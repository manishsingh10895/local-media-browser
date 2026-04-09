<script lang="ts">
  import GalleryGrid from '$lib/components/GalleryGrid.svelte';
  import { mediaUrl } from '$lib/api';
  import type {
    BreadcrumbItem,
    FolderEntry,
    MediaItem,
    SortDirection,
    SortField
  } from '$lib/types';

  let {
    data
  }: {
    data: {
      currentPath: string;
      folders: FolderEntry[];
      media: MediaItem[];
      breadcrumbs: BreadcrumbItem[];
      publicApiBaseUrl: string;
      sortField: SortField;
      sortDirection: SortDirection;
    };
  } = $props();

  function folderLabel(itemCount: number): string {
    return itemCount === 1 ? '1 media item' : `${itemCount} media items`;
  }

  function sortHref(field: SortField, direction: SortDirection): string {
    const params = new URLSearchParams();
    params.set('sort', field);
    params.set('dir', direction);
    const query = params.toString();
    return query ? `?${query}` : '';
  }
</script>

<svelte:head>
  <title>{data.currentPath ? `${data.currentPath} · Local Gallery` : 'Local Gallery'}</title>
  <meta
    name="description"
    content="Browse images and videos hosted by the local Rust media server."
  />
</svelte:head>

<div class="shell">
  <header class="hero">
    <div class="hero-copy">
      <nav class="breadcrumbs" aria-label="Breadcrumb">
        {#each data.breadcrumbs as crumb, index}
          {#if index > 0}
            <span class="crumb-separator">/</span>
          {/if}
          <a href={crumb.href}>{crumb.name}</a>
        {/each}
      </nav>

      <h2>{data.currentPath ? data.currentPath.split('/').at(-1) : 'Local Media Browser'}</h2>
      <p class="lede">
        {#if data.currentPath}
          Browsing this folder and its immediate media items.
        {:else}
          Browse folders first or open media directly from the root collection.
        {/if}
      </p>
    </div>

    <div class="hero-panel">
      <span class="panel-label">Current View</span>
      <strong>{data.folders.length + data.media.length}</strong>
      <p>
        {data.folders.length} {data.folders.length === 1 ? 'folder' : 'folders'} ·
        {data.media.length} {data.media.length === 1 ? 'media item' : 'media items'}
      </p>
    </div>
  </header>

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
  }
</style>
