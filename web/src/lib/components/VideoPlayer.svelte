<script lang="ts">
  import 'vidstack/player';
  import 'vidstack/player/layouts/default';
  import 'vidstack/player/styles/default/theme.css';
  import 'vidstack/player/styles/default/layouts/video.css';

  let { src, title }: { src: string; title: string } = $props();
</script>

<div class="player-shell">
  <media-player
    class="vidstack-player"
    src={src}
    title={title}
    autoplay
    playsinline
    crossorigin
    view-type="video"
    stream-type="on-demand"
  >
    <media-provider></media-provider>
    <media-video-layout></media-video-layout>
  </media-player>
</div>

<style>
  .player-shell {
    width: 100%;
    height: 100%;
  }

  .player-shell :global(.vidstack-player) {
    width: 100%;
    height: 100%;
    --media-brand: #f4b36b;
    --media-focus-ring-color: #f4b36b;
    --media-tooltip-background: rgba(6, 11, 18, 0.92);
    --media-menu-background: rgba(8, 14, 22, 0.96);
    --media-control-background: rgba(7, 12, 18, 0.68);
    --media-controls-color: #f5f7fa;
    background: #000;
  }

  .player-shell :global(video) {
    width: 100%;
    height: 100%;
    object-fit: contain;
    background: #000;
  }
</style>
