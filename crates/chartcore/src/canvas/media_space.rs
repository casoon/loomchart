//! Media Space - CSS Pixel Coordinate System
//!
//! Handles coordinates in CSS/layout space.
//! Used for text rendering and user interaction.

use super::coordinate_space::{CssPixels, DevicePixels, PixelRatio};

/// Media Space - manages CSS pixel coordinates
#[derive(Debug, Clone)]
pub struct MediaSpace {
    pub width: CssPixels,
    pub height: CssPixels,
    pub pixel_ratio: PixelRatio,
}

impl MediaSpace {
    /// Create a new media space with CSS dimensions
    pub fn new(width: CssPixels, height: CssPixels, pixel_ratio: PixelRatio) -> Self {
        Self {
            width,
            height,
            pixel_ratio,
        }
    }

    /// Create from device pixel dimensions
    pub fn from_device(width: DevicePixels, height: DevicePixels, pixel_ratio: PixelRatio) -> Self {
        Self {
            width: pixel_ratio.to_css(width),
            height: pixel_ratio.to_css(height),
            pixel_ratio,
        }
    }

    /// Get width in device pixels
    pub fn device_width(&self) -> DevicePixels {
        self.pixel_ratio.to_device(self.width)
    }

    /// Get height in device pixels
    pub fn device_height(&self) -> DevicePixels {
        self.pixel_ratio.to_device(self.height)
    }

    /// Convert device X coordinate to CSS X coordinate
    pub fn to_css_x(&self, device_x: DevicePixels) -> CssPixels {
        self.pixel_ratio.to_css(device_x)
    }

    /// Convert device Y coordinate to CSS Y coordinate
    pub fn to_css_y(&self, device_y: DevicePixels) -> CssPixels {
        self.pixel_ratio.to_css(device_y)
    }

    /// Convert CSS X coordinate to device X coordinate
    pub fn to_device_x(&self, css_x: CssPixels) -> DevicePixels {
        self.pixel_ratio.to_device(css_x)
    }

    /// Convert CSS Y coordinate to device Y coordinate
    pub fn to_device_y(&self, css_y: CssPixels) -> DevicePixels {
        self.pixel_ratio.to_device(css_y)
    }

    /// Calculate font size in device pixels for rendering
    /// Text should be sized in CSS pixels but rendered at device pixel scale
    pub fn font_size_device(&self, css_font_size: f64) -> f64 {
        css_font_size * self.pixel_ratio.0
    }

    /// Check if a point in CSS coordinates is within bounds
    pub fn contains(&self, x: CssPixels, y: CssPixels) -> bool {
        x.0 >= 0.0 && x.0 <= self.width.0 && y.0 >= 0.0 && y.0 <= self.height.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_space_creation() {
        let css_w = CssPixels(800.0);
        let css_h = CssPixels(600.0);
        let ratio = PixelRatio::new(2.0);

        let media = MediaSpace::new(css_w, css_h, ratio);

        assert_eq!(media.width.0, 800.0);
        assert_eq!(media.height.0, 600.0);
        assert_eq!(media.device_width().0, 1600.0);
        assert_eq!(media.device_height().0, 1200.0);
    }

    #[test]
    fn test_from_device() {
        let device_w = DevicePixels(1600.0);
        let device_h = DevicePixels(1200.0);
        let ratio = PixelRatio::new(2.0);

        let media = MediaSpace::from_device(device_w, device_h, ratio);

        assert_eq!(media.width.0, 800.0);
        assert_eq!(media.height.0, 600.0);
    }

    #[test]
    fn test_font_size_scaling() {
        let media = MediaSpace::new(CssPixels(800.0), CssPixels(600.0), PixelRatio::new(2.0));

        let css_font = 12.0;
        let device_font = media.font_size_device(css_font);
        assert_eq!(device_font, 24.0);
    }

    #[test]
    fn test_contains() {
        let media = MediaSpace::new(CssPixels(800.0), CssPixels(600.0), PixelRatio::new(1.0));

        assert!(media.contains(CssPixels(400.0), CssPixels(300.0)));
        assert!(media.contains(CssPixels(0.0), CssPixels(0.0)));
        assert!(media.contains(CssPixels(800.0), CssPixels(600.0)));
        assert!(!media.contains(CssPixels(801.0), CssPixels(300.0)));
        assert!(!media.contains(CssPixels(400.0), CssPixels(601.0)));
        assert!(!media.contains(CssPixels(-1.0), CssPixels(300.0)));
    }
}
