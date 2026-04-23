/**
 * Parse an HH:MM:SS, MM:SS, or bare-seconds string into a number of seconds.
 * Fractional seconds are accepted in the lowest component only. Returns null
 * for malformed input (empty, non-numeric, components out of range).
 *
 * Mirrors the CLI's `parse_time_spec` in `crates/cli/src/args.rs` so the
 * roundtrip from GUI input -> argv stays consistent.
 */
export function parseTimeSpec(s: string): number | null {
  const trimmed = s.trim();
  if (!trimmed) return null;
  const parts = trimmed.split(":");
  if (parts.length === 1) {
    const n = Number(parts[0]);
    return Number.isFinite(n) && n >= 0 ? n : null;
  }
  if (parts.length === 2) {
    const m = Number(parts[0]);
    const sec = Number(parts[1]);
    if (!Number.isInteger(m) || m < 0) return null;
    if (!Number.isFinite(sec) || sec < 0 || sec >= 60) return null;
    return m * 60 + sec;
  }
  if (parts.length === 3) {
    const h = Number(parts[0]);
    const m = Number(parts[1]);
    const sec = Number(parts[2]);
    if (!Number.isInteger(h) || h < 0) return null;
    if (!Number.isInteger(m) || m < 0 || m >= 60) return null;
    if (!Number.isFinite(sec) || sec < 0 || sec >= 60) return null;
    return h * 3600 + m * 60 + sec;
  }
  return null;
}

/**
 * Format a number of seconds as `HH:MM:SS` (or `HH:MM:SS.mmm` when the
 * fractional part is non-zero). Negative or non-finite inputs render as
 * `00:00:00`.
 */
export function formatTimeSpec(secs: number): string {
  if (!Number.isFinite(secs) || secs < 0) return "00:00:00";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs - h * 3600 - m * 60;
  const hh = String(h).padStart(2, "0");
  const mm = String(m).padStart(2, "0");
  if (Math.abs(s - Math.round(s)) < 1e-6) {
    return `${hh}:${mm}:${String(Math.round(s)).padStart(2, "0")}`;
  }
  return `${hh}:${mm}:${s.toFixed(3).padStart(6, "0")}`;
}
