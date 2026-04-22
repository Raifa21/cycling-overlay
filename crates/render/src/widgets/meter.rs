use activity::{Activity, Metric, Sample};
use layout::{Indicator, IndicatorKind, Orientation, Rect, Theme, Ticks};
use std::time::Duration;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::widgets::scale::frac;

/// Render a linear meter widget into `pixmap`.
///
/// v1 (this task): horizontal orientation, `IndicatorKind::Fill` only. Ticks
/// and other indicators land in later tasks; callers must pass their defaults
/// for those fields but the values are currently ignored.
#[allow(clippy::too_many_arguments)]
pub fn render_meter(
    pixmap: &mut Pixmap,
    theme: &Theme,
    rect: Rect,
    metric_name: &str,
    min: f32,
    max: f32,
    _orientation: Orientation,   // Task 5
    indicator: Indicator,
    _ticks: Ticks,               // Task 5
    _show_value: bool,           // Task 6
    activity: &Activity,
    t: Duration,
) {
    let Some(metric) = Metric::from_str(metric_name) else {
        return;
    };
    let sample = activity.sample_at(t);
    let Some(current) = pull_value(metric, &sample) else {
        return;
    };

    let fg = super::parse_hex(&theme.fg).unwrap_or(Color::WHITE);
    let accent = super::parse_hex(&theme.accent).unwrap_or(fg);

    let f = frac(current, min, max);

    // Track geometry: centered band within the rect, thickness = half the
    // smaller dimension so numbers (future) have space above/below.
    let thickness = (rect.h as f32 * 0.5).min(rect.w as f32 * 0.5);
    let track_y = rect.y as f32 + (rect.h as f32 - thickness) * 0.5;
    let track_x = rect.x as f32;
    let track_w = rect.w as f32;

    // Fill portion.
    if matches!(indicator.kind, IndicatorKind::Fill) || indicator.fill_under {
        let filled_w = track_w * f;
        if filled_w > 0.0 {
            let mut pb = PathBuilder::new();
            pb.push_rect(
                tiny_skia::Rect::from_xywh(track_x, track_y, filled_w, thickness).unwrap(),
            );
            if let Some(path) = pb.finish() {
                let mut paint = Paint::default();
                paint.set_color(accent);
                paint.anti_alias = true;
                pixmap.fill_path(
                    &path,
                    &paint,
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }
    }

    // Track outline (always drawn).
    let mut pb = PathBuilder::new();
    pb.push_rect(tiny_skia::Rect::from_xywh(track_x, track_y, track_w, thickness).unwrap());
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(fg);
        paint.anti_alias = true;
        let stroke = Stroke {
            width: 1.5,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }
}

/// Pull a scalar value from `sample` for a given metric. Speed is returned in
/// km/h (the common display unit); later tasks wire `Units`-aware conversion.
/// Returns `None` if the metric has no value on the sample.
pub(crate) fn pull_value(m: Metric, s: &Sample) -> Option<f32> {
    match m {
        Metric::Speed => s.speed_mps.map(|v| v * 3.6),
        Metric::HeartRate => s.heart_rate_bpm.map(|v| v as f32),
        Metric::Power => s.power_w.map(|v| v as f32),
        Metric::Cadence => s.cadence_rpm.map(|v| v as f32),
        Metric::Altitude => s.altitude_m,
        Metric::Distance => s.distance_m.map(|v| v as f32),
        Metric::Gradient => s.gradient_pct,
        Metric::ElevGain => s.elev_gain_cum_m,
        _ => None, // TimeElapsed/TimeOfDay/PowerToWeight — not supported by Meter/Gauge yet.
    }
}
