import type { HTMLAttributes } from 'svelte/elements';

declare global {
  namespace svelteHTML {
    interface IntrinsicElements {
      'media-player': HTMLAttributes<HTMLElement> & {
        class?: string;
        src?: string;
        title?: string;
        autoplay?: boolean;
        playsinline?: boolean;
        crossorigin?: boolean | string;
        'view-type'?: string;
        'stream-type'?: string;
      };
      'media-provider': HTMLAttributes<HTMLElement>;
      'media-video-layout': HTMLAttributes<HTMLElement>;
    }
  }
}

export {};
