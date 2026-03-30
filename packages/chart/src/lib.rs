//! # loom-chart
//!
//! Chart drawing primitives and plugin system for trading charts.
//!
//! ## Features
//!
//! - **Drawing Primitives**: Lines, labels, shapes, arrows, zones
//! - **Plugin System**: Rust plugins that draw on charts
//! - **Built-in Plugins**: Pivots, Wyckoff, SMC, Elliott Waves
//! - **WASM Bridge**: Seamless integration with JS charts
//!
//! ## Example
//!
//! ```rust
//! use loom_chart::{ChartPlugin, DrawingContext, Line, Label};
//!
//! struct MyIndicatorPlugin;
//!
//! impl ChartPlugin for MyIndicatorPlugin {
//!     fn name(&self) -> &str { "My Indicator" }
//!
//!     fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle) {
//!         // Draw support line
//!         ctx.draw(Line::horizontal(candle.low)
//!             .color(Color::GREEN)
//!             .style(LineStyle::Dashed));
//!
//!         // Add label
//!         ctx.draw(Label::new("Support", candle.time, candle.low)
//!             .color(Color::GREEN));
//!     }
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod primitives;
pub mod plugin;
pub mod context;
pub mod plugins;

#[cfg(feature = "wasm")]
pub mod wasm;

// Re-exports
pub use primitives::{
    // Basic types
    Color, Point, Anchor, LineStyle, FillPattern,
    // Drawings
    Drawing, DrawingId,
    Line, HorizontalLine, VerticalLine, TrendLine, Ray, ExtendedLine,
    Label, PriceLabel, TimeLabel,
    Shape, Rectangle, Triangle, Circle, Ellipse, Arc,
    Arrow, Bracket,
    Zone, PriceZone, TimeZone,
    FibonacciRetracement, FibonacciExtension, FibonacciFan,
    Channel, PitchFork,
    Text, TextStyle,
    Icon, IconType,
    Path, PathCommand,
};

pub use plugin::{ChartPlugin, PluginConfig, PluginState};
pub use context::{DrawingContext, DrawingBuffer, LayerId};

pub use plugins::{
    PivotPlugin, PivotConfig, PivotType,
    WyckoffPlugin, WyckoffVisualConfig,
    SmcPlugin, SmcVisualConfig,
    ElliottPlugin, ElliottVisualConfig,
    SupportResistancePlugin,
    TrendlinePlugin,
};
