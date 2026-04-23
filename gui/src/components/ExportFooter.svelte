<script lang="ts">
  import { onMount } from "svelte";
  import {
    exportStatus,
    exportProgress,
    exportLog,
  } from "../lib/stores";
  import {
    onExportProgress,
    onExportLog,
    onExportDone,
    cancelExport,
  } from "../lib/tauri";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  let collapsed = true;

  onMount(() => {
    const unlistens: UnlistenFn[] = [];

    onExportProgress((p) => {
      exportProgress.set(p);
    }).then((u) => unlistens.push(u));

    onExportLog((p) => {
      exportLog.update((l) => {
        const next = [...l, p.line];
        return next.length > 500 ? next.slice(next.length - 500) : next;
      });
    }).then((u) => unlistens.push(u));

    onExportDone((p) => {
      exportStatus.set(p.status);
      if (p.message) {
        exportLog.update((l) => [...l, `[export-done ${p.status}] ${p.message}`]);
      }
    }).then((u) => unlistens.push(u));

    return () => unlistens.forEach((u) => u());
  });

  $: pct = $exportProgress
    ? Math.round(($exportProgress.frame / Math.max(1, $exportProgress.total)) * 100)
    : 0;

  $: etaLabel = $exportProgress?.eta_seconds != null
    ? `${Math.round($exportProgress.eta_seconds)}s`
    : "—";

  $: showFooter = $exportStatus === "running" || $exportProgress !== null;
</script>

<footer class="footer" class:running={$exportStatus === "running"}>
  {#if showFooter}
    <div class="top">
      <div class="bar">
        <div class="fill" style="width: {pct}%"></div>
      </div>
      <div class="stats">
        {#if $exportProgress}
          {$exportProgress.frame} / {$exportProgress.total}
          · {$exportProgress.fps.toFixed(1)} fps
          · ETA {etaLabel}
          · {pct}%
        {:else}
          starting…
        {/if}
      </div>
      {#if $exportStatus === "running"}
        <button class="cancel" on:click={cancelExport}>Cancel</button>
      {/if}
      <button class="toggle" on:click={() => collapsed = !collapsed}>
        {collapsed ? "Show log" : "Hide log"}
      </button>
    </div>

    {#if !collapsed}
      <div class="log">
        {#each $exportLog as line}
          <div class="logline">{line}</div>
        {/each}
        {#if $exportLog.length === 0}
          <div class="logline muted">(no log yet)</div>
        {/if}
      </div>
    {/if}
  {/if}
</footer>

<style>
  .footer { background: #1a1a1a; }
  .top {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding: 0.5rem 1rem;
  }
  .bar {
    flex: 1;
    background: #2a2a2a;
    height: 10px;
    border-radius: 5px;
    overflow: hidden;
  }
  .fill {
    height: 100%;
    background: #4ac;
    transition: width 0.1s linear;
  }
  .stats {
    font-size: 0.85rem;
    color: #bbb;
    min-width: 20ch;
    text-align: right;
  }
  .cancel {
    background: #a33; color: white; border: 0;
    padding: 0.3rem 0.8rem; cursor: pointer;
  }
  .cancel:hover { background: #b44; }
  .toggle {
    background: #333; color: #ddd; border: 0;
    padding: 0.3rem 0.8rem; cursor: pointer;
  }
  .log {
    max-height: 200px;
    overflow-y: auto;
    font-family: ui-monospace, "Cascadia Code", Menlo, monospace;
    font-size: 12px;
    padding: 0.5rem 1rem;
    background: #111;
    border-top: 1px solid #222;
  }
  .logline { color: #aaa; white-space: pre-wrap; }
  .muted { color: #666; }
</style>
