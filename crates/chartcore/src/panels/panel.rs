//! Panel types and configuration

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique panel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PanelId(Uuid);

impl PanelId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for PanelId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PanelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of panel content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PanelType {
    /// Main chart with candlesticks/lines and overlays
    Chart {
        /// Indicators to overlay on the chart (e.g., EMA, BB, MFI)
        overlays: Vec<OverlayConfig>,
    },
    /// Dedicated indicator panel with own scale
    Indicator {
        indicator_id: String,
        #[serde(default)]
        params: serde_json::Value,
    },
    /// Volume panel
    Volume,
}

/// Configuration for an overlay indicator on the chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    /// Indicator ID (e.g., "ema21", "mfi14")
    pub id: String,
    /// Parameters for the indicator
    #[serde(default)]
    pub params: serde_json::Value,
    /// Color for rendering
    pub color: String,
    /// Whether this indicator uses a different scale (e.g., MFI 0-100 vs price)
    pub separate_scale: bool,
    /// If separate_scale=true, the scale range
    pub scale_min: Option<f64>,
    pub scale_max: Option<f64>,
}

/// Price scale configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceScale {
    /// Auto-scale to visible data
    pub auto_scale: bool,
    /// Fixed minimum (if not auto-scaling)
    pub min: Option<f64>,
    /// Fixed maximum (if not auto-scaling)
    pub max: Option<f64>,
    /// Invert scale (top = low, bottom = high)
    pub invert: bool,
    /// Logarithmic scale
    pub logarithmic: bool,
    /// Percentage mode (relative to first visible bar)
    pub percentage: bool,
}

impl Default for PriceScale {
    fn default() -> Self {
        Self {
            auto_scale: true,
            min: None,
            max: None,
            invert: false,
            logarithmic: false,
            percentage: false,
        }
    }
}

impl PriceScale {
    /// Create a fixed scale (e.g., for RSI 0-100)
    pub fn fixed(min: f64, max: f64) -> Self {
        Self {
            auto_scale: false,
            min: Some(min),
            max: Some(max),
            ..Default::default()
        }
    }

    /// Create a percentage scale
    pub fn percentage() -> Self {
        Self {
            auto_scale: true,
            percentage: true,
            ..Default::default()
        }
    }
}

/// Panel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Unique panel ID
    pub id: PanelId,
    /// Panel type and content
    pub panel_type: PanelType,
    /// Display title
    pub title: String,

    // Layout
    /// Relative height weight (default: 1.0)
    /// Example: Chart=3.0, RSI=1.0, MACD=1.0 → Chart gets 60%, others 20% each
    pub stretch_factor: f64,
    /// Minimum height in pixels
    pub min_height: u32,
    /// Whether panel is collapsed
    pub collapsed: bool,

    // Rendering
    /// Price scale configuration
    pub price_scale: PriceScale,
    /// Show grid lines
    pub show_grid: bool,
    /// Show crosshair
    pub show_crosshair: bool,

    // Position
    /// Order index (lower = higher in the layout)
    pub order_index: usize,
}

impl PanelConfig {
    /// Create a main chart panel
    pub fn new_chart() -> Self {
        Self {
            id: PanelId::new(),
            panel_type: PanelType::Chart {
                overlays: Vec::new(),
            },
            title: "Chart".to_string(),
            stretch_factor: 3.0,
            min_height: 100,
            collapsed: false,
            price_scale: PriceScale::default(),
            show_grid: true,
            show_crosshair: true,
            order_index: 0,
        }
    }

    /// Create an indicator panel with its own scale
    pub fn new_indicator(indicator_id: &str, params: serde_json::Value) -> Self {
        // Determine default scale based on indicator type
        let (price_scale, min_height) = match indicator_id {
            "rsi" | "rsi14" | "rsi7" | "rsi21" => (PriceScale::fixed(0.0, 100.0), 80),
            "mfi" | "mfi14" => (PriceScale::fixed(0.0, 100.0), 80),
            "stoch" | "stochastic" => (PriceScale::fixed(0.0, 100.0), 80),
            "cci" | "cci14" => (PriceScale::fixed(-200.0, 200.0), 80),
            "williams_r" | "wpr" => (PriceScale::fixed(-100.0, 0.0), 80),
            _ => (PriceScale::default(), 100),
        };

        Self {
            id: PanelId::new(),
            panel_type: PanelType::Indicator {
                indicator_id: indicator_id.to_string(),
                params,
            },
            title: indicator_id.to_uppercase(),
            stretch_factor: 1.0,
            min_height,
            collapsed: false,
            price_scale,
            show_grid: true,
            show_crosshair: true,
            order_index: 1,
        }
    }

    /// Create a volume panel
    pub fn new_volume() -> Self {
        Self {
            id: PanelId::new(),
            panel_type: PanelType::Volume,
            title: "Volume".to_string(),
            stretch_factor: 0.5,
            min_height: 60,
            collapsed: false,
            price_scale: PriceScale::default(),
            show_grid: false,
            show_crosshair: false,
            order_index: 2,
        }
    }

    /// Add an overlay to a chart panel
    pub fn add_overlay(&mut self, overlay: OverlayConfig) {
        if let PanelType::Chart { overlays } = &mut self.panel_type {
            overlays.push(overlay);
        }
    }

    /// Remove an overlay from a chart panel
    pub fn remove_overlay(&mut self, id: &str) {
        if let PanelType::Chart { overlays } = &mut self.panel_type {
            overlays.retain(|o| o.id != id);
        }
    }
}

/// Panel with computed layout information
#[derive(Debug, Clone)]
pub struct Panel {
    /// Panel configuration
    pub config: PanelConfig,

    // Computed layout (set by PanelManager)
    /// Computed height in pixels
    pub computed_height: u32,
    /// Y offset from top in pixels
    pub y_offset: u32,
}

impl Panel {
    pub fn new(config: PanelConfig) -> Self {
        Self {
            config,
            computed_height: 0,
            y_offset: 0,
        }
    }

    pub fn id(&self) -> PanelId {
        self.config.id
    }
}
