<script lang="ts">
  import GalleryGrid from "$lib/components/GalleryGrid.svelte";
  import type { MediaItem } from "$lib/types";

  let { data }: { data: { media: MediaItem[]; publicApiBaseUrl: string } } = $props();
</script>

<svelte:head>
  <title>Local Gallery</title>
  <meta
    name="description"
    content="Browse images and videos hosted by the local Rust media server."
  />
</svelte:head>

<div class="shell">
  <header class="hero">
    <div class="hero-copy">
      <h2>Local Media Browser</h2>
    </div>

    <div class="hero-panel">
      <span class="panel-label">Collection</span>
      <strong>{data.media.length}</strong>
      <p>
        {data.media.length === 1
          ? "media item ready to browse"
          : "media items ready to browse"}
      </p>
    </div>
  </header>

  <GalleryGrid items={data.media} apiBaseUrl={data.publicApiBaseUrl} />
</div>

<style>
  :global(body) {
    margin: 0;
    color: #e8edf2;
    font-family: "Iowan Old Style", "Palatino Linotype", "Book Antiqua",
      Palatino, Georgia, serif;
    background: radial-gradient(
        circle at 15% 10%,
        rgba(244, 179, 107, 0.22),
        transparent 24%
      ),
      radial-gradient(
        circle at 84% 18%,
        rgba(99, 154, 196, 0.2),
        transparent 28%
      ),
      radial-gradient(
        circle at 50% 110%,
        rgba(31, 103, 90, 0.16),
        transparent 38%
      ),
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
    background-image: linear-gradient(
        rgba(255, 255, 255, 0.025) 1px,
        transparent 1px
      ),
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

  .hero-copy h2 {
    margin: 0;
    max-width: 11ch;
    font-size: clamp(2.8rem, 6vw, 4.25rem);
    line-height: 0.93;
    letter-spacing: -0.04em;
    text-wrap: balance;
    color: #f7f4ef;
  }

  .hero-panel {
    padding: 1.15rem 1.15rem 1.05rem;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 1.3rem;
    background: linear-gradient(
        180deg,
        rgba(255, 255, 255, 0.08),
        rgba(255, 255, 255, 0.03)
      ),
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

  @media (max-width: 840px) {
    .hero {
      grid-template-columns: 1fr;
      align-items: start;
    }

    .hero-panel {
      max-width: 280px;
    }
  }

  @media (max-width: 640px) {
    .shell {
      padding: 2.4rem 1rem 3.5rem;
    }

    .hero-copy h2 {
      font-size: clamp(2.35rem, 11vw, 3.7rem);
    }
  }
</style>
