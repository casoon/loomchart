# Visualization Readiness Checklist — Real Market Data

Tracks what must be verified before the Rust/WASM + Canvas engine is considered ready
for real market data. Check items off as the corresponding work is merged.

Related issues: #57, #58, #59, #61, #62, #63

---

## 1. Data Schema

- [ ] Timestamps are Unix milliseconds (not seconds, not microseconds) — validate on ingest (#57)
- [ ] OHLCV fields present and non-null: `open`, `high`, `low`, `close`, `volume`, `ts`
- [ ] `is_final` flag present on every candle message from the Phoenix channel
- [ ] `high >= max(open, close)` and `low <= min(open, close)` enforced before rendering
- [ ] `volume` is `f64 >= 0`; negative or NaN volume is rejected

## 2. Sorting & Gaps

- [ ] Candle slice delivered to the engine is sorted ascending by `ts` (#57)
- [ ] Engine handles non-contiguous candles (gaps for weekends, halts, illiquid instruments) without visual artifacts
- [ ] Gaps wider than N timeframe buckets are rendered as blank space, not collapsed
- [ ] Missing closing candle for the current bar does not crash or stall the render loop

## 3. Corrections & Out-of-Order Updates

- [ ] Late-arriving candle for a closed bar (correction) is applied correctly (#58)
- [ ] Out-of-order `candle_update` messages do not corrupt ring buffer state
- [ ] Duplicate `ts` for the same bar is idempotent (last-write-wins)
- [ ] `candle_final` followed by another `candle_update` for the same bar is handled gracefully

## 4. Snapshots & History Loading

- [ ] `candle_snapshot` on channel join is merged without duplication with REST history (#59)
- [ ] REST + snapshot overlap window is de-duplicated before passing to the engine
- [ ] Symbol/timeframe switch flushes all previous candles from engine state before loading new data
- [ ] History load of 1 000 candles renders without perceptible jank (< 100 ms to first paint)
- [ ] History load of 10 000 candles renders without crash or OOM
- [ ] History load of 50 000 candles: viewport-culled render stays under 16 ms per frame (#61)
- [ ] History load of 100 000 candles: memory stays within acceptable bounds; test on a 4 GB device

## 5. Live Updates

- [ ] `candle_update` for the running bar updates the last candle in-place without a full re-render (#62)
- [ ] `candle_final` promotes the running bar and allocates a new bar slot
- [ ] Crosshair position is stable during rapid `candle_update` bursts (no jitter or flicker)
- [ ] Pan (drag) during live feed does not lose the current viewport offset
- [ ] Zoom during live feed does not reset to the latest bar
- [ ] Auto-scroll to latest bar re-engages only when the user was already at the right edge

## 6. Overlays & Indicators

- [ ] Indicator values are only committed on `is_final=true`; running-bar preview uses a separate slot
- [ ] Toggling an indicator mid-stream does not corrupt existing series data
- [ ] Multiple overlays rendered on the same pane do not bleed z-order (#63)
- [ ] Sub-pane (separate panel) indicators resize correctly when the main chart height changes
- [ ] Indicator with NaN output for initial warm-up period renders a gap, not zero

## 7. Test Data Fixtures

The following fixture files are needed under `tests/fixtures/candles/`:

- [ ] `normal_1h_500.json` — 500 clean 1-hour EURUSD candles, no gaps
- [ ] `sparse_1d_with_gaps.json` — daily candles with multi-day weekend/holiday gaps
- [ ] `volatile_spike.json` — candles with extreme wicks (> 3× ATR body)
- [ ] `flat_consolidation.json` — 200 candles with near-zero range (low-volatility period)
- [ ] `malformed_mixed.json` — mix of valid candles and entries with null fields, wrong field types, negative volume; engine must reject bad entries and render the rest
- [ ] `out_of_order.json` — 100 candles shuffled; consumer must sort before handing to engine
- [ ] `correction_sequence.json` — late correction candle for a closed bar followed by normal live updates

---

## Definition of Done

All checkboxes in sections 1–6 are ticked, all fixture files in section 7 exist and are
used by at least one automated test (`cargo test` or Playwright).
