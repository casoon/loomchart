//! Coordinate Space Types
//!
//! Defines type-safe coordinate space types to prevent mixing CSS pixels with device pixels.

use std::ops::{Add, Div, Mul, Sub};

/// CSS Pixels - coordinates in DOM/layout space
/// Used for: mouse events, element positioning, logical dimensions
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CssPixels(pub f64);

/// Device Pixels - coordinates in canvas bitmap space
/// Used for: actual canvas rendering, physical pixel operations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DevicePixels(pub f64);

/// Pixel Ratio - ratio between device pixels and CSS pixels
/// Common values: 1.0 (standard), 1.5, 2.0 (Retina), 3.0 (high-DPI mobile)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelRatio(pub f64);

impl PixelRatio {
    /// Get the current device pixel ratio from the environment
    #[cfg(feature = "wasm")]
    pub fn from_window() -> Self {
        let window = web_sys::window().expect("no global window");
        PixelRatio(window.device_pixel_ratio())
    }

    /// Create a pixel ratio with a specific value
    pub fn new(ratio: f64) -> Self {
        PixelRatio(ratio.max(1.0))
    }

    /// Convert CSS pixels to device pixels
    pub fn to_device(&self, css: CssPixels) -> DevicePixels {
        DevicePixels(css.0 * self.0)
    }

    /// Convert device pixels to CSS pixels
    pub fn to_css(&self, device: DevicePixels) -> CssPixels {
        CssPixels(device.0 / self.0)
    }
}

/// Coordinate Scope - identifies which coordinate system is being used
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordinateScope {
    /// CSS pixel coordinates (layout space)
    Media,
    /// Device pixel coordinates (bitmap space)
    Bitmap,
}

// Implement arithmetic operations for CssPixels
impl Add for CssPixels {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        CssPixels(self.0 + other.0)
    }
}

impl Sub for CssPixels {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        CssPixels(self.0 - other.0)
    }
}

impl Mul<f64> for CssPixels {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        CssPixels(self.0 * scalar)
    }
}

impl Div<f64> for CssPixels {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        CssPixels(self.0 / scalar)
    }
}

// Implement arithmetic operations for DevicePixels
impl Add for DevicePixels {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        DevicePixels(self.0 + other.0)
    }
}

impl Sub for DevicePixels {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        DevicePixels(self.0 - other.0)
    }
}

impl Mul<f64> for DevicePixels {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        DevicePixels(self.0 * scalar)
    }
}

impl Div<f64> for DevicePixels {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        DevicePixels(self.0 / scalar)
    }
}

impl From<f64> for CssPixels {
    fn from(value: f64) -> Self {
        CssPixels(value)
    }
}

impl From<CssPixels> for f64 {
    fn from(pixels: CssPixels) -> Self {
        pixels.0
    }
}

impl From<f64> for DevicePixels {
    fn from(value: f64) -> Self {
        DevicePixels(value)
    }
}

impl From<DevicePixels> for f64 {
    fn from(pixels: DevicePixels) -> Self {
        pixels.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pixel_ratio_conversion() {
        let ratio = PixelRatio::new(2.0);
        let css = CssPixels(100.0);
        let device = ratio.to_device(css);

        assert_eq!(device.0, 200.0);

        let back_to_css = ratio.to_css(device);
        assert_eq!(back_to_css.0, 100.0);
    }

    #[test]
    fn test_css_arithmetic() {
        let a = CssPixels(10.0);
        let b = CssPixels(5.0);

        assert_eq!((a + b).0, 15.0);
        assert_eq!((a - b).0, 5.0);
        assert_eq!((a * 2.0).0, 20.0);
        assert_eq!((a / 2.0).0, 5.0);
    }

    #[test]
    fn test_device_arithmetic() {
        let a = DevicePixels(20.0);
        let b = DevicePixels(10.0);

        assert_eq!((a + b).0, 30.0);
        assert_eq!((a - b).0, 10.0);
        assert_eq!((a * 2.0).0, 40.0);
        assert_eq!((a / 2.0).0, 10.0);
    }

    #[test]
    fn test_minimum_pixel_ratio() {
        let ratio = PixelRatio::new(0.5);
        assert_eq!(ratio.0, 1.0); // Should clamp to minimum of 1.0
    }
}
