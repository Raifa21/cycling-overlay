use base64::Engine;
use std::sync::Mutex;
use std::time::Duration;
use tauri::State;
use tiny_skia::{Color, Pixmap};

use crate::state::AppState;

pub struct TextState(pub Mutex<render::TextCtx>);

impl Default for TextState {
    fn default() -> Self {
        Self(Mutex::new(render::TextCtx::new()))
    }
}

/// Render a single frame at `t_seconds`. If `downscale_width` is provided
/// and smaller than the layout's canvas width, the output is scaled down to
/// that width (preserving aspect ratio).
///
/// Returns a `data:image/png;base64,…` URL ready to drop into an <img> src.
#[tauri::command]
pub fn preview_frame(
    app_state: State<AppState>,
    text_state: State<TextState>,
    t_seconds: f64,
    downscale_width: Option<u32>,
) -> Result<String, String> {
    // Snapshot layout dimensions (small copy, release the lock quickly)
    let (src_w, src_h) = app_state
        .with_layout(|l| (l.canvas.width, l.canvas.height))
        .ok_or("no layout loaded")?;

    let mut pixmap = Pixmap::new(src_w, src_h).ok_or("pixmap alloc failed")?;
    let mut text = text_state.0.lock().map_err(|e| e.to_string())?;

    // Render into full-res pixmap.
    app_state
        .with_both(|layout, activity| {
            render::render_frame(
                layout,
                activity,
                Duration::from_secs_f64(t_seconds.max(0.0)),
                &mut text,
                &mut pixmap,
                Color::TRANSPARENT,
            )
        })
        .ok_or("no activity/layout loaded")?
        .map_err(|e| e.to_string())?;

    drop(text); // release TextCtx lock early

    // Optionally downscale into a second pixmap.
    let png_bytes = if let Some(dw) = downscale_width {
        if dw < src_w {
            let scale = dw as f32 / src_w as f32;
            let dh = (src_h as f32 * scale).round().max(1.0) as u32;
            let mut small = Pixmap::new(dw, dh).ok_or("downscale pixmap alloc failed")?;
            small.draw_pixmap(
                0,
                0,
                pixmap.as_ref(),
                &tiny_skia::PixmapPaint::default(),
                tiny_skia::Transform::from_scale(scale, scale),
                None,
            );
            small.encode_png().map_err(|e| e.to_string())?
        } else {
            pixmap.encode_png().map_err(|e| e.to_string())?
        }
    } else {
        pixmap.encode_png().map_err(|e| e.to_string())?
    };

    let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
    Ok(format!("data:image/png;base64,{}", b64))
}
