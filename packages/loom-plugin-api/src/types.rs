use serde::{Deserialize, Serialize};

/// OHLCV candle — stable layout, matches chartcore's Candle.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Candle {
    /// Unix timestamp in seconds
    pub time: u64,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
}

/// RGBA colour value (0–255 per channel).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

// ---------------------------------------------------------------------------
// Indicator input/output types
// ---------------------------------------------------------------------------

/// Which price component to use as the series source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Open,
    High,
    Low,
    Close,
    Hl2,
    Hlc3,
    Ohlc4,
    Hlcc4,
    Volume,
}

impl SourceType {
    pub fn extract(&self, c: &Candle) -> f64 {
        match self {
            Self::Open => c.o,
            Self::High => c.h,
            Self::Low => c.l,
            Self::Close => c.c,
            Self::Hl2 => (c.h + c.l) / 2.0,
            Self::Hlc3 => (c.h + c.l + c.c) / 3.0,
            Self::Ohlc4 => (c.o + c.h + c.l + c.c) / 4.0,
            Self::Hlcc4 => (c.h + c.l + c.c + c.c) / 4.0,
            Self::Volume => c.v,
        }
    }
}

/// Describes what kind of input control to show in the UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum InputType {
    Int {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<i32>,
    },
    Float {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
    },
    Bool,
    Select {
        options: Vec<String>,
    },
    Source,
    Color,
}

/// Configuration for a single input parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    pub id: String,
    pub title: String,
    #[serde(flatten)]
    pub input_type: InputType,
    pub default: InputValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<String>,
}

/// Runtime value of an input parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum InputValue {
    Int(i32),
    Float(f64),
    Bool(bool),
    Str(String),
    Source(SourceType),
    Color(Color),
}

/// Visualisation style for a plot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlotStyle {
    Line,
    Histogram,
    Area,
    Circles,
}

/// Configuration for a single output plot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotConfig {
    pub id: String,
    pub title: String,
    pub color: Color,
    #[serde(default = "default_line_width")]
    pub line_width: u8,
    #[serde(default)]
    pub style: PlotStyle,
}

fn default_line_width() -> u8 {
    2
}

impl Default for PlotStyle {
    fn default() -> Self {
        Self::Line
    }
}

// ---------------------------------------------------------------------------
// Strategy signal
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignalKind {
    Buy,
    Sell,
}

/// A trading signal emitted by a strategy plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub kind: SignalKind,
    pub price: f64,
    pub time: u64,
    #[serde(default)]
    pub reason: String,
}

// ---------------------------------------------------------------------------
// Renderer commands
// ---------------------------------------------------------------------------

/// Drawing command emitted by a renderer plugin.
///
/// Coordinates are in data space (time + price); the host transforms them to
/// canvas pixels. `time` values are Unix timestamps in seconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum RenderCommand {
    DrawLine {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
        width: f32,
    },
    FillRect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        color: Color,
    },
    DrawText {
        x: f64,
        y: f64,
        text: String,
        color: Color,
        size: f32,
    },
    DrawCircle {
        cx: f64,
        cy: f64,
        radius: f32,
        color: Color,
        filled: bool,
    },
}
