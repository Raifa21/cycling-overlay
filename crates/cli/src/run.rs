use activity::{load_fit, load_gpx, metric_present_on_activity, Activity, Metric};
use anyhow::{anyhow, Context, Result};
use layout::{Layout, MetricCatalog};
use std::fs;
use std::time::Duration;

use crate::args::RenderArgs;

/// Everything a render path needs once parsing + validation have succeeded.
pub struct Loaded {
    pub activity: Activity,
    pub layout: Layout,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub fps: u32,
    pub from: Duration,
    pub to: Duration,
    pub warnings: Vec<layout::Warning>,
}

/// Load the activity + layout, apply CLI overrides, validate the layout, and
/// compute the time range. Shared between `--dry-run` and the real render path.
pub fn load_and_validate(args: &RenderArgs) -> Result<Loaded> {
    // 1. Load activity by extension.
    let input_ext = args
        .input
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow!("input file has no extension: {:?}", args.input))?
        .to_ascii_lowercase();
    let mut activity = match input_ext.as_str() {
        "gpx" => load_gpx(&args.input)
            .with_context(|| format!("loading GPX {:?}", args.input))?,
        "fit" => load_fit(&args.input)
            .with_context(|| format!("loading FIT {:?}", args.input))?,
        other => {
            return Err(anyhow!(
                "unsupported input extension '{}'; use .gpx or .fit",
                other
            ))
        }
    };

    // 2. Prepare derived metrics (distance, speed, gradient, elev-gain, etc.).
    activity.prepare();

    // 3. Load and parse layout.
    let layout_text = fs::read_to_string(&args.layout)
        .with_context(|| format!("reading layout {:?}", args.layout))?;
    let mut layout: Layout = serde_json::from_str(&layout_text)
        .with_context(|| format!("parsing layout {:?}", args.layout))?;

    // 4. Apply CLI overrides (size + fps) before validation so that rect
    //    bounds checks use the final canvas dimensions.
    if let Some((w, h)) = args.size {
        layout.canvas.width = w;
        layout.canvas.height = h;
    }
    if let Some(fps) = args.fps {
        layout.canvas.fps = fps;
    }

    // 5. Build metric catalog.
    let known_owned: Vec<&'static str> = Metric::ALL.iter().map(|m| m.as_str()).collect();
    let present_owned: Vec<&'static str> = Metric::ALL
        .iter()
        .filter(|m| metric_present_on_activity(**m, &activity.samples))
        .map(|m| m.as_str())
        .collect();
    let catalog = MetricCatalog {
        known: &known_owned,
        present: &present_owned,
    };

    // 6. Validate.
    let warnings = layout
        .validate(&catalog)
        .with_context(|| "validating layout")?;

    // 7. Compute from/to with defaults and sanity-check.
    let activity_duration = activity.duration();
    let from = args.from.unwrap_or(Duration::ZERO);
    let to = args.to.unwrap_or(activity_duration);
    if from > to {
        return Err(anyhow!(
            "--from ({:?}) is after --to ({:?})",
            from,
            to
        ));
    }
    if from > activity_duration {
        return Err(anyhow!(
            "--from ({:?}) exceeds activity duration ({:?})",
            from,
            activity_duration
        ));
    }

    Ok(Loaded {
        canvas_width: layout.canvas.width,
        canvas_height: layout.canvas.height,
        fps: layout.canvas.fps,
        activity,
        layout,
        from,
        to,
        warnings,
    })
}

/// Load + validate, then print a summary to stdout. Never writes any output file.
pub fn dry_run(args: &RenderArgs) -> Result<()> {
    let l = load_and_validate(args)?;
    let range = l.to - l.from;
    let frame_count = (range.as_secs_f64() * l.fps as f64).round() as u64;

    println!(
        "Activity: {}s, {} samples",
        l.activity.duration().as_secs(),
        l.activity.samples.len()
    );
    let available: Vec<&str> = Metric::ALL
        .iter()
        .filter(|m| metric_present_on_activity(**m, &l.activity.samples))
        .map(|m| m.as_str())
        .collect();
    println!("Available metrics: {}", available.join(", "));
    println!("Layout: {} widgets", l.layout.widgets.len());
    println!(
        "Time range: {:?} -> {:?} ({}s)",
        l.from,
        l.to,
        range.as_secs()
    );
    println!(
        "Output: {:?}, {}x{} @ {} fps ({} frames)",
        args.output, l.canvas_width, l.canvas_height, l.fps, frame_count
    );
    if !l.warnings.is_empty() {
        println!("Warnings:");
        for w in &l.warnings {
            match w {
                layout::Warning::MetricAbsent { widget_id, metric } => {
                    println!(
                        "  widget '{}' refs metric '{}' absent in activity",
                        widget_id, metric
                    );
                }
            }
        }
    }
    Ok(())
}
