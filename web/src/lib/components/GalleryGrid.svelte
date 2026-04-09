<script lang="ts">
  import { mediaUrl } from '$lib/api';
  import VideoPlayer from '$lib/components/VideoPlayer.svelte';
  import type { MediaItem } from '$lib/types';

  let { items }: { items: MediaItem[] } = $props();
  let selectedIndex = $state<number | null>(null);
  let touchStartX = 0;
  let touchStartY = 0;

  const swipeThreshold = 60;

  function parentPath(relativePath: string): string | null {
    const parts = relativePath.split('/');

    if (parts.length <= 1) {
      return null;
    }

    return parts.slice(0, -1).join('/');
  }

  function openViewer(index: number): void {
    selectedIndex = index;
  }

  function closeViewer(): void {
    selectedIndex = null;
  }

  function showPrevious(): void {
    if (selectedIndex === null || items.length === 0) {
      return;
    }

    selectedIndex = (selectedIndex - 1 + items.length) % items.length;
  }

  function showNext(): void {
    if (selectedIndex === null || items.length === 0) {
      return;
    }

    selectedIndex = (selectedIndex + 1) % items.length;
  }

  function handleWindowKeydown(event: KeyboardEvent): void {
    if (selectedIndex === null) {
      return;
    }

    if (event.key === 'Escape') {
      closeViewer();
    } else if (event.key === 'ArrowLeft') {
      event.preventDefault();
      showPrevious();
    } else if (event.key === 'ArrowRight') {
      event.preventDefault();
      showNext();
    }
  }

  function handleTouchStart(event: TouchEvent): void {
    const touch = event.changedTouches[0];
    touchStartX = touch.clientX;
    touchStartY = touch.clientY;
  }

  function handleTouchEnd(event: TouchEvent): void {
    const touch = event.changedTouches[0];
    const deltaX = touch.clientX - touchStartX;
    const deltaY = touch.clientY - touchStartY;

    if (Math.abs(deltaX) < swipeThreshold || Math.abs(deltaX) < Math.abs(deltaY)) {
      return;
    }

    if (deltaX > 0) {
      showPrevious();
    } else {
      showNext();
    }
  }
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if items.length === 0}
  <div class="empty-state">
    <p>No images or videos were found in the configured media folder.</p>
  </div>
{:else}
  <section class="grid">
    {#each items as item, index}
      <article class="card">
        <button
          class="preview-button"
          type="button"
          aria-label={`Open ${item.name}`}
          onclick={() => openViewer(index)}
        >
          <div class="preview">
            {#if item.media_type === 'image'}
              <img src={mediaUrl(item.relative_path)} alt={item.name} loading="lazy" />
            {:else}
              <video preload="metadata" muted playsinline>
                <source src={mediaUrl(item.relative_path)} type={item.mime} />
              </video>
            {/if}
          </div>
        </button>

        <div class="meta">
          <h2>{item.name}</h2>
          {#if parentPath(item.relative_path)}
            <p>{parentPath(item.relative_path)}</p>
          {/if}
        </div>
      </article>
    {/each}
  </section>

  {#if selectedIndex !== null}
    {@const selectedItem = items[selectedIndex]}

    <div
      class="viewer-backdrop"
      role="presentation"
      onclick={closeViewer}
    >
      <div
        class="viewer-shell"
        role="dialog"
        aria-modal="true"
        aria-label={selectedItem.name}
        tabindex="-1"
        onclick={(event) => event.stopPropagation()}
        onkeydown={(event) => event.stopPropagation()}
      >
        <div
          class="viewer-stage"
          role="presentation"
          ontouchstart={handleTouchStart}
          ontouchend={handleTouchEnd}
        >
          <button class="nav-button prev-button" type="button" aria-label="Previous media" onclick={showPrevious}>
            ‹
          </button>

          {#if selectedItem.media_type === 'image'}
            <img
              class="viewer-media"
              src={mediaUrl(selectedItem.relative_path)}
              alt={selectedItem.name}
            />
          {:else}
            <div class="viewer-media viewer-video">
              <VideoPlayer src={mediaUrl(selectedItem.relative_path)} title={selectedItem.name} />
            </div>
          {/if}

          <button class="nav-button next-button" type="button" aria-label="Next media" onclick={showNext}>
            ›
          </button>

          <button class="icon-button close-button" type="button" aria-label="Close viewer" onclick={closeViewer}>
            ×
          </button>
        </div>
      </div>
    </div>
  {/if}
{/if}

<style>
  .empty-state {
    padding: 2.6rem 1.4rem;
    border: 1px dashed rgba(255, 255, 255, 0.12);
    border-radius: 1.5rem;
    color: #d0d9e1;
    text-align: center;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.04), rgba(255, 255, 255, 0.02)),
      rgba(7, 15, 24, 0.6);
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1.4rem;
  }

  .card {
    position: relative;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.45rem;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.02)),
      linear-gradient(135deg, rgba(244, 179, 107, 0.06), transparent 28%),
      rgba(9, 18, 28, 0.86);
    box-shadow:
      0 22px 60px rgba(0, 0, 0, 0.26),
      inset 0 1px 0 rgba(255, 255, 255, 0.04);
    transition:
      transform 180ms ease,
      border-color 180ms ease,
      box-shadow 180ms ease;
  }

  .card::after {
    content: '';
    position: absolute;
    inset: auto 1.15rem 0.95rem;
    height: 1px;
    background: linear-gradient(90deg, rgba(255, 255, 255, 0.08), transparent);
    pointer-events: none;
  }

  .card:hover {
    transform: translateY(-4px);
    border-color: rgba(244, 179, 107, 0.2);
    box-shadow:
      0 28px 70px rgba(0, 0, 0, 0.3),
      0 0 0 1px rgba(244, 179, 107, 0.04);
  }

  .preview-button {
    display: block;
    width: 100%;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    cursor: pointer;
    text-align: left;
  }

  .preview {
    position: relative;
    aspect-ratio: 4 / 3;
    background:
      radial-gradient(circle at top, rgba(244, 179, 107, 0.09), transparent 28%),
      #081018;
    overflow: hidden;
  }

  .preview::after {
    content: '';
    position: absolute;
    inset: auto 0 0;
    height: 42%;
    background: linear-gradient(180deg, transparent, rgba(4, 9, 14, 0.3));
    pointer-events: none;
  }

  .preview-button:hover :global(img),
  .preview-button:hover :global(video) {
    transform: scale(1.06);
  }

  .preview-button:focus-visible {
    outline: 2px solid #f4b36b;
    outline-offset: -3px;
  }

  .preview :global(img),
  .preview :global(video) {
    width: 100%;
    height: 100%;
    display: block;
    object-fit: cover;
    transition: transform 180ms ease;
  }

  .meta {
    padding: 1rem 1.05rem 1.1rem;
  }

  h2 {
    margin: 0;
    font-size: 1rem;
    line-height: 1.35;
    color: #f7f4ef;
    letter-spacing: -0.01em;
  }

  p {
    margin: 0.45rem 0 0;
    font-size: 0.82rem;
    color: #89a0b2;
    letter-spacing: 0.01em;
    word-break: break-word;
  }

  @media (max-width: 640px) {
    .grid {
      grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
      gap: 1rem;
    }

    .card {
      border-radius: 1.15rem;
    }

    .meta {
      padding: 0.85rem 0.85rem 0.95rem;
    }
  }

  .viewer-backdrop {
    position: fixed;
    inset: 0;
    z-index: 20;
    display: block;
    background: rgba(3, 8, 14, 0.88);
  }

  .viewer-shell {
    width: 100vw;
    height: 100vh;
  }

  .viewer-stage {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background: #000;
  }

  .viewer-media {
    max-width: 100vw;
    max-height: 100vh;
    display: block;
    object-fit: contain;
  }

  .viewer-video {
    width: 100%;
    height: 100%;
  }

  .icon-button,
  .nav-button {
    border: 0;
    color: #f5f7fa;
    cursor: pointer;
    background: rgba(9, 16, 25, 0.7);
    backdrop-filter: blur(8px);
  }

  .icon-button {
    position: absolute;
    top: 1rem;
    right: 1rem;
    z-index: 2;
    width: 2.75rem;
    height: 2.75rem;
    border-radius: 999px;
    font-size: 1.8rem;
    line-height: 1;
  }

  .nav-button {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    width: 3.25rem;
    height: 3.25rem;
    border-radius: 999px;
    font-size: 2.1rem;
    line-height: 1;
  }

  .prev-button {
    left: 1rem;
  }

  .next-button {
    right: 1rem;
  }

  .icon-button:hover,
  .nav-button:hover {
    background: rgba(244, 179, 107, 0.18);
  }

  .icon-button:focus-visible,
  .nav-button:focus-visible {
    outline: 2px solid #f4b36b;
    outline-offset: 2px;
  }

  @media (max-width: 720px) {
    .nav-button {
      top: auto;
      bottom: 1rem;
      transform: none;
      width: 2.8rem;
      height: 2.8rem;
      font-size: 1.8rem;
    }

    .prev-button {
      left: 1rem;
    }

    .next-button {
      right: 1rem;
    }

    .icon-button {
      top: 0.75rem;
      right: 0.75rem;
    }
  }
</style>
