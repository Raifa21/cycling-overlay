use activity::{Activity, Metric};
use layout::{DistanceUnit, ElevationUnit, Rect, Rider, Theme, Units};
use std::time::Duration;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::text::TextCtx;

/// Render a horizontal progress-bar widget.
///
/// The bar fills from left to right. Current value, min, and max are all in
/// the metric's base unit (meters for distance/elev_gain, watts for power,
/// etc.). Display formatting for `show_text` uses `units` for distance
/// conversions. For `distance` and `elev_gain`, `min` defaults to 0 and
/// `max` auto-computes from the activity's final value when not provided.
/// Other metrics require explicit `min` and `max`; otherwise the widget
/// renders nothing.
#[allow(clippy::too_many_arguments)]
pub fn render_bar(
    pixmap: &mut Pixmap,
    text_ctx: &mut TextCtx,
    theme: &Theme,
    units: &Units,
    _rider: Option<&Rider>,
    rect: Rect,
    metric_name: &str,
    min: Option<f32>,
    max: Option<f32>,
    show_text: bool,
    decimals: u32,
    activity: &Activity,
    t: Duration,
) {
    let Some(metric) = Metric::from_str(metric_name) else {
        return;
    };
    let (min_v, max_v, current) = match resolve_range(metric, min, max, activity, t) {
        Some(v) => v,
        None => return,
    };
    let span = (max_v - min_v).max(1e-6);
    let frac = ((current - min_v) / span).clamp(0.0, 1.0);

    let fg = super::parse_hex(&theme.fg).unwrap_or(Color::WHITE);
    let accent = super::parse_hex(&theme.accent).unwrap_or(fg);

    let rx = rect.x as f32;
    let ry = rect.y as f32;
    let rw = rect.w as f32;
    let rh = rect.h as f32;

    // Filled portion.
    let filled_w = rw * frac;
    if filled_w > 0.0 {
        let mut pb = PathBuilder::new();
        pb.push_rect(tiny_skia::Rect::from_xywh(rx, ry, filled_w, rh).unwrap());
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

    // Outline.
    let mut pb = PathBuilder::new();
    pb.push_rect(tiny_skia::Rect::from_xywh(rx, ry, rw, rh).unwrap());
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(fg);
        paint.anti_alias = true;
        let stroke = Stroke {
            width: 2.0,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    if show_text {
        let text = format_progress(metric, current, max_v, units, decimals);
        // Text sized to roughly 60% of bar height, padded 8px from left.
        let font_size = rh * 0.6;
        let ty = ry + (rh - font_size) * 0.5;
        text_ctx.draw(pixmap, &text, rx + 8.0, ty, font_size, fg);
    }
}

/// Resolve (min, max, current) for the bar, all in the metric's base unit.
/// Returns None when the metric isn't bar-able or required bounds are missing.
fn resolve_range(
    metric: Metric,
    min: Option<f32>,
    max: Option<f32>,
    activity: &Activity,
    t: Duration,
) -> Option<(f32, f32, f32)> {
    let sample = activity.sample_at(t);
    match metric {
        Metric::Distance => {
            let current = sample.distance_m? as f32;
            let total = activity.samples.iter().rev().find_map(|s| s.distance_m)? as f32;
            Some((min.unwrap_or(0.0), max.unwrap_or(total), current))
        }
        Metric::ElevGain => {
            let current = sample.elev_gain_cum_m?;
            let total = activity
                .samples
                .iter()
                .rev()
                .find_map(|s| s.elev_gain_cum_m)?;
            Some((min.unwrap_or(0.0), max.unwrap_or(total), current))
        }
        _ => {
            // Non-auto metrics: require explicit min + max.
            let lo = min?;
            let hi = max?;
            let current = match metric {
                Metric::Speed => sample.speed_mps?,
                Metric::HeartRate => sample.heart_rate_bpm? as f32,
                Metric::Power => sample.power_w? as f32,
                Metric::Cadence => sample.cadence_rpm? as f32,
                Metric::Altitude => sample.altitude_m?,
                Metric::Gradient => sample.gradient_pct?,
                _ => return None, // TimeElapsed/TimeOfDay/PowerToWeight: not supported
            };
            Some((lo, hi, current))
        }
    }
}

fn format_progress(metric: Metric, current: f32, max: f32, units: &Units, decimals: u32) -> String {
    let dec = decimals as usize;
    match metric {
        Metric::Distance => {
            let (cur, mx, unit) = match units.distance {
                DistanceUnit::Km => (current / 1000.0, max / 1000.0, "km"),
                DistanceUnit::Mi => (current / 1609.344, max / 1609.344, "mi"),
            };
            format!("{:.*} / {:.*} {}", dec, cur, dec, mx, unit)
        }
        Metric::ElevGain => {
            let (cur, mx, unit) = match units.elevation {
                ElevationUnit::M => (current, max, "m"),
                ElevationUnit::Ft => (current * 3.280_84, max * 3.280_84, "ft"),
            };
            format!("{:.*} / {:.*} {}", dec, cur, dec, mx, unit)
        }
        _ => format!("{:.*} / {:.*}", dec, current, dec, max),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use activity::{Activity, Sample};
    use chrono::Utc;

    fn make_activity(distances: &[f64]) -> Activity {
        let samples: Vec<Sample> = distances
            .iter()
            .enumerate()
            .map(|(i, d)| Sample {
                t: Duration::from_secs(i as u64),
                lat: 0.0,
                lon: 0.0,
                altitude_m: None,
                speed_mps: None,
                heart_rate_bpm: None,
                cadence_rpm: None,
                power_w: None,
                distance_m: Some(*d),
                elev_gain_cum_m: None,
                gradient_pct: None,
            })
            .collect();
        Activity::from_samples(Utc::now(), samples)
    }

    #[test]
    fn distance_auto_resolves_range() {
        let a = make_activity(&[0.0, 500.0, 1000.0, 2000.0]);
        let r = resolve_range(Metric::Distance, None, None, &a, Duration::from_secs(1)).unwrap();
        assert!((r.0 - 0.0).abs() < 0.01);
        assert!((r.1 - 2000.0).abs() < 0.01);
        assert!((r.2 - 500.0).abs() < 0.01);
    }

    #[test]
    fn format_progress_distance_km() {
        let units = Units {
            speed: layout::SpeedUnit::Kmh,
            distance: DistanceUnit::Km,
            elevation: ElevationUnit::M,
            temp: layout::TempUnit::C,
        };
        let s = format_progress(Metric::Distance, 1234.0, 5678.0, &units, 1);
        assert_eq!(s, "1.2 / 5.7 km");
    }

    #[test]
    fn unknown_metric_returns_none() {
        let a = make_activity(&[0.0, 100.0]);
        assert!(resolve_range(Metric::TimeElapsed, None, None, &a, Duration::ZERO).is_none());
        assert!(resolve_range(Metric::PowerToWeight, None, None, &a, Duration::ZERO).is_none());
    }
}
