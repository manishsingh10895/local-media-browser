import { env } from '$env/dynamic/private';
import { error } from '@sveltejs/kit';

import type {
  BreadcrumbItem,
  FolderEntry,
  MediaItem,
  SortDirection,
  SortField
} from '$lib/types';

const DEFAULT_INTERNAL_API_BASE_URL = 'http://localhost:6677';
const DEFAULT_PUBLIC_API_PORT = '6677';
const DEFAULT_SORT_FIELD: SortField = 'name';
const DEFAULT_SORT_DIRECTION: SortDirection = 'asc';

function normalizePath(path: string | undefined): string {
  return path?.split('/').filter(Boolean).join('/') ?? '';
}

function buildBreadcrumbs(path: string): BreadcrumbItem[] {
  const segments = path ? path.split('/') : [];

  return [
    { name: 'Library', href: '/' },
    ...segments.map((segment, index) => ({
      name: segment,
      href: `/${segments.slice(0, index + 1).join('/')}`
    }))
  ];
}

function parseSortField(value: string | null): SortField {
  return value === 'date' || value === 'size' || value === 'name' ? value : DEFAULT_SORT_FIELD;
}

function parseSortDirection(value: string | null): SortDirection {
  return value === 'desc' || value === 'asc' ? value : DEFAULT_SORT_DIRECTION;
}

function compareText(left: string, right: string, direction: SortDirection): number {
  const result = left.localeCompare(right, undefined, { sensitivity: 'base' });
  return direction === 'asc' ? result : -result;
}

function compareNumber(left: number, right: number, direction: SortDirection): number {
  const result = left - right;
  return direction === 'asc' ? result : -result;
}

function sortMedia(items: MediaItem[], field: SortField, direction: SortDirection): MediaItem[] {
  return [...items].sort((left, right) => {
    if (field === 'date') {
      return (
        compareNumber(left.modified_ms, right.modified_ms, direction) ||
        compareText(left.name, right.name, 'asc')
      );
    }

    if (field === 'size') {
      return (
        compareNumber(left.size_bytes, right.size_bytes, direction) ||
        compareText(left.name, right.name, 'asc')
      );
    }

    return compareText(left.name, right.name, direction);
  });
}

function sortFolders(items: FolderEntry[], field: SortField, direction: SortDirection): FolderEntry[] {
  return [...items].sort((left, right) => {
    if (field === 'date') {
      return (
        compareNumber(left.newest_modified_ms, right.newest_modified_ms, direction) ||
        compareText(left.name, right.name, 'asc')
      );
    }

    if (field === 'size') {
      return (
        compareNumber(left.total_size_bytes, right.total_size_bytes, direction) ||
        compareText(left.name, right.name, 'asc')
      );
    }

    return compareText(left.name, right.name, direction);
  });
}

function buildFolderEntries(allMedia: MediaItem[], currentPath: string): FolderEntry[] {
  const folderMap = new Map<string, FolderEntry>();
  const prefix = currentPath ? `${currentPath}/` : '';

  for (const item of allMedia) {
    if (currentPath && !item.relative_path.startsWith(prefix)) {
      continue;
    }

    const remainder = currentPath ? item.relative_path.slice(prefix.length) : item.relative_path;
    const [nextSegment, ...rest] = remainder.split('/');

    if (!nextSegment || rest.length === 0) {
      continue;
    }

    const relativePath = currentPath ? `${currentPath}/${nextSegment}` : nextSegment;
    const existing = folderMap.get(relativePath);

    if (existing) {
      existing.item_count += 1;
      existing.total_size_bytes += item.size_bytes;
      if (item.modified_ms > existing.newest_modified_ms) {
        existing.newest_modified_ms = item.modified_ms;
        existing.cover = item;
      }
    } else {
      folderMap.set(relativePath, {
        name: nextSegment,
        relative_path: relativePath,
        item_count: 1,
        total_size_bytes: item.size_bytes,
        newest_modified_ms: item.modified_ms,
        cover: item
      });
    }
  }

  return [...folderMap.values()];
}

function buildVisibleMedia(allMedia: MediaItem[], currentPath: string): MediaItem[] {
  const prefix = currentPath ? `${currentPath}/` : '';

  return allMedia
    .filter((item) => {
      if (currentPath && !item.relative_path.startsWith(prefix)) {
        return false;
      }

      const remainder = currentPath ? item.relative_path.slice(prefix.length) : item.relative_path;
      return !remainder.includes('/');
    });
}

export async function load({ fetch, url, params }) {
  const apiBaseUrl = env.INTERNAL_API_BASE_URL || DEFAULT_INTERNAL_API_BASE_URL;
  const response = await fetch(`${apiBaseUrl}/api/media`);

  if (!response.ok) {
    throw error(response.status, 'Failed to fetch media');
  }

  const publicApiBaseUrl =
    env.PUBLIC_API_BASE_URL || `${url.protocol}//${url.hostname}:${DEFAULT_PUBLIC_API_PORT}`;
  const currentPath = normalizePath(params.path);
  const sortField = parseSortField(url.searchParams.get('sort'));
  const sortDirection = parseSortDirection(url.searchParams.get('dir'));
  const allMedia = (await response.json()) as MediaItem[];
  const folders = sortFolders(buildFolderEntries(allMedia, currentPath), sortField, sortDirection);
  const media = sortMedia(buildVisibleMedia(allMedia, currentPath), sortField, sortDirection);

  if (currentPath && folders.length === 0 && media.length === 0) {
    throw error(404, 'Folder not found');
  }

  return {
    currentPath,
    folders,
    media,
    breadcrumbs: buildBreadcrumbs(currentPath),
    publicApiBaseUrl,
    sortField,
    sortDirection
  };
}
