//! Canvas DPI Management
//!
//! This module provides DPI-aware canvas rendering with proper coordinate space handling.
//! It implements a two-coordinate system approach:
//! - **Media Space (CSS Pixels)**: The coordinate system for DOM layout and user interaction
//! - **Bitmap Space (Device Pixels)**: The coordinate system for actual canvas rendering
//!
//! This ensures pixel-perfect rendering on displays with different device pixel ratios (1x, 1.5x, 2x, 3x, etc.)

pub mod bitmap_space;
pub mod coordinate_space;
pub mod media_space;

pub use bitmap_space::BitmapSpace;
pub use coordinate_space::{CoordinateScope, CssPixels, DevicePixels, PixelRatio};
pub use media_space::MediaSpace;
