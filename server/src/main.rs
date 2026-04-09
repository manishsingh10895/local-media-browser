mod config;
mod frontend;
mod handlers;
mod http_utils;
mod indexer;
mod models;
mod paths;
mod responses;
mod sorting;
mod state;
mod thumbnails;

use anyhow::Result;
use axum::{Router, routing::get};
use dotenvy::dotenv;
use tokio::{fs, sync::RwLock};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{
    config::{FrontendMode, RuntimeConfig},
    handlers::{assets::serve_media, assets::serve_thumbnail, api::get_folder, api::get_folder_root, api::health},
    http_utils::build_cors,
    indexer::{scan_media_index, start_media_watcher},
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    init_tracing();

    let config = RuntimeConfig::from_env()?;
    fs::create_dir_all(&config.thumbnail_root).await?;

    let initial_index = scan_media_index(&config.media_root).await?;
    let state = AppState {
        media_root: config.media_root.clone().into(),
        thumbnail_root: config.thumbnail_root.clone().into(),
        index: std::sync::Arc::new(RwLock::new(initial_index)),
        frontend_mode: config.frontend_mode,
    };

    start_media_watcher(state.clone())?;

    let mut app = Router::new()
        .route("/api/health", get(health))
        .route("/api/folder", get(get_folder_root))
        .route("/api/folder/{*path}", get(get_folder))
        .route("/media/{*path}", get(serve_media))
        .route("/thumbs/{*path}", get(serve_thumbnail));

    if matches!(config.frontend_mode, FrontendMode::Axum) {
        app = frontend::mount_routes(app);
    }

    let app = app
        .with_state(state)
        .layer(build_cors(&config.cors_allow_origin)?)
        .layer(TraceLayer::new_for_http());

    log_startup(&config);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

fn log_startup(config: &RuntimeConfig) {
    info!("MEDIA_ROOT env value: {:?}", config.env_media_root);
    info!("media root CLI override: {:?}", config.cli_media_root);
    info!("effective media root: {}", config.media_root.display());
    info!("frontend mode: {}", config.frontend_mode.as_str());
    info!("listening on http://{}", config.bind_addr);
}
