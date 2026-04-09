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
export type GridSize = 'compact' | 'comfortable' | 'large';

export interface FolderResponse {
  current_path: string;
  breadcrumbs: BreadcrumbItem[];
  folders: FolderEntry[];
  media: MediaItem[];
  next_offset: number | null;
  total_media_count: number;
  limit: number;
  sort_field: SortField;
  sort_direction: SortDirection;
  grid_size?: GridSize;
}
