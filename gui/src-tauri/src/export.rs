use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use crate::progress::{eta_seconds, parse_line, ProgressLine};

#[derive(Deserialize)]
pub struct ExportArgs {
    pub cli_path: PathBuf,
    pub input: PathBuf,
    pub layout: PathBuf,
    pub output: PathBuf,
    pub codec: String,
    pub quality: u32,
    pub chromakey: String,
    pub from_seconds: f64,
    pub to_seconds: f64,
    #[serde(default)]
    pub ffmpeg_path_override: Option<PathBuf>,
}

#[derive(Serialize, Clone)]
pub struct ProgressPayload {
    pub frame: u64,
    pub total: u64,
    pub fps: f64,
    pub eta_seconds: Option<f64>,
}

#[derive(Serialize, Clone)]
pub struct LogPayload {
    pub line: String,
    pub stream: &'static str,
}

#[derive(Serialize, Clone)]
pub struct DonePayload {
    pub status: String,
    pub message: Option<String>,
}

/// Handle to the running child process. `Some` while a render is in flight,
/// `None` between runs. The canceller takes the Child out of the Option when
/// it kills the process so the stderr-reader task sees an empty slot and
/// knows a cancel was intentional.
pub struct ExportHandle(pub Mutex<Option<Child>>);

impl Default for ExportHandle {
    fn default() -> Self {
        Self(Mutex::new(None))
    }
}

fn fmt_time(secs: f64) -> String {
    let s = secs.max(0.0);
    let h = (s / 3600.0) as u64;
    let m = ((s % 3600.0) / 60.0) as u64;
    let sec = s - (h as f64) * 3600.0 - (m as f64) * 60.0;
    // Keep 3 decimals when there's a fractional component; use integer
    // format when clean, for readability.
    if sec.fract().abs() < 1e-6 {
        format!("{:02}:{:02}:{:02}", h, m, sec as u64)
    } else {
        format!("{:02}:{:02}:{:06.3}", h, m, sec)
    }
}

/// Spawn the CLI. Returns immediately; progress events stream via `emit`.
#[tauri::command]
pub async fn start_export(
    app: AppHandle,
    args: ExportArgs,
) -> Result<(), String> {
    let mut cmd = Command::new(&args.cli_path);
    cmd.arg("render")
        .arg("-i").arg(&args.input)
        .arg("-l").arg(&args.layout)
        .arg("-o").arg(&args.output)
        .arg("--codec").arg(&args.codec)
        .arg("--crf").arg(args.quality.to_string())
        .arg("--qscale").arg(args.quality.to_string())
        .arg("--chromakey").arg(&args.chromakey)
        .arg("--from").arg(fmt_time(args.from_seconds))
        .arg("--to").arg(fmt_time(args.to_seconds))
        .arg("--progress-json")
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    // If the GUI has an ffmpeg override, prepend its directory to PATH so
    // the CLI's spawned ffmpeg resolves to the override binary.
    if let Some(override_path) = args.ffmpeg_path_override.as_ref() {
        if let Some(dir) = override_path.parent() {
            let existing = std::env::var_os("PATH").unwrap_or_default();
            let mut paths: Vec<PathBuf> =
                std::env::split_paths(&existing).collect();
            paths.insert(0, dir.to_path_buf());
            if let Ok(joined) = std::env::join_paths(paths) {
                cmd.env("PATH", joined);
            }
        }
    }

    // On Unix, put the child in its own process group so cancel can signal
    // the whole tree (CLI + its ffmpeg grandchild) with one killpg.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                // setpgid(0, 0) makes this process the leader of a new
                // process group whose pgid == its pid. Must be called in
                // the child before exec.
                if libc::setpgid(0, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let stderr = child.stderr.take().ok_or("no stderr pipe")?;

    {
        let handle_state = app.state::<ExportHandle>();
        *handle_state.0.lock().map_err(|e| e.to_string())? = Some(child);
    }

    // Reader task: parse stderr line by line, emit events.
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        let started = Instant::now();
        let mut saw_done = false;

        while let Ok(Some(line)) = reader.next_line().await {
            match parse_line(&line) {
                Some(ProgressLine::Progress { frame, total }) => {
                    let elapsed = started.elapsed().as_secs_f64();
                    let fps = if elapsed > 0.0 { frame as f64 / elapsed } else { 0.0 };
                    let eta = eta_seconds(frame, total, elapsed);
                    let _ = app_clone.emit(
                        "export-progress",
                        ProgressPayload { frame, total, fps, eta_seconds: eta },
                    );
                }
                Some(ProgressLine::Done) => {
                    saw_done = true;
                    let _ = app_clone.emit(
                        "export-done",
                        DonePayload { status: "success".into(), message: None },
                    );
                }
                Some(ProgressLine::Error { message }) => {
                    let _ = app_clone.emit(
                        "export-done",
                        DonePayload { status: "error".into(), message: Some(message) },
                    );
                }
                None => {
                    let _ = app_clone.emit(
                        "export-log",
                        LogPayload { line: line.clone(), stream: "stderr" },
                    );
                }
            }
        }

        // Reader EOF. Reap the child if it's still in state (i.e., not cancelled).
        let handle_state = app_clone.state::<ExportHandle>();
        let child_opt = {
            let mut g = match handle_state.0.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            g.take()
        };
        if let Some(mut child) = child_opt {
            if let Ok(status) = child.wait().await {
                if !status.success() && !saw_done {
                    let _ = app_clone.emit(
                        "export-done",
                        DonePayload {
                            status: "error".into(),
                            message: Some(format!("exited with code {:?}", status.code())),
                        },
                    );
                }
            }
        }
    });
    Ok(())
}

#[cfg(windows)]
fn kill_process_tree(child: &mut Child) {
    if let Some(pid) = child.id() {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/T", "/PID", &pid.to_string()])
            .output();
    }
}

#[cfg(not(windows))]
fn kill_process_tree(child: &mut Child) {
    if let Some(pid) = child.id() {
        // Child was spawned with setpgid(0,0), so its pgid == pid. Signal
        // the whole group so ffmpeg (grandchild) goes down with the CLI
        // even if the CLI's SIGTERM handler doesn't cascade quickly.
        unsafe {
            libc::kill(-(pid as i32), libc::SIGTERM);
        }
    }
}

#[tauri::command]
pub fn cancel_export(app: AppHandle) -> Result<(), String> {
    let handle_state = app.state::<ExportHandle>();
    let child_opt = {
        let mut g = handle_state.0.lock().map_err(|e| e.to_string())?;
        g.take()
    };
    if let Some(mut child) = child_opt {
        kill_process_tree(&mut child);
        let _ = app.emit(
            "export-done",
            DonePayload { status: "canceled".into(), message: None },
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::fmt_time;

    #[test]
    fn whole_seconds_no_decimals() {
        assert_eq!(fmt_time(0.0), "00:00:00");
        assert_eq!(fmt_time(65.0), "00:01:05");
        assert_eq!(fmt_time(3661.0), "01:01:01");
    }

    #[test]
    fn fractional_seconds_preserved() {
        assert_eq!(fmt_time(3.7), "00:00:03.700");
        assert_eq!(fmt_time(65.25), "00:01:05.250");
    }

    #[test]
    fn negative_clamped() {
        assert_eq!(fmt_time(-1.0), "00:00:00");
    }
}
