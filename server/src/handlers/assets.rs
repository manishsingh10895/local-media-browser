use axum::{
    body::Body,
    extract::{Path as AxumPath, Query, State},
    http::{HeaderValue, Response, StatusCode, header},
    response::IntoResponse,
};
use tokio::fs;
use tracing::error;

use crate::{
    http_utils::{apply_no_cache_headers, apply_thumbnail_cache_headers},
    models::{MediaQuery, MediaType, ThumbnailOutcome},
    paths::sanitize_relative_path,
    state::AppState,
    thumbnails::{generate_thumbnail, svg_thumbnail_response, thumbnail_cache_path},
};

pub async fn serve_media(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    Query(query): Query<MediaQuery>,
) -> impl IntoResponse {
    let Some(sanitized) = sanitize_relative_path(&path) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let relative_path = sanitized.to_string_lossy().replace('\\', "/");
    let item = {
        let index = state.index.read().await;
        match index.media_by_path.get(&relative_path).cloned() {
            Some(item) => item,
            None => return StatusCode::NOT_FOUND.into_response(),
        }
    };

    match fs::read(state.media_root.join(&sanitized)).await {
        Ok(bytes) => {
            let mut response = Response::new(Body::from(bytes));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_str(&item.mime)
                    .unwrap_or(HeaderValue::from_static("application/octet-stream")),
            );
            if query.download.unwrap_or(false) {
                let file_name = item.name.replace('"', "");
                if let Ok(value) =
                    HeaderValue::from_str(&format!("attachment; filename=\"{file_name}\""))
                {
                    response
                        .headers_mut()
                        .insert(header::CONTENT_DISPOSITION, value);
                }
            }
            apply_no_cache_headers(response.headers_mut());
            response.into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn serve_thumbnail(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> impl IntoResponse {
    let Some(sanitized) = sanitize_relative_path(&path) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let relative_path = sanitized.to_string_lossy().replace('\\', "/");
    let Some(item) = ({
        let index = state.index.read().await;
        index.media_by_path.get(&relative_path).cloned()
    }) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if matches!(item.media_type, MediaType::Video) {
        return svg_thumbnail_response("VIDEO").into_response();
    }

    let cache_path = thumbnail_cache_path(&state.thumbnail_root, &item);
    if fs::metadata(&cache_path).await.is_err() {
        match generate_thumbnail(state.media_root.join(&sanitized), cache_path.clone()).await {
            Ok(ThumbnailOutcome::Generated) => {}
            Ok(ThumbnailOutcome::UnsupportedSource) => return svg_thumbnail_response("IMAGE").into_response(),
            Err(err) => {
                error!("failed to generate thumbnail for {}: {err:#}", item.relative_path);
                return svg_thumbnail_response("IMAGE").into_response();
            }
        }
    }

    match fs::read(&cache_path).await {
        Ok(bytes) => {
            let mut response = Response::new(Body::from(bytes));
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg"));
            apply_thumbnail_cache_headers(response.headers_mut());
            response.into_response()
        }
        Err(_) => svg_thumbnail_response("IMAGE").into_response(),
    }
}
