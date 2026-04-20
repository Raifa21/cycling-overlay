use chrono::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct Sample {
    pub t: Duration,
    pub lat: f64,
    pub lon: f64,
    pub altitude_m: Option<f32>,
    pub speed_mps: Option<f32>,
    pub heart_rate_bpm: Option<u8>,
    pub cadence_rpm: Option<u8>,
    pub power_w: Option<u16>,
    pub distance_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    pub start_time: DateTime<Utc>,
    pub samples: Vec<Sample>,
}

impl Activity {
    pub fn from_samples(start_time: DateTime<Utc>, samples: Vec<Sample>) -> Self {
        Self { start_time, samples }
    }

    pub fn duration(&self) -> Duration {
        self.samples.last().map(|s| s.t).unwrap_or_default()
    }

    /// If any sample already has `distance_m`, assume all do and return unchanged.
    /// Otherwise, cumulate haversine distance between consecutive lat/lon pairs,
    /// starting from 0.0 on the first sample.
    pub fn fill_derived_distance(&mut self) {
        if self.samples.iter().all(|s| s.distance_m.is_some()) {
            return;
        }
        let mut acc = 0.0f64;
        for i in 0..self.samples.len() {
            if i == 0 {
                self.samples[0].distance_m = Some(0.0);
            } else {
                let prev = &self.samples[i - 1];
                let curr = &self.samples[i];
                let step = crate::geo::haversine_m(prev.lat, prev.lon, curr.lat, curr.lon);
                acc += step;
                self.samples[i].distance_m = Some(acc);
            }
        }
    }

    /// Fill `speed_mps` on samples where it is missing, by finite-differencing
    /// `distance_m` against `t`.
    ///
    /// - Requires `distance_m` to be populated — call `fill_derived_distance`
    ///   first when loading GPS-only data.
    /// - No-op for samples that already have a `speed_mps` value; per-sample
    ///   decision, not all-or-nothing.
    /// - Interior samples use a central difference: `(d[i+1] - d[i-1]) / (t[i+1] - t[i-1])`.
    /// - First and last samples use a one-sided (forward/backward) difference.
    /// - Single-sample activities leave speed unset.
    pub fn fill_derived_speed(&mut self) {
        let n = self.samples.len();
        if n < 2 {
            return;
        }
        for i in 0..n {
            if self.samples[i].speed_mps.is_some() {
                continue;
            }
            let (j_lo, j_hi) = if i == 0 {
                (0, 1)
            } else if i == n - 1 {
                (n - 2, n - 1)
            } else {
                (i - 1, i + 1)
            };
            let (Some(d_lo), Some(d_hi)) = (
                self.samples[j_lo].distance_m,
                self.samples[j_hi].distance_m,
            ) else {
                continue; // can't derive without both distances
            };
            let dt = self.samples[j_hi].t.as_secs_f64() - self.samples[j_lo].t.as_secs_f64();
            if dt <= 0.0 {
                continue;
            }
            let v = ((d_hi - d_lo) / dt) as f32;
            self.samples[i].speed_mps = Some(v);
        }
    }
}

#[cfg(test)]
impl Sample {
    /// Test helper: a blank Sample at t=0, (0.0, 0.0), all metrics None.
    pub(crate) fn blank() -> Self {
        Sample {
            t: std::time::Duration::ZERO,
            lat: 0.0, lon: 0.0,
            altitude_m: None, speed_mps: None,
            heart_rate_bpm: None, cadence_rpm: None,
            power_w: None, distance_m: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use std::time::Duration;

    #[test]
    fn from_samples_builds_activity() {
        let samples = vec![
            Sample { t: Duration::from_secs(0), lat: 0.0, lon: 0.0,
                     altitude_m: Some(100.0), speed_mps: None,
                     heart_rate_bpm: None, cadence_rpm: None,
                     power_w: None, distance_m: None },
        ];
        let a = Activity::from_samples(Utc.timestamp_opt(0, 0).unwrap(), samples);
        assert_eq!(a.samples.len(), 1);
        assert_eq!(a.duration(), Duration::from_secs(0));
    }

    #[test]
    fn fill_distance_cumulates() {
        use chrono::Utc;
        use std::time::Duration;
        let samples = vec![
            Sample { t: Duration::ZERO, lat: 0.0, lon: 0.0, ..Sample::blank() },
            Sample {
                t: Duration::from_secs(1),
                lat: 0.0,
                lon: 0.001, // ~111 m east at equator (cos(0)=1)
                ..Sample::blank()
            },
        ];
        let mut a = Activity::from_samples(Utc::now(), samples);
        a.fill_derived_distance();
        assert_eq!(a.samples[0].distance_m, Some(0.0));
        let d1 = a.samples[1].distance_m.unwrap();
        assert!(d1 > 100.0 && d1 < 120.0, "got {}", d1);
    }

    #[test]
    fn fill_distance_noop_if_all_present() {
        use chrono::Utc;
        use std::time::Duration;
        let samples = vec![
            Sample { t: Duration::ZERO, lat: 0.0, lon: 0.0, distance_m: Some(5.0), ..Sample::blank() },
            Sample { t: Duration::from_secs(1), lat: 0.0, lon: 0.001, distance_m: Some(20.0), ..Sample::blank() },
        ];
        let mut a = Activity::from_samples(Utc::now(), samples);
        a.fill_derived_distance();
        assert_eq!(a.samples[0].distance_m, Some(5.0));
        assert_eq!(a.samples[1].distance_m, Some(20.0));
    }

    #[test]
    fn fill_speed_from_constant_distance_rate() {
        // 11 samples at 1 Hz, distance grows 10 m/s (0, 10, 20, ..., 100).
        let samples: Vec<Sample> = (0..11)
            .map(|i| Sample {
                t: Duration::from_secs(i as u64),
                lat: 0.0, lon: 0.0,
                distance_m: Some(i as f64 * 10.0),
                ..Sample::blank()
            })
            .collect();
        let mut a = Activity::from_samples(Utc::now(), samples);
        a.fill_derived_speed();
        // Middle samples should have speed very close to 10 m/s.
        for s in &a.samples[1..a.samples.len() - 1] {
            let v = s.speed_mps.unwrap();
            assert!((v - 10.0).abs() < 0.01, "got {}", v);
        }
    }

    #[test]
    fn fill_speed_noop_when_present() {
        let samples = vec![
            Sample {
                t: Duration::ZERO, lat: 0.0, lon: 0.0,
                distance_m: Some(0.0), speed_mps: Some(5.0),
                ..Sample::blank()
            },
            Sample {
                t: Duration::from_secs(1), lat: 0.0, lon: 0.0,
                distance_m: Some(10.0), speed_mps: Some(5.0),
                ..Sample::blank()
            },
        ];
        let mut a = Activity::from_samples(Utc::now(), samples);
        a.fill_derived_speed();
        assert_eq!(a.samples[0].speed_mps, Some(5.0));
        assert_eq!(a.samples[1].speed_mps, Some(5.0));
    }

    #[test]
    fn fill_speed_endpoints_use_one_sided_difference() {
        // Constant 10 m/s: endpoints should use forward/backward difference and
        // still land on ~10.0.
        let samples: Vec<Sample> = (0..11)
            .map(|i| Sample {
                t: Duration::from_secs(i as u64),
                lat: 0.0, lon: 0.0,
                distance_m: Some(i as f64 * 10.0),
                ..Sample::blank()
            })
            .collect();
        let mut a = Activity::from_samples(Utc::now(), samples);
        a.fill_derived_speed();
        assert!((a.samples[0].speed_mps.unwrap() - 10.0).abs() < 0.01);
        assert!((a.samples[10].speed_mps.unwrap() - 10.0).abs() < 0.01);
    }

    #[test]
    fn fill_speed_irregular_dt() {
        // t = [0, 1, 3], d = [0, 5, 25] → at i=1 central diff = (25-0)/(3-0) = 8.333
        let samples = vec![
            Sample { t: Duration::from_secs(0), lat: 0.0, lon: 0.0, distance_m: Some(0.0),  ..Sample::blank() },
            Sample { t: Duration::from_secs(1), lat: 0.0, lon: 0.0, distance_m: Some(5.0),  ..Sample::blank() },
            Sample { t: Duration::from_secs(3), lat: 0.0, lon: 0.0, distance_m: Some(25.0), ..Sample::blank() },
        ];
        let mut a = Activity::from_samples(Utc::now(), samples);
        a.fill_derived_speed();
        let v = a.samples[1].speed_mps.unwrap();
        assert!((v - 25.0 / 3.0).abs() < 0.01, "got {}", v);
    }

    #[test]
    fn fill_speed_skips_when_neighbor_distance_missing() {
        // Middle sample has no neighbors with distance → speed stays None.
        let samples = vec![
            Sample { t: Duration::from_secs(0), lat: 0.0, lon: 0.0, distance_m: None,       ..Sample::blank() },
            Sample { t: Duration::from_secs(1), lat: 0.0, lon: 0.0, distance_m: None,       ..Sample::blank() },
            Sample { t: Duration::from_secs(2), lat: 0.0, lon: 0.0, distance_m: None,       ..Sample::blank() },
        ];
        let mut a = Activity::from_samples(Utc::now(), samples);
        a.fill_derived_speed();
        assert!(a.samples.iter().all(|s| s.speed_mps.is_none()));
    }
}
