use activity::Activity;
use layout::Layout;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Default)]
pub struct AppState {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    activity: Option<Activity>,
    activity_path: Option<PathBuf>,
    layout: Option<Layout>,
    layout_path: Option<PathBuf>,
}

impl AppState {
    pub fn set_activity(&self, a: Activity, p: PathBuf) {
        let mut g = self.inner.lock().unwrap();
        g.activity = Some(a);
        g.activity_path = Some(p);
    }
    pub fn set_layout(&self, l: Layout, p: PathBuf) {
        let mut g = self.inner.lock().unwrap();
        g.layout = Some(l);
        g.layout_path = Some(p);
    }
    pub fn layout_path(&self) -> Option<PathBuf> {
        self.inner.lock().ok().and_then(|g| g.layout_path.clone())
    }
    pub fn with_layout<R>(&self, f: impl FnOnce(&Layout) -> R) -> Option<R> {
        let g = self.inner.lock().ok()?;
        Some(f(g.layout.as_ref()?))
    }
    pub fn with_activity<R>(&self, f: impl FnOnce(&Activity) -> R) -> Option<R> {
        let g = self.inner.lock().ok()?;
        Some(f(g.activity.as_ref()?))
    }
    /// Borrow both for a single critical section. Used by preview rendering.
    pub fn with_both<R>(&self, f: impl FnOnce(&Layout, &Activity) -> R) -> Option<R> {
        let g = self.inner.lock().ok()?;
        let l = g.layout.as_ref()?;
        let a = g.activity.as_ref()?;
        Some(f(l, a))
    }
}

#[derive(Serialize)]
pub struct ActivityInfo {
    pub duration_seconds: f64,
    pub sample_count: usize,
    pub metrics_present: Vec<String>,
}

#[derive(Serialize)]
pub struct LayoutInfo {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub widget_count: usize,
    pub warnings: Vec<String>,
}

#[tauri::command]
pub fn load_activity(state: tauri::State<AppState>, path: PathBuf) -> Result<ActivityInfo, String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    let mut activity = match ext.as_deref() {
        Some("fit") => activity::load_fit(&path).map_err(|e| e.to_string())?,
        Some("gpx") => activity::load_gpx(&path).map_err(|e| e.to_string())?,
        _ => return Err("unsupported file type (expected .fit or .gpx)".into()),
    };
    activity.prepare();

    let duration_seconds = activity.duration().as_secs_f64();
    let sample_count = activity.samples.len();
    let metrics_present: Vec<String> = activity::Metric::ALL
        .iter()
        .filter(|m| activity::metric_present_on_activity(**m, &activity.samples))
        .map(|m| m.as_str().to_string())
        .collect();

    state.set_activity(activity, path);
    Ok(ActivityInfo {
        duration_seconds,
        sample_count,
        metrics_present,
    })
}

#[tauri::command]
pub fn load_layout(state: tauri::State<AppState>, path: PathBuf) -> Result<LayoutInfo, String> {
    let s = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let layout: Layout = serde_json::from_str(&s).map_err(|e| format!("parse error: {}", e))?;
    let info = LayoutInfo {
        width: layout.canvas.width,
        height: layout.canvas.height,
        fps: layout.canvas.fps,
        widget_count: layout.widgets.len(),
        warnings: Vec::new(),
    };
    state.set_layout(layout, path);
    Ok(info)
}
