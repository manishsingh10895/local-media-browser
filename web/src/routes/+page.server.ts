import { env } from '$env/dynamic/private';
import { error } from '@sveltejs/kit';

import type { MediaItem } from '$lib/types';

const DEFAULT_INTERNAL_API_BASE_URL = 'http://localhost:6677';
const DEFAULT_PUBLIC_API_PORT = '6677';

export async function load({ fetch, url }) {
  const apiBaseUrl = env.INTERNAL_API_BASE_URL || DEFAULT_INTERNAL_API_BASE_URL;
  const response = await fetch(`${apiBaseUrl}/api/media`);

  if (!response.ok) {
    throw error(response.status, 'Failed to fetch media');
  }

  const publicApiBaseUrl =
    env.PUBLIC_API_BASE_URL || `${url.protocol}//${url.hostname}:${DEFAULT_PUBLIC_API_PORT}`;

  return {
    media: (await response.json()) as MediaItem[],
    publicApiBaseUrl
  };
}
