use activity::{Activity, Metric, Sample};
use layout::{DistanceUnit, ElevationUnit, Rect, Rider, SpeedUnit, Theme, Units};
use std::time::Duration;
use tiny_skia::{Color, Pixmap};

use crate::text::TextCtx;

/// Render a readout widget into `pixmap`.
#[allow(clippy::too_many_arguments)]
pub fn render_readout(
    pixmap: &mut Pixmap,
    text_ctx: &mut TextCtx,
    theme: &Theme,
    units: &Units,
    rider: Option<&Rider>,
    rect: Rect,
    metric_name: &str,
    label: &str,
    decimals: u32,
    font_size: f32,
    activity: &Activity,
    t: Duration,
) {
    let sample = activity.sample_at(t);
    let (value_str, unit_str) = match Metric::from_str(metric_name) {
        Some(m) => format_metric(m, &sample, units, rider, t, decimals),
        None => ("--".to_string(), ""),
    };

    // Colors: parse hex strings into tiny_skia::Color.
    let fg = super::parse_hex(&theme.fg).unwrap_or(Color::WHITE);
    let accent = super::parse_hex(&theme.accent).unwrap_or(fg);

    // Layout: label takes ~1/3 of height (top), value takes ~2/3 (below).
    let label_size = font_size * 0.35;
    let label_x = rect.x as f32;
    let label_y = rect.y as f32;
    let value_x = rect.x as f32;
    let value_y = rect.y as f32 + label_size * 1.4;

    text_ctx.draw(pixmap, label, label_x, label_y, label_size, accent);

    // Value + unit in one string: "42.5 km/h"
    let value_full = if unit_str.is_empty() {
        value_str
    } else {
        format!("{} {}", value_str, unit_str)
    };
    text_ctx.draw(pixmap, &value_full, value_x, value_y, font_size, fg);
}

fn format_metric(
    m: Metric,
    s: &Sample,
    units: &Units,
    rider: Option<&Rider>,
    t: Duration,
    decimals: u32,
) -> (String, &'static str) {
    let dec = decimals as usize;
    match m {
        Metric::Speed => match s.speed_mps {
            Some(mps) => match units.speed {
                SpeedUnit::Kmh => (format!("{:.*}", dec, mps * 3.6), "km/h"),
                SpeedUnit::Mph => (format!("{:.*}", dec, mps * 2.236_936_3), "mph"),
            },
            None => ("--".into(), ""),
        },
        Metric::HeartRate => match s.heart_rate_bpm {
            Some(v) => (format!("{}", v), "bpm"),
            None => ("--".into(), ""),
        },
        Metric::Power => match s.power_w {
            Some(v) => (format!("{}", v), "W"),
            None => ("--".into(), ""),
        },
        Metric::Cadence => match s.cadence_rpm {
            Some(v) => (format!("{}", v), "rpm"),
            None => ("--".into(), ""),
        },
        Metric::Altitude => match s.altitude_m {
            Some(m) => match units.elevation {
                ElevationUnit::M => (format!("{:.*}", dec, m), "m"),
                ElevationUnit::Ft => (format!("{:.*}", dec, m * 3.280_84), "ft"),
            },
            None => ("--".into(), ""),
        },
        Metric::Distance => match s.distance_m {
            Some(v) => match units.distance {
                DistanceUnit::Km => (format!("{:.*}", dec, v / 1000.0), "km"),
                DistanceUnit::Mi => (format!("{:.*}", dec, v / 1609.344), "mi"),
            },
            None => ("--".into(), ""),
        },
        Metric::ElevGain => match s.elev_gain_cum_m {
            Some(v) => match units.elevation {
                ElevationUnit::M => (format!("{:.*}", dec, v), "m"),
                ElevationUnit::Ft => (format!("{:.*}", dec, v * 3.280_84), "ft"),
            },
            None => ("--".into(), ""),
        },
        Metric::Gradient => match s.gradient_pct {
            Some(v) => (format!("{:.*}", dec, v), "%"),
            None => ("--".into(), ""),
        },
        Metric::TimeElapsed => (format_duration(t), ""),
        Metric::TimeOfDay => (format_duration(t), ""), // placeholder — v1 ignores start_time
        Metric::PowerToWeight => match (s.power_w, rider.map(|r| r.weight_kg)) {
            (Some(p), Some(w)) if w > 0.0 => (format!("{:.*}", dec, p as f32 / w), "W/kg"),
            _ => ("--".into(), ""),
        },
    }
}

fn format_duration(t: Duration) -> String {
    let total = t.as_secs();
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let sec = total % 60;
    if h > 0 {
        format!("{}:{:02}:{:02}", h, m, sec)
    } else {
        format!("{}:{:02}", m, sec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_under_hour() {
        assert_eq!(format_duration(Duration::from_secs(65)), "1:05");
    }

    #[test]
    fn format_duration_over_hour() {
        assert_eq!(format_duration(Duration::from_secs(3725)), "1:02:05");
    }

    #[test]
    fn format_metric_speed_kmh() {
        let s = Sample {
            t: Duration::ZERO,
            lat: 0.0,
            lon: 0.0,
            altitude_m: None,
            speed_mps: Some(10.0), // 36 km/h
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_w: None,
            distance_m: None,
            elev_gain_cum_m: None,
            gradient_pct: None,
        };
        let units = Units {
            speed: SpeedUnit::Kmh,
            distance: DistanceUnit::Km,
            elevation: ElevationUnit::M,
            temp: layout::TempUnit::C,
        };
        let (v, u) = format_metric(Metric::Speed, &s, &units, None, Duration::ZERO, 1);
        assert_eq!(v, "36.0");
        assert_eq!(u, "km/h");
    }

    #[test]
    fn format_metric_speed_missing() {
        let s = Sample {
            t: Duration::ZERO,
            lat: 0.0,
            lon: 0.0,
            altitude_m: None,
            speed_mps: None,
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_w: None,
            distance_m: None,
            elev_gain_cum_m: None,
            gradient_pct: None,
        };
        let units = Units {
            speed: SpeedUnit::Kmh,
            distance: DistanceUnit::Km,
            elevation: ElevationUnit::M,
            temp: layout::TempUnit::C,
        };
        let (v, u) = format_metric(Metric::Speed, &s, &units, None, Duration::ZERO, 1);
        assert_eq!(v, "--");
        assert_eq!(u, "");
    }

    #[test]
    fn format_metric_distance_km() {
        let s = Sample {
            t: Duration::ZERO,
            lat: 0.0,
            lon: 0.0,
            altitude_m: None,
            speed_mps: None,
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_w: None,
            distance_m: Some(2500.0),
            elev_gain_cum_m: None,
            gradient_pct: None,
        };
        let units = Units {
            speed: SpeedUnit::Kmh,
            distance: DistanceUnit::Km,
            elevation: ElevationUnit::M,
            temp: layout::TempUnit::C,
        };
        let (v, u) = format_metric(Metric::Distance, &s, &units, None, Duration::ZERO, 2);
        assert_eq!(v, "2.50");
        assert_eq!(u, "km");
    }

    #[test]
    fn format_metric_w_per_kg() {
        let s = Sample {
            t: Duration::ZERO,
            lat: 0.0,
            lon: 0.0,
            altitude_m: None,
            speed_mps: None,
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_w: Some(220),
            distance_m: None,
            elev_gain_cum_m: None,
            gradient_pct: None,
        };
        let units = Units {
            speed: SpeedUnit::Kmh,
            distance: DistanceUnit::Km,
            elevation: ElevationUnit::M,
            temp: layout::TempUnit::C,
        };
        let rider = Rider { weight_kg: 73.3 };
        let (v, u) = format_metric(
            Metric::PowerToWeight,
            &s,
            &units,
            Some(&rider),
            Duration::ZERO,
            2,
        );
        assert_eq!(v, "3.00");
        assert_eq!(u, "W/kg");
    }

    #[test]
    fn format_metric_w_per_kg_missing_weight() {
        let s = Sample {
            t: Duration::ZERO,
            lat: 0.0,
            lon: 0.0,
            altitude_m: None,
            speed_mps: None,
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_w: Some(250),
            distance_m: None,
            elev_gain_cum_m: None,
            gradient_pct: None,
        };
        let units = Units {
            speed: SpeedUnit::Kmh,
            distance: DistanceUnit::Km,
            elevation: ElevationUnit::M,
            temp: layout::TempUnit::C,
        };
        let (v, u) = format_metric(Metric::PowerToWeight, &s, &units, None, Duration::ZERO, 1);
        assert_eq!(v, "--");
        assert_eq!(u, "");
    }

    #[test]
    fn format_metric_distance_mi() {
        let s = Sample {
            t: Duration::ZERO,
            lat: 0.0,
            lon: 0.0,
            altitude_m: None,
            speed_mps: None,
            heart_rate_bpm: None,
            cadence_rpm: None,
            power_w: None,
            distance_m: Some(1609.344),
            elev_gain_cum_m: None,
            gradient_pct: None,
        };
        let units = Units {
            speed: SpeedUnit::Kmh,
            distance: DistanceUnit::Mi,
            elevation: ElevationUnit::M,
            temp: layout::TempUnit::C,
        };
        let (v, u) = format_metric(Metric::Distance, &s, &units, None, Duration::ZERO, 2);
        assert_eq!(v, "1.00");
        assert_eq!(u, "mi");
    }
}
