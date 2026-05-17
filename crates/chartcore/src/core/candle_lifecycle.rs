use crate::core::types::Candle;

/// Events from the data feed driving candle state transitions.
pub enum CandleEvent {
    /// Full history replace — discards all existing state.
    Snapshot(Vec<Candle>),
    /// Running (in-flight) candle update; same timestamp as the current running bar.
    Update(Candle),
    /// The running candle is closed; move it to the finalized list.
    Final(Candle),
    /// Out-of-order correction for an already-finalized candle.
    Correction { time: i64, candle: Candle },
    /// Reconnect backfill — merge into finalized list without creating duplicates.
    Backfill(Vec<Candle>),
}

/// Holds finalized candles and an optional in-flight running bar.
pub struct CandleStore {
    candles: Vec<Candle>, // finalized, sorted by time ascending
    running: Option<Candle>,
}

impl CandleStore {
    pub fn new() -> Self {
        Self {
            candles: Vec::new(),
            running: None,
        }
    }

    pub fn apply(&mut self, event: CandleEvent) {
        match event {
            CandleEvent::Snapshot(candles) => {
                let mut sorted = candles;
                sorted.sort_by_key(|c| c.time);
                self.candles = sorted;
                self.running = None;
            }

            CandleEvent::Update(candle) => {
                self.running = Some(candle);
            }

            CandleEvent::Final(candle) => {
                self.running = None;
                // Insert maintaining sort order; replace if same timestamp exists.
                match self.candles.binary_search_by_key(&candle.time, |c| c.time) {
                    Ok(idx) => self.candles[idx] = candle,
                    Err(idx) => self.candles.insert(idx, candle),
                }
            }

            CandleEvent::Correction { time, candle } => {
                if let Ok(idx) = self.candles.binary_search_by_key(&time, |c| c.time) {
                    self.candles[idx] = candle;
                }
                // If not found in finalized list, silently ignore (no-op).
            }

            CandleEvent::Backfill(candles) => {
                for candle in candles {
                    match self.candles.binary_search_by_key(&candle.time, |c| c.time) {
                        Ok(_) => {} // duplicate — skip
                        Err(idx) => self.candles.insert(idx, candle),
                    }
                }
            }
        }
    }

    /// Returns only the finalized candles, sorted by time ascending.
    pub fn candles(&self) -> &[Candle] {
        &self.candles
    }

    /// Returns the in-flight running candle, if any.
    pub fn running(&self) -> Option<&Candle> {
        self.running.as_ref()
    }

    /// Iterates finalized candles followed by the running candle (if present).
    pub fn all_candles(&self) -> impl Iterator<Item = &Candle> {
        self.candles.iter().chain(self.running.iter())
    }
}

impl Default for CandleStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(time: i64, price: f64) -> Candle {
        Candle::new(time, price, price, price, price, 1.0)
    }

    // ------------------------------------------------------------------
    // Snapshot
    // ------------------------------------------------------------------

    #[test]
    fn snapshot_replaces_all_state() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Update(c(100, 1.0)));
        store.apply(CandleEvent::Snapshot(vec![c(10, 1.0), c(20, 2.0)]));

        assert_eq!(store.candles().len(), 2);
        assert!(store.running().is_none());
        assert_eq!(store.candles()[0].time, 10);
        assert_eq!(store.candles()[1].time, 20);
    }

    #[test]
    fn snapshot_sorts_unsorted_input() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Snapshot(vec![c(30, 3.0), c(10, 1.0), c(20, 2.0)]));

        let times: Vec<i64> = store.candles().iter().map(|c| c.time).collect();
        assert_eq!(times, vec![10, 20, 30]);
    }

    // ------------------------------------------------------------------
    // Update
    // ------------------------------------------------------------------

    #[test]
    fn update_creates_running_candle() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Update(c(100, 5.0)));

        assert_eq!(store.running().unwrap().time, 100);
        assert!(store.candles().is_empty());
    }

    #[test]
    fn update_overwrites_existing_running_candle() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Update(c(100, 5.0)));
        store.apply(CandleEvent::Update(c(100, 6.0)));

        assert_eq!(store.running().unwrap().c, 6.0);
    }

    // ------------------------------------------------------------------
    // Final
    // ------------------------------------------------------------------

    #[test]
    fn final_moves_running_to_finalized() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Update(c(100, 5.0)));
        store.apply(CandleEvent::Final(c(100, 5.5)));

        assert!(store.running().is_none());
        assert_eq!(store.candles().len(), 1);
        assert_eq!(store.candles()[0].time, 100);
        assert_eq!(store.candles()[0].c, 5.5);
    }

    #[test]
    fn final_maintains_sort_order() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Snapshot(vec![c(50, 1.0), c(150, 2.0)]));
        store.apply(CandleEvent::Final(c(100, 3.0)));

        let times: Vec<i64> = store.candles().iter().map(|c| c.time).collect();
        assert_eq!(times, vec![50, 100, 150]);
    }

    #[test]
    fn final_out_of_order_stays_sorted() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Snapshot(vec![c(200, 1.0), c(300, 2.0)]));
        // Finalize a candle that belongs before existing ones.
        store.apply(CandleEvent::Final(c(100, 3.0)));

        let times: Vec<i64> = store.candles().iter().map(|c| c.time).collect();
        assert_eq!(times, vec![100, 200, 300]);
    }

    // ------------------------------------------------------------------
    // Correction
    // ------------------------------------------------------------------

    #[test]
    fn correction_replaces_finalized_candle() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Snapshot(vec![c(100, 1.0), c(200, 2.0)]));
        store.apply(CandleEvent::Correction {
            time: 100,
            candle: c(100, 9.0),
        });

        assert_eq!(store.candles()[0].c, 9.0);
        assert_eq!(store.candles().len(), 2); // no extra entry added
    }

    #[test]
    fn correction_unknown_time_is_noop() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Snapshot(vec![c(100, 1.0)]));
        store.apply(CandleEvent::Correction {
            time: 999,
            candle: c(999, 9.0),
        });

        assert_eq!(store.candles().len(), 1);
    }

    // ------------------------------------------------------------------
    // Backfill
    // ------------------------------------------------------------------

    #[test]
    fn backfill_merges_without_duplicates() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Snapshot(vec![c(100, 1.0), c(200, 2.0)]));
        // 100 already exists; 150 is new.
        store.apply(CandleEvent::Backfill(vec![c(100, 9.0), c(150, 1.5)]));

        let times: Vec<i64> = store.candles().iter().map(|c| c.time).collect();
        assert_eq!(times, vec![100, 150, 200]);
        // Existing entry at 100 must NOT be overwritten.
        assert_eq!(store.candles()[0].c, 1.0);
    }

    #[test]
    fn backfill_maintains_sort_order() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Backfill(vec![c(300, 3.0), c(100, 1.0), c(200, 2.0)]));

        let times: Vec<i64> = store.candles().iter().map(|c| c.time).collect();
        assert_eq!(times, vec![100, 200, 300]);
    }

    // ------------------------------------------------------------------
    // all_candles
    // ------------------------------------------------------------------

    #[test]
    fn all_candles_includes_running() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Snapshot(vec![c(100, 1.0), c(200, 2.0)]));
        store.apply(CandleEvent::Update(c(300, 3.0)));

        let all: Vec<i64> = store.all_candles().map(|c| c.time).collect();
        assert_eq!(all, vec![100, 200, 300]);
    }

    #[test]
    fn all_candles_without_running() {
        let mut store = CandleStore::new();
        store.apply(CandleEvent::Snapshot(vec![c(100, 1.0)]));

        let count = store.all_candles().count();
        assert_eq!(count, 1);
    }
}
