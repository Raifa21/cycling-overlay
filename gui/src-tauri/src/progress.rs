use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProgressLine {
    Progress { frame: u64, total: u64 },
    Done,
    Error { message: String },
}

/// Parse one stderr line. Returns `None` for non-JSON lines (warnings,
/// ffmpeg chatter). Returns `None` for JSON that doesn't match a known shape.
pub fn parse_line(line: &str) -> Option<ProgressLine> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') {
        return None;
    }
    serde_json::from_str::<ProgressLine>(trimmed).ok()
}

/// Compute seconds remaining given frames done out of total and seconds elapsed.
/// Returns None at edges (frame == 0 or frame >= total) or when the rate isn't
/// positive (protects against zero/negative elapsed).
pub fn eta_seconds(frame: u64, total: u64, elapsed_secs: f64) -> Option<f64> {
    if frame == 0 || frame >= total {
        return None;
    }
    let rate = frame as f64 / elapsed_secs;
    if rate <= 0.0 || !rate.is_finite() {
        return None;
    }
    Some((total - frame) as f64 / rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_progress() {
        assert_eq!(
            parse_line(r#"{"type":"progress","frame":42,"total":900}"#).unwrap(),
            ProgressLine::Progress {
                frame: 42,
                total: 900
            }
        );
    }

    #[test]
    fn parses_done() {
        assert_eq!(
            parse_line(r#"{"type":"done"}"#).unwrap(),
            ProgressLine::Done
        );
    }

    #[test]
    fn parses_error() {
        assert_eq!(
            parse_line(r#"{"type":"error","message":"boom"}"#).unwrap(),
            ProgressLine::Error {
                message: "boom".into()
            }
        );
    }

    #[test]
    fn non_json_ignored() {
        assert!(parse_line("warning: widget 'x' missing metric").is_none());
        assert!(parse_line("").is_none());
        assert!(parse_line("{not-json").is_none()); // starts with { but is malformed
    }

    #[test]
    fn eta_mid_run() {
        // 450 done of 900, 10s elapsed -> rate 45fps -> 450/45 = 10s ETA
        assert_eq!(eta_seconds(450, 900, 10.0).unwrap(), 10.0);
    }

    #[test]
    fn eta_none_at_edges() {
        assert!(eta_seconds(0, 900, 0.0).is_none());
        assert!(eta_seconds(900, 900, 10.0).is_none());
        assert!(eta_seconds(450, 900, 0.0).is_none());
    }
}
