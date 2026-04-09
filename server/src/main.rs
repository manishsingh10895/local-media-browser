use std::{
    collections::{BTreeMap, BTreeSet, hash_map::DefaultHasher},
    env,
    hash::{Hash, Hasher},
    net::SocketAddr,
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use axum::{
    Json, Router,
    body::Body,
    extract::{Path as AxumPath, Query, State},
    http::{HeaderValue, Response, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use dotenvy::dotenv;
use image::ImageFormat;
use mime_guess::mime;
use notify::{RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use tokio::{fs, sync::RwLock, task};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};

const DEFAULT_PAGE_LIMIT: usize = 60;
const MAX_PAGE_LIMIT: usize = 200;
const THUMBNAIL_SIZE: u32 = 480;

#[derive(Clone)]
struct AppState {
    media_root: Arc<PathBuf>,
    thumbnail_root: Arc<PathBuf>,
    index: Arc<RwLock<MediaIndex>>,
}

#[derive(Default, Clone)]
struct MediaIndex {
    media_by_path: BTreeMap<String, MediaItem>,
    folders: BTreeMap<String, FolderNode>,
}

#[derive(Clone, Default)]
struct FolderNode {
    path: String,
    name: String,
    child_folders: BTreeSet<String>,
    immediate_media: Vec<String>,
    item_count: u64,
    total_size_bytes: u64,
    newest_modified_ms: u64,
    cover_relative_path: Option<String>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
struct MediaItem {
    name: String,
    relative_path: String,
    media_type: MediaType,
    mime: String,
    size_bytes: u64,
    modified_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum MediaType {
    Image,
    Video,
}

#[derive(Clone)]
struct MediaClassification {
    media_type: MediaType,
    mime: mime::Mime,
}

enum ThumbnailOutcome {
    Generated,
    UnsupportedSource,
}

#[derive(Serialize)]
struct FolderEntry {
    name: String,
    relative_path: String,
    item_count: u64,
    total_size_bytes: u64,
    newest_modified_ms: u64,
    cover: Option<MediaItem>,
}

#[derive(Serialize)]
struct BreadcrumbItem {
    name: String,
    href: String,
}

#[derive(Serialize)]
struct FolderResponse {
    current_path: String,
    breadcrumbs: Vec<BreadcrumbItem>,
    folders: Vec<FolderEntry>,
    media: Vec<MediaItem>,
    next_offset: Option<usize>,
    total_media_count: usize,
    limit: usize,
    sort_field: &'static str,
    sort_direction: &'static str,
}

#[derive(Deserialize, Default)]
struct MediaQuery {
    download: Option<bool>,
}

#[derive(Deserialize, Default)]
struct FolderQuery {
    sort: Option<String>,
    dir: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[derive(Clone, Copy)]
enum SortField {
    Name,
    Date,
    Size,
}

#[derive(Clone, Copy)]
enum SortDirection {
    Asc,
    Desc,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let cli_media_root = env::args().nth(1);
    let env_media_root = env::var("MEDIA_ROOT").ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let bind_addr: SocketAddr = read_env("BIND_ADDR", "0.0.0.0:6677")
        .parse()
        .context("invalid BIND_ADDR")?;
    let media_root = cli_media_root
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(read_env("MEDIA_ROOT", "../media")));
    let cors_allow_origin = read_env("CORS_ALLOW_ORIGIN", "*");
    let thumbnail_root = build_thumbnail_root(&media_root);
    fs::create_dir_all(&thumbnail_root).await?;

    let initial_index = scan_media_index(&media_root).await?;
    let state = AppState {
        media_root: Arc::new(media_root.clone()),
        thumbnail_root: Arc::new(thumbnail_root),
        index: Arc::new(RwLock::new(initial_index)),
    };

    start_media_watcher(state.clone())?;

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/folder", get(get_folder_root))
        .route("/api/folder/{*path}", get(get_folder))
        .route("/media/{*path}", get(serve_media))
        .route("/thumbs/{*path}", get(serve_thumbnail))
        .with_state(state)
        .layer(build_cors(&cors_allow_origin)?)
        .layer(TraceLayer::new_for_http());

    info!("MEDIA_ROOT env value: {:?}", env_media_root);
    info!("media root CLI override: {:?}", cli_media_root);
    info!("effective media root: {}", media_root.display());
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

fn apply_thumbnail_cache_headers(headers: &mut axum::http::HeaderMap) {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
}

fn build_thumbnail_root(media_root: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    media_root.to_string_lossy().hash(&mut hasher);
    std::env::temp_dir()
        .join("local-gallery-thumbnails")
        .join(format!("{:x}", hasher.finish()))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn get_folder_root(
    State(state): State<AppState>,
    Query(query): Query<FolderQuery>,
) -> impl IntoResponse {
    build_folder_response(state, String::new(), query).await
}

async fn get_folder(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    Query(query): Query<FolderQuery>,
) -> impl IntoResponse {
    let sanitized = match sanitize_relative_path(&path) {
        Some(path) => path,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let current_path = sanitized.to_string_lossy().replace('\\', "/");
    build_folder_response(state, current_path, query).await
}

async fn build_folder_response(
    state: AppState,
    current_path: String,
    query: FolderQuery,
) -> Response<Body> {
    let sort_field = parse_sort_field(query.sort.as_deref());
    let sort_direction = parse_sort_direction(query.dir.as_deref());
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT);

    let index = state.index.read().await;
    let Some(folder) = index.folders.get(&current_path) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let mut folders = folder
        .child_folders
        .iter()
        .filter_map(|path| index.folders.get(path))
        .map(|folder| FolderEntry {
            name: folder.name.clone(),
            relative_path: folder.path.clone(),
            item_count: folder.item_count,
            total_size_bytes: folder.total_size_bytes,
            newest_modified_ms: folder.newest_modified_ms,
            cover: folder
                .cover_relative_path
                .as_ref()
                .and_then(|path| index.media_by_path.get(path))
                .cloned(),
        })
        .collect::<Vec<_>>();
    sort_folder_entries(&mut folders, sort_field, sort_direction);

    let mut immediate_media = folder
        .immediate_media
        .iter()
        .filter_map(|path| index.media_by_path.get(path))
        .cloned()
        .collect::<Vec<_>>();
    sort_media_items(&mut immediate_media, sort_field, sort_direction);

    let total_media_count = immediate_media.len();
    let media = immediate_media
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    let next_offset = if offset + media.len() < total_media_count {
        Some(offset + media.len())
    } else {
        None
    };

    let mut response = Json(FolderResponse {
        current_path: current_path.clone(),
        breadcrumbs: build_breadcrumbs(&current_path),
        folders,
        media,
        next_offset,
        total_media_count,
        limit,
        sort_field: sort_field.as_str(),
        sort_direction: sort_direction.as_str(),
    })
    .into_response();
    apply_no_cache_headers(response.headers_mut());
    response
}

async fn serve_media(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    Query(query): Query<MediaQuery>,
) -> impl IntoResponse {
    let sanitized = match sanitize_relative_path(&path) {
        Some(path) => path,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let relative_path = sanitized.to_string_lossy().replace('\\', "/");
    let item = {
        let index = state.index.read().await;
        match index.media_by_path.get(&relative_path).cloned() {
            Some(item) => item,
            None => return StatusCode::NOT_FOUND.into_response(),
        }
    };

    let full_path = state.media_root.join(&sanitized);
    match fs::read(&full_path).await {
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

async fn serve_thumbnail(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
) -> impl IntoResponse {
    let sanitized = match sanitize_relative_path(&path) {
        Some(path) => path,
        None => return StatusCode::BAD_REQUEST.into_response(),
    };

    let relative_path = sanitized.to_string_lossy().replace('\\', "/");
    let Some(item) = ({
        let index = state.index.read().await;
        index.media_by_path.get(&relative_path).cloned()
    }) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    if matches!(item.media_type, MediaType::Video) {
        return svg_thumbnail_response("VIDEO");
    }

    let cache_path = thumbnail_cache_path(&state.thumbnail_root, &item);
    if fs::metadata(&cache_path).await.is_err() {
        let full_path = state.media_root.join(&sanitized);
        match generate_thumbnail(full_path, cache_path.clone()).await {
            Ok(ThumbnailOutcome::Generated) => {}
            Ok(ThumbnailOutcome::UnsupportedSource) => {
                return svg_thumbnail_response("IMAGE");
            }
            Err(error) => {
                error!("failed to generate thumbnail for {}: {error:#}", item.relative_path);
                return svg_thumbnail_response("IMAGE");
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
            response
        }
        Err(_) => svg_thumbnail_response("IMAGE"),
    }
}

fn svg_thumbnail_response(label: &str) -> Response<Body> {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 480 360">
<defs>
  <linearGradient id="bg" x1="0" x2="1" y1="0" y2="1">
    <stop offset="0%" stop-color="#132130"/>
    <stop offset="100%" stop-color="#0a121b"/>
  </linearGradient>
</defs>
<rect width="480" height="360" fill="url(#bg)"/>
<rect x="32" y="32" width="416" height="296" rx="28" fill="rgba(255,255,255,0.04)" stroke="rgba(255,255,255,0.08)"/>
<text x="240" y="190" text-anchor="middle" fill="#f0c48c" font-family="Arial, sans-serif" font-size="40" letter-spacing="8">{label}</text>
</svg>"##
    );
    let mut response = Response::new(Body::from(svg));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/svg+xml"));
    apply_thumbnail_cache_headers(response.headers_mut());
    response
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

fn classify_media_path(path: &Path) -> Option<MediaClassification> {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let media_type = if mime.type_() == mime::IMAGE {
        Some(MediaType::Image)
    } else if mime.type_() == mime::VIDEO {
        Some(MediaType::Video)
    } else {
        None
    }?;

    Some(MediaClassification { media_type, mime })
}

fn parse_sort_field(value: Option<&str>) -> SortField {
    match value {
        Some("date") => SortField::Date,
        Some("size") => SortField::Size,
        _ => SortField::Name,
    }
}

fn parse_sort_direction(value: Option<&str>) -> SortDirection {
    match value {
        Some("desc") => SortDirection::Desc,
        _ => SortDirection::Asc,
    }
}

impl SortField {
    fn as_str(self) -> &'static str {
        match self {
            SortField::Name => "name",
            SortField::Date => "date",
            SortField::Size => "size",
        }
    }
}

impl SortDirection {
    fn as_str(self) -> &'static str {
        match self {
            SortDirection::Asc => "asc",
            SortDirection::Desc => "desc",
        }
    }
}

fn compare_text(left: &str, right: &str, direction: SortDirection) -> std::cmp::Ordering {
    let ordering = left.to_lowercase().cmp(&right.to_lowercase());
    match direction {
        SortDirection::Asc => ordering,
        SortDirection::Desc => ordering.reverse(),
    }
}

fn compare_number(left: u64, right: u64, direction: SortDirection) -> std::cmp::Ordering {
    match direction {
        SortDirection::Asc => left.cmp(&right),
        SortDirection::Desc => right.cmp(&left),
    }
}

fn sort_media_items(items: &mut [MediaItem], field: SortField, direction: SortDirection) {
    items.sort_by(|left, right| match field {
        SortField::Date => compare_number(left.modified_ms, right.modified_ms, direction)
            .then_with(|| compare_text(&left.name, &right.name, SortDirection::Asc)),
        SortField::Size => compare_number(left.size_bytes, right.size_bytes, direction)
            .then_with(|| compare_text(&left.name, &right.name, SortDirection::Asc)),
        SortField::Name => compare_text(&left.name, &right.name, direction),
    });
}

fn sort_folder_entries(items: &mut [FolderEntry], field: SortField, direction: SortDirection) {
    items.sort_by(|left, right| match field {
        SortField::Date => compare_number(left.newest_modified_ms, right.newest_modified_ms, direction)
            .then_with(|| compare_text(&left.name, &right.name, SortDirection::Asc)),
        SortField::Size => compare_number(left.total_size_bytes, right.total_size_bytes, direction)
            .then_with(|| compare_text(&left.name, &right.name, SortDirection::Asc)),
        SortField::Name => compare_text(&left.name, &right.name, direction),
    });
}

fn build_breadcrumbs(path: &str) -> Vec<BreadcrumbItem> {
    let mut crumbs = vec![BreadcrumbItem {
        name: "Library".to_string(),
        href: "/".to_string(),
    }];

    let mut accumulated = String::new();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        if !accumulated.is_empty() {
            accumulated.push('/');
        }
        accumulated.push_str(segment);
        crumbs.push(BreadcrumbItem {
            name: segment.to_string(),
            href: format!("/{accumulated}"),
        });
    }

    crumbs
}

fn parent_folder_path(relative_path: &str) -> String {
    relative_path
        .rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

fn folder_name(path: &str) -> String {
    if path.is_empty() {
        "Library".to_string()
    } else {
        path.rsplit('/').next().unwrap_or(path).to_string()
    }
}

fn ensure_folder(index: &mut MediaIndex, path: &str) {
    index
        .folders
        .entry(path.to_string())
        .or_insert_with(|| FolderNode {
            path: path.to_string(),
            name: folder_name(path),
            ..FolderNode::default()
        });
}

fn ancestor_paths(path: &str) -> Vec<String> {
    if path.is_empty() {
        return vec![String::new()];
    }

    let mut ancestors = vec![String::new()];
    let mut current = String::new();
    for segment in path.split('/').filter(|segment| !segment.is_empty()) {
        if !current.is_empty() {
            current.push('/');
        }
        current.push_str(segment);
        ancestors.push(current.clone());
    }

    ancestors
}

fn build_media_index(items: Vec<MediaItem>) -> MediaIndex {
    let mut index = MediaIndex::default();
    ensure_folder(&mut index, "");

    for item in items {
        let relative_path = item.relative_path.clone();
        let parent_path = parent_folder_path(&relative_path);
        let segments = parent_path
            .split('/')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();

        ensure_folder(&mut index, &parent_path);

        let mut current = String::new();
        for segment in segments {
            let child = if current.is_empty() {
                segment.to_string()
            } else {
                format!("{current}/{segment}")
            };

            ensure_folder(&mut index, &current);
            ensure_folder(&mut index, &child);
            if let Some(parent) = index.folders.get_mut(&current) {
                parent.child_folders.insert(child.clone());
            }
            current = child;
        }

        if let Some(folder) = index.folders.get_mut(&parent_path) {
            folder.immediate_media.push(relative_path.clone());
        }

        for ancestor in ancestor_paths(&parent_path) {
            ensure_folder(&mut index, &ancestor);
            if let Some(folder) = index.folders.get_mut(&ancestor) {
                folder.item_count += 1;
                folder.total_size_bytes += item.size_bytes;
                if item.modified_ms >= folder.newest_modified_ms {
                    folder.newest_modified_ms = item.modified_ms;
                    folder.cover_relative_path = Some(relative_path.clone());
                }
            }
        }

        index.media_by_path.insert(relative_path, item);
    }

    index
}

async fn scan_media_index(root: &Path) -> Result<MediaIndex> {
    let canonical_root = fs::canonicalize(root)
        .await
        .with_context(|| format!("failed to access media root {}", root.display()))?;
    let mut items = Vec::new();
    collect_media_recursive(&canonical_root, &canonical_root, &mut items).await?;
    items.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(build_media_index(items))
}

async fn collect_media_recursive(root: &Path, current: &Path, items: &mut Vec<MediaItem>) -> Result<()> {
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

        let Some(media) = classify_media_path(&path) else {
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
            media_type: media.media_type,
            mime: media.mime.to_string(),
            size_bytes: metadata.len(),
            modified_ms: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis() as u64)
                .unwrap_or(0),
        });
    }

    Ok(())
}

async fn rebuild_media_index(state: &AppState) -> Result<()> {
    let next_index = scan_media_index(state.media_root.as_ref()).await?;
    let mut index = state.index.write().await;
    *index = next_index;
    Ok(())
}

fn start_media_watcher(state: AppState) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let media_root = state.media_root.clone();

    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        match result {
            Ok(_) => {
                let _ = tx.send(());
            }
            Err(error) => {
                error!("file watcher error: {error}");
            }
        }
    })?;
    watcher.watch(media_root.as_ref(), RecursiveMode::Recursive)?;

    tokio::spawn(async move {
        let _watcher = watcher;
        while rx.recv().await.is_some() {
            tokio::time::sleep(Duration::from_millis(300)).await;
            while rx.try_recv().is_ok() {}
            if let Err(error) = rebuild_media_index(&state).await {
                error!("failed to rebuild media index: {error:#}");
            } else {
                info!("media index refreshed");
            }
        }
    });

    Ok(())
}

fn thumbnail_cache_path(thumbnail_root: &Path, item: &MediaItem) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    item.relative_path.hash(&mut hasher);
    item.modified_ms.hash(&mut hasher);
    thumbnail_root.join(format!("{:x}.jpg", hasher.finish()))
}

async fn generate_thumbnail(source_path: PathBuf, cache_path: PathBuf) -> Result<ThumbnailOutcome> {
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
        let thumbnail = image.thumbnail(THUMBNAIL_SIZE, THUMBNAIL_SIZE);
        thumbnail.save_with_format(&cache_path, ImageFormat::Jpeg)?;
        Ok(ThumbnailOutcome::Generated)
    })
    .await
    .context("thumbnail generation task failed")?
}
