use activity::Activity;
use layout::{Layout, Widget};
use std::time::Duration;
use tiny_skia::{Color, Pixmap};

/// Render one frame of the overlay into `pixmap`.
///
/// This is the pure per-frame entry point — given an immutable layout and
/// activity plus a time `t`, it clears the pixmap to transparent and draws
/// every widget from `layout.widgets` into it.
///
/// The caller is responsible for allocating the pixmap. `pixmap.width()` and
/// `pixmap.height()` must match `layout.canvas.width`/`height`.
#[allow(unused_variables)] // widget arms unfilled until Tasks 17-19
pub fn render_frame(
    layout: &Layout,
    activity: &Activity,
    t: Duration,
    pixmap: &mut Pixmap,
) -> anyhow::Result<()> {
    if pixmap.width() != layout.canvas.width || pixmap.height() != layout.canvas.height {
        anyhow::bail!(
            "pixmap size {}x{} does not match layout canvas {}x{}",
            pixmap.width(),
            pixmap.height(),
            layout.canvas.width,
            layout.canvas.height,
        );
    }
    pixmap.fill(Color::TRANSPARENT);

    for widget in &layout.widgets {
        match widget {
            Widget::Readout { .. } => {
                // Implemented in Task 17.
            }
            Widget::Course { .. } => {
                // Implemented in Task 18.
            }
            Widget::ElevationProfile { .. } => {
                // Implemented in Task 19.
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use activity::{Activity, Sample};
    use chrono::{TimeZone, Utc};
    use layout::{Canvas, DistanceUnit, ElevationUnit, Layout, SpeedUnit, TempUnit, Theme, Units};
    use std::time::Duration;
    use tiny_skia::Pixmap;

    fn minimal_layout(w: u32, h: u32, fps: u32) -> Layout {
        Layout {
            version: 1,
            canvas: Canvas {
                width: w,
                height: h,
                fps,
            },
            units: Units {
                speed: SpeedUnit::Kmh,
                distance: DistanceUnit::Km,
                elevation: ElevationUnit::M,
                temp: TempUnit::C,
            },
            theme: Theme {
                font: "Inter".into(),
                fg: "#ffffff".into(),
                accent: "#ffcc00".into(),
                shadow: None,
            },
            widgets: vec![],
        }
    }

    fn one_sample_activity() -> Activity {
        let s = Sample {
            t: Duration::ZERO,
            lat: 0.0,
            lon: 0.0,
            altitude_m: Some(100.0),
            speed_mps: Some(5.0),
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_w: None,
            distance_m: Some(0.0),
            elev_gain_cum_m: Some(0.0),
            gradient_pct: Some(0.0),
        };
        Activity::from_samples(Utc.timestamp_opt(0, 0).unwrap(), vec![s])
    }

    #[test]
    fn empty_layout_renders_transparent() {
        let layout = minimal_layout(100, 100, 30);
        let activity = one_sample_activity();
        let mut pix = Pixmap::new(100, 100).unwrap();
        // Pre-fill with red so a successful clear is observable.
        pix.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
        render_frame(&layout, &activity, Duration::ZERO, &mut pix).unwrap();
        // Every pixel must be fully transparent after render.
        assert!(
            pix.data().chunks_exact(4).all(|p| p[3] == 0),
            "found non-transparent pixel"
        );
    }

    #[test]
    fn render_frame_fails_on_mismatched_pixmap_size() {
        let layout = minimal_layout(200, 100, 30);
        let activity = one_sample_activity();
        let mut pix = Pixmap::new(100, 100).unwrap();
        let r = render_frame(&layout, &activity, Duration::ZERO, &mut pix);
        assert!(r.is_err());
    }
}
