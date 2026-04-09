use std::path::{Component, Path, PathBuf};

use mime_guess::mime;

use crate::{
    config::{GridSize, SortDirection},
    models::{BreadcrumbItem, MediaClassification, MediaItem, MediaType},
};

pub fn sanitize_relative_path(path: &str) -> Option<PathBuf> {
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

pub fn classify_media_path(path: &Path) -> Option<MediaClassification> {
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

pub fn build_breadcrumbs(path: &str) -> Vec<BreadcrumbItem> {
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

pub fn parent_folder_path(relative_path: &str) -> String {
    relative_path
        .rsplit_once('/')
        .map(|(parent, _)| parent.to_string())
        .unwrap_or_default()
}

pub fn folder_name(path: &str) -> String {
    if path.is_empty() {
        "Library".to_string()
    } else {
        path.rsplit('/').next().unwrap_or(path).to_string()
    }
}

pub fn ancestor_paths(path: &str) -> Vec<String> {
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

pub fn url_for_folder(path: &str, sort: &str, dir: &str, view: GridSize, offset: usize) -> String {
    let mut base = if path.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", path)
    };
    let query = format!("sort={sort}&dir={dir}&view={}&offset={offset}", view.as_str());
    base.push(if base.contains('?') { '&' } else { '?' });
    base.push_str(&query);
    base
}

pub fn media_route(path: &str) -> String {
    format!("/media/{}", encode_path_segments(path))
}

pub fn media_download_route(path: &str) -> String {
    format!("/media/{}?download=true", encode_path_segments(path))
}

pub fn thumbnail_route(item: &MediaItem) -> String {
    format!(
        "/thumbs/{}?v={}",
        encode_path_segments(&item.relative_path),
        item.modified_ms
    )
}

pub fn sort_direction_label(direction: SortDirection) -> &'static str {
    match direction {
        SortDirection::Asc => "Ascending",
        SortDirection::Desc => "Descending",
    }
}

fn encode_path_segments(path: &str) -> String {
    path.split('/').map(percent_encode).collect::<Vec<_>>().join("/")
}

fn percent_encode(segment: &str) -> String {
    let mut output = String::new();
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                output.push(byte as char)
            }
            _ => output.push_str(&format!("%{:02X}", byte)),
        }
    }
    output
}
