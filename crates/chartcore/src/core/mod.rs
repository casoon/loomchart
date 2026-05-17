// Core types and chart state management

mod adaptive_fps;
mod bar_index;
mod candle_buffer;
mod candle_lifecycle;
mod buffer;
mod chart;
mod chart_renderer;
mod chart_state;
mod config;
mod events;
mod generator;
mod indicator_renderer;
mod invalidation;
pub mod overlay;
pub mod overlays;
mod pane;
mod scale;
mod types;
mod viewport;
mod volume_pane;

pub use adaptive_fps::{AdaptiveFPSConfig, AdaptiveFrameScheduler, FPSStats, RenderComplexity};
pub use bar_index::{BarCoordMapper, BarIndex};
pub use candle_buffer::CandleBuffer;
pub use candle_lifecycle::{CandleEvent, CandleStore};
pub use buffer::ChartBuffer;
pub use chart::Chart;
pub use chart_renderer::{ChartRenderer, ChartTheme};
pub use chart_state::{ChartOptions, ChartState, CrosshairState, InteractionState};
pub use config::ChartConfig;
pub use events::{EventHandler, KeyboardEvent, MouseButton, MouseEvent, TouchEvent};
pub use generator::{
    CandleGenerator, GeneratorConfig, MarketType, Scenario, Trend, VolatilityRegime,
};
pub use indicator_renderer::IndicatorRenderer;
pub use invalidation::{InvalidationLevel, InvalidationLevels, InvalidationMask, LayeredInvalidation, RenderLayer};
pub use types::{Candle, Point, Timeframe};
pub use viewport::{Dimensions, PriceRange, TimeRange, Viewport};
pub use volume_pane::{VolumePane, VolumePaneConfig};
pub use overlay::{ChartOverlay, OverlayRegistry};
pub use pane::{Pane, PaneLayout};
pub use scale::{PriceScale, ScaleMode};
