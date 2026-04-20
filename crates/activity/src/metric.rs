use crate::Sample;

/// Enumerates every metric a widget can reference by string.
///
/// Kept in sync with `Sample`'s field set. Each variant serializes to its
/// `widget.metric` string name (lowercase, snake_case).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Metric {
    Speed,
    HeartRate,
    Power,
    Cadence,
    Altitude,
    Distance,
    ElevGain,
    Gradient,
    TimeElapsed,   // synthetic — derivable from Sample.t
    TimeOfDay,     // synthetic — derivable from Activity.start_time + Sample.t
    PowerToWeight, // synthetic — power ÷ rider weight (layout-configured)
}

impl Metric {
    pub fn as_str(self) -> &'static str {
        match self {
            Metric::Speed => "speed",
            Metric::HeartRate => "heart_rate",
            Metric::Power => "power",
            Metric::Cadence => "cadence",
            Metric::Altitude => "altitude",
            Metric::Distance => "distance",
            Metric::ElevGain => "elev_gain",
            Metric::Gradient => "gradient",
            Metric::TimeElapsed => "time_elapsed",
            Metric::TimeOfDay => "time_of_day",
            Metric::PowerToWeight => "w_per_kg",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Metric> {
        match s {
            "speed" => Some(Metric::Speed),
            "heart_rate" | "hr" => Some(Metric::HeartRate),
            "power" => Some(Metric::Power),
            "cadence" => Some(Metric::Cadence),
            "altitude" | "elevation" => Some(Metric::Altitude),
            "distance" => Some(Metric::Distance),
            "elev_gain" | "elevation_gain" => Some(Metric::ElevGain),
            "gradient" => Some(Metric::Gradient),
            "time_elapsed" => Some(Metric::TimeElapsed),
            "time_of_day" => Some(Metric::TimeOfDay),
            "w_per_kg" | "power_to_weight" => Some(Metric::PowerToWeight),
            _ => None,
        }
    }

    pub const ALL: [Metric; 11] = [
        Metric::Speed,
        Metric::HeartRate,
        Metric::Power,
        Metric::Cadence,
        Metric::Altitude,
        Metric::Distance,
        Metric::ElevGain,
        Metric::Gradient,
        Metric::TimeElapsed,
        Metric::TimeOfDay,
        Metric::PowerToWeight,
    ];
}

/// Predicate: is this metric present on at least one sample of the activity?
///
/// Synthetic metrics (`TimeElapsed`, `TimeOfDay`) are always true.
pub fn metric_present_on_activity(m: Metric, samples: &[Sample]) -> bool {
    match m {
        Metric::Speed => samples.iter().any(|s| s.speed_mps.is_some()),
        Metric::HeartRate => samples.iter().any(|s| s.heart_rate_bpm.is_some()),
        Metric::Power => samples.iter().any(|s| s.power_w.is_some()),
        Metric::Cadence => samples.iter().any(|s| s.cadence_rpm.is_some()),
        Metric::Altitude => samples.iter().any(|s| s.altitude_m.is_some()),
        Metric::Distance => samples.iter().any(|s| s.distance_m.is_some()),
        Metric::ElevGain => samples.iter().any(|s| s.elev_gain_cum_m.is_some()),
        Metric::Gradient => samples.iter().any(|s| s.gradient_pct.is_some()),
        Metric::TimeElapsed | Metric::TimeOfDay => true,
        // PowerToWeight is available iff power is — the weight half of the
        // ratio comes from the layout's rider config, which is validated
        // elsewhere (runtime falls back to "--" if weight is missing).
        Metric::PowerToWeight => samples.iter().any(|s| s.power_w.is_some()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_string_round_trip() {
        for m in Metric::ALL {
            assert_eq!(Metric::from_str(m.as_str()), Some(m));
        }
    }

    #[test]
    fn hr_and_heart_rate_both_parse() {
        assert_eq!(Metric::from_str("hr"), Some(Metric::HeartRate));
        assert_eq!(Metric::from_str("heart_rate"), Some(Metric::HeartRate));
    }
}
