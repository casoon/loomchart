//! Scale mapping for multi-scale overlays
//!
//! Allows overlaying indicators with different scales (e.g., MFI 0-100)
//! on top of price charts by normalizing to pixel coordinates.

use serde::{Deserialize, Serialize};

/// Value range for scaling
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScaleRange {
    pub min: f64,
    pub max: f64,
}

impl ScaleRange {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub fn from_values(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for &v in values {
            if v.is_finite() {
                min = min.min(v);
                max = max.max(v);
            }
        }

        if min.is_finite() && max.is_finite() {
            Some(Self { min, max })
        } else {
            None
        }
    }

    pub fn span(&self) -> f64 {
        self.max - self.min
    }

    /// Add padding (e.g., 10% on each side)
    pub fn with_padding(&self, percentage: f64) -> Self {
        let padding = self.span() * percentage;
        Self {
            min: self.min - padding,
            max: self.max + padding,
        }
    }
}

/// Overlay scale configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayScale {
    /// Original data range
    pub data_range: ScaleRange,
    /// Target range to map to (usually the panel's price scale)
    pub target_range: ScaleRange,
}

impl OverlayScale {
    pub fn new(data_range: ScaleRange, target_range: ScaleRange) -> Self {
        Self {
            data_range,
            target_range,
        }
    }

    /// Map a value from data range to target range
    pub fn map_value(&self, value: f64) -> f64 {
        if !value.is_finite() {
            return value;
        }

        let data_span = self.data_range.span();
        if data_span == 0.0 {
            return self.target_range.min;
        }

        // Normalize to 0-1
        let normalized = (value - self.data_range.min) / data_span;

        // Map to target range
        self.target_range.min + normalized * self.target_range.span()
    }

    /// Map multiple values
    pub fn map_values(&self, values: &[f64]) -> Vec<f64> {
        values.iter().map(|&v| self.map_value(v)).collect()
    }
}

/// Scale mapper for a panel
pub struct ScaleMapper {
    /// Panel height in pixels
    panel_height: u32,
    /// Primary scale (e.g., price)
    primary_scale: ScaleRange,
    /// Overlay scales for indicators with different ranges
    overlay_scales: Vec<(String, OverlayScale)>,
}

impl ScaleMapper {
    pub fn new(panel_height: u32, primary_scale: ScaleRange) -> Self {
        Self {
            panel_height,
            primary_scale,
            overlay_scales: Vec::new(),
        }
    }

    /// Add an overlay scale mapping
    pub fn add_overlay(&mut self, id: String, data_range: ScaleRange) {
        let scale = OverlayScale::new(data_range, self.primary_scale);
        self.overlay_scales.push((id, scale));
    }

    /// Get mapped value for primary scale
    pub fn map_primary(&self, value: f64) -> f64 {
        self.value_to_y(value, &self.primary_scale)
    }

    /// Get mapped value for an overlay
    pub fn map_overlay(&self, overlay_id: &str, value: f64) -> f64 {
        if let Some((_, scale)) = self.overlay_scales.iter().find(|(id, _)| id == overlay_id) {
            let mapped = scale.map_value(value);
            self.value_to_y(mapped, &self.primary_scale)
        } else {
            self.map_primary(value)
        }
    }

    /// Convert a value to Y pixel coordinate
    fn value_to_y(&self, value: f64, range: &ScaleRange) -> f64 {
        if !value.is_finite() {
            return 0.0;
        }

        let span = range.span();
        if span == 0.0 {
            return (self.panel_height / 2) as f64;
        }

        // Normalize to 0-1 (inverted: high value = low Y)
        let normalized = (range.max - value) / span;

        // Map to pixel coordinates
        normalized * (self.panel_height as f64)
    }

    /// Convert Y pixel coordinate to value (for crosshair/interaction)
    pub fn y_to_value(&self, y: f64) -> f64 {
        if self.panel_height == 0 {
            return self.primary_scale.min;
        }

        let normalized = y / (self.panel_height as f64);
        self.primary_scale.max - normalized * self.primary_scale.span()
    }

    /// Update panel height
    pub fn set_panel_height(&mut self, height: u32) {
        self.panel_height = height;
    }

    /// Update primary scale
    pub fn set_primary_scale(&mut self, range: ScaleRange) {
        self.primary_scale = range;

        // Update all overlay mappings to new target range
        for (_, overlay) in &mut self.overlay_scales {
            overlay.target_range = range;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_range() {
        let range = ScaleRange::new(0.0, 100.0);
        assert_eq!(range.span(), 100.0);

        let padded = range.with_padding(0.1);
        assert_eq!(padded.min, -10.0);
        assert_eq!(padded.max, 110.0);
    }

    #[test]
    fn test_overlay_scale_mapping() {
        // Map MFI (0-100) to price range (40000-42000)
        let mfi_range = ScaleRange::new(0.0, 100.0);
        let price_range = ScaleRange::new(40000.0, 42000.0);
        let scale = OverlayScale::new(mfi_range, price_range);

        // MFI 0 -> 40000
        assert_eq!(scale.map_value(0.0), 40000.0);
        // MFI 100 -> 42000
        assert_eq!(scale.map_value(100.0), 42000.0);
        // MFI 50 -> 41000
        assert_eq!(scale.map_value(50.0), 41000.0);
    }

    #[test]
    fn test_scale_mapper() {
        let panel_height = 400;
        let price_range = ScaleRange::new(40000.0, 42000.0);
        let mut mapper = ScaleMapper::new(panel_height, price_range);

        // Add MFI overlay
        mapper.add_overlay("mfi14".to_string(), ScaleRange::new(0.0, 100.0));

        // Price 42000 (max) should be at Y=0 (top)
        assert_eq!(mapper.map_primary(42000.0), 0.0);

        // Price 40000 (min) should be at Y=400 (bottom)
        assert_eq!(mapper.map_primary(40000.0), 400.0);

        // MFI 50 should map to middle of price range, then to middle of panel
        let mfi_y = mapper.map_overlay("mfi14", 50.0);
        assert!((mfi_y - 200.0).abs() < 1.0);
    }

    #[test]
    fn test_y_to_value() {
        let panel_height = 400;
        let price_range = ScaleRange::new(40000.0, 42000.0);
        let mapper = ScaleMapper::new(panel_height, price_range);

        // Y=0 (top) -> max price
        assert_eq!(mapper.y_to_value(0.0), 42000.0);

        // Y=400 (bottom) -> min price
        assert_eq!(mapper.y_to_value(400.0), 40000.0);

        // Y=200 (middle) -> middle price
        assert_eq!(mapper.y_to_value(200.0), 41000.0);
    }
}
