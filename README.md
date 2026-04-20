# gpx-overlay

Render a transparent ProRes 4444 video overlay of ride/run telemetry (speed, distance, elevation, gradient, course map, elevation profile) from a GPX or FIT activity file, driven by a JSON layout.

## Prerequisites

- **Rust stable** — the workspace pins the channel via `rust-toolchain.toml`.
- **ffmpeg on PATH** — any recent build (tested with 7.1.1). Must include the `prores_ks` encoder, which ships in any full ffmpeg build.

## Build

```bash
cargo build --release
```

The CLI binary is produced at `target/release/gpx-overlay`.

## Quick start

Render the bundled example activity against the bundled layout:

```bash
cargo run --release -- render \
    --input examples/short.gpx \
    --layout examples/layout.json \
    --output out.mov
```

The output `out.mov` is a transparent-background ProRes 4444 clip sized to match the layout's canvas.

## CLI flags

From `gpx-overlay render --help`:

| Flag | Description |
| --- | --- |
| `-i, --input <PATH>` | Activity file (`.gpx` or `.fit`). |
| `-l, --layout <PATH>` | Layout JSON file. |
| `-o, --output <PATH>` | Output video path (use `.mov` for ProRes 4444). |
| `--from <TIME>` | Start offset: `HH:MM:SS`, `MM:SS`, or seconds. |
| `--to <TIME>` | End offset (defaults to activity end). |
| `--fps <N>` | Override the layout's `canvas.fps`. |
| `--size <WxH>` | Override the layout's `canvas.width`x`canvas.height`. |
| `--threads <N>` | Render threads (defaults to `num_cpus::get()`). |
| `--qscale <N>` | ProRes qscale, 0=lossless, 13=aggressive. Default 11. |
| `--dry-run` | Parse and validate inputs; don't render. |

## Layout format

Layouts are JSON documents that describe the canvas, units, theme, and a list of widgets (readouts, course map, elevation profile). See [`docs/plans/2026-04-20-gpx-overlay-design.md`](docs/plans/2026-04-20-gpx-overlay-design.md) for the complete schema.

A minimal layout with a single speed readout:

```json
{
    "version": 1,
    "canvas": { "width": 1920, "height": 1080, "fps": 30 },
    "units": { "speed": "kmh", "distance": "km", "elevation": "m", "temp": "c" },
    "theme": { "font": "Inter", "fg": "#ffffff", "accent": "#ffcc00", "shadow": null },
    "widgets": [
        {
            "type": "readout", "id": "speed_readout",
            "metric": "speed",
            "rect": { "x": 80, "y": 880, "w": 280, "h": 140 },
            "label": "SPEED", "decimals": 1, "font_size": 80.0
        }
    ]
}
```

See `examples/layout.json` for a fuller example using all widget types.

## Known v1 limitations

- **`--size` does not scale widget rects.** It overrides canvas dimensions only, so shrinking a 1920x1080-designed layout to 640x360 will fail validation when widgets fall outside the new canvas.
- **Fonts.** Only the bundled Inter variable font is available; system fonts are not loaded.
- **GPX extensions.** If a `.gpx` file has no `<extensions>` block, HR / power / cadence will be absent.
- **Golden-image tests are Windows-specific.** `cosmic-text` rasterization varies across platforms, so the image-comparison tests are pinned to Windows.

## Testing

Default (skips ffmpeg integration tests):

```bash
cargo test --workspace
```

Full suite (requires `ffmpeg` and `ffprobe` on PATH):

```bash
cargo test --workspace --features ffmpeg-tests
```

## Project structure

```
crates/
    activity/   # GPX + FIT parsing, sampling, derived metrics
    layout/     # Layout JSON schema, validation, color/unit helpers
    render/     # Frame compositing (tiny-skia + cosmic-text), widgets
    cli/        # Binary entry point, ffmpeg pipe, render pipeline
```

## Further reading

- [Design doc](docs/plans/2026-04-20-gpx-overlay-design.md) — architecture, layout schema, metric definitions.
- [v1 implementation plan](docs/plans/2026-04-20-gpx-overlay-v1-impl.md) — the task breakdown this repo was built from.
