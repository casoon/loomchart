//! Sorted, deduplicated candle buffer for the chart engine.

use crate::core::Candle;

/// Sorted `Vec<Candle>` with dedup on timestamp.
///
/// All mutating operations maintain ascending `time` order and keep at most
/// one entry per timestamp (last write wins for `append`/`update_running`).
pub struct CandleBuffer {
    candles: Vec<Candle>,
}

impl CandleBuffer {
    /// Create an empty buffer.
    pub fn new() -> Self {
        Self {
            candles: Vec::new(),
        }
    }

    /// Replace all candles with a sorted, deduped snapshot.
    ///
    /// If `candles` contains duplicate timestamps the last occurrence in the
    /// input slice wins (after sorting, earlier entries are overwritten).
    pub fn snapshot(&mut self, candles: Vec<Candle>) {
        self.candles = candles;
        self.sort_and_dedup();
    }

    /// Add new candles without resetting existing data.
    ///
    /// Incoming candles are merged, sorted, and deduped.  When a timestamp
    /// already exists the incoming value replaces the stored one.
    pub fn append(&mut self, candles: &[Candle]) {
        self.candles.extend_from_slice(candles);
        self.sort_and_dedup();
    }

    /// Upsert a single candle by timestamp (for live tick updates).
    ///
    /// If a candle with the same `time` already exists it is replaced in-place
    /// (without re-sorting, since the position cannot change).  Otherwise the
    /// candle is inserted at the correct sorted position.
    pub fn update_running(&mut self, candle: Candle) {
        match self.candles.binary_search_by_key(&candle.time, |c| c.time) {
            Ok(pos) => self.candles[pos] = candle,
            Err(pos) => self.candles.insert(pos, candle),
        }
    }

    /// Mark a running candle as final.
    ///
    /// This is currently a no-op: `Candle` has no `is_final` flag at this
    /// layer (finality is tracked at the WASM-bridge level).  The method
    /// exists so callers can express intent without breaking the API if a
    /// flag is added later.
    pub fn finalize(&mut self, _timestamp: i64) {
        // No-op – see doc comment.
    }

    /// Read-only view of all candles in ascending time order.
    pub fn candles(&self) -> &[Candle] {
        &self.candles
    }

    // ---- internal helpers -----------------------------------------------

    fn sort_and_dedup(&mut self) {
        // Stable sort so that among equal timestamps the *last* item in the
        // original order survives the dedup step below.
        self.candles.sort_by_key(|c| c.time);

        // `dedup_by` retains the *first* of each run, so reverse first so
        // that the originally-last duplicate ends up first after sorting, then
        // dedup, then reverse back.  Simpler: just do a manual dedup pass
        // that keeps the last occurrence (i.e. iterate and overwrite).
        let mut write = 0usize;
        let mut i = 0usize;
        let len = self.candles.len();

        while i < len {
            // Find the last index with the same timestamp.
            let mut j = i;
            while j + 1 < len && self.candles[j + 1].time == self.candles[i].time {
                j += 1;
            }
            // Keep the last occurrence.
            self.candles.swap(write, j);
            write += 1;
            i = j + 1;
        }

        self.candles.truncate(write);
        // The swap above may have disrupted order within equal-time groups
        // (there are none after dedup), but if timestamps are all unique after
        // the pass, the overall order from the initial sort is still correct
        // because we only moved items *within* equal-key runs.
    }
}

impl Default for CandleBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(time: i64, price: f64) -> Candle {
        Candle::new(time, price, price, price, price, 0.0)
    }

    #[test]
    fn snapshot_replaces_all() {
        let mut buf = CandleBuffer::new();
        buf.snapshot(vec![c(3, 3.0), c(1, 1.0), c(2, 2.0)]);
        assert_eq!(buf.candles().len(), 3);
        assert_eq!(buf.candles()[0].time, 1);
        assert_eq!(buf.candles()[2].time, 3);

        buf.snapshot(vec![c(10, 10.0)]);
        assert_eq!(buf.candles().len(), 1);
        assert_eq!(buf.candles()[0].time, 10);
    }

    #[test]
    fn append_dedup_keeps_last_value() {
        let mut buf = CandleBuffer::new();
        buf.snapshot(vec![c(1, 1.0), c(2, 2.0)]);

        // Append with a duplicate timestamp – new value should win.
        buf.append(&[c(2, 99.0), c(3, 3.0)]);

        assert_eq!(buf.candles().len(), 3);
        assert_eq!(buf.candles()[1].time, 2);
        assert_eq!(buf.candles()[1].o, 99.0); // overwritten
        assert_eq!(buf.candles()[2].time, 3);
    }

    #[test]
    fn append_out_of_order_sorts() {
        let mut buf = CandleBuffer::new();
        buf.append(&[c(5, 5.0), c(1, 1.0), c(3, 3.0)]);
        let times: Vec<i64> = buf.candles().iter().map(|c| c.time).collect();
        assert_eq!(times, vec![1, 3, 5]);
    }

    #[test]
    fn update_running_upsert_existing() {
        let mut buf = CandleBuffer::new();
        buf.snapshot(vec![c(1, 1.0), c(2, 2.0), c(3, 3.0)]);

        // Update an existing candle.
        buf.update_running(c(2, 42.0));
        assert_eq!(buf.candles().len(), 3);
        assert_eq!(buf.candles()[1].o, 42.0);
    }

    #[test]
    fn update_running_insert_new() {
        let mut buf = CandleBuffer::new();
        buf.snapshot(vec![c(1, 1.0), c(3, 3.0)]);

        // Insert a new candle in the middle.
        buf.update_running(c(2, 2.0));
        assert_eq!(buf.candles().len(), 3);
        let times: Vec<i64> = buf.candles().iter().map(|c| c.time).collect();
        assert_eq!(times, vec![1, 2, 3]);
    }

    #[test]
    fn mixed_order_snapshot_then_append() {
        let mut buf = CandleBuffer::new();
        buf.snapshot(vec![c(10, 10.0), c(5, 5.0), c(8, 8.0)]);
        buf.append(&[c(7, 7.0), c(5, 55.0)]); // 5 is a duplicate with new value

        let times: Vec<i64> = buf.candles().iter().map(|c| c.time).collect();
        assert_eq!(times, vec![5, 7, 8, 10]);
        assert_eq!(buf.candles()[0].o, 55.0); // new value for ts=5
    }

    #[test]
    fn finalize_is_noop() {
        let mut buf = CandleBuffer::new();
        buf.snapshot(vec![c(1, 1.0)]);
        buf.finalize(1);
        assert_eq!(buf.candles().len(), 1);
    }
}
