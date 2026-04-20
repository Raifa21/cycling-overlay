use activity::Activity;
use layout::{Rect, Theme};
use std::time::Duration;
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Render a course polyline + moving dot into `pixmap`.
///
/// Projects lat/lon using a simple equirectangular projection, scaling
/// longitude by `cos(center_lat)` so the aspect looks roughly right for
/// small regions. The bbox of the activity's samples is fit into `rect`
/// with letterboxing so the polyline is centered and preserves its
/// native aspect ratio. North is up (lat axis inverted).
///
/// Indoor / no-GPS activities (empty samples, or all samples at the same
/// point) render nothing rather than panicking.
pub fn render_course(
    pixmap: &mut Pixmap,
    theme: &Theme,
    rect: Rect,
    line_width: f32,
    dot_radius: f32,
    activity: &Activity,
    t: Duration,
) {
    if activity.samples.is_empty() {
        return;
    }

    // Compute bbox over all samples.
    let mut min_lat = f64::INFINITY;
    let mut max_lat = f64::NEG_INFINITY;
    let mut min_lon = f64::INFINITY;
    let mut max_lon = f64::NEG_INFINITY;
    for s in &activity.samples {
        if s.lat < min_lat {
            min_lat = s.lat;
        }
        if s.lat > max_lat {
            max_lat = s.lat;
        }
        if s.lon < min_lon {
            min_lon = s.lon;
        }
        if s.lon > max_lon {
            max_lon = s.lon;
        }
    }

    // Degenerate bbox (indoor, no GPS fix, or all-zero samples): bail.
    let lat_span = max_lat - min_lat;
    let lon_span = max_lon - min_lon;
    if lat_span < 1e-9 && lon_span < 1e-9 {
        return;
    }

    // Equirectangular: scale lon by cos(center_lat) for visual aspect.
    let center_lat = (min_lat + max_lat) / 2.0;
    let x_scale = center_lat.to_radians().cos().max(0.01); // avoid div-by-zero near poles
    let world_w = lon_span * x_scale;
    let world_h = lat_span;

    // Fit into rect preserving aspect ratio (letterbox).
    let rx = rect.x as f32;
    let ry = rect.y as f32;
    let rw = rect.w as f32;
    let rh = rect.h as f32;
    let rect_aspect = rw as f64 / rh.max(1e-6) as f64;
    let world_aspect = world_w / world_h.max(1e-12);

    let (scale, offset_x, offset_y) = if world_aspect >= rect_aspect {
        // World is wider than rect — width-constrained.
        let s = rw as f64 / world_w.max(1e-12);
        let drawn_h = (world_h * s) as f32;
        (s, 0.0_f32, (rh - drawn_h) / 2.0)
    } else {
        // World is taller than rect — height-constrained.
        let s = rh as f64 / world_h.max(1e-12);
        let drawn_w = (world_w * s) as f32;
        (s, (rw - drawn_w) / 2.0, 0.0_f32)
    };

    let project = |lat: f64, lon: f64| -> (f32, f32) {
        let x_world = (lon - min_lon) * x_scale;
        let y_world = max_lat - lat; // invert so north is up
        let x = rx + offset_x + (x_world * scale) as f32;
        let y = ry + offset_y + (y_world * scale) as f32;
        (x, y)
    };

    // Theme colors.
    let fg = crate::widgets::parse_hex(&theme.fg).unwrap_or(Color::WHITE);
    let accent = crate::widgets::parse_hex(&theme.accent).unwrap_or(fg);

    // Build the polyline path.
    let mut pb = PathBuilder::new();
    let mut first = true;
    for s in &activity.samples {
        let (x, y) = project(s.lat, s.lon);
        if first {
            pb.move_to(x, y);
            first = false;
        } else {
            pb.line_to(x, y);
        }
    }
    if let Some(path) = pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(fg);
        paint.anti_alias = true;
        let stroke = Stroke {
            width: line_width,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    // Dot at the interpolated current position.
    let cur = activity.sample_at(t);
    let (cx, cy) = project(cur.lat, cur.lon);
    let mut pb_dot = PathBuilder::new();
    pb_dot.push_circle(cx, cy, dot_radius);
    if let Some(path) = pb_dot.finish() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use activity::Sample;
    use chrono::{TimeZone, Utc};

    fn theme() -> Theme {
        Theme {
            font: "Inter".into(),
            fg: "#ffffff".into(),
            accent: "#ffcc00".into(),
            shadow: None,
        }
    }

    fn mk(secs: u64, lat: f64, lon: f64) -> Sample {
        Sample {
            t: Duration::from_secs(secs),
            lat,
            lon,
            altitude_m: None,
            speed_mps: None,
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_w: None,
            distance_m: None,
            elev_gain_cum_m: None,
            gradient_pct: None,
        }
    }

    #[test]
    fn empty_activity_renders_nothing() {
        let mut pix = Pixmap::new(64, 64).unwrap();
        let activity = Activity::from_samples(Utc.timestamp_opt(0, 0).unwrap(), vec![]);
        render_course(
            &mut pix,
            &theme(),
            Rect {
                x: 0,
                y: 0,
                w: 64,
                h: 64,
            },
            2.0,
            4.0,
            &activity,
            Duration::ZERO,
        );
        assert!(pix.data().chunks_exact(4).all(|p| p[3] == 0));
    }

    #[test]
    fn indoor_activity_renders_nothing() {
        let mut pix = Pixmap::new(64, 64).unwrap();
        let samples = vec![mk(0, 0.0, 0.0), mk(10, 0.0, 0.0), mk(20, 0.0, 0.0)];
        let activity = Activity::from_samples(Utc.timestamp_opt(0, 0).unwrap(), samples);
        render_course(
            &mut pix,
            &theme(),
            Rect {
                x: 0,
                y: 0,
                w: 64,
                h: 64,
            },
            2.0,
            4.0,
            &activity,
            Duration::from_secs(5),
        );
        assert!(pix.data().chunks_exact(4).all(|p| p[3] == 0));
    }

    #[test]
    fn single_sample_renders_nothing() {
        // Single sample → bbox has zero span → bail.
        let mut pix = Pixmap::new(64, 64).unwrap();
        let samples = vec![mk(0, 35.0, 139.0)];
        let activity = Activity::from_samples(Utc.timestamp_opt(0, 0).unwrap(), samples);
        render_course(
            &mut pix,
            &theme(),
            Rect {
                x: 0,
                y: 0,
                w: 64,
                h: 64,
            },
            2.0,
            4.0,
            &activity,
            Duration::ZERO,
        );
        assert!(pix.data().chunks_exact(4).all(|p| p[3] == 0));
    }

    #[test]
    fn real_track_draws_some_pixels() {
        let mut pix = Pixmap::new(64, 64).unwrap();
        let samples = vec![
            mk(0, 0.0, 0.0),
            mk(10, 0.0, 0.001),
            mk(20, 0.001, 0.001),
            mk(30, 0.001, 0.0),
        ];
        let activity = Activity::from_samples(Utc.timestamp_opt(0, 0).unwrap(), samples);
        render_course(
            &mut pix,
            &theme(),
            Rect {
                x: 0,
                y: 0,
                w: 64,
                h: 64,
            },
            2.0,
            4.0,
            &activity,
            Duration::from_secs(15),
        );
        assert!(pix.data().chunks_exact(4).any(|p| p[3] > 0));
    }
}
