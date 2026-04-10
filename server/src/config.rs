use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
    net::SocketAddr,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

pub const DEFAULT_PAGE_LIMIT: usize = 60;
pub const MAX_PAGE_LIMIT: usize = 200;
pub const THUMBNAIL_SIZE: u32 = 480;

pub struct RuntimeConfig {
    pub bind_addr: SocketAddr,
    pub media_root: PathBuf,
    pub thumbnail_root: PathBuf,
    pub cors_allow_origin: String,
    pub frontend_mode: FrontendMode,
    pub cli_media_root: Option<String>,
    pub env_media_root: Option<String>,
}

#[derive(Clone, Copy)]
pub enum FrontendMode {
    Svelte,
    Axum,
}

#[derive(Clone, Copy)]
pub enum SortField {
    Name,
    Date,
    Size,
}

#[derive(Clone, Copy)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Copy)]
pub enum GridSize {
    Compact,
    Comfortable,
    Large,
}

impl RuntimeConfig {
    pub fn from_env() -> Result<Self> {
        let cli_media_root = env::args().nth(1);
        let env_media_root = env::var("MEDIA_ROOT").ok();
        let media_root = cli_media_root
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(read_env("MEDIA_ROOT", "../media")));

        Ok(Self {
            bind_addr: read_env("BIND_ADDR", "0.0.0.0:6677")
                .parse()
                .context("invalid BIND_ADDR")?,
            thumbnail_root: build_thumbnail_root(&media_root),
            cors_allow_origin: read_env("CORS_ALLOW_ORIGIN", "*"),
            frontend_mode: parse_frontend_mode(env::var("FRONTEND_MODE").ok().as_deref()),
            media_root,
            cli_media_root,
            env_media_root,
        })
    }
}

impl FrontendMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Svelte => "svelte",
            Self::Axum => "axum",
        }
    }
}

impl SortField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Date => "date",
            Self::Size => "size",
        }
    }
}

impl SortDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

impl GridSize {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Comfortable => "comfortable",
            Self::Large => "large",
        }
    }

    pub fn min_width(self) -> &'static str {
        match self {
            Self::Compact => "160px",
            Self::Comfortable => "220px",
            Self::Large => "300px",
        }
    }
}

pub fn parse_sort_field(value: Option<&str>) -> SortField {
    match value {
        Some("date") => SortField::Date,
        Some("size") => SortField::Size,
        _ => SortField::Name,
    }
}

pub fn parse_sort_direction(value: Option<&str>) -> SortDirection {
    match value {
        Some("desc") => SortDirection::Desc,
        _ => SortDirection::Asc,
    }
}

pub fn parse_grid_size(value: Option<&str>) -> GridSize {
    match value {
        Some("compact") => GridSize::Compact,
        Some("large") => GridSize::Large,
        _ => GridSize::Comfortable,
    }
}

pub fn parse_frontend_mode(value: Option<&str>) -> FrontendMode {
    match value {
        Some("axum") => FrontendMode::Axum,
        _ => FrontendMode::Svelte,
    }
}

fn read_env(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn build_thumbnail_root(media_root: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    media_root.to_string_lossy().hash(&mut hasher);
    std::env::temp_dir()
        .join("local-gallery-thumbnails")
        .join(format!("{:x}", hasher.finish()))
}
