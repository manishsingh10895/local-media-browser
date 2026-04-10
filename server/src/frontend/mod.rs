mod style;

use axum::{
    Router,
    body::Body,
    extract::{Path as AxumPath, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use maud::{DOCTYPE, Markup, html};

use crate::{
    config::{FrontendMode, GridSize, parse_grid_size, parse_sort_direction},
    http_utils::apply_no_cache_headers,
    models::{FolderQuery, FolderResponse, HealthResponse},
    paths::{media_download_route, media_route, sanitize_relative_path, sort_direction_label, thumbnail_route, url_for_folder},
    responses::build_folder_response,
    state::AppState,
};

pub fn mount_routes(router: Router<AppState>) -> Router<AppState> {
    router
        .route("/", get(render_root))
        .route("/{*path}", get(render_path))
}

async fn render_root(
    State(state): State<AppState>,
    Query(query): Query<FolderQuery>,
) -> impl IntoResponse {
    render_folder_page(state, String::new(), query).await
}

async fn render_path(
    State(state): State<AppState>,
    AxumPath(path): AxumPath<String>,
    Query(query): Query<FolderQuery>,
) -> impl IntoResponse {
    let Some(sanitized) = sanitize_relative_path(&path) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    render_folder_page(state, sanitized.to_string_lossy().replace('\\', "/"), query).await
}

async fn render_folder_page(state: AppState, current_path: String, query: FolderQuery) -> Response<Body> {
    let grid_size = parse_grid_size(query.view.as_deref());
    let offset = query.offset.unwrap_or(0);
    if !state.initial_index_ready().await {
        return render_indexing_page(state, grid_size).await;
    }

    match build_folder_response(state.clone(), current_path, query).await {
        Ok(folder) => {
            let markup = render_axum_frontend(&folder, state.frontend_mode, grid_size, offset, state.media_root.display().to_string());
            let mut response = markup.into_response();
            apply_no_cache_headers(response.headers_mut());
            response
        }
        Err(status) => status.into_response(),
    }
}

async fn render_indexing_page(state: AppState, grid_size: GridSize) -> Response<Body> {
    let status = state.current_status().await;
    let markup = render_axum_indexing(&status, state.frontend_mode, state.media_root.display().to_string(), grid_size);
    let mut response = (StatusCode::SERVICE_UNAVAILABLE, markup).into_response();
    apply_no_cache_headers(response.headers_mut());
    response
}

fn render_axum_frontend(
    folder: &FolderResponse,
    frontend_mode: FrontendMode,
    grid_size: GridSize,
    offset: usize,
    media_root_label: String,
) -> Markup {
    let title = if folder.current_path.is_empty() {
        "Local Gallery".to_string()
    } else {
        format!("{} · Local Gallery", folder.current_path)
    };

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                style { (style::AXUM_FRONTEND_CSS) }
            }
            body {
                main class="shell" {
                    (render_header(folder, frontend_mode, media_root_label))
                    (render_controls(folder, grid_size))
                    (render_folders(folder, grid_size))
                    (render_media(folder, grid_size, offset))
                }
            }
        }
    }
}

fn render_axum_indexing(
    status: &HealthResponse,
    frontend_mode: FrontendMode,
    media_root_label: String,
    grid_size: GridSize,
) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Indexing · Local Gallery" }
                meta http-equiv="refresh" content="3";
                style { (style::AXUM_FRONTEND_CSS) }
            }
            body {
                main class="shell" {
                    header class="hero" {
                        div class="hero-copy" {
                            h1 { "Indexing Media Library" }
                            p class="lede" { "The server is running and scanning your media folder. This page refreshes automatically." }
                        }
                        div class="hero-panel" {
                            span class="panel-label" { "Frontend" }
                            strong { (frontend_mode.as_str()) }
                            p { "Media root: " (media_root_label) }
                        }
                    }
                    section class="controls" {
                        p class="empty-note" { "Phase: " (match status.phase { crate::models::IndexPhase::Starting => "starting", crate::models::IndexPhase::Indexing => "indexing", crate::models::IndexPhase::Ready => "ready", crate::models::IndexPhase::Error => "error" }) }
                        p class="empty-note" { "Indexed items so far: " (status.indexed_media_count) }
                        @if let Some(error) = &status.last_error {
                            p class="empty-note" { "Last error: " (error) }
                        }
                        p class="empty-note" { "Preferred view after indexing: " (grid_size.as_str()) }
                    }
                }
            }
        }
    }
}

fn render_header(folder: &FolderResponse, mode: FrontendMode, media_root_label: String) -> Markup {
    html! {
        header class="hero" {
            div class="hero-copy" {
                nav class="breadcrumbs" {
                    @for (index, crumb) in folder.breadcrumbs.iter().enumerate() {
                        @if index > 0 { span class="crumb-separator" { "/" } }
                        a href=(crumb.href) { (&crumb.name) }
                    }
                }
                h1 { (folder.current_path.rsplit('/').next().filter(|_| !folder.current_path.is_empty()).unwrap_or("Local Media Browser")) }
                p class="lede" { "Axum-rendered frontend over the same indexed media backend." }
            }
            div class="hero-panel" {
                span class="panel-label" { "Frontend" }
                strong { (mode.as_str()) }
                p { "Media root: " (media_root_label) }
            }
        }
    }
}

fn render_controls(folder: &FolderResponse, grid_size: GridSize) -> Markup {
    let current_dir = parse_sort_direction(Some(folder.sort_direction));
    html! {
        section class="controls" {
            form method="get" class="controls-form" {
                input type="hidden" name="offset" value="0";
                label { span { "Sort" } select name="sort" {
                    option value="name" selected[folder.sort_field == "name"] { "Name" }
                    option value="date" selected[folder.sort_field == "date"] { "Date" }
                    option value="size" selected[folder.sort_field == "size"] { "Size" }
                }}
                label { span { "Direction" } select name="dir" {
                    option value="asc" selected[folder.sort_direction == "asc"] { "Ascending" }
                    option value="desc" selected[folder.sort_direction == "desc"] { "Descending" }
                }}
                label { span { "View" } select name="view" {
                    option value="compact" selected[matches!(grid_size, GridSize::Compact)] { "Compact" }
                    option value="comfortable" selected[matches!(grid_size, GridSize::Comfortable)] { "Comfortable" }
                    option value="large" selected[matches!(grid_size, GridSize::Large)] { "Large" }
                }}
                button type="submit" { "Apply" }
                a class="refresh-link" href=(url_for_folder(&folder.current_path, folder.sort_field, current_dir.as_str(), grid_size, 0)) { "Refresh" }
            }
        }
    }
}

fn render_folders(folder: &FolderResponse, grid_size: GridSize) -> Markup {
    html! {
        section class="folder-section" {
            h2 { "Folders" }
            @if folder.folders.is_empty() {
                p class="empty-note" { "No child folders in this location." }
            } @else {
                div class="folder-grid" {
                    @for child in &folder.folders {
                        a class="folder-card" href=(url_for_folder(&child.relative_path, folder.sort_field, folder.sort_direction, grid_size, 0)) {
                            div class="folder-cover" {
                                @if let Some(cover) = &child.cover { img src=(thumbnail_route(cover)) alt=(child.name); }
                                @else { div class="folder-placeholder" { "Folder" } }
                            }
                            div class="folder-meta" { h3 { (&child.name) } p { (child.item_count) " media items" } }
                        }
                    }
                }
            }
        }
    }
}

fn render_media(folder: &FolderResponse, grid_size: GridSize, offset: usize) -> Markup {
    let prev_offset = offset.saturating_sub(folder.limit);
    let next_offset = folder.next_offset.unwrap_or(offset);
    html! {
        section class="media-section" {
            h2 { "Media" } p class="sort-note" { (sort_direction_label(parse_sort_direction(Some(folder.sort_direction)))) }
            @if folder.media.is_empty() {
                p class="empty-note" { "No media in this folder." }
            } @else {
                div class="media-grid" style=(format!("grid-template-columns: repeat(auto-fill, minmax({}, 1fr));", grid_size.min_width())) {
                    @for item in &folder.media {
                        article class="media-card" {
                            a class="media-thumb" href=(media_route(&item.relative_path)) target="_blank" rel="noreferrer" {
                                img src=(thumbnail_route(item)) alt=(item.name);
                            }
                            div class="media-meta" {
                                h3 { (&item.name) }
                                div class="media-actions" {
                                    a href=(media_route(&item.relative_path)) target="_blank" rel="noreferrer" { "Open" }
                                    a href=(media_download_route(&item.relative_path)) { "Download" }
                                }
                            }
                        }
                    }
                }
            }
            div class="pager" {
                @if offset > 0 { a href=(url_for_folder(&folder.current_path, folder.sort_field, folder.sort_direction, grid_size, prev_offset)) { "Previous" } }
                @if folder.next_offset.is_some() { a href=(url_for_folder(&folder.current_path, folder.sort_field, folder.sort_direction, grid_size, next_offset)) { "Next" } }
            }
        }
    }
}
