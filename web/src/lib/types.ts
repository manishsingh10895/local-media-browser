export type MediaType = 'image' | 'video';

export interface MediaItem {
  name: string;
  relative_path: string;
  media_type: MediaType;
  mime: string;
  size_bytes: number;
}
