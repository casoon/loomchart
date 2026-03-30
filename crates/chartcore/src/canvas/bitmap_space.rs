//! Bitmap Space - Device Pixel Coordinate System
//!
//! Handles coordinates in the actual canvas bitmap (device pixels).
//! All rendering operations should use bitmap space coordinates.

use super::coordinate_space::{CssPixels, DevicePixels, PixelRatio};

/// Bitmap Space - manages device pixel coordinates
#[derive(Debug, Clone)]
pub struct BitmapSpace {
    pub width: DevicePixels,
    pub height: DevicePixels,
    pub pixel_ratio: PixelRatio,
}

impl BitmapSpace {
    /// Create a new bitmap space from CSS dimensions and pixel ratio
    pub fn new(css_width: CssPixels, css_height: CssPixels, pixel_ratio: PixelRatio) -> Self {
        Self {
            width: pixel_ratio.to_device(css_width),
            height: pixel_ratio.to_device(css_height),
            pixel_ratio,
        }
    }

    /// Create from device pixel dimensions
    pub fn from_device(width: DevicePixels, height: DevicePixels, pixel_ratio: PixelRatio) -> Self {
        Self {
            width,
            height,
            pixel_ratio,
        }
    }

    /// Get width in CSS pixels
    pub fn css_width(&self) -> CssPixels {
        self.pixel_ratio.to_css(self.width)
    }

    /// Get height in CSS pixels
    pub fn css_height(&self) -> CssPixels {
        self.pixel_ratio.to_css(self.height)
    }

    /// Convert CSS X coordinate to device X coordinate
    pub fn to_device_x(&self, css_x: CssPixels) -> DevicePixels {
        self.pixel_ratio.to_device(css_x)
    }

    /// Convert CSS Y coordinate to device Y coordinate
    pub fn to_device_y(&self, css_y: CssPixels) -> DevicePixels {
        self.pixel_ratio.to_device(css_y)
    }

    /// Convert device X coordinate to CSS X coordinate
    pub fn to_css_x(&self, device_x: DevicePixels) -> CssPixels {
        self.pixel_ratio.to_css(device_x)
    }

    /// Convert device Y coordinate to CSS Y coordinate
    pub fn to_css_y(&self, device_y: DevicePixels) -> CssPixels {
        self.pixel_ratio.to_css(device_y)
    }

    /// Apply pixel-perfect correction for odd line widths
    /// Shifts coordinates by 0.5 device pixels to align with pixel grid
    pub fn align_to_pixel_grid(&self, coord: DevicePixels, line_width: f64) -> DevicePixels {
        if line_width % 2.0 == 1.0 {
            // Odd line width - shift by 0.5 to center on pixel
            DevicePixels(coord.0.floor() + 0.5)
        } else {
            // Even line width - align to whole pixel
            DevicePixels(coord.0.floor())
        }
    }

    /// Round to nearest device pixel
    pub fn round(&self, coord: DevicePixels) -> DevicePixels {
        DevicePixels(coord.0.round())
    }

    /// Floor to device pixel
    pub fn floor(&self, coord: DevicePixels) -> DevicePixels {
        DevicePixels(coord.0.floor())
    }

    /// Ceil to device pixel
    pub fn ceil(&self, coord: DevicePixels) -> DevicePixels {
        DevicePixels(coord.0.ceil())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_space_creation() {
        let css_w = CssPixels(800.0);
        let css_h = CssPixels(600.0);
        let ratio = PixelRatio::new(2.0);

        let bitmap = BitmapSpace::new(css_w, css_h, ratio);

        assert_eq!(bitmap.width.0, 1600.0);
        assert_eq!(bitmap.height.0, 1200.0);
        assert_eq!(bitmap.css_width().0, 800.0);
        assert_eq!(bitmap.css_height().0, 600.0);
    }

    #[test]
    fn test_coordinate_conversion() {
        let bitmap = BitmapSpace::from_device(
            DevicePixels(1600.0),
            DevicePixels(1200.0),
            PixelRatio::new(2.0),
        );

        let css_x = CssPixels(100.0);
        let device_x = bitmap.to_device_x(css_x);
        assert_eq!(device_x.0, 200.0);

        let back = bitmap.to_css_x(device_x);
        assert_eq!(back.0, 100.0);
    }

    #[test]
    fn test_pixel_grid_alignment() {
        let bitmap = BitmapSpace::from_device(
            DevicePixels(800.0),
            DevicePixels(600.0),
            PixelRatio::new(1.0),
        );

        // Odd line width (1px) - should shift by 0.5
        let coord = DevicePixels(10.7);
        let aligned = bitmap.align_to_pixel_grid(coord, 1.0);
        assert_eq!(aligned.0, 10.5);

        // Even line width (2px) - should floor
        let aligned = bitmap.align_to_pixel_grid(coord, 2.0);
        assert_eq!(aligned.0, 10.0);
    }

    #[test]
    fn test_rounding() {
        let bitmap = BitmapSpace::from_device(
            DevicePixels(800.0),
            DevicePixels(600.0),
            PixelRatio::new(1.0),
        );

        assert_eq!(bitmap.round(DevicePixels(10.4)).0, 10.0);
        assert_eq!(bitmap.round(DevicePixels(10.6)).0, 11.0);
        assert_eq!(bitmap.floor(DevicePixels(10.9)).0, 10.0);
        assert_eq!(bitmap.ceil(DevicePixels(10.1)).0, 11.0);
    }
}
