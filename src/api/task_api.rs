use std::{fs::File, io::BufReader};

use actix_web::{HttpResponse, get, post, web};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    conf,
    sd3::{self, ImagineRequest},
    utils::result::{CqResult, Nothing},
    ws,
};
#[derive(Debug, Serialize, Deserialize)]
struct ImaRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    base64_array: Option<Vec<String>>,
    prompt: String,
    author: String,
    style: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "default_steps")]
    steps: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default = "default_workflow")]
    workflow: Option<serde_json::Value>,
}

fn default_workflow() -> Option<serde_json::Value> {
    let conf = conf::config::Config::from_env().ok()?;
    let workflowfile = conf.wf_json_path;
    let prompt_file = File::open(workflowfile).ok()?;
    let prompt_reader = BufReader::new(prompt_file);
    serde_json::from_reader(prompt_reader).ok()?
}

fn default_steps() -> Option<i32> {
    Some(4)
}

#[post("/submit_imageine")]
async fn submit_imageine(
    config: web::Data<crate::conf::config::Config>,
    req: web::Json<ImaRequest>,
) -> HttpResponse {
    if req.author.clone().trim().is_empty()
        || req.prompt.clone().trim().is_empty()
        || req.style.clone().trim().is_empty()
    {
        return HttpResponse::BadRequest().json(CqResult::<Nothing>::error(
            500,
            "author prompt style can not be empty",
        ));
    }
    let prompt = format!("{},{}", req.prompt.clone(), req.style.clone());
    let imagine_request = ImagineRequest {
        prompt: prompt.clone(),
        steps: req.steps.unwrap(),
        workflow: req.workflow.clone().unwrap(),
        sd3_model_file: config.sd3_model_file_name.clone(),
        sd3_clip_name1: config.sd3_clip_name1.clone(),
        sd3_clip_name2: config.sd3_clip_name2.clone(),
        sd3_clip_name3: config.sd3_clip_name3.clone(),
    };
    let sd3_client = sd3::SD3Client::new(&config.sd3_base_server);
    match sd3_client.submit_imagine(imagine_request).await {
        Ok(res) => {
            ws::task_ws::update_task_status(&res, ws::task_ws::TaskStatus::Submited).await;
            return HttpResponse::Ok().json(CqResult::<String>::success(res));
        }
        Err(e) => {
            error!("{} ERROR!!!", e);
            return HttpResponse::BadRequest().json(CqResult::<Nothing>::error(
                500,
                "submit task failed , please check your prompt",
            ));
        }
    }
}

#[get("/fetch_task/{prompt_id}")]

pub async fn fetch_task(
    config: web::Data<crate::conf::config::Config>,
    path: web::Path<String>,
) -> HttpResponse {
    let prompt_id = path.into_inner();
    let sd3client = sd3::SD3Client::new(&config.sd3_base_server);

    match sd3client
        .fetch_sd3_image(&config.img_tmp_path, &prompt_id)
        .await
    {
        Ok((state, img_name)) => {
            if img_name.is_empty() {
                return HttpResponse::Ok()
                    .json(CqResult::success("task is not finish yet".to_string()));
            }
            return HttpResponse::Ok().json(CqResult::<serde_json::Value>::success(
                json!({"task_state":state,"img_url":format!("{}/{}",config.img_tmp_point,img_name)}),
            ));
        }
        Err(e) => {
            error!("{} ERROR!!!", e);
            return HttpResponse::BadRequest().json(CqResult::<Nothing>::error(
                500,
                "fetch task failed , please check your prompt_id",
            ));
        }
    };
}
