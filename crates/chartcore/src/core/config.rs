// Chart configuration

use super::types::Timeframe;

/// Chart configuration
#[derive(Debug, Clone)]
pub struct ChartConfig {
    /// Maximum number of candles to keep in buffer
    pub max_candles: usize,
    /// Default timeframe
    pub timeframe: Timeframe,
    /// Chart title
    pub title: Option<String>,
    /// Symbol being charted
    pub symbol: Option<String>,
    /// Auto-scale Y axis
    pub auto_scale: bool,
    /// Show volume
    pub show_volume: bool,
    /// Show grid
    pub show_grid: bool,
    /// Show crosshair
    pub show_crosshair: bool,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            max_candles: 2000,
            timeframe: Timeframe::M5,
            title: None,
            symbol: None,
            auto_scale: true,
            show_volume: true,
            show_grid: true,
            show_crosshair: true,
        }
    }
}

impl ChartConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_candles(mut self, max: usize) -> Self {
        self.max_candles = max;
        self
    }

    pub fn with_timeframe(mut self, tf: Timeframe) -> Self {
        self.timeframe = tf;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    pub fn auto_scale(mut self, enabled: bool) -> Self {
        self.auto_scale = enabled;
        self
    }

    pub fn show_volume(mut self, show: bool) -> Self {
        self.show_volume = show;
        self
    }

    pub fn show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    pub fn show_crosshair(mut self, show: bool) -> Self {
        self.show_crosshair = show;
        self
    }
}
