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
use tracing::info;

use crate::{
    config::{FrontendMode, RuntimeConfig},
    handlers::{assets::serve_media, assets::serve_thumbnail, api::get_folder, api::get_folder_root, api::health},
    http_utils::build_cors,
    indexer::{spawn_initial_index, start_media_watcher},
    models::MediaIndex,
    state::{AppState, IndexStatusState},
};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    init_tracing();

    let config = RuntimeConfig::from_env()?;
    fs::create_dir_all(&config.thumbnail_root).await?;

    let state = AppState {
        media_root: config.media_root.clone().into(),
        thumbnail_root: config.thumbnail_root.clone().into(),
        index: std::sync::Arc::new(RwLock::new(MediaIndex::default())),
        status: std::sync::Arc::new(RwLock::new(IndexStatusState::default())),
        index_run_lock: std::sync::Arc::new(tokio::sync::Mutex::new(())),
        frontend_mode: config.frontend_mode,
    };

    start_media_watcher(state.clone())?;
    spawn_initial_index(state.clone());

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
        .layer(build_cors(&config.cors_allow_origin)?);

    log_startup(&config);

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("local_gallery_server=info,tower_http=warn"));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();
}

fn log_startup(config: &RuntimeConfig) {
    info!("media root: {}", config.media_root.display());
    if config.cli_media_root.is_some() || config.env_media_root.is_some() {
        info!("media root source resolved");
    }
    info!("frontend mode: {}", config.frontend_mode.as_str());
    info!("listening on http://{}", config.bind_addr);
}
