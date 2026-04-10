use std::{
    path::Path,
    time::{Duration, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use notify::{RecursiveMode, Watcher};
use tokio::fs;
use tracing::{error, info};

use crate::{
    models::{FolderNode, MediaIndex, MediaItem},
    paths::{ancestor_paths, classify_media_path, folder_name, parent_folder_path},
    state::AppState,
};

pub async fn scan_media_index(root: &Path) -> Result<MediaIndex> {
    let canonical_root = fs::canonicalize(root)
        .await
        .with_context(|| format!("failed to access media root {}", root.display()))?;
    let mut items = Vec::new();
    collect_media_recursive(&canonical_root, &canonical_root, &mut items).await?;
    items.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(build_media_index(items))
}

pub async fn rebuild_media_index(state: &AppState) -> Result<()> {
    let next_index = scan_media_index(state.media_root.as_ref()).await?;
    let mut index = state.index.write().await;
    *index = next_index;
    Ok(())
}

pub fn start_media_watcher(state: AppState) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    let media_root = state.media_root.clone();

    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        match result {
            Ok(_) => {
                let _ = tx.send(());
            }
            Err(error) => error!("file watcher error: {error}"),
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

pub fn build_media_index(items: Vec<MediaItem>) -> MediaIndex {
    let mut index = MediaIndex::default();
    ensure_folder(&mut index, "");

    for item in items {
        let relative_path = item.relative_path.clone();
        let parent_path = parent_folder_path(&relative_path);
        let segments = parent_path.split('/').filter(|segment| !segment.is_empty());

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

fn ensure_folder(index: &mut MediaIndex, path: &str) {
    index.folders.entry(path.to_string()).or_insert_with(|| FolderNode {
        path: path.to_string(),
        name: folder_name(path),
        ..FolderNode::default()
    });
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
        items.push(build_media_item(root, &path, entry.file_name(), metadata.len(), metadata.modified().ok(), media)?);
    }
    Ok(())
}

fn build_media_item(
    root: &Path,
    path: &Path,
    file_name: std::ffi::OsString,
    size_bytes: u64,
    modified: Option<std::time::SystemTime>,
    media: crate::models::MediaClassification,
) -> Result<MediaItem> {
    let relative_path = path
        .strip_prefix(root)
        .context("failed to strip media root from file path")?
        .to_string_lossy()
        .replace('\\', "/");

    Ok(MediaItem {
        name: file_name.to_string_lossy().to_string(),
        relative_path,
        media_type: media.media_type,
        mime: media.mime.to_string(),
        size_bytes,
        modified_ms: modified
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0),
    })
}
