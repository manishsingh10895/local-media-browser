use axum::http::StatusCode;

use crate::{
    config::{DEFAULT_PAGE_LIMIT, MAX_PAGE_LIMIT, parse_sort_direction, parse_sort_field},
    models::{FolderEntry, FolderQuery, FolderResponse},
    paths::build_breadcrumbs,
    sorting::{sort_folder_entries, sort_media_items},
    state::AppState,
};

pub async fn build_folder_response(
    state: AppState,
    current_path: String,
    query: FolderQuery,
) -> Result<FolderResponse, StatusCode> {
    let sort_field = parse_sort_field(query.sort.as_deref());
    let sort_direction = parse_sort_direction(query.dir.as_deref());
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(DEFAULT_PAGE_LIMIT).min(MAX_PAGE_LIMIT);

    let index = state.index.read().await;
    let Some(folder) = index.folders.get(&current_path) else {
        return Err(StatusCode::NOT_FOUND);
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
    let media = immediate_media.into_iter().skip(offset).take(limit).collect::<Vec<_>>();
    let next_offset = (offset + media.len() < total_media_count).then_some(offset + media.len());

    Ok(FolderResponse {
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
}
