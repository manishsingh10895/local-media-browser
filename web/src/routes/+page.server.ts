import { env } from '$env/dynamic/private';
import { error } from '@sveltejs/kit';

import type { MediaItem } from '$lib/types';

const DEFAULT_INTERNAL_API_BASE_URL = 'http://localhost:6677';

export async function load({ fetch }) {
  const apiBaseUrl = env.INTERNAL_API_BASE_URL || DEFAULT_INTERNAL_API_BASE_URL;
  const response = await fetch(`${apiBaseUrl}/api/media`);

  if (!response.ok) {
    throw error(response.status, 'Failed to fetch media');
  }

  return {
    media: (await response.json()) as MediaItem[]
  };
}
