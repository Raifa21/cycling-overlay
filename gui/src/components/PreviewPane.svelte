<script lang="ts">
  import { previewImage, previewBusy, layoutInfo } from "../lib/stores";

  // The inner `.canvas` wrapper is sized to the layout's aspect ratio so
  // the outlined box shows the actual render surface, not the whole pane.
  // The checkerboard lives inside the canvas only — transparent regions
  // inside the rect show the board; area outside the rect is solid dark.
  $: aspect = $layoutInfo ? `${$layoutInfo.width} / ${$layoutInfo.height}` : "16 / 9";
</script>

<section class="preview" class:busy={$previewBusy}>
  {#if $layoutInfo}
    <div class="canvas" style="aspect-ratio: {aspect};">
      {#if $previewImage}
        <img src={$previewImage} alt="Preview frame" />
      {/if}
    </div>
    <div class="caption">{$layoutInfo.width} × {$layoutInfo.height}</div>
  {:else}
    <div class="empty">Load an activity and layout to see a preview.</div>
  {/if}
</section>

<style>
  .preview {
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    gap: 0.35rem;
    padding: 1rem;
    background: #111;
  }
  .canvas {
    max-width: 100%;
    max-height: 100%;
    display: flex;
    outline: 1px solid #555;
    background-color: #333;
    background-image:
      linear-gradient(45deg, #444 25%, transparent 25%),
      linear-gradient(-45deg, #444 25%, transparent 25%),
      linear-gradient(45deg, transparent 75%, #444 75%),
      linear-gradient(-45deg, transparent 75%, #444 75%);
    background-size: 20px 20px;
    background-position: 0 0, 0 10px, 10px -10px, -10px 0;
  }
  .canvas img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    display: block;
  }
  .caption {
    font-size: 0.75rem;
    color: #666;
    font-family: ui-monospace, "Cascadia Code", Menlo, monospace;
  }
  .empty { color: #888; font-size: 0.95rem; }
  .busy { opacity: 0.85; }
</style>
