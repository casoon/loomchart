//! Production pane layout: splits the chart canvas into vertically stacked panes.

use super::scale::PriceScale;

/// A single chart pane (e.g. the main price chart, an RSI panel, a volume panel).
#[derive(Debug, Clone)]
pub struct Pane {
    /// Stable identifier for this pane.
    pub id: String,
    /// Fraction of the total chart height assigned to this pane (0.0..1.0).
    /// All panes' fractions must sum to 1.0 after normalisation.
    pub height_fraction: f64,
    /// IDs of indicators/series rendered in this pane.
    pub series_ids: Vec<String>,
    /// Price/value scale for this pane.
    pub scale: PriceScale,
}

impl Pane {
    fn new(id: &str, height_fraction: f64, price_range: (f64, f64), height_px: u32) -> Self {
        Self {
            id: id.to_string(),
            height_fraction,
            series_ids: Vec::new(),
            scale: PriceScale::new(price_range.0, price_range.1, height_px),
        }
    }
}

/// The complete pane layout for one chart instance.
pub struct PaneLayout {
    pub panes: Vec<Pane>,
    pub total_height: u32,
    pub total_width: u32,
}

impl PaneLayout {
    /// Create a layout with a single pane that fills the entire chart.
    pub fn single(width: u32, height: u32, price_range: (f64, f64)) -> Self {
        let pane = Pane::new("main", 1.0, price_range, height);
        Self {
            panes: vec![pane],
            total_height: height,
            total_width: width,
        }
    }

    /// Add a new pane with `height_fraction` of total height.
    ///
    /// Fractions are normalised so that all panes always sum to 1.0.
    /// The new pane is appended at the bottom; existing panes are
    /// compressed proportionally.
    pub fn add_pane(&mut self, id: &str, height_fraction: f64) -> &mut Pane {
        let frac = height_fraction.clamp(0.0, 1.0);

        // Scale existing panes down so the total stays at 1.0.
        let remaining = (1.0 - frac).max(0.0);
        let existing_total: f64 = self.panes.iter().map(|p| p.height_fraction).sum();
        if existing_total > 0.0 {
            for pane in &mut self.panes {
                pane.height_fraction = pane.height_fraction / existing_total * remaining;
            }
        }

        let height_px = self.pane_height_for_fraction(frac);
        self.panes.push(Pane::new(
            id,
            frac,
            (0.0, 1.0), // caller should set a real range on scale afterwards
            height_px,
        ));

        self.update_scale_heights();

        self.panes.last_mut().unwrap()
    }

    /// Pixel Y-offset (top edge) of pane at `pane_index`.
    pub fn pane_y_offset(&self, pane_index: usize) -> u32 {
        self.panes[..pane_index]
            .iter()
            .map(|p| self.pane_height_for_fraction(p.height_fraction))
            .sum()
    }

    /// Pixel height of pane at `pane_index`.
    pub fn pane_height(&self, pane_index: usize) -> u32 {
        if pane_index >= self.panes.len() {
            return 0;
        }
        self.pane_height_for_fraction(self.panes[pane_index].height_fraction)
    }

    /// Update total dimensions, recompute each pane's pixel height.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.total_width = width;
        self.total_height = height;
        self.update_scale_heights();
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn pane_height_for_fraction(&self, fraction: f64) -> u32 {
        (self.total_height as f64 * fraction).round() as u32
    }

    /// Push updated pixel heights into each pane's `scale.height`.
    fn update_scale_heights(&mut self) {
        for pane in &mut self.panes {
            let h = (self.total_height as f64 * pane.height_fraction).round() as u32;
            pane.scale.height = h;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── single() ─────────────────────────────────────────────────────────────

    #[test]
    fn single_has_one_pane_with_full_height() {
        let layout = PaneLayout::single(800, 600, (100.0, 200.0));
        assert_eq!(layout.panes.len(), 1);
        assert_eq!(layout.panes[0].height_fraction, 1.0);
        assert_eq!(layout.pane_height(0), 600);
        assert_eq!(layout.pane_y_offset(0), 0);
    }

    #[test]
    fn single_scale_range_preserved() {
        let layout = PaneLayout::single(800, 400, (1000.0, 2000.0));
        let scale = &layout.panes[0].scale;
        assert_eq!(scale.min, 1000.0);
        assert_eq!(scale.max, 2000.0);
        assert_eq!(scale.height, 400);
    }

    // ── add_pane() ───────────────────────────────────────────────────────────

    #[test]
    fn add_pane_fractions_sum_to_one() {
        let mut layout = PaneLayout::single(800, 600, (100.0, 200.0));
        layout.add_pane("rsi", 0.25);

        let total: f64 = layout.panes.iter().map(|p| p.height_fraction).sum();
        assert!((total - 1.0).abs() < 1e-10, "sum={total}");
    }

    #[test]
    fn add_pane_fractions_sum_to_one_after_two_additions() {
        let mut layout = PaneLayout::single(800, 600, (100.0, 200.0));
        layout.add_pane("rsi", 0.20);
        layout.add_pane("macd", 0.20);

        let total: f64 = layout.panes.iter().map(|p| p.height_fraction).sum();
        assert!((total - 1.0).abs() < 1e-10, "sum={total}");
    }

    #[test]
    fn add_pane_returns_correct_pane() {
        let mut layout = PaneLayout::single(800, 600, (100.0, 200.0));
        let pane = layout.add_pane("volume", 0.15);
        assert_eq!(pane.id, "volume");
        assert!((pane.height_fraction - 0.15).abs() < 1e-10);
    }

    // ── pane_y_offset / pane_height ──────────────────────────────────────────

    #[test]
    fn pane_y_offsets_are_contiguous() {
        let mut layout = PaneLayout::single(800, 600, (100.0, 200.0));
        layout.add_pane("rsi", 0.25);

        // pane[0] starts at 0; pane[1] starts at end of pane[0].
        let h0 = layout.pane_height(0);
        let h1 = layout.pane_height(1);
        assert_eq!(layout.pane_y_offset(0), 0);
        assert_eq!(layout.pane_y_offset(1), h0);
        // Sum of heights should equal total (allow ±1 rounding).
        assert!((h0 + h1).abs_diff(600) <= 1);
    }

    // ── resize() ─────────────────────────────────────────────────────────────

    #[test]
    fn resize_updates_pixel_heights() {
        let mut layout = PaneLayout::single(800, 600, (100.0, 200.0));
        layout.add_pane("rsi", 0.25);
        layout.resize(1024, 768);

        assert_eq!(layout.total_height, 768);
        assert_eq!(layout.total_width, 1024);

        // scale.height in each pane should reflect new total.
        for pane in &layout.panes {
            let expected = (768.0 * pane.height_fraction).round() as u32;
            assert_eq!(pane.scale.height, expected);
        }
    }
}
