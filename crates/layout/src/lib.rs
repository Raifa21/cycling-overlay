use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layout {
    pub version: u32,
    pub canvas: Canvas,
    pub units: Units,
    pub theme: Theme,
    pub widgets: Vec<Widget>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Units {
    pub speed: SpeedUnit,
    pub distance: DistanceUnit,
    pub elevation: ElevationUnit,
    pub temp: TempUnit,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SpeedUnit {
    Kmh,
    Mph,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DistanceUnit {
    Km,
    Mi,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ElevationUnit {
    M,
    Ft,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TempUnit {
    C,
    F,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    pub font: String,
    pub fg: String,
    pub accent: String,
    pub shadow: Option<Shadow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Shadow {
    pub blur: f32,
    pub color: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Widget {
    Readout {
        id: String,
        metric: String,
        rect: Rect,
        label: String,
        decimals: u32,
        font_size: f32,
    },
    Course {
        id: String,
        rect: Rect,
        line_width: f32,
        dot_radius: f32,
    },
    ElevationProfile {
        id: String,
        rect: Rect,
    },
}

impl Widget {
    pub fn id(&self) -> &str {
        match self {
            Widget::Readout { id, .. }
            | Widget::Course { id, .. }
            | Widget::ElevationProfile { id, .. } => id,
        }
    }

    pub fn rect(&self) -> Rect {
        match self {
            Widget::Readout { rect, .. }
            | Widget::Course { rect, .. }
            | Widget::ElevationProfile { rect, .. } => *rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_example_layout() {
        let json = r##"{
            "version": 1,
            "canvas": { "width": 1920, "height": 1080, "fps": 30 },
            "units": { "speed": "kmh", "distance": "km", "elevation": "m", "temp": "c" },
            "theme": {
                "font": "Inter",
                "fg": "#ffffff",
                "accent": "#ffcc00",
                "shadow": { "blur": 4.0, "color": "#000000cc" }
            },
            "widgets": [
                {
                    "type": "readout",
                    "id": "speed_readout",
                    "metric": "speed",
                    "rect": { "x": 80, "y": 900, "w": 260, "h": 120 },
                    "label": "SPEED",
                    "decimals": 1,
                    "font_size": 72.0
                },
                {
                    "type": "course",
                    "id": "course_map",
                    "rect": { "x": 1560, "y": 60, "w": 300, "h": 300 },
                    "line_width": 4.0,
                    "dot_radius": 8.0
                },
                {
                    "type": "elevation_profile",
                    "id": "elev_profile",
                    "rect": { "x": 80, "y": 60, "w": 500, "h": 120 }
                }
            ]
        }"##;
        let layout: Layout = serde_json::from_str(json).unwrap();
        let back = serde_json::to_string(&layout).unwrap();
        let layout2: Layout = serde_json::from_str(&back).unwrap();
        assert_eq!(layout, layout2);
        assert_eq!(layout.widgets.len(), 3);
    }

    #[test]
    fn widget_tagged_by_type() {
        let w: Widget = serde_json::from_str(
            r#"{
            "type": "readout", "id": "x", "metric": "hr",
            "rect": { "x": 0, "y": 0, "w": 10, "h": 10 },
            "label": "HR", "decimals": 0, "font_size": 48.0
        }"#,
        )
        .unwrap();
        match w {
            Widget::Readout { .. } => {}
            _ => panic!("expected Readout variant"),
        }
    }
}
