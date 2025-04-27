use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CqResult<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Nothing;

impl<T: Serialize> CqResult<T> {
    pub fn success(data: T) -> Self {
        CqResult {
            code: 200,
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(code: i32, message: &str) -> Self {
        CqResult {
            code,
            message: message.to_string(),
            data: None,
        }
    }
}
