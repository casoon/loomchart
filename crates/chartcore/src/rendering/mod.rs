//! Rendering system
//!
//! Defines the Renderer trait and implementations for different backends

pub mod optimizations;
mod renderer;

#[cfg(feature = "wasm")]
mod canvas2d;

pub use renderer::{
    CandleData, DrawStyle, LineStyle, RenderCommand, Renderer, TextAlign, TextBaseline,
};

#[cfg(feature = "wasm")]
pub use canvas2d::Canvas2DRenderer;

pub use optimizations::{
    calculate_indicator_complexity, calculate_visible_count, cull_candles, should_render_detail,
    should_render_indicator, RenderDetail,
};
