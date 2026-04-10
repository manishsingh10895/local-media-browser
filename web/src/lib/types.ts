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
export type IndexPhase = 'starting' | 'indexing' | 'ready' | 'error';
export type IndexJob = 'initial' | 'refresh';

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

export interface HealthResponse {
  status: 'indexing' | 'ready' | 'error';
  phase: IndexPhase;
  is_ready: boolean;
  is_indexing: boolean;
  current_job: IndexJob | null;
  indexed_media_count: number;
  last_completed_ms: number | null;
  last_error: string | null;
  started_at_ms: number | null;
}

export type FolderFetchResult =
  | {
      kind: 'ready';
      folder: FolderResponse;
    }
  | {
      kind: 'indexing';
      status: HealthResponse;
    };
