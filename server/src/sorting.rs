use std::cmp::Ordering;

use crate::{
    config::{SortDirection, SortField},
    models::{FolderEntry, MediaItem},
};

fn compare_text(left: &str, right: &str, direction: SortDirection) -> Ordering {
    let ordering = left.to_lowercase().cmp(&right.to_lowercase());
    match direction {
        SortDirection::Asc => ordering,
        SortDirection::Desc => ordering.reverse(),
    }
}

fn compare_number(left: u64, right: u64, direction: SortDirection) -> Ordering {
    match direction {
        SortDirection::Asc => left.cmp(&right),
        SortDirection::Desc => right.cmp(&left),
    }
}

pub fn sort_media_items(items: &mut [MediaItem], field: SortField, direction: SortDirection) {
    items.sort_by(|left, right| match field {
        SortField::Date => compare_number(left.modified_ms, right.modified_ms, direction)
            .then_with(|| compare_text(&left.name, &right.name, SortDirection::Asc)),
        SortField::Size => compare_number(left.size_bytes, right.size_bytes, direction)
            .then_with(|| compare_text(&left.name, &right.name, SortDirection::Asc)),
        SortField::Name => compare_text(&left.name, &right.name, direction),
    });
}

pub fn sort_folder_entries(items: &mut [FolderEntry], field: SortField, direction: SortDirection) {
    items.sort_by(|left, right| match field {
        SortField::Date => compare_number(left.newest_modified_ms, right.newest_modified_ms, direction)
            .then_with(|| compare_text(&left.name, &right.name, SortDirection::Asc)),
        SortField::Size => compare_number(left.total_size_bytes, right.total_size_bytes, direction)
            .then_with(|| compare_text(&left.name, &right.name, SortDirection::Asc)),
        SortField::Name => compare_text(&left.name, &right.name, direction),
    });
}
