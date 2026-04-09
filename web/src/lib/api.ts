import type { MediaItem } from '$lib/types';

export function mediaUrl(relativePath: string, apiBaseUrl: string): string {
  const encodedPath = relativePath
    .split('/')
    .map((segment) => encodeURIComponent(segment))
    .join('/');

  return `${apiBaseUrl}/media/${encodedPath}`;
}

export async function fetchMedia(fetchImpl: typeof fetch): Promise<MediaItem[]> {
  const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:6677';
  const response = await fetchImpl(`${apiBaseUrl}/api/media`);

  if (!response.ok) {
    throw new Error(`Failed to fetch media: ${response.status}`);
  }

  return (await response.json()) as MediaItem[];
}
