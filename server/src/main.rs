use std::{
    env,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path as AxumPath, State},
    http::{HeaderValue, Response, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use dotenvy::dotenv;
use mime_guess::mime;
use serde::Serialize;
use tokio::fs;
use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
use tracing::{error, info};

#[derive(Clone)]
struct AppState {
    media_root: Arc<PathBuf>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct MediaItem {
    name: String,
    relative_path: String,
    media_type: MediaType,
    mime: String,
    size_bytes: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum MediaType {
    Image,
    Video,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let cli_media_root = env::args().nth(1);

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let bind_addr: SocketAddr = read_env("BIND_ADDR", "0.0.0.0:6677")
        .parse()
        .context("invalid BIND_ADDR")?;
    let media_root = cli_media_root
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(read_env("MEDIA_ROOT", "../media")));
    let cors_allow_origin = read_env("CORS_ALLOW_ORIGIN", "*");

    let state = AppState {
        media_root: Arc::new(media_root.clone()),
    };

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/media", get(list_media))
        .route("/media/{*path}", get(serve_media))
        .with_state(state)
        .layer(build_cors(&cors_allow_origin)?)
        .layer(TraceLayer::new_for_http());

    info!("serving media root {}", media_root.display());
    info!("listening on http://{}", bind_addr);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn read_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn build_cors(allow_origin: &str) -> Result<CorsLayer> {
    if allow_origin == "*" {
        return Ok(CorsLayer::new().allow_origin(Any));
    }

    let origin = HeaderValue::from_str(allow_origin).context("invalid CORS_ALLOW_ORIGIN")?;
    Ok(CorsLayer::new().allow_origin(origin))
}

fn apply_no_cache_headers(headers: &mut axum::http::HeaderMap) {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn list_media(State(state): State<AppState>) -> impl IntoResponse {
    match collect_media(&state.media_root).await {
        Ok(items) => {
            let mut response = Json(items).into_response();
            apply_no_cache_headers(response.headers_mut());
            response
        }
        Err(error) => {
            error!("failed to list media: {error:#}");
            let mut response = (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "failed to list media" })),
            )
                .into_response();
            apply_no_cache_headers(response.headers_mut());
            response
        }
    }
}

async fn serve_media(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> impl IntoResponse {
    let sanitized = match sanitize_relative_path(&path) {
        Some(path) => path,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let full_path = state.media_root.join(sanitized);
    match fs::read(&full_path).await {
        Ok(bytes) => {
            let mime = mime_guess::from_path(&full_path).first_or_octet_stream();
            let mut response = Response::new(Body::from(bytes));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime.as_ref()).unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            apply_no_cache_headers(response.headers_mut());
            response.into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn sanitize_relative_path(path: &str) -> Option<PathBuf> {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return None;
    }

    let mut clean = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::RootDir | Component::ParentDir | Component::Prefix(_) => return None,
        }
    }

    Some(clean)
}

async fn collect_media(root: &Path) -> Result<Vec<MediaItem>> {
    let mut items = Vec::new();
    let root = fs::canonicalize(root)
        .await
        .with_context(|| format!("failed to access media root {}", root.display()))?;
    collect_media_recursive(&root, &root, &mut items).await?;
    items.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(items)
}

async fn collect_media_recursive(
    root: &Path,
    current: &Path,
    items: &mut Vec<MediaItem>,
) -> Result<()> {
    let mut entries = fs::read_dir(current).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        let metadata = entry.metadata().await?;

        if metadata.is_dir() {
            Box::pin(collect_media_recursive(root, &path, items)).await?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let mime = mime_guess::from_path(&path).first_or_octet_stream();
        let media_type = if mime.type_() == mime::IMAGE {
            Some(MediaType::Image)
        } else if mime.type_() == mime::VIDEO {
            Some(MediaType::Video)
        } else {
            None
        };

        let Some(media_type) = media_type else {
            continue;
        };

        let relative_path = path
            .strip_prefix(root)
            .context("failed to strip media root from file path")?
            .to_string_lossy()
            .replace('\\', "/");

        items.push(MediaItem {
            name: entry.file_name().to_string_lossy().to_string(),
            relative_path,
            media_type,
            mime: mime.to_string(),
            size_bytes: metadata.len(),
        });
    }

    Ok(())
}
