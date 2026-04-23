export interface SessionState {
  input_path: string | null;
  layout_path: string | null;
  output_path: string | null;
  codec: string;
  quality: number;
  chromakey: string;
  from_seconds: number;
  to_seconds: number | null;
  cli_path_override: string | null;
  ffmpeg_path_override: string | null;
}

export interface ActivityInfo {
  duration_seconds: number;
  sample_count: number;
  metrics_present: string[];
}

export interface LayoutInfo {
  width: number;
  height: number;
  fps: number;
  widget_count: number;
  warnings: string[];
}

export interface ProgressPayload {
  frame: number;
  total: number;
  fps: number;
  eta_seconds: number | null;
}

export interface LogPayload {
  line: string;
  stream: string;
}

export interface DonePayload {
  status: "success" | "canceled" | "error";
  message: string | null;
}

export interface ExportArgs {
  cli_path: string;
  input: string;
  layout: string;
  output: string;
  codec: string;
  quality: number;
  chromakey: string;
  from_seconds: number;
  to_seconds: number;
  ffmpeg_path_override: string | null;
}
