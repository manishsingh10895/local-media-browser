export type MediaType = 'image' | 'video';

export interface MediaItem {
  name: string;
  relative_path: string;
  media_type: MediaType;
  mime: string;
  size_bytes: number;
  modified_ms: number;
}

export interface FolderEntry {
  name: string;
  relative_path: string;
  item_count: number;
  total_size_bytes: number;
  newest_modified_ms: number;
  cover?: MediaItem;
}

export interface BreadcrumbItem {
  name: string;
  href: string;
}

export type SortField = 'name' | 'date' | 'size';
export type SortDirection = 'asc' | 'desc';
