use std::{io::ErrorKind, sync::Arc};

use actix_web::{App, HttpServer, web};
use log::info;

mod api;
mod conf;
mod sd3;
mod utils;
mod ws;
mod solana;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = conf::config::Config::from_env().expect("Failed to load configuration");
    conf::logging::init_logger(&config);
    let config_arc = Arc::new(config.clone());

    let serve_addr = config.server_addr.clone();
    let serve = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .configure(router_config)
    })
    .bind(&serve_addr)?
    .run();
    info!("serve was running: {}", &serve_addr);
    let server_handle = serve.handle();
    tokio::spawn(serve);
    tokio::spawn(async move {
        let _ = ws::task_ws::ws_connect(config_arc).await;
    });
    tokio::signal::ctrl_c()
        .await
        .map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;
    server_handle.stop(true).await;
    Ok(())
}

pub fn router_config(cfg: &mut web::ServiceConfig) {
    cfg.service(api::task_api::submit_imageine)
        .service(api::task_api::fetch_task)
        .service(api::file_api::file);
}
