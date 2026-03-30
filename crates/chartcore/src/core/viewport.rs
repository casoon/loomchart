//! Viewport - Manages visible range and coordinate transformations
//!
//! The viewport tracks what portion of the data is visible and provides
//! transformations between different coordinate spaces:
//! - Data space (time, price)
//! - Logical space (bar indices)
//! - Screen space (pixels)

use crate::core::types::Timeframe;

/// Time range in seconds (unix timestamp)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeRange {
    pub start: i64,
    pub end: i64,
}

/// Price range
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceRange {
    pub min: f64,
    pub max: f64,
}

/// Pixel dimensions
#[derive(Debug, Clone, Copy)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
    pub pixel_ratio: f64,
}

/// Viewport state
#[derive(Debug, Clone)]
pub struct Viewport {
    /// Visible time range
    pub time: TimeRange,
    /// Visible price range
    pub price: PriceRange,
    /// Screen dimensions
    pub dimensions: Dimensions,
    /// Timeframe (for bar spacing)
    pub timeframe: Timeframe,
}

impl Viewport {
    /// Create new viewport
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            time: TimeRange { start: 0, end: 0 },
            price: PriceRange {
                min: 0.0,
                max: 100.0,
            },
            dimensions: Dimensions {
                width,
                height,
                pixel_ratio: 1.0,
            },
            timeframe: Timeframe::M5,
        }
    }

    /// Set dimensions
    pub fn set_dimensions(&mut self, width: u32, height: u32, pixel_ratio: f64) {
        self.dimensions = Dimensions {
            width,
            height,
            pixel_ratio,
        };
    }

    /// Fit viewport to data range
    pub fn fit_to_data(&mut self, time_range: TimeRange, price_range: PriceRange) {
        let duration = time_range.end - time_range.start;
        let padding = (duration as f64 * 0.05) as i64;

        self.time = TimeRange {
            start: time_range.start - padding,
            end: time_range.end + padding,
        };

        self.price = price_range;
    }

    /// Pan by pixel delta
    pub fn pan(&mut self, delta_x: i32, _delta_y: i32) {
        let time_per_pixel =
            (self.time.end - self.time.start) as f64 / self.dimensions.width as f64;
        let time_delta = (-delta_x as f64 * time_per_pixel) as i64;

        self.time.start += time_delta;
        self.time.end += time_delta;
    }

    /// Zoom around a point
    pub fn zoom(&mut self, factor: f64, center_x: Option<u32>) {
        let cx = center_x.unwrap_or(self.dimensions.width / 2);

        // Calculate center in time space
        let center_time = self.x_to_time(cx as f64);

        // Calculate new range
        let current_range = self.time.end - self.time.start;
        let new_range = (current_range as f64 * factor) as i64;

        // Clamp zoom levels
        let min_bars = 10;
        let max_bars = 5000;
        let bar_duration = self.timeframe.duration_ms() / 1000;

        let bars_in_view = new_range / bar_duration;
        if bars_in_view < min_bars || bars_in_view > max_bars {
            return;
        }

        // Calculate how much time before/after center
        let time_before = ((center_time - self.time.start) as f64 * factor) as i64;
        let time_after = ((self.time.end - center_time) as f64 * factor) as i64;

        self.time.start = center_time - time_before;
        self.time.end = center_time + time_after;
    }

    /// Convert time to x pixel coordinate
    pub fn time_to_x(&self, time: i64) -> f64 {
        let t_start = self.time.start as f64;
        let t_end = self.time.end as f64;
        let t = time as f64;

        (t - t_start) / (t_end - t_start) * self.dimensions.width as f64
    }

    /// Convert x pixel to time
    pub fn x_to_time(&self, x: f64) -> i64 {
        let t_start = self.time.start as f64;
        let t_end = self.time.end as f64;

        (t_start + (x / self.dimensions.width as f64) * (t_end - t_start)) as i64
    }

    /// Convert price to y pixel coordinate
    pub fn price_to_y(&self, price: f64) -> f64 {
        let p_min = self.price.min;
        let p_max = self.price.max;

        // Y is inverted (0 at top)
        (p_max - price) / (p_max - p_min) * self.dimensions.height as f64
    }

    /// Convert y pixel to price
    pub fn y_to_price(&self, y: f64) -> f64 {
        let p_min = self.price.min;
        let p_max = self.price.max;

        // Y is inverted
        p_max - (y / self.dimensions.height as f64) * (p_max - p_min)
    }

    /// Get bar width in pixels
    pub fn bar_width(&self) -> f64 {
        let time_range = (self.time.end - self.time.start) as f64;
        let bar_duration = self.timeframe.duration_ms() as f64 / 1000.0;
        let bars_visible = time_range / bar_duration;

        (self.dimensions.width as f64 / bars_visible)
            .max(1.0)
            .min(50.0)
    }

    /// Get number of visible bars
    pub fn visible_bars(&self) -> usize {
        let time_range = (self.time.end - self.time.start) as f64;
        let bar_duration = self.timeframe.duration_ms() as f64 / 1000.0;

        (time_range / bar_duration).ceil() as usize
    }

    /// Get viewport time start (for optimizations)
    pub fn time_start(&self) -> i64 {
        self.time.start
    }

    /// Get viewport time end (for optimizations)
    pub fn time_end(&self) -> i64 {
        self.time.end
    }

    /// Get viewport width (for optimizations)
    pub fn width(&self) -> u32 {
        self.dimensions.width
    }

    /// Get viewport height (for optimizations)
    pub fn height(&self) -> u32 {
        self.dimensions.height
    }

    /// Scale price range around center (for interactive scaling)
    /// Similar to lightweight-charts implementation
    pub fn scale_price_around_center(&mut self, scale_coefficient: f64) {
        let center = (self.price.min + self.price.max) / 2.0;
        let range = self.price.max - self.price.min;
        let new_range = range * scale_coefficient;

        // Clamp to prevent extreme zoom (minimum 0.1x, maximum 10x of original)
        let clamped_range = new_range.max(range * 0.1).min(range * 10.0);

        self.price.min = center - clamped_range / 2.0;
        self.price.max = center + clamped_range / 2.0;
    }

    /// Start price scaling - captures initial Y position
    /// Returns the inverted Y coordinate for tracking
    pub fn start_price_scale(&self, y: f64) -> f64 {
        // Invert Y (0 is top, height is bottom)
        self.dimensions.height as f64 - y
    }

    /// Apply price scaling based on Y movement
    /// start_y: Initial Y position (inverted) from start_price_scale()
    /// current_y: Current Y position (not inverted)
    /// initial_price_range: Snapshot of price range when scaling started
    pub fn apply_price_scale(
        &mut self,
        start_y: f64,
        current_y: f64,
        initial_price_range: &PriceRange,
    ) {
        // Invert current Y
        let y = self.dimensions.height as f64 - current_y;

        // Clamp to valid range
        let y = y.max(0.0);

        // Calculate scale coefficient with 20% padding (like lightweight-charts)
        let height = self.dimensions.height as f64;
        let padding_factor = 0.2;

        let scale_coeff =
            (start_y + (height - 1.0) * padding_factor) / (y + (height - 1.0) * padding_factor);

        // Limit scale coefficient to minimum 0.1 (10x minimum zoom)
        let scale_coeff = scale_coeff.max(0.1);

        // Calculate new range from initial snapshot
        let center = (initial_price_range.min + initial_price_range.max) / 2.0;
        let initial_range = initial_price_range.max - initial_price_range.min;
        let new_range = initial_range * scale_coeff;

        // Apply new range
        self.price.min = center - new_range / 2.0;
        self.price.max = center + new_range / 2.0;
    }

    /// Start time scaling - captures initial X position
    /// Returns the X coordinate for tracking
    pub fn start_time_scale(&self, x: f64) -> f64 {
        x
    }

    /// Apply time scaling based on X movement
    /// start_x: Initial X position from start_time_scale()
    /// current_x: Current X position
    /// initial_time_range: Snapshot of time range when scaling started
    pub fn apply_time_scale(
        &mut self,
        start_x: f64,
        current_x: f64,
        initial_time_range: &TimeRange,
    ) {
        // Clamp to valid range
        let current_x = current_x.max(0.0);
        let start_x = start_x.max(0.0);

        // Calculate scale coefficient with 20% padding
        let width = self.dimensions.width as f64;
        let padding_factor = 0.2;

        let scale_coeff = (start_x + (width - 1.0) * padding_factor)
            / (current_x + (width - 1.0) * padding_factor);

        // Limit scale coefficient to minimum 0.1 (10x minimum zoom)
        let scale_coeff = scale_coeff.max(0.1);

        // Calculate new range from initial snapshot
        let center = (initial_time_range.start + initial_time_range.end) / 2;
        let initial_range = (initial_time_range.end - initial_time_range.start) as f64;
        let new_range = (initial_range * scale_coeff) as i64;

        // Apply new range
        self.time.start = center - new_range / 2;
        self.time.end = center + new_range / 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_transform() {
        let mut vp = Viewport::new(800, 600);
        vp.time = TimeRange {
            start: 1000,
            end: 2000,
        };
        vp.price = PriceRange {
            min: 100.0,
            max: 200.0,
        };

        // Time to X
        assert_eq!(vp.time_to_x(1000), 0.0);
        assert_eq!(vp.time_to_x(2000), 800.0);
        assert_eq!(vp.time_to_x(1500), 400.0);

        // Price to Y (inverted)
        assert_eq!(vp.price_to_y(200.0), 0.0);
        assert_eq!(vp.price_to_y(100.0), 600.0);
        assert_eq!(vp.price_to_y(150.0), 300.0);
    }

    #[test]
    fn test_pan() {
        let mut vp = Viewport::new(800, 600);
        vp.time = TimeRange {
            start: 1000,
            end: 2000,
        };

        vp.pan(100, 0); // Pan right 100px

        // Time should shift left (negative delta)
        assert!(vp.time.start < 1000);
        assert!(vp.time.end < 2000);
    }

    #[test]
    fn test_zoom() {
        let mut vp = Viewport::new(800, 600);
        vp.time = TimeRange {
            start: 1000,
            end: 2000,
        };

        vp.zoom(0.5, Some(400)); // Zoom in 2x at center

        let new_range = vp.time.end - vp.time.start;
        assert_eq!(new_range, 500); // Half the range
    }
}
