use activity::{Activity, Metric};
use layout::{Indicator, IndicatorKind, Rect, Theme, Ticks, Units};
use std::time::Duration;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::text::TextCtx;
use crate::widgets::meter::pull_value;
use crate::widgets::scale::{angle_lerp, frac, to_skia_angle};

/// Render a radial gauge widget.
///
/// v1 (this task): default 270° arc, Fill indicator only, no ticks / markers /
/// center label. Later tasks extend this to major/minor ticks (Task 8),
/// Rect/Arrow/Needle markers, fill_under combined mode, and show_value
/// (Task 9).
#[allow(clippy::too_many_arguments)]
pub fn render_gauge(
    pixmap: &mut Pixmap,
    _text_ctx: &mut TextCtx, // Task 8 (tick numbers) + Task 9 (show_value)
    theme: &Theme,
    units: &Units,
    rect: Rect,
    metric_name: &str,
    min: f32,
    max: f32,
    start_deg: f32,
    end_deg: f32,
    indicator: Indicator,
    _ticks: Ticks, // Task 8
    _show_value: bool, // Task 9
    _value_font_size: Option<f32>,
    activity: &Activity,
    t: Duration,
) {
    let Some(metric) = Metric::from_str(metric_name) else {
        return;
    };
    let sample = activity.sample_at(t);
    let current = pull_value(metric, &sample, units);

    let fg = super::parse_hex(&theme.fg).unwrap_or(Color::WHITE);
    let accent = super::parse_hex(&theme.accent).unwrap_or(fg);

    // Geometry: largest centered square inside rect, with padding to keep
    // the arc (and future tick numbers) inside the rect.
    let cx = rect.x as f32 + rect.w as f32 * 0.5;
    let cy = rect.y as f32 + rect.h as f32 * 0.5;
    let padding = 8.0;
    let r_outer = (rect.w.min(rect.h) as f32) * 0.5 - padding;
    let thickness = (r_outer * 0.15).max(4.0);
    let r_center = r_outer - thickness * 0.5;

    // Full arc (track): stroke the polyline from start_deg to end_deg.
    draw_arc_stroke(pixmap, cx, cy, r_center, thickness, start_deg, end_deg, fg);

    // Fill portion: stroke a shorter arc from start_deg to the current angle,
    // using accent color. Skip when metric is missing.
    if let Some(v) = current {
        if matches!(indicator.kind, IndicatorKind::Fill) || indicator.fill_under {
            let f = frac(v, min, max);
            if f > 0.0 {
                let cur_deg = angle_lerp(start_deg, end_deg, f);
                draw_arc_stroke(
                    pixmap, cx, cy, r_center, thickness, start_deg, cur_deg, accent,
                );
            }
        }
    }
}

/// Stroke a circular arc from `start_deg_user` to `end_deg_user` (user
/// convention: 0° up, CW+) at the given center and radius. Approximates the
/// arc with a polyline — roughly 2° per segment is smooth at overlay
/// resolutions (~180 segments per full 360° sweep).
fn draw_arc_stroke(
    pixmap: &mut Pixmap,
    cx: f32,
    cy: f32,
    radius: f32,
    stroke_w: f32,
    start_deg_user: f32,
    end_deg_user: f32,
    color: Color,
) {
    // Normalize the end angle so a "less than start" end means "wraps through top."
    let user_start = start_deg_user;
    let user_end = if end_deg_user >= start_deg_user {
        end_deg_user
    } else {
        end_deg_user + 360.0
    };
    let span = (user_end - user_start).abs();
    if span < 1e-4 {
        return; // degenerate sweep
    }
    let steps = ((span / 2.0).ceil() as i32).max(1);

    let mut pb = PathBuilder::new();
    for i in 0..=steps {
        let u = user_start + (user_end - user_start) * (i as f32 / steps as f32);
        let s = to_skia_angle(u).to_radians();
        let x = cx + radius * s.cos();
        let y = cy - radius * s.sin(); // flip math-y to screen-y
        if i == 0 {
            pb.move_to(x, y);
        } else {
            pb.line_to(x, y);
        }
    }
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        let stroke = Stroke {
            width: stroke_w,
            line_cap: tiny_skia::LineCap::Butt,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

#[cfg(test)]
mod tests {
    // Defer visual correctness to the golden test; no pure-math unit tests
    // needed for this task.
}
