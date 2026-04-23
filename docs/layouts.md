# Layout reference

An overlay layout is a JSON document that describes the render canvas, unit preferences, a visual theme, and a list of widgets to draw. The CLI and GUI both consume the same format.

Working examples you can copy and edit live in [`../examples/`](../examples/):

- [`layout.json`](../examples/layout.json) — a compact layout touching every widget type.
- [`layout-cycling.json`](../examples/layout-cycling.json) — a full cycling dashboard (speed + power meters, radial power gauge, HR, distance bar, elevation profile, course map, time-elapsed, W/kg).

## Top-level structure

```json
{
    "version": 1,
    "canvas": { "width": 1920, "height": 1080, "fps": 30 },
    "units":  { "speed": "kmh", "distance": "km", "elevation": "m", "temp": "c" },
    "theme":  { "font": "Inter", "fg": "#ffffff", "accent": "#ffcc00", "shadow": null },
    "rider":  { "weight_kg": 73.0 },
    "widgets": [ ... ]
}
```

| Field | Description |
| --- | --- |
| `version` | Schema version. Must be `1`. |
| `canvas` | Output pixel dimensions and frame rate. All widget rects are positioned in these pixels. |
| `units` | Display units. `speed`: `kmh` or `mph`. `distance`: `km` or `mi`. `elevation`: `m` or `ft`. `temp`: `c` or `f`. |
| `theme` | Colors and font family. `fg` is the default widget color; `accent` is used for labels and highlights. Both are `#rrggbb`. `shadow` is reserved for future use — set to `null`. |
| `rider` *(optional)* | Rider properties. `weight_kg` is used to compute the W/kg (`power_to_weight`) metric. |
| `widgets` | Array of widget objects described below. |

## Widgets

Every widget object has a `type`, an `id` (unique within the layout), and a `rect` specifying its bounding box in canvas pixels:

```json
"rect": { "x": 80, "y": 880, "w": 280, "h": 140 }
```

Widget validation rejects any rect that falls outside the canvas.

### `readout` — numeric value with label and unit

Displays a single metric as a formatted number with a unit (e.g. `36.5 km/h`). Label sits above the number.

```json
{
    "type": "readout", "id": "speed_readout",
    "metric": "speed",
    "rect": { "x": 80, "y": 880, "w": 280, "h": 140 },
    "label": "SPEED",
    "decimals": 1,
    "font_size": 80.0,
    "label_font_size": 28.0,
    "unit_font_size": 36.0
}
```

`metric` is any of: `speed`, `heart_rate` (alias `hr`), `power`, `cadence`, `altitude` (alias `elevation`), `distance`, `elev_gain` (alias `elevation_gain`), `gradient`, `time_elapsed`, `time_of_day`, `w_per_kg` (alias `power_to_weight`).

`label_font_size` and `unit_font_size` are optional; both default to derivations of `font_size`.

### `bar` — horizontal progress bar

A fill bar for cumulative metrics (distance, elevation gain) against a configured `max`.

```json
{
    "type": "bar", "id": "distance_bar",
    "metric": "distance",
    "rect": { "x": 80, "y": 1020, "w": 800, "h": 30 },
    "max": 50,
    "label": "DIST"
}
```

### `course` — top-down map of the track

Projects the activity's lat/lon onto the rect and draws the full path, with a moving dot at the current time.

```json
{
    "type": "course", "id": "course_map",
    "rect": { "x": 1520, "y": 60, "w": 340, "h": 340 },
    "stroke": 3.0
}
```

### `elevation_profile` — elevation-vs-distance trace

Plots the elevation profile with a progress marker at the current distance.

```json
{
    "type": "elevation_profile", "id": "elev_profile",
    "rect": { "x": 80, "y": 60, "w": 600, "h": 180 }
}
```

### `meter` — linear scale for scalar metrics

Horizontal or vertical bar with optional ticks, numbers, and one of four indicator styles.

```json
{
    "type": "meter", "id": "speed_meter",
    "metric": "speed",
    "rect": { "x": 80, "y": 760, "w": 600, "h": 60 },
    "min": 0, "max": 60,
    "orientation": "horizontal",
    "indicator": { "kind": "arrow", "thickness": 8, "fill_track": true },
    "ticks": { "major_interval": 10, "minor_per_major": 5, "show_numbers": true },
    "show_value": true
}
```

Fields:

- `orientation`: `horizontal` or `vertical`.
- `indicator.kind`: `fill` (filled from min to current), `rect` (solid marker at current), `arrow` (chevron pointing at current), `needle` (line from track to current).
- `indicator.fill_track`: when true, also fills the track behind the indicator (useful with `rect`/`arrow`/`needle`).
- `ticks.major_interval` / `minor_per_major`: tick spacing.
- `show_numbers`: render numeric labels at the major ticks.
- `show_value`: render the current value above the scale.

### `gauge` — radial/arc version of `meter`

```json
{
    "type": "gauge", "id": "power_gauge",
    "metric": "power",
    "rect": { "x": 1520, "y": 440, "w": 340, "h": 340 },
    "min": 0, "max": 600,
    "start_deg": -135, "end_deg": 135,
    "indicator": { "kind": "needle", "thickness": 6, "fill_track": false },
    "ticks": { "major_interval": 100, "minor_per_major": 4, "show_numbers": true },
    "show_value": true
}
```

Angles are in degrees with **0° = up, clockwise positive** (so a 270° speedometer sweep is `start_deg: -135`, `end_deg: 135`). The gauge progresses clockwise from start to end.

All other fields mirror `meter`.

## Minimal example

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

## Iterating on a layout

The GUI watches the loaded layout file on disk. Save in your editor and the preview reloads within ~200 ms — you can scrub the seekbar to check how a change looks at different points in the activity. Parse errors show as a red banner; the previous good layout remains visible.

## Further detail

The original design doc at [`plans/2026-04-20-gpx-overlay-design.md`](plans/2026-04-20-gpx-overlay-design.md) has more on metric derivation, validation rules, and rendering behavior. The Meter / Gauge design at [`plans/2026-04-22-meter-gauge-design.md`](plans/2026-04-22-meter-gauge-design.md) covers those two widget types in depth.
