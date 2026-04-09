use axum::{
    Json,
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{
    http_utils::apply_no_cache_headers,
    models::{FolderQuery, HealthResponse},
    paths::sanitize_relative_path,
    responses::build_folder_response,
    state::AppState,
};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn get_folder_root(
    State(state): State<AppState>,
    Query(query): Query<FolderQuery>,
) -> impl IntoResponse {
    folder_response(state, String::new(), query).await
}

pub async fn get_folder(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    Query(query): Query<FolderQuery>,
) -> impl IntoResponse {
    let Some(sanitized) = sanitize_relative_path(&path) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    folder_response(state, sanitized.to_string_lossy().replace('\\', "/"), query).await
}

async fn folder_response(state: AppState, current_path: String, query: FolderQuery) -> Response {
    match build_folder_response(state, current_path, query).await {
        Ok(payload) => {
            let mut response = Json(payload).into_response();
            apply_no_cache_headers(response.headers_mut());
            response
        }
        Err(status) => status.into_response(),
    }
}
