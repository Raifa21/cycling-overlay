import { writable } from "svelte/store";

export const ffmpegMissing = writable(false);
export const cliMissing = writable(false);

/** Most-recent user-facing file-load failure (activity or layout). Cleared
 *  on next successful load or when the user dismisses the banner. */
export const loadError = writable<string | null>(null);
