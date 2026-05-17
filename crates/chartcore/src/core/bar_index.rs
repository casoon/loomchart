//! Logical bar indexing and session-aware coordinate mapping.
//!
//! `BarIndex` maps timestamps to contiguous logical bar indices, eliminating
//! weekend/session gaps from the x-axis. `BarCoordMapper` converts those
//! indices to screen pixels independently of timestamps.

use crate::core::Candle;

/// Maps timestamps to logical bar indices, handling gaps, sessions, and synthetic bars.
pub struct BarIndex {
    /// Sorted list of (timestamp, bar_index) pairs.
    entries: Vec<(i64, usize)>,
}

impl BarIndex {
    /// Build index from a sorted candle slice.
    pub fn from_candles(candles: &[Candle]) -> Self {
        let entries = candles
            .iter()
            .enumerate()
            .map(|(i, c)| (c.time, i))
            .collect();
        Self { entries }
    }

    /// Logical index for a timestamp (None if not found).
    pub fn time_to_index(&self, time: i64) -> Option<usize> {
        self.entries
            .binary_search_by_key(&time, |&(t, _)| t)
            .ok()
            .map(|pos| self.entries[pos].1)
    }

    /// Timestamp for a logical index.
    pub fn index_to_time(&self, index: usize) -> Option<i64> {
        // entries are stored in ascending index order (from_candles guarantees this)
        self.entries
            .iter()
            .find(|&&(_, i)| i == index)
            .map(|&(t, _)| t)
    }

    /// Total bar count.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Visible bar range (inclusive start, exclusive end) for a time window [start, end].
    ///
    /// Returns `(0, 0)` when no bars fall within the window.
    pub fn visible_range(&self, start: i64, end: i64) -> (usize, usize) {
        if self.entries.is_empty() {
            return (0, 0);
        }

        // First entry whose timestamp >= start
        let first = self
            .entries
            .partition_point(|&(t, _)| t < start);

        // First entry whose timestamp > end
        let last = self
            .entries
            .partition_point(|&(t, _)| t <= end);

        if first >= last {
            return (0, 0);
        }

        let start_idx = self.entries[first].1;
        let end_idx = self.entries[last - 1].1 + 1; // exclusive
        (start_idx, end_idx)
    }
}

/// Converts a logical bar index to a screen x-coordinate, independent of timestamps.
pub struct BarCoordMapper {
    /// Pixels per bar.
    bar_width: f64,
    /// First visible bar index (may be negative when panned past the beginning).
    offset: i64,
    /// Canvas width in pixels.
    canvas_width: u32,
}

impl BarCoordMapper {
    pub fn new(bar_width: f64, offset: i64, canvas_width: u32) -> Self {
        Self {
            bar_width,
            offset,
            canvas_width,
        }
    }

    /// Center x-coordinate for `bar_index`.
    pub fn bar_to_x(&self, bar_index: usize) -> f64 {
        (bar_index as i64 - self.offset) as f64 * self.bar_width + self.bar_width / 2.0
    }

    /// Bar index (can be negative when x is left of the visible area) for pixel `x`.
    pub fn x_to_bar(&self, x: f64) -> i64 {
        ((x - self.bar_width / 2.0) / self.bar_width) as i64 + self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(time: i64) -> Candle {
        Candle::new(time, 1.0, 2.0, 0.5, 1.5, 100.0)
    }

    // --- BarIndex ---

    #[test]
    fn test_empty_index() {
        let idx = BarIndex::from_candles(&[]);
        assert!(idx.is_empty());
        assert_eq!(idx.len(), 0);
        assert_eq!(idx.time_to_index(0), None);
        assert_eq!(idx.index_to_time(0), None);
        assert_eq!(idx.visible_range(0, 1000), (0, 0));
    }

    #[test]
    fn test_regular_spacing() {
        let candles: Vec<Candle> = (0..5).map(|i| make_candle(i * 300)).collect();
        let idx = BarIndex::from_candles(&candles);

        assert_eq!(idx.len(), 5);
        assert_eq!(idx.time_to_index(0), Some(0));
        assert_eq!(idx.time_to_index(300), Some(1));
        assert_eq!(idx.time_to_index(1200), Some(4));
        assert_eq!(idx.time_to_index(500), None); // not a bar boundary

        assert_eq!(idx.index_to_time(0), Some(0));
        assert_eq!(idx.index_to_time(4), Some(1200));
        assert_eq!(idx.index_to_time(5), None);
    }

    #[test]
    fn test_gaps() {
        // Bars at 0, 300, 900 — gap between index 1 and 2 (missing 600)
        let times = [0i64, 300, 900];
        let candles: Vec<Candle> = times.iter().map(|&t| make_candle(t)).collect();
        let idx = BarIndex::from_candles(&candles);

        assert_eq!(idx.len(), 3);
        assert_eq!(idx.time_to_index(300), Some(1));
        assert_eq!(idx.time_to_index(600), None); // gap — not in index
        assert_eq!(idx.time_to_index(900), Some(2));

        // Logical indices are still 0,1,2 — no hole in the index itself
        assert_eq!(idx.index_to_time(2), Some(900));
    }

    #[test]
    fn test_visible_range_full_overlap() {
        let candles: Vec<Candle> = (0..5).map(|i| make_candle(i * 300)).collect();
        let idx = BarIndex::from_candles(&candles);

        // Window covers all bars
        let (s, e) = idx.visible_range(0, 1200);
        assert_eq!(s, 0);
        assert_eq!(e, 5);
    }

    #[test]
    fn test_visible_range_partial_overlap() {
        let candles: Vec<Candle> = (0..5).map(|i| make_candle(i * 300)).collect();
        let idx = BarIndex::from_candles(&candles);

        // Window covers bars at 300, 600, 900 (indices 1..3)
        let (s, e) = idx.visible_range(300, 900);
        assert_eq!(s, 1);
        assert_eq!(e, 4); // exclusive: bar at 900 is index 3, so end = 4
    }

    #[test]
    fn test_visible_range_no_overlap() {
        let candles: Vec<Candle> = (0..3).map(|i| make_candle(i * 300)).collect();
        let idx = BarIndex::from_candles(&candles);

        let (s, e) = idx.visible_range(5000, 9000);
        assert_eq!((s, e), (0, 0));
    }

    // --- BarCoordMapper ---

    #[test]
    fn test_bar_to_x_basic() {
        // bar_width=10, offset=0 → bar 0 centers at x=5, bar 1 at x=15
        let mapper = BarCoordMapper::new(10.0, 0, 800);
        assert_eq!(mapper.bar_to_x(0), 5.0);
        assert_eq!(mapper.bar_to_x(1), 15.0);
        assert_eq!(mapper.bar_to_x(9), 95.0);
    }

    #[test]
    fn test_bar_to_x_with_offset() {
        // offset=5 means the 5th bar is the first visible one (x=5)
        let mapper = BarCoordMapper::new(10.0, 5, 800);
        assert_eq!(mapper.bar_to_x(5), 5.0);
        assert_eq!(mapper.bar_to_x(6), 15.0);
    }

    #[test]
    fn test_x_to_bar_basic() {
        let mapper = BarCoordMapper::new(10.0, 0, 800);
        assert_eq!(mapper.x_to_bar(5.0), 0);
        assert_eq!(mapper.x_to_bar(15.0), 1);
    }

    #[test]
    fn test_x_to_bar_negative() {
        // offset=0, bar_width=10 → x<5 maps to bar -1 (left of visible area)
        let mapper = BarCoordMapper::new(10.0, 0, 800);
        assert_eq!(mapper.x_to_bar(-5.0), -1);
    }

    #[test]
    fn test_round_trip() {
        let mapper = BarCoordMapper::new(8.0, 3, 800);
        for bar_index in 3usize..20 {
            let x = mapper.bar_to_x(bar_index);
            let recovered = mapper.x_to_bar(x);
            assert_eq!(recovered, bar_index as i64, "round-trip failed for bar {bar_index}");
        }
    }
}
