import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  SessionState,
  ActivityInfo,
  LayoutInfo,
  ProgressPayload,
  LogPayload,
  DonePayload,
  ExportArgs,
} from "./types";

export const sessionLoad = () => invoke<SessionState>("session_load");
export const sessionSave = (state: SessionState) => invoke<void>("session_save", { state });

export const probeFfmpeg = (overridePath?: string) =>
  invoke<string>("probe_ffmpeg", { overridePath: overridePath ?? null });
export const probeCli = (overridePath?: string) =>
  invoke<string>("probe_cli", { overridePath: overridePath ?? null });

export const loadActivity = (path: string) => invoke<ActivityInfo>("load_activity", { path });
export const loadLayout = (path: string) => invoke<LayoutInfo>("load_layout", { path });

export const previewFrame = (tSeconds: number, downscaleWidth?: number) =>
  invoke<string>("preview_frame", {
    tSeconds,
    downscaleWidth: downscaleWidth ?? null,
  });

export const watchLayout = (path: string) => invoke<void>("watch_layout", { path });
export const unwatchLayout = () => invoke<void>("unwatch_layout");

export const startExport = (args: ExportArgs) => invoke<void>("start_export", { args });
export const cancelExport = () => invoke<void>("cancel_export");

export const onLayoutReloaded = (fn: () => void): Promise<UnlistenFn> =>
  listen("layout-reloaded", fn);
export const onLayoutError = (fn: (msg: string) => void): Promise<UnlistenFn> =>
  listen<string>("layout-error", (e) => fn(e.payload));
export const onExportProgress = (fn: (p: ProgressPayload) => void): Promise<UnlistenFn> =>
  listen<ProgressPayload>("export-progress", (e) => fn(e.payload));
export const onExportLog = (fn: (p: LogPayload) => void): Promise<UnlistenFn> =>
  listen<LogPayload>("export-log", (e) => fn(e.payload));
export const onExportDone = (fn: (p: DonePayload) => void): Promise<UnlistenFn> =>
  listen<DonePayload>("export-done", (e) => fn(e.payload));
