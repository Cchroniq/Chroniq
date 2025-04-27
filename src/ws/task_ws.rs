use std::collections::HashMap;
use std::sync::Arc;

use futures_util::StreamExt;

use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;

use crate::conf::config;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TaskStatus {
    Submited,
    Status,
    ExecutionStart,
    ExecutionCached,
    Executing,
    Progress,
    Executed,
    ExecutionSuccess,
    ExecutionFailed,
}

static GLOBAL_TASK_STATUS: Lazy<Arc<Mutex<HashMap<String, TaskStatus>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

pub async fn get_task_status(task_id: &str) -> Option<TaskStatus> {
    let map = GLOBAL_TASK_STATUS.lock().await;
    map.get(task_id).cloned()
}

pub async fn update_task_status(task_id: &str, new_status: TaskStatus) -> bool {
    let mut map = GLOBAL_TASK_STATUS.lock().await;
    if let Some(status) = map.get_mut(task_id) {
        *status = new_status;
        true
    } else {
        map.insert(task_id.to_string(), new_status);
        true
    }
}

pub async fn ws_connect(config: Arc<config::Config>) -> anyhow::Result<()> {
    let ws_url = format!(
        "ws://{}/ws?clientId={}",
        config.sd3_base_server, config.sd3_client_id
    );

    let (ws_stream, _) = connect_async(&ws_url).await?;
    let (mut _write, mut read) = ws_stream.split();
    while let Some(message) = read.next().await {
        match message {
            std::result::Result::Ok(msg) if msg.is_text() => {
                let text = msg.to_text()?;
                let message: serde_json::Value = serde_json::from_str(text)?;
                if message["type"] == "status" {
                    if message["type"]["data"]["exec_info"]["queue_remaining"].is_number() {
                        let queue_remaining = message["data"]["exec_info"]["queue_remaining"]
                            .as_i64()
                            .unwrap_or(0);
                        info!("queue_remaining: {}", queue_remaining);
                    } else {
                        let prompt_id = message["data"]["prompt_id"].as_str().unwrap_or("");
                        let status = message["data"]["status"].as_str().unwrap_or("");
                        let task_status = match status {
                            "submited" => TaskStatus::Submited,
                            "execution_start" => TaskStatus::ExecutionStart,
                            "execution_cached" => TaskStatus::ExecutionCached,
                            "executing" => TaskStatus::Executing,
                            "progress" => TaskStatus::Progress,
                            "executed" => TaskStatus::Executed,
                            "execution_success" => TaskStatus::ExecutionSuccess,
                            "execution_failed" => TaskStatus::ExecutionFailed,
                            _ => continue,
                        };
                        update_task_status(prompt_id, task_status).await;
                    }
                }
            }
            Err(e) => {
                error!("Error receiving message: {:?}", e);
                return Err(e.into());
            }
            _ => continue,
        }
    }
    Ok(())
}
