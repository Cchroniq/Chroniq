use std::{
    fs,
    path::{Path, PathBuf},
};

use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, get, http::header, web};

use crate::utils::result::{CqResult, Nothing};

#[get("/file/{file_name}")]
async fn file(
    config: web::Data<crate::conf::config::Config>,
    path: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let file_name = path.into_inner();
    if file_name.trim().is_empty() {
        let result = CqResult::<Nothing>::error(500, "file_name is empty");
        return HttpResponse::BadRequest().json(result);
    }
    let root_dir = PathBuf::from(config.img_tmp_path.clone())
        .canonicalize()
        .map_err(|e| {
            let result = CqResult::<Nothing>::error(500, &format!("Invalid FILE_ROOT: {}", e));
            HttpResponse::InternalServerError().json(result)
        })
        .unwrap();
    let full_path = match validate_path(&file_name, &root_dir) {
        Ok(path) => path,
        Err(e) => {
            let result = CqResult::<Nothing>::error(500, &e);
            return HttpResponse::BadRequest().json(result);
        }
    };
    let metadata = match fs::metadata(&full_path) {
        Ok(meta) => meta,
        Err(e) => {
            let result =
                CqResult::<Nothing>::error(500, &format!("Failed to read metadata: {}", e));
            return HttpResponse::NotFound().json(result);
        }
    };
    let file_name_only = full_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let file_size = metadata.len();
    match NamedFile::open(&full_path) {
        Ok(named_file) => {
            let mut response = named_file.into_response(&req);
            if let Ok(name_value) = header::HeaderValue::from_str(&file_name_only) {
                response
                    .headers_mut()
                    .insert(header::HeaderName::from_static("x-file-name"), name_value);
            }
            if let Ok(size_value) = header::HeaderValue::from_str(&file_size.to_string()) {
                response
                    .headers_mut()
                    .insert(header::HeaderName::from_static("x-file-size"), size_value);
            }
            return response;
        }
        Err(e) => {
            let result = CqResult::<Nothing>::error(500, &format!("Failed to open file: {}", e));
            return HttpResponse::NotFound().json(result);
        }
    }
}

fn validate_path(input_path: &str, root_dir: &Path) -> Result<PathBuf, String> {
    let full_path = root_dir.join(input_path);
    let path = full_path
        .canonicalize()
        .map_err(|e| format!("Invalid path: {}", e))?;

    if !path.starts_with(root_dir) {
        return Err("Path exceeds allowed directories".to_string());
    }
    if !path.is_file() {
        return Err("Path is not a file".to_string());
    }

    Ok(path)
}
