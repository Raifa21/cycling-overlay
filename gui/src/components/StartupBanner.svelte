<script lang="ts">
  export let kind: "ffmpeg" | "cli";
  export let onSetPath: () => void;
</script>

<div class="banner" role="alert">
  <span>
    <strong>{kind}</strong> not found. Export is disabled until this is configured.
  </span>
  <button on:click={onSetPath}>Set path…</button>
</div>

<style>
  .banner {
    background: #a33;
    color: white;
    padding: 0.5rem 1rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
  }
  button {
    background: white;
    color: #a33;
    border: 0;
    padding: 0.25rem 0.7rem;
    cursor: pointer;
    font-weight: 600;
  }
  button:hover {
    background: #eee;
  }
</style>
