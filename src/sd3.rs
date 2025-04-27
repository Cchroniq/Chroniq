use anyhow::{Ok, Result, anyhow};
use log::info;
use rand::Rng;
use rand::rng;
use reqwest::Client;
use serde_json::Value;

use std::fs::File;
use std::io::Write;

use crate::ws;
use crate::ws::task_ws::TaskStatus;
use crate::ws::task_ws::update_task_status;

pub struct SD3Client {
    client: Client,
    server_address: String,
}

pub struct ImagineRequest {
    pub prompt: String,
    pub steps: i32,
    pub workflow: serde_json::Value,
    pub sd3_model_file: String,
    pub sd3_clip_name1: String,
    pub sd3_clip_name2: String,
    pub sd3_clip_name3: String,
}

impl SD3Client {
    pub fn new(server_address: &str) -> Self {
        SD3Client {
            client: Client::new(),
            server_address: server_address.to_string(),
        }
    }

    pub async fn submit_imagine(&self, imagine: ImagineRequest) -> anyhow::Result<String> {
        let workflow_id = uuid::Uuid::new_v4().to_string();
        let client_id = uuid::Uuid::new_v4().to_string();
        let seed = rng().random_range(0..=u32::MAX) as i32;
        let images = self
            .submit_workflow(imagine, seed, workflow_id, client_id)
            .await?;
        Ok(images)
    }

    async fn queue_prompt(&self, workflow_data: Value) -> anyhow::Result<String> {
        let url = format!("http://{}/api/prompt", self.server_address);
        let res = self.client.post(&url).json(&workflow_data).send().await?;
        let json: Value = res.json().await?;
        if json["prompt_id"].is_null() {
            return Err(anyhow::anyhow!("submit task failed: No prompt_id found"));
        }
        anyhow::Ok(json["prompt_id"].as_str().unwrap().to_string())
    }

    async fn submit_sd3_queue(&self, workflow_data: Value) -> anyhow::Result<String> {
        let prompt_id = self.queue_prompt(workflow_data).await?;
        info!("prompt_id: {}", prompt_id);
        Ok(prompt_id)
    }

    pub async fn fetch_sd3_image(
        &self,
        img_tmp_path: &str,
        prompt_id: &str,
    ) -> anyhow::Result<(TaskStatus, String)> {
        match ws::task_ws::get_task_status(prompt_id).await {
            Some(state) => {
                let file_name = format!("{}.png", prompt_id);
                let file_path = format!("{}/{}", img_tmp_path, file_name);
                if File::open(file_path.clone()).is_ok() {
                    return Ok((state, file_name));
                }
                let history = self.get_history(&prompt_id).await?;
                let mut output_images = std::collections::HashMap::new();
                if let Some(history_data) = history.get(&prompt_id) {
                    for (node_id, node_output) in history_data["outputs"]
                        .as_object()
                        .ok_or_else(|| anyhow!("Outputs not found"))?
                    {
                        if let Some(images) = node_output.get("images") {
                            let mut images_output = Vec::new();
                            for image in images
                                .as_array()
                                .ok_or_else(|| anyhow!("Images not an array"))?
                            {
                                let image_data = self
                                    .get_image(
                                        image["filename"]
                                            .as_str()
                                            .ok_or_else(|| anyhow!("Filename missing"))?,
                                        image["subfolder"]
                                            .as_str()
                                            .ok_or_else(|| anyhow!("Subfolder missing"))?,
                                        image["type"]
                                            .as_str()
                                            .ok_or_else(|| anyhow!("Type missing"))?,
                                    )
                                    .await?;
                                images_output.push(image_data);
                            }
                            output_images.insert(node_id.clone(), images_output);
                        }
                    }

                    for (_node_id, image_data_vec) in output_images {
                        for image_data in image_data_vec {
                            self.save_image(&image_data, &file_path)?;
                        }
                    }
                    update_task_status(prompt_id, TaskStatus::ExecutionSuccess).await;
                    return Ok((state, file_name));
                }
            }
            None => {
                return Err(anyhow!("Failed to get task status"));
            }
        };

        Ok((TaskStatus::Submited, String::new()))
    }
    async fn get_image(
        &self,
        filename: &str,
        subfolder: &str,
        folder_type: &str,
    ) -> Result<Vec<u8>> {
        let url = format!(
            "http://{}/view?filename={}&subfolder={}&type={}",
            self.server_address, filename, subfolder, folder_type
        );
        let res = self.client.get(&url).send().await?;
        let bytes = res.bytes().await?;
        Ok(bytes.to_vec())
    }

    async fn submit_workflow(
        &self,
        imagine: ImagineRequest,
        _seed: i32,
        workflow_id: String,
        client_id: String,
    ) -> anyhow::Result<String> {
        let mut workflow_data: Value = imagine.workflow.clone();
        workflow_data["prompt"]["6"]["inputs"]["text"] = Value::String(imagine.prompt);
        workflow_data["prompt"]["294"]["inputs"]["steps"] = Value::Number(imagine.steps.into());
        replace_placeholder(&mut workflow_data,"${model_name}",&imagine.sd3_model_file);
        replace_placeholder(&mut workflow_data,"${sd3_clip_name1}",&imagine.sd3_clip_name1,        );
        replace_placeholder(&mut workflow_data,"${sd3_clip_name2}",&imagine.sd3_clip_name2,);
        replace_placeholder(&mut workflow_data,"${sd3_clip_name3}",&imagine.sd3_clip_name3,);
        replace_placeholder(&mut workflow_data, "${client_id}", client_id.as_str());
        replace_placeholder(&mut workflow_data, "${workflow_id}", workflow_id.as_str());
        self.submit_sd3_queue(workflow_data).await
    }

    #[allow(dead_code)]
    pub async fn get_history(&self, prompt_id: &str) -> anyhow::Result<Value> {
        let url = format!("http://{}/api/history/{}", self.server_address, prompt_id);
        let res = self.client.get(&url).send().await?;
        let json: Value = res.json().await?;
        anyhow::Ok(json)
    }

    #[allow(dead_code)]
    fn save_image(&self, image_data: &[u8], path: &str) -> Result<()> {
        let mut file = File::create(path)?;
        file.write_all(image_data)?;
        Ok(())
    }
}

fn replace_placeholder(value: &mut Value, placeholder: &str, replacement: &str) {
    match value {
        Value::String(s) => {
            if s.contains(placeholder) {
                *s = s.replace(placeholder, replacement);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                replace_placeholder(item, placeholder, replacement);
            }
        }
        Value::Object(obj) => {
            for (_key, val) in obj {
                replace_placeholder(val, placeholder, replacement);
            }
        }
        _ => {} 
    }
}
