use std::env;

use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(&'static str),
}

#[derive(Debug, Clone, Default)]
pub struct Config {
    pub server_addr: String,
    pub log_level: String,
    pub sd3_model_file_name: String,
    pub sd3_clip_name1: String,
    pub sd3_clip_name2: String,
    pub sd3_clip_name3: String,
    pub sd3_base_server: String,
    pub sd3_client_id: String,
    pub img_tmp_point: String,
    pub img_tmp_path: String,
    pub wf_json_path: String,
    pub solana_points: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();
        let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".to_string());
        let img_tmp_path =
            env::var("IMG_TEMP_PATH").map_err(|_| ConfigError::MissingEnvVar("IMG_TEMP_PATH"))?;
        let img_tmp_point =
            env::var("IMG_TMP_POINT").map_err(|_| ConfigError::MissingEnvVar("IMG_TMP_POINT"))?;
        let sd3_base_server = env::var("SD3_BASE_SERVER")
            .map_err(|_| ConfigError::MissingEnvVar("SD3_BASE_SERVER"))?;
        let sd3_client_id =
            env::var("SD3_CLIENT_ID").unwrap_or_else(|_| Uuid::new_v4().to_string());

        let sd3_model_file_name = env::var("SD3_MODEL_FILE_NAME")
            .map_err(|_| ConfigError::MissingEnvVar("SD3_MODEL_FILE_NAME"))?;

        let sd3_clip_name1 =
            env::var("SD3_CLIP_NAME1").map_err(|_| ConfigError::MissingEnvVar("SD3_CLIP_NAME1"))?;
        let sd3_clip_name2 =
            env::var("SD3_CLIP_NAME2").map_err(|_| ConfigError::MissingEnvVar("SD3_CLIP_NAME2"))?;

        let sd3_clip_name3 =
            env::var("SD3_CLIP_NAME3").map_err(|_| ConfigError::MissingEnvVar("SD3_CLIP_NAME3"))?;
        let wf_json_path =
            env::var("WF_JSON_PATH").map_err(|_| ConfigError::MissingEnvVar("WF_JSON_PATH"))?;
        let solana_points_str =
            env::var("SOLANA_POINTS").map_err(|_| ConfigError::MissingEnvVar("SOLANA_POINTS"))?;
        let solana_points: Vec<String> = solana_points_str
            .split(",")
            .map(|s| s.to_string())
            .collect();
        Ok(Config {
            server_addr,
            log_level,
            sd3_base_server,
            sd3_client_id,
            sd3_model_file_name,
            sd3_clip_name1,
            sd3_clip_name2,
            sd3_clip_name3,
            img_tmp_path,
            img_tmp_point,
            wf_json_path,
            solana_points,
        })
    }
}
