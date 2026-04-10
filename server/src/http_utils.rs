use anyhow::{Context, Result};
use axum::http::{HeaderMap, HeaderValue, header};
use tower_http::cors::{Any, CorsLayer};

pub fn build_cors(allow_origin: &str) -> Result<CorsLayer> {
    if allow_origin == "*" {
        return Ok(CorsLayer::new().allow_origin(Any));
    }

    let origin = HeaderValue::from_str(allow_origin).context("invalid CORS_ALLOW_ORIGIN")?;
    Ok(CorsLayer::new().allow_origin(origin))
}

pub fn apply_no_cache_headers(headers: &mut HeaderMap) {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(header::EXPIRES, HeaderValue::from_static("0"));
}

pub fn apply_thumbnail_cache_headers(headers: &mut HeaderMap) {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
}
