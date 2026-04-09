use std::collections::{BTreeMap, BTreeSet};

use mime_guess::mime;
use serde::{Deserialize, Serialize};

#[derive(Default, Clone)]
pub struct MediaIndex {
    pub media_by_path: BTreeMap<String, MediaItem>,
    pub folders: BTreeMap<String, FolderNode>,
}

#[derive(Clone, Default)]
pub struct FolderNode {
    pub path: String,
    pub name: String,
    pub child_folders: BTreeSet<String>,
    pub immediate_media: Vec<String>,
    pub item_count: u64,
    pub total_size_bytes: u64,
    pub newest_modified_ms: u64,
    pub cover_relative_path: Option<String>,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaItem {
    pub name: String,
    pub relative_path: String,
    pub media_type: MediaType,
    pub mime: String,
    pub size_bytes: u64,
    pub modified_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
}

#[derive(Clone)]
pub struct MediaClassification {
    pub media_type: MediaType,
    pub mime: mime::Mime,
}

pub enum ThumbnailOutcome {
    Generated,
    UnsupportedSource,
}

#[derive(Serialize)]
pub struct FolderEntry {
    pub name: String,
    pub relative_path: String,
    pub item_count: u64,
    pub total_size_bytes: u64,
    pub newest_modified_ms: u64,
    pub cover: Option<MediaItem>,
}

#[derive(Serialize)]
pub struct BreadcrumbItem {
    pub name: String,
    pub href: String,
}

#[derive(Serialize)]
pub struct FolderResponse {
    pub current_path: String,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub folders: Vec<FolderEntry>,
    pub media: Vec<MediaItem>,
    pub next_offset: Option<usize>,
    pub total_media_count: usize,
    pub limit: usize,
    pub sort_field: &'static str,
    pub sort_direction: &'static str,
}

#[derive(Deserialize, Default)]
pub struct MediaQuery {
    pub download: Option<bool>,
}

#[derive(Deserialize, Default, Clone)]
pub struct FolderQuery {
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub view: Option<String>,
}
