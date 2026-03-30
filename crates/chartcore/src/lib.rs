// chartcore - Lightweight trading chart engine in Rust
//
// A modern alternative to TradingView Lightweight Charts
//
// # Quick Start
//
// ```rust
// use chartcore::prelude::*;
//
// // Create chart with candles
// let candles = vec![
//     Candle::new(1000, 100.0, 101.0, 99.0, 100.5, 1000.0),
//     // ... more candles
// ];
//
// // Add indicator
// let registry = IndicatorRegistry::default();
// let context = CalculationContext::new(&candles)
//     .with_input("length", InputValue::Int(14));
// let result = registry.calculate("rsi", &context).unwrap();
// ```

pub mod canvas; // Canvas DPI management
pub mod commands; // Command pattern for undo/redo
pub mod core;
pub mod drawings; // Phase 4: Drawing tools
pub mod indicators;
pub mod panels; // Panel system for multi-pane layouts
pub mod plugins; // Indicator plugins and chart analysis plugins
pub mod primitives;

// Legacy renderers module - kept for backward compatibility with unused core modules
// WASM uses the new `rendering` module instead
#[allow(dead_code)]
pub(crate) mod renderers;

pub mod rendering; // Renderer trait and implementations
pub mod state; // State import/export
pub mod ta;
pub mod tools; // Drawing tools framework
pub mod utils; // Utility functions for rendering calculations

// WASM bindings
#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export commonly used types from all modules
pub use core::{
    Candle, CandleGenerator, Chart, ChartBuffer, ChartConfig, ChartOptions, ChartState,
    CrosshairState, Dimensions, EventHandler, GeneratorConfig, InteractionState, KeyboardEvent,
    MarketType, MouseButton, MouseEvent, Point, PriceRange, TimeRange, Timeframe, TouchEvent,
    Trend, Viewport, VolatilityRegime,
};

pub use commands::{Command, CommandHistory};

pub use drawings::{
    drawing::{LineStyle as DrawingLineStyle, Point as DrawingPoint},
    renderer::{DrawCommand, Viewport as DrawingViewport},
    Drawing, DrawingManager, DrawingRenderer, DrawingStyle, DrawingType,
};

pub use panels::{
    OverlayConfig, OverlayScale, Panel, PanelConfig, PanelId, PanelManager, PanelType, PriceScale,
    ScaleMapper, ScaleRange,
};

pub use plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    InputType, InputValue, PluginRegistry as IndicatorRegistry, SourceType, WasmPluginLoader,
};

pub use primitives::{Color, LineStyle, PlotConfig};

pub use rendering::{DrawStyle, RenderCommand, Renderer, TextAlign, TextBaseline};

#[cfg(feature = "wasm")]
pub use rendering::Canvas2DRenderer;

// Convenience prelude for glob imports
pub mod prelude {
    pub use crate::core::*;
    pub use crate::plugins::{
        CalculationContext, IndicatorPlugin, IndicatorResult, InputValue, SourceType,
    };
    pub use crate::primitives::*;
    pub use crate::ta;
    pub use crate::IndicatorRegistry;
}
