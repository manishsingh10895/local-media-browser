import { env } from '$env/dynamic/private';
import { error } from '@sveltejs/kit';

import { fetchFolderResponse } from '$lib/api';
import type { FolderResponse, GridSize, SortDirection, SortField } from '$lib/types';

const DEFAULT_INTERNAL_API_BASE_URL = 'http://localhost:6677';
const DEFAULT_PUBLIC_API_PORT = '6677';
const DEFAULT_SORT_FIELD: SortField = 'name';
const DEFAULT_SORT_DIRECTION: SortDirection = 'asc';
const DEFAULT_GRID_SIZE: GridSize = 'comfortable';

function normalizePath(path: string | undefined): string {
  return path?.split('/').filter(Boolean).join('/') ?? '';
}

function parseSortField(value: string | null): SortField {
  return value === 'date' || value === 'size' || value === 'name' ? value : DEFAULT_SORT_FIELD;
}

function parseSortDirection(value: string | null): SortDirection {
  return value === 'desc' || value === 'asc' ? value : DEFAULT_SORT_DIRECTION;
}

function parseGridSize(value: string | null): GridSize {
  return value === 'compact' || value === 'large' || value === 'comfortable'
    ? value
    : DEFAULT_GRID_SIZE;
}

export async function load({ fetch, url, params }) {
  const apiBaseUrl = env.INTERNAL_API_BASE_URL || DEFAULT_INTERNAL_API_BASE_URL;
  const publicApiBaseUrl =
    env.PUBLIC_API_BASE_URL || `${url.protocol}//${url.hostname}:${DEFAULT_PUBLIC_API_PORT}`;
  const currentPath = normalizePath(params.path);
  const sortField = parseSortField(url.searchParams.get('sort'));
  const sortDirection = parseSortDirection(url.searchParams.get('dir'));
  const gridSize = parseGridSize(url.searchParams.get('view'));
  let folder: FolderResponse;

  try {
    folder = await fetchFolderResponse(
      fetch,
      apiBaseUrl,
      currentPath,
      sortField,
      sortDirection,
      0,
      60
    );
  } catch {
    throw error(404, 'Folder not found');
  }

  return {
    folder: {
      ...folder,
      grid_size: gridSize
    },
    publicApiBaseUrl,
    apiBaseUrl: publicApiBaseUrl
  };
}
