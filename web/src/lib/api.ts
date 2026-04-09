import type { FolderResponse, SortDirection, SortField } from '$lib/types';

function encodeRelativePath(relativePath: string): string {
  return relativePath
    .split('/')
    .map((segment) => encodeURIComponent(segment))
    .join('/');
}

export function mediaUrl(relativePath: string, apiBaseUrl: string): string {
  const encodedPath = encodeRelativePath(relativePath);

  return `${apiBaseUrl}/media/${encodedPath}`;
}

export function thumbnailUrl(relativePath: string, apiBaseUrl: string, modifiedMs: number): string {
  const encodedPath = encodeRelativePath(relativePath);
  return `${apiBaseUrl}/thumbs/${encodedPath}?v=${modifiedMs}`;
}

export function downloadUrl(relativePath: string, apiBaseUrl: string): string {
  const encodedPath = encodeRelativePath(relativePath);

  return `${apiBaseUrl}/media/${encodedPath}?download=true`;
}

export function folderApiPath(
  currentPath: string,
  sortField: SortField,
  sortDirection: SortDirection,
  offset = 0,
  limit = 60
): string {
  const params = new URLSearchParams();
  params.set('sort', sortField);
  params.set('dir', sortDirection);
  params.set('offset', String(offset));
  params.set('limit', String(limit));

  if (!currentPath) {
    return `/api/folder?${params.toString()}`;
  }

  return `/api/folder/${encodeRelativePath(currentPath)}?${params.toString()}`;
}

export async function fetchFolderResponse(
  fetchImpl: typeof fetch,
  apiBaseUrl: string,
  currentPath: string,
  sortField: SortField,
  sortDirection: SortDirection,
  offset = 0,
  limit = 60
): Promise<FolderResponse> {
  const response = await fetchImpl(
    `${apiBaseUrl}${folderApiPath(currentPath, sortField, sortDirection, offset, limit)}`
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch folder: ${response.status}`);
  }

  return (await response.json()) as FolderResponse;
}
