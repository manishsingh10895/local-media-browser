import type { MediaItem } from '$lib/types';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:6677';

export function mediaUrl(relativePath: string): string {
  const encodedPath = relativePath
    .split('/')
    .map((segment) => encodeURIComponent(segment))
    .join('/');

  return `${API_BASE_URL}/media/${encodedPath}`;
}

export async function fetchMedia(fetchImpl: typeof fetch): Promise<MediaItem[]> {
  const response = await fetchImpl(`${API_BASE_URL}/api/media`);

  if (!response.ok) {
    throw new Error(`Failed to fetch media: ${response.status}`);
  }

  return (await response.json()) as MediaItem[];
}
