use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use axum::{
    body::Body,
    http::{HeaderValue, Response, header},
};
use image::ImageFormat;
use tokio::task;

use crate::{
    config::THUMBNAIL_SIZE,
    http_utils::apply_thumbnail_cache_headers,
    models::{MediaItem, ThumbnailOutcome},
};

pub fn thumbnail_cache_path(thumbnail_root: &Path, item: &MediaItem) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    item.relative_path.hash(&mut hasher);
    item.modified_ms.hash(&mut hasher);
    thumbnail_root.join(format!("{:x}.jpg", hasher.finish()))
}

pub fn svg_thumbnail_response(label: &str) -> Response<Body> {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 480 360">
<defs><linearGradient id="bg" x1="0" x2="1" y1="0" y2="1"><stop offset="0%" stop-color="#132130"/><stop offset="100%" stop-color="#0a121b"/></linearGradient></defs>
<rect width="480" height="360" fill="url(#bg)"/><rect x="32" y="32" width="416" height="296" rx="28" fill="rgba(255,255,255,0.04)" stroke="rgba(255,255,255,0.08)"/>
<text x="240" y="190" text-anchor="middle" fill="#f0c48c" font-family="Arial, sans-serif" font-size="40" letter-spacing="8">{label}</text></svg>"##
    );
    let mut response = Response::new(Body::from(svg));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"));
    apply_thumbnail_cache_headers(response.headers_mut());
    response
}

pub async fn generate_thumbnail(source_path: PathBuf, cache_path: PathBuf) -> Result<ThumbnailOutcome> {
    task::spawn_blocking(move || -> Result<ThumbnailOutcome> {
        if let Some(parent) = cache_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let reader = image::ImageReader::open(&source_path)?.with_guessed_format()?;
        let Some(format) = reader.format() else {
            return Ok(ThumbnailOutcome::UnsupportedSource);
        };
        if matches!(format, ImageFormat::Gif) {
            return Ok(ThumbnailOutcome::UnsupportedSource);
        }

        let image = match reader.decode() {
            Ok(image) => image,
            Err(_) => return Ok(ThumbnailOutcome::UnsupportedSource),
        };
        image
            .thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE)
            .save_with_format(&cache_path, ImageFormat::Jpeg)?;
        Ok(ThumbnailOutcome::Generated)
    })
    .await
    .context("thumbnail generation task failed")?
}
