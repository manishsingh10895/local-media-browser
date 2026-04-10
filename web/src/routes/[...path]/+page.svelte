<script lang="ts">
  import { browser } from '$app/environment';
  import { onMount } from 'svelte';
  import { fetchFolderResponse, thumbnailUrl } from '$lib/api';
  import GalleryGrid from '$lib/components/GalleryGrid.svelte';
  import type {
    HealthResponse,
    FolderResponse,
    GridSize,
    MediaItem,
    SortDirection,
    SortField
  } from '$lib/types';

  let {
    data
  }: {
    data:
      | {
          mode: 'ready';
          folder: FolderResponse;
          currentPath: string;
          sortField: SortField;
          sortDirection: SortDirection;
          gridSize: GridSize;
          publicApiBaseUrl: string;
          apiBaseUrl: string;
        }
      | {
          mode: 'indexing';
          status: HealthResponse;
          currentPath: string;
          sortField: SortField;
          sortDirection: SortDirection;
          gridSize: GridSize;
          publicApiBaseUrl: string;
          apiBaseUrl: string;
        };
  } = $props();
  let folder = $state<FolderResponse | null>(null);
  let indexingStatus = $state<HealthResponse | null>(null);
  let mediaItems = $state<MediaItem[]>([]);
  let nextOffset = $state<number | null>(null);
  let loadingMore = $state(false);
  let refreshing = $state(false);
  let sentinel = $state<HTMLDivElement | null>(null);
  let observer: IntersectionObserver | null = null;
  let indexingPollTimer: ReturnType<typeof window.setInterval> | null = null;

  function folderLabel(itemCount: number): string {
    return itemCount === 1 ? '1 media item' : `${itemCount} media items`;
  }

  function sortHref(field: SortField, direction: SortDirection): string {
    const params = new URLSearchParams();
    params.set('sort', field);
    params.set('dir', direction);
    params.set('view', folder?.grid_size ?? data.gridSize ?? 'comfortable');
    const query = params.toString();
    return query ? `?${query}` : '';
  }

  function viewHref(size: GridSize): string {
    const params = new URLSearchParams();
    params.set('sort', folder?.sort_field ?? data.sortField);
    params.set('dir', folder?.sort_direction ?? data.sortDirection);
    params.set('view', size);
    const query = params.toString();
    return query ? `?${query}` : '';
  }

  async function loadMore(): Promise<void> {
    if (!browser || loadingMore || nextOffset === null || !folder) {
      return;
    }

    loadingMore = true;

    try {
      const nextPage = await fetchFolderResponse(
        window.fetch.bind(window),
        data.publicApiBaseUrl,
        folder.current_path,
        folder.sort_field,
        folder.sort_direction,
        nextOffset,
        folder.limit
      );
      if (nextPage.kind !== 'ready') {
        return;
      }
      mediaItems = [...mediaItems, ...nextPage.folder.media];
      nextOffset = nextPage.folder.next_offset;
    } finally {
      loadingMore = false;
    }
  }

  async function refreshCurrentFolder(): Promise<void> {
    if (!browser || refreshing || !folder) {
      return;
    }

    refreshing = true;

    try {
      const refreshed = await fetchFolderResponse(
        window.fetch.bind(window),
        data.publicApiBaseUrl,
        folder.current_path,
        folder.sort_field,
        folder.sort_direction,
        0,
        folder.limit
      );
      if (refreshed.kind !== 'ready') {
        indexingStatus = refreshed.status;
        folder = null;
        mediaItems = [];
        nextOffset = null;
        return;
      }
      folder = {
        ...refreshed.folder,
        grid_size: folder.grid_size
      };
      mediaItems = refreshed.folder.media;
      nextOffset = refreshed.folder.next_offset;
    } finally {
      refreshing = false;
    }
  }

  async function pollUntilReady(): Promise<void> {
    if (!browser || !indexingStatus) {
      return;
    }

    const result = await fetchFolderResponse(
      window.fetch.bind(window),
      data.publicApiBaseUrl,
      data.currentPath,
      data.sortField,
      data.sortDirection,
      0,
      60
    );

    if (result.kind === 'indexing') {
      indexingStatus = result.status;
      return;
    }

    folder = {
      ...result.folder,
      grid_size: data.gridSize
    };
    indexingStatus = null;
    mediaItems = result.folder.media;
    nextOffset = result.folder.next_offset;
  }

  onMount(() => {
    observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          void loadMore();
        }
      },
      { rootMargin: '320px' }
    );

    if (sentinel) {
      observer.observe(sentinel);
    }

    if (browser && indexingStatus) {
      indexingPollTimer = window.setInterval(() => {
        void pollUntilReady();
      }, 2500);
    }

    return () => {
      observer?.disconnect();
      if (indexingPollTimer) {
        window.clearInterval(indexingPollTimer);
      }
    };
  });

  $effect(() => {
    if (data.mode === 'ready') {
      folder = data.folder;
      indexingStatus = null;
      mediaItems = data.folder.media;
      nextOffset = data.folder.next_offset;
    } else {
      folder = null;
      indexingStatus = data.status;
      mediaItems = [];
      nextOffset = null;
    }
  });

  $effect(() => {
    if (observer && sentinel) {
      observer.disconnect();
      observer.observe(sentinel);
    }
  });

  $effect(() => {
    if (!browser) {
      return;
    }

    if (indexingPollTimer) {
      window.clearInterval(indexingPollTimer);
      indexingPollTimer = null;
    }

    if (indexingStatus) {
      indexingPollTimer = window.setInterval(() => {
        void pollUntilReady();
      }, 2500);
    }
  });
</script>

<svelte:head>
  <title>{folder?.current_path ? `${folder.current_path} · Local Gallery` : indexingStatus ? 'Indexing · Local Gallery' : 'Local Gallery'}</title>
  <meta
    name="description"
    content="Browse images and videos hosted by the local Rust media server."
  />
</svelte:head>

<div class="shell">
  <header class="hero">
    <div class="hero-copy">
      {#if folder}
        <nav class="breadcrumbs" aria-label="Breadcrumb">
          {#each folder.breadcrumbs as crumb, index}
            {#if index > 0}
              <span class="crumb-separator">/</span>
            {/if}
            <a href={crumb.href}>{crumb.name}</a>
          {/each}
        </nav>

        <h2>{folder.current_path ? folder.current_path.split('/').at(-1) : 'Local Media Browser'}</h2>
        <p class="lede">
          {#if folder.current_path}
            Browsing this folder and its immediate media items.
          {:else}
            Browse folders first or open media directly from the root collection.
          {/if}
        </p>
      {:else}
        <span class="status-chip">Server Live</span>
        <h2>Indexing Media Library</h2>
        <p class="lede">
          The server is up and scanning your media folder. This page updates automatically when the initial index is ready.
        </p>
      {/if}
    </div>

    <div class="hero-panel">
      {#if folder}
        <span class="panel-label">Current View</span>
        <strong>{folder.folders.length + folder.total_media_count}</strong>
        <p>
          {folder.folders.length} {folder.folders.length === 1 ? 'folder' : 'folders'} ·
          {folder.total_media_count} {folder.total_media_count === 1 ? 'media item' : 'media items'}
        </p>
      {:else if indexingStatus}
        <span class="panel-label">Index Status</span>
        <strong>{indexingStatus.phase}</strong>
        <p>
          {indexingStatus.current_job ?? 'initial'} job · {indexingStatus.indexed_media_count} indexed
        </p>
      {/if}
    </div>
  </header>

  {#if !folder && indexingStatus}
    <section class="indexing-panel">
      <div class="indexing-copy">
        <h3>Preparing your library</h3>
        <p>The backend is scanning the media root. Large folders on Docker Desktop, especially on Windows, can take time on the first pass.</p>
      </div>
      <dl class="indexing-stats">
        <div>
          <dt>Phase</dt>
          <dd>{indexingStatus.phase}</dd>
        </div>
        <div>
          <dt>Job</dt>
          <dd>{indexingStatus.current_job ?? 'initial'}</dd>
        </div>
        <div>
          <dt>Indexed</dt>
          <dd>{indexingStatus.indexed_media_count}</dd>
        </div>
      </dl>
      {#if indexingStatus.last_error}
        <p class="indexing-error">{indexingStatus.last_error}</p>
      {/if}
    </section>
  {:else if folder}
    {@const currentFolder = folder}
    <section class="folder-section">
      <div class="section-header">
        <h3>Folders</h3>
        <div class="sort-controls">
          <label>
            <span>Sort</span>
            <select onchange={(event) => (window.location.href = sortHref(event.currentTarget.value as SortField, currentFolder.sort_direction))}>
              <option value="name" selected={currentFolder.sort_field === 'name'}>Name</option>
              <option value="date" selected={currentFolder.sort_field === 'date'}>Date</option>
              <option value="size" selected={currentFolder.sort_field === 'size'}>Size</option>
            </select>
          </label>

          <a class="direction-toggle" href={sortHref(currentFolder.sort_field, currentFolder.sort_direction === 'asc' ? 'desc' : 'asc')}>
            {currentFolder.sort_direction === 'asc' ? 'Ascending' : 'Descending'}
          </a>
        </div>
      </div>

      {#if currentFolder.folders.length > 0}
        <div class="folder-grid">
          {#each currentFolder.folders as child}
            <a class="folder-card" href={`/${child.relative_path}`}>
              <div class="folder-cover">
                {#if child.cover}
                  <img
                    src={thumbnailUrl(child.cover.relative_path, data.publicApiBaseUrl, child.cover.modified_ms)}
                    alt={child.name}
                  />
                {:else}
                  <div class="folder-icon" aria-hidden="true">◫</div>
                {/if}
              </div>
              <div class="folder-meta">
                <h4>{child.name}</h4>
                <p>{folderLabel(child.item_count)}</p>
              </div>
            </a>
          {/each}
        </div>
      {:else}
        <div class="empty-inline">
          <p>No child folders in this location.</p>
        </div>
      {/if}
    </section>

    <section class="media-section">
      <div class="section-header">
        <h3>Media</h3>
        <div class="media-toolbar">
          <button
            type="button"
            class="refresh-button"
            onclick={refreshCurrentFolder}
            disabled={refreshing}
          >
            {refreshing ? 'Refreshing…' : 'Refresh'}
          </button>

          <div class="view-controls" aria-label="Grid size">
            <span>View</span>
            <a class:active={currentFolder.grid_size === 'compact'} href={viewHref('compact')}>Compact</a>
            <a class:active={currentFolder.grid_size === 'comfortable'} href={viewHref('comfortable')}>Comfortable</a>
            <a class:active={currentFolder.grid_size === 'large'} href={viewHref('large')}>Large</a>
          </div>
        </div>
      </div>

      <GalleryGrid
        items={mediaItems}
        apiBaseUrl={data.publicApiBaseUrl}
        gridSize={currentFolder.grid_size ?? 'comfortable'}
      />

      {#if nextOffset !== null}
        <div class="load-more-zone" bind:this={sentinel}>
          <p>{loadingMore ? 'Loading more media...' : 'Scroll to load more'}</p>
          <button type="button" class="load-more-button" onclick={loadMore} disabled={loadingMore}>
            {loadingMore ? 'Loading…' : 'Load more'}
          </button>
        </div>
      {/if}
    </section>

    <div class="mobile-refresh">
      <button
        type="button"
        class="refresh-button mobile-refresh-button"
        onclick={refreshCurrentFolder}
        disabled={refreshing}
      >
        {refreshing ? 'Refreshing…' : 'Refresh'}
      </button>
    </div>
  {/if}

  <!--
  {#if data.folders.length > 0}
    <section class="folder-section">
      <div class="section-header">
        <h3>Folders</h3>
        <div class="sort-controls">
          <label>
            <span>Sort</span>
            <select onchange={(event) => (window.location.href = sortHref(event.currentTarget.value as SortField, data.sortDirection))}>
              <option value="name" selected={data.sortField === 'name'}>Name</option>
              <option value="date" selected={data.sortField === 'date'}>Date</option>
              <option value="size" selected={data.sortField === 'size'}>Size</option>
            </select>
          </label>

          <a class="direction-toggle" href={sortHref(data.sortField, data.sortDirection === 'asc' ? 'desc' : 'asc')}>
            {data.sortDirection === 'asc' ? 'Ascending' : 'Descending'}
          </a>
        </div>
      </div>

      <div class="folder-grid">
        {#each data.folders as folder}
          <a class="folder-card" href={`/${folder.relative_path}`}>
            <div class="folder-cover">
              {#if folder.cover}
                {#if folder.cover.media_type === 'image'}
                  <img src={mediaUrl(folder.cover.relative_path, data.publicApiBaseUrl)} alt={folder.name} />
                {:else}
                  <video muted playsinline preload="metadata">
                    <source src={mediaUrl(folder.cover.relative_path, data.publicApiBaseUrl)} type={folder.cover.mime} />
                  </video>
                {/if}
              {:else}
                <div class="folder-icon" aria-hidden="true">◫</div>
              {/if}
            </div>
            <div class="folder-meta">
              <h4>{folder.name}</h4>
              <p>{folderLabel(folder.item_count)}</p>
            </div>
          </a>
        {/each}
      </div>
    </section>
  {/if}

  <section class="media-section">
    <div class="section-header">
      <h3>Media</h3>
    </div>

    <GalleryGrid items={data.media} apiBaseUrl={data.publicApiBaseUrl} />
  </section>
  -->
</div>

<style>
  :global(body) {
    margin: 0;
    color: #e8edf2;
    font-family: "Iowan Old Style", "Palatino Linotype", "Book Antiqua",
      Palatino, Georgia, serif;
    background:
      radial-gradient(circle at 15% 10%, rgba(244, 179, 107, 0.22), transparent 24%),
      radial-gradient(circle at 84% 18%, rgba(99, 154, 196, 0.2), transparent 28%),
      radial-gradient(circle at 50% 110%, rgba(31, 103, 90, 0.16), transparent 38%),
      linear-gradient(180deg, #061019 0%, #091722 40%, #0d1821 100%);
    min-height: 100vh;
    position: relative;
  }

  :global(body::before) {
    content: "";
    position: fixed;
    inset: 0;
    pointer-events: none;
    opacity: 0.34;
    background-image:
      linear-gradient(rgba(255, 255, 255, 0.025) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.025) 1px, transparent 1px);
    background-size: 32px 32px;
    mask-image: linear-gradient(180deg, rgba(0, 0, 0, 0.5), transparent 78%);
  }

  .shell {
    max-width: 1320px;
    margin: 0 auto;
    padding: 3.5rem 1.4rem 4.5rem;
  }

  .hero {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 240px;
    gap: 1.5rem;
    align-items: end;
    margin-bottom: 2.4rem;
  }

  .hero-copy {
    max-width: 760px;
  }

  .breadcrumbs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    margin-bottom: 1rem;
    color: #a8bccc;
    font-family: "Avenir Next", "Segoe UI", sans-serif;
    font-size: 0.82rem;
    letter-spacing: 0.03em;
  }

  .breadcrumbs a {
    color: inherit;
    text-decoration: none;
  }

  .breadcrumbs a:hover {
    color: #fff4e6;
  }

  .crumb-separator {
    color: #6f8798;
  }

  .hero-copy h2 {
    margin: 0;
    max-width: 14ch;
    font-size: clamp(2.4rem, 6vw, 4rem);
    line-height: 0.93;
    letter-spacing: -0.04em;
    text-wrap: balance;
    color: #f7f4ef;
  }

  .lede {
    margin: 0.95rem 0 0;
    color: #acc0ce;
    font-size: 1rem;
    line-height: 1.6;
  }

  .hero-panel {
    padding: 1.15rem 1.15rem 1.05rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.3rem;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.08), rgba(255, 255, 255, 0.03)),
      rgba(8, 18, 27, 0.84);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.22);
    backdrop-filter: blur(10px);
  }

  .panel-label {
    display: block;
    margin-bottom: 0.55rem;
    color: #8fa8b8;
    font-size: 0.74rem;
    letter-spacing: 0.16em;
    text-transform: uppercase;
    font-family: "Avenir Next", "Segoe UI", sans-serif;
  }

  .hero-panel strong {
    display: block;
    font-size: clamp(2rem, 4vw, 3rem);
    line-height: 1;
    color: #fff4e6;
    font-weight: 700;
  }

  .hero-panel p {
    margin: 0.7rem 0 0;
    color: #acc0ce;
    font-size: 0.92rem;
    line-height: 1.55;
  }

  .status-chip {
    display: inline-flex;
    align-items: center;
    margin-bottom: 0.9rem;
    padding: 0.32rem 0.7rem;
    border-radius: 999px;
    background: rgba(244, 179, 107, 0.12);
    color: #f4c98f;
    font-size: 0.78rem;
    font-family: "Avenir Next", "Segoe UI", sans-serif;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .indexing-panel {
    padding: 1.35rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.3rem;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.03)),
      rgba(8, 18, 27, 0.84);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.22);
  }

  .indexing-copy h3 {
    margin: 0;
    color: #fff4e6;
    font-size: 1.2rem;
  }

  .indexing-copy p,
  .indexing-error {
    color: #acc0ce;
    line-height: 1.6;
  }

  .indexing-stats {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.9rem;
    margin: 1.2rem 0 0;
  }

  .indexing-stats div {
    padding: 0.9rem 1rem;
    border-radius: 1rem;
    background: rgba(255, 255, 255, 0.04);
  }

  .indexing-stats dt {
    margin-bottom: 0.3rem;
    color: #8fa8b8;
    font-size: 0.72rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    font-family: "Avenir Next", "Segoe UI", sans-serif;
  }

  .indexing-stats dd {
    margin: 0;
    color: #f7f4ef;
    font-size: 1.05rem;
  }

  .folder-section,
  .media-section {
    margin-top: 2rem;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .section-header h3 {
    margin: 0;
    color: #f7f4ef;
    font-size: 1rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    font-family: "Avenir Next", "Segoe UI", sans-serif;
  }

  .media-toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .refresh-button {
    padding: 0.48rem 0.82rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    background: rgba(8, 18, 27, 0.84);
    color: #f7f4ef;
    cursor: pointer;
    font-size: 0.8rem;
    font-family: "Avenir Next", "Segoe UI", sans-serif;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .refresh-button:disabled {
    opacity: 0.65;
    cursor: default;
  }

  .mobile-refresh {
    position: sticky;
    bottom: 1rem;
    z-index: 8;
    display: none;
    justify-content: center;
    margin-top: 1.25rem;
    pointer-events: none;
  }

  .mobile-refresh-button {
    min-width: 150px;
    box-shadow: 0 18px 40px rgba(0, 0, 0, 0.28);
    backdrop-filter: blur(10px);
    pointer-events: auto;
  }

  .view-controls {
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
    color: #a8bccc;
    font-size: 0.8rem;
    font-family: "Avenir Next", "Segoe UI", sans-serif;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .view-controls a {
    padding: 0.45rem 0.72rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    background: rgba(8, 18, 27, 0.84);
    color: #f7f4ef;
    text-decoration: none;
    font-size: 0.78rem;
  }

  .view-controls a.active {
    border-color: rgba(244, 179, 107, 0.22);
    background: rgba(244, 179, 107, 0.12);
    color: #fff4e6;
  }

  .sort-controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .sort-controls label {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    color: #a8bccc;
    font-size: 0.8rem;
    font-family: "Avenir Next", "Segoe UI", sans-serif;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .sort-controls select,
  .direction-toggle {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    background: rgba(8, 18, 27, 0.84);
    color: #f7f4ef;
    font-size: 0.86rem;
    text-decoration: none;
  }

  .sort-controls select {
    padding: 0.55rem 0.85rem;
  }

  .direction-toggle {
    padding: 0.58rem 0.9rem;
  }

  .folder-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 1rem;
  }

  .folder-card {
    display: block;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.25rem;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.06), rgba(255, 255, 255, 0.02)),
      linear-gradient(135deg, rgba(116, 171, 214, 0.09), transparent 30%),
      rgba(9, 18, 28, 0.82);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.22);
    color: inherit;
    text-decoration: none;
    transition: transform 180ms ease, border-color 180ms ease, box-shadow 180ms ease;
  }

  .folder-cover {
    aspect-ratio: 4 / 3;
    overflow: hidden;
    background: rgba(255, 255, 255, 0.04);
  }

  .folder-cover :global(img),
  .folder-cover :global(video) {
    width: 100%;
    height: 100%;
    display: block;
    object-fit: cover;
  }

  .folder-card:hover {
    transform: translateY(-3px);
    border-color: rgba(116, 171, 214, 0.22);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.28);
  }

  .folder-icon {
    display: grid;
    place-items: center;
    width: 100%;
    height: 100%;
    min-height: 180px;
    color: #f0c48c;
    font-size: 2rem;
    background: rgba(255, 255, 255, 0.06);
  }

  .folder-meta h4 {
    margin: 0;
    font-size: 1rem;
    color: #f7f4ef;
  }

  .folder-meta p {
    margin: 0.35rem 0 0;
    font-size: 0.86rem;
    color: #89a0b2;
  }

  .folder-meta {
    padding: 1rem 1.05rem 1.05rem;
  }

  .empty-inline,
  .load-more-zone {
    margin-top: 1rem;
    padding: 1rem 1.1rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1rem;
    background: rgba(8, 18, 27, 0.6);
    color: #a8bccc;
  }

  .empty-inline p,
  .load-more-zone p {
    margin: 0;
  }

  .load-more-zone {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
  }

  .load-more-button {
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 999px;
    background: rgba(8, 18, 27, 0.84);
    color: #f7f4ef;
    padding: 0.58rem 0.9rem;
    cursor: pointer;
  }

  @media (max-width: 840px) {
    .hero {
      grid-template-columns: 1fr;
      align-items: start;
    }

    .hero-panel {
      max-width: 320px;
    }
  }

  @media (max-width: 640px) {
    .shell {
      padding: 2.4rem 1rem 3.5rem;
    }

    .hero-copy h2 {
      font-size: clamp(2.1rem, 11vw, 3.4rem);
    }

    .folder-grid {
      grid-template-columns: 1fr;
    }

    .section-header {
      align-items: start;
      flex-direction: column;
    }

    .view-controls {
      width: 100%;
    }

    .media-toolbar {
      width: 100%;
      align-items: start;
      flex-direction: column;
    }

    .media-toolbar > .refresh-button {
      display: none;
    }

    .mobile-refresh {
      display: flex;
    }

    .load-more-zone {
      align-items: start;
      flex-direction: column;
    }
  }
</style>
