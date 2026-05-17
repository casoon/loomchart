//! Production price scale: coordinate mapping for all 4 scale modes.

/// The four scale modes supported by the price axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScaleMode {
    /// Equal pixel spacing per unit.
    Linear,
    /// Logarithmic spacing — equal pixels per percentage change.
    Log,
    /// Values expressed as percentage change from `base_price`.
    Percent,
    /// Values expressed as index (base = 100) relative to `base_price`.
    IndexedToBase,
}

/// Maps prices to pixel Y coordinates for a single pane.
///
/// Y=0 is the **top** of the pane; Y=height is the **bottom**.
#[derive(Debug, Clone)]
pub struct PriceScale {
    pub mode: ScaleMode,
    pub min: f64,
    pub max: f64,
    /// Pixel height of this scale's pane.
    pub height: u32,
    /// Reference price used by `Percent` and `IndexedToBase` modes.
    pub base_price: f64,
}

impl PriceScale {
    pub fn new(min: f64, max: f64, height: u32) -> Self {
        Self {
            mode: ScaleMode::Linear,
            min,
            max,
            height,
            base_price: 1.0,
        }
    }

    pub fn with_mode(mut self, mode: ScaleMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_base(mut self, base: f64) -> Self {
        self.base_price = base;
        self
    }

    /// Convert a price (or value) to a Y pixel coordinate.
    ///
    /// Returns a value in `[0.0, height]`; values outside the visible range
    /// produce coordinates outside that interval (no clamping).
    pub fn price_to_y(&self, price: f64) -> f64 {
        let h = self.height as f64;
        let (lo, hi, v) = self.to_internal(self.min, self.max, price);

        let span = hi - lo;
        if span == 0.0 {
            return h / 2.0;
        }

        // Y is inverted: high value → low Y.
        (hi - v) / span * h
    }

    /// Convert a Y pixel coordinate back to a price.
    pub fn y_to_price(&self, y: f64) -> f64 {
        let h = self.height as f64;
        if h == 0.0 {
            return self.min;
        }

        let (lo, hi, _) = self.to_internal(self.min, self.max, self.min);
        let span = hi - lo;

        // Invert: y=0 → hi, y=h → lo.
        let internal_val = hi - (y / h) * span;
        self.from_internal(internal_val)
    }

    /// Generate `count` human-friendly tick values covering `[min, max]`.
    ///
    /// Ticks are aligned to "nice" round numbers (1, 2, 2.5, 5 × 10^n).
    pub fn nice_ticks(&self, count: usize) -> Vec<f64> {
        if count == 0 {
            return Vec::new();
        }

        let span = self.max - self.min;
        if span <= 0.0 || !span.is_finite() {
            return vec![self.min];
        }

        // Raw step size.
        let raw_step = span / count as f64;

        // Magnitude of the step.
        let mag = raw_step.log10().floor();
        let scale = 10f64.powf(mag);

        // Round up to nearest "nice" step.
        let norm = raw_step / scale;
        let nice_norm = if norm <= 1.0 {
            1.0
        } else if norm <= 2.0 {
            2.0
        } else if norm <= 2.5 {
            2.5
        } else if norm <= 5.0 {
            5.0
        } else {
            10.0
        };
        let step = nice_norm * scale;

        // First tick at or above min.
        let first = (self.min / step).ceil() * step;

        let mut ticks = Vec::new();
        let mut t = first;
        // Add a small epsilon to guard against floating-point overshoot.
        let epsilon = step * 1e-9;
        while t <= self.max + epsilon {
            ticks.push(t);
            t += step;
        }
        ticks
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    /// Transform (min, max, value) into internal space according to `mode`.
    /// Returns `(internal_min, internal_max, internal_value)`.
    fn to_internal(&self, min: f64, max: f64, value: f64) -> (f64, f64, f64) {
        match self.mode {
            ScaleMode::Linear => (min, max, value),
            ScaleMode::Log => {
                let eps = 1e-10;
                (
                    min.max(eps).ln(),
                    max.max(eps).ln(),
                    value.max(eps).ln(),
                )
            }
            ScaleMode::Percent => {
                let base = if self.base_price == 0.0 {
                    1.0
                } else {
                    self.base_price
                };
                (
                    (min / base - 1.0) * 100.0,
                    (max / base - 1.0) * 100.0,
                    (value / base - 1.0) * 100.0,
                )
            }
            ScaleMode::IndexedToBase => {
                let base = if self.base_price == 0.0 {
                    1.0
                } else {
                    self.base_price
                };
                (
                    min / base * 100.0,
                    max / base * 100.0,
                    value / base * 100.0,
                )
            }
        }
    }

    /// Convert an internal-space value back to price space.
    fn from_internal(&self, v: f64) -> f64 {
        match self.mode {
            ScaleMode::Linear => v,
            ScaleMode::Log => v.exp(),
            ScaleMode::Percent => {
                let base = if self.base_price == 0.0 {
                    1.0
                } else {
                    self.base_price
                };
                (v / 100.0 + 1.0) * base
            }
            ScaleMode::IndexedToBase => {
                let base = if self.base_price == 0.0 {
                    1.0
                } else {
                    self.base_price
                };
                v / 100.0 * base
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── price_to_y ───────────────────────────────────────────────────────────

    #[test]
    fn price_to_y_linear_top_bottom_middle() {
        let s = PriceScale::new(100.0, 200.0, 400);
        // max price → y=0 (top), min price → y=height (bottom)
        assert_eq!(s.price_to_y(200.0), 0.0);
        assert_eq!(s.price_to_y(100.0), 400.0);
        assert!((s.price_to_y(150.0) - 200.0).abs() < 1e-9);
    }

    #[test]
    fn price_to_y_log() {
        // For log scale: ln(100)..ln(1000), height=300
        let s = PriceScale::new(100.0, 1000.0, 300).with_mode(ScaleMode::Log);
        // max → top
        assert!((s.price_to_y(1000.0)).abs() < 1e-6);
        // min → bottom
        assert!((s.price_to_y(100.0) - 300.0).abs() < 1e-6);
        // geometric midpoint → pixel midpoint
        let mid = (100f64 * 1000f64).sqrt(); // ≈ 316.2
        assert!((s.price_to_y(mid) - 150.0).abs() < 1e-6);
    }

    #[test]
    fn price_to_y_percent() {
        // base=100, min=90 (-10%), max=110 (+10%), height=200
        let s = PriceScale::new(90.0, 110.0, 200)
            .with_mode(ScaleMode::Percent)
            .with_base(100.0);
        // max (+10%) → top (y=0)
        assert!((s.price_to_y(110.0)).abs() < 1e-9);
        // min (-10%) → bottom (y=200)
        assert!((s.price_to_y(90.0) - 200.0).abs() < 1e-9);
        // base (0%) → middle (y=100)
        assert!((s.price_to_y(100.0) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn price_to_y_indexed_to_base() {
        // base=200, min=100 (index 50), max=300 (index 150), height=200
        let s = PriceScale::new(100.0, 300.0, 200)
            .with_mode(ScaleMode::IndexedToBase)
            .with_base(200.0);
        assert!((s.price_to_y(300.0)).abs() < 1e-9);
        assert!((s.price_to_y(100.0) - 200.0).abs() < 1e-9);
        assert!((s.price_to_y(200.0) - 100.0).abs() < 1e-9);
    }

    // ── round-trip ───────────────────────────────────────────────────────────

    #[test]
    fn round_trip_linear() {
        let s = PriceScale::new(1000.0, 2000.0, 500);
        for price in [1000.0, 1234.5, 1500.0, 1999.9, 2000.0] {
            let y = s.price_to_y(price);
            let back = s.y_to_price(y);
            assert!((back - price).abs() < 1e-6, "price={price} back={back}");
        }
    }

    #[test]
    fn round_trip_log() {
        let s = PriceScale::new(100.0, 10000.0, 400).with_mode(ScaleMode::Log);
        for price in [100.0, 316.2, 1000.0, 3162.0, 10000.0] {
            let y = s.price_to_y(price);
            let back = s.y_to_price(y);
            assert!((back - price).abs() < 1e-4, "price={price} back={back}");
        }
    }

    #[test]
    fn round_trip_percent() {
        let s = PriceScale::new(80.0, 120.0, 400)
            .with_mode(ScaleMode::Percent)
            .with_base(100.0);
        for price in [80.0, 90.0, 100.0, 110.0, 120.0] {
            let y = s.price_to_y(price);
            let back = s.y_to_price(y);
            assert!((back - price).abs() < 1e-6, "price={price} back={back}");
        }
    }

    #[test]
    fn round_trip_indexed() {
        let s = PriceScale::new(100.0, 300.0, 200)
            .with_mode(ScaleMode::IndexedToBase)
            .with_base(200.0);
        for price in [100.0, 150.0, 200.0, 250.0, 300.0] {
            let y = s.price_to_y(price);
            let back = s.y_to_price(y);
            assert!((back - price).abs() < 1e-6, "price={price} back={back}");
        }
    }

    // ── nice_ticks ───────────────────────────────────────────────────────────

    #[test]
    fn nice_ticks_covers_range() {
        let s = PriceScale::new(0.0, 100.0, 400);
        let ticks = s.nice_ticks(5);
        assert!(!ticks.is_empty());
        assert!(ticks.first().unwrap() >= &0.0);
        assert!(*ticks.last().unwrap() <= 100.0 + 1.0); // allow one step overshoot
    }

    #[test]
    fn nice_ticks_round_numbers() {
        // 0..1000, 5 ticks → step should be 200
        let s = PriceScale::new(0.0, 1000.0, 400);
        let ticks = s.nice_ticks(5);
        // Steps should all be multiples of 200
        for w in ticks.windows(2) {
            let diff = (w[1] - w[0]).round() as i64;
            assert_eq!(diff % 200, 0, "unexpected step: {}", diff);
        }
    }

    #[test]
    fn nice_ticks_price_range() {
        // Realistic BTC-ish range: 42000..44000, 4 ticks
        let s = PriceScale::new(42000.0, 44000.0, 400);
        let ticks = s.nice_ticks(4);
        assert!(!ticks.is_empty());
        // All ticks within [42000, 44000]
        for &t in &ticks {
            assert!(t >= 42000.0 && t <= 44000.0, "tick out of range: {t}");
        }
    }

    #[test]
    fn nice_ticks_zero_count_is_empty() {
        let s = PriceScale::new(0.0, 100.0, 400);
        assert!(s.nice_ticks(0).is_empty());
    }
}
