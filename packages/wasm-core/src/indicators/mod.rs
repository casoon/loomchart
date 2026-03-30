//! Indicator calculation engine
//!
//! Uses the loom-indicators library for calculations and wraps them
//! for the WASM UI context.

use std::collections::{HashMap, VecDeque};
use crate::types::Candle;
use crate::bridge;

// Re-export from loom-indicators for convenience
use loom_indicators::indicators::{
    Ema as LibEma,
    Rsi as LibRsi,
    Macd as LibMacd,
    BollingerBands as LibBollinger,
    Atr as LibAtr,
    Next, Reset,
};
use loom_indicators::types::Ohlcv;

/// Wrapper trait for all indicators in the UI context
pub trait UiIndicator {
    /// Process a new candle and return the calculated value
    fn on_candle(&mut self, candle: &Candle) -> Option<f64>;

    /// Reset the indicator state
    fn reset(&mut self);

    /// Recalculate from historical data
    fn recalculate(&mut self, candles: &VecDeque<Candle>) -> Vec<(i64, f64)>;

    /// Get the indicator name/id
    fn id(&self) -> &str;
}

/// Convert our Candle to loom-indicators Ohlcv
fn to_ohlcv(candle: &Candle) -> Ohlcv {
    Ohlcv::new(candle.o, candle.h, candle.l, candle.c, candle.v)
}

/// Parse timestamp to unix seconds
fn parse_ts(ts: &str) -> Option<i64> {
    if ts.len() < 19 {
        return None;
    }

    let year: i32 = ts[0..4].parse().ok()?;
    let month: u32 = ts[5..7].parse().ok()?;
    let day: u32 = ts[8..10].parse().ok()?;
    let hour: u32 = ts[11..13].parse().ok()?;
    let min: u32 = ts[14..16].parse().ok()?;
    let sec: u32 = ts[17..19].parse().ok()?;

    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m as i32 - 3) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;

    Some((days as i64) * 86400 + (hour as i64) * 3600 + (min as i64) * 60 + (sec as i64))
}

// === EMA Wrapper ===

pub struct Ema {
    id: String,
    inner: LibEma,
}

impl Ema {
    pub fn new(id: &str, period: usize) -> Self {
        Self {
            id: id.to_string(),
            inner: LibEma::new(period),
        }
    }
}

impl UiIndicator for Ema {
    fn on_candle(&mut self, candle: &Candle) -> Option<f64> {
        self.inner.next(candle.c)
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn recalculate(&mut self, candles: &VecDeque<Candle>) -> Vec<(i64, f64)> {
        self.reset();
        let mut results = Vec::new();

        for candle in candles {
            if let Some(value) = self.inner.next(candle.c) {
                if let Some(ts) = parse_ts(&candle.ts) {
                    results.push((ts, value));
                }
            }
        }

        results
    }

    fn id(&self) -> &str {
        &self.id
    }
}

// === RSI Wrapper ===

pub struct Rsi {
    id: String,
    inner: LibRsi,
}

impl Rsi {
    pub fn new(id: &str, period: usize) -> Self {
        Self {
            id: id.to_string(),
            inner: LibRsi::new(period),
        }
    }
}

impl UiIndicator for Rsi {
    fn on_candle(&mut self, candle: &Candle) -> Option<f64> {
        self.inner.next(candle.c)
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn recalculate(&mut self, candles: &VecDeque<Candle>) -> Vec<(i64, f64)> {
        self.reset();
        let mut results = Vec::new();

        for candle in candles {
            if let Some(value) = self.inner.next(candle.c) {
                if let Some(ts) = parse_ts(&candle.ts) {
                    results.push((ts, value));
                }
            }
        }

        results
    }

    fn id(&self) -> &str {
        &self.id
    }
}

// === MACD Wrapper (returns MACD line only for simple chart) ===

pub struct MacdLine {
    id: String,
    inner: LibMacd,
}

impl MacdLine {
    pub fn new(id: &str, fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            id: id.to_string(),
            inner: LibMacd::new(fast, slow, signal),
        }
    }
}

impl UiIndicator for MacdLine {
    fn on_candle(&mut self, candle: &Candle) -> Option<f64> {
        self.inner.next(candle.c).map(|out| out.macd)
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn recalculate(&mut self, candles: &VecDeque<Candle>) -> Vec<(i64, f64)> {
        self.reset();
        let mut results = Vec::new();

        for candle in candles {
            if let Some(out) = self.inner.next(candle.c) {
                if let Some(ts) = parse_ts(&candle.ts) {
                    results.push((ts, out.macd));
                }
            }
        }

        results
    }

    fn id(&self) -> &str {
        &self.id
    }
}

// === Bollinger Bands Wrapper (returns middle band) ===

pub struct BollingerMiddle {
    id: String,
    inner: LibBollinger,
}

impl BollingerMiddle {
    pub fn new(id: &str, period: usize, multiplier: f64) -> Self {
        Self {
            id: id.to_string(),
            inner: LibBollinger::new(period, multiplier),
        }
    }
}

impl UiIndicator for BollingerMiddle {
    fn on_candle(&mut self, candle: &Candle) -> Option<f64> {
        self.inner.next(candle.c).map(|out| out.middle)
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn recalculate(&mut self, candles: &VecDeque<Candle>) -> Vec<(i64, f64)> {
        self.reset();
        let mut results = Vec::new();

        for candle in candles {
            if let Some(out) = self.inner.next(candle.c) {
                if let Some(ts) = parse_ts(&candle.ts) {
                    results.push((ts, out.middle));
                }
            }
        }

        results
    }

    fn id(&self) -> &str {
        &self.id
    }
}

// === ATR Wrapper ===

pub struct Atr {
    id: String,
    inner: LibAtr,
}

impl Atr {
    pub fn new(id: &str, period: usize) -> Self {
        Self {
            id: id.to_string(),
            inner: LibAtr::new(period),
        }
    }
}

impl UiIndicator for Atr {
    fn on_candle(&mut self, candle: &Candle) -> Option<f64> {
        let ohlcv = to_ohlcv(candle);
        self.inner.next(&ohlcv)
    }

    fn reset(&mut self) {
        self.inner.reset();
    }

    fn recalculate(&mut self, candles: &VecDeque<Candle>) -> Vec<(i64, f64)> {
        self.reset();
        let mut results = Vec::new();

        for candle in candles {
            let ohlcv = to_ohlcv(candle);
            if let Some(value) = self.inner.next(&ohlcv) {
                if let Some(ts) = parse_ts(&candle.ts) {
                    results.push((ts, value));
                }
            }
        }

        results
    }

    fn id(&self) -> &str {
        &self.id
    }
}

/// Manages active indicators
pub struct IndicatorEngine {
    indicators: HashMap<String, Box<dyn UiIndicator>>,
}

impl IndicatorEngine {
    pub fn new() -> Self {
        Self {
            indicators: HashMap::new(),
        }
    }

    /// Enable an indicator with given parameters
    pub fn enable(&mut self, name: &str, candles: &VecDeque<Candle>) {
        let indicator: Box<dyn UiIndicator> = match name {
            // EMA variants
            "ema" | "ema21" => Box::new(Ema::new("ema21", 21)),
            "ema9" => Box::new(Ema::new("ema9", 9)),
            "ema50" => Box::new(Ema::new("ema50", 50)),
            "ema200" => Box::new(Ema::new("ema200", 200)),

            // RSI
            "rsi" | "rsi14" => Box::new(Rsi::new("rsi14", 14)),
            "rsi7" => Box::new(Rsi::new("rsi7", 7)),
            "rsi21" => Box::new(Rsi::new("rsi21", 21)),

            // MACD
            "macd" => Box::new(MacdLine::new("macd", 12, 26, 9)),

            // Bollinger
            "bb" | "bollinger" => Box::new(BollingerMiddle::new("bollinger", 20, 2.0)),

            // ATR
            "atr" | "atr14" => Box::new(Atr::new("atr14", 14)),

            _ => {
                log::warn!("Unknown indicator: {}", name);
                return;
            }
        };

        let id = indicator.id().to_string();
        let mut indicator = indicator;

        // Calculate historical values
        let points = indicator.recalculate(candles);

        // Send to chart
        bridge::set_line_series(&id, &points);

        self.indicators.insert(id, indicator);
    }

    /// Disable an indicator
    pub fn disable(&mut self, name: &str) {
        if self.indicators.remove(name).is_some() {
            bridge::remove_indicator(name);
            log::info!("Indicator {} disabled and removed from chart", name);
        }
    }

    /// Process new candle for all active indicators
    pub fn on_candle(&mut self, candle: &Candle) {
        for (id, indicator) in &mut self.indicators {
            if let Some(value) = indicator.on_candle(candle) {
                let time = parse_ts(&candle.ts).unwrap_or(0);
                bridge::update_line_point(id, time, value);
            }
        }
    }

    /// Recalculate all indicators from historical data
    pub fn recalculate(&mut self, candles: &VecDeque<Candle>) {
        for (id, indicator) in &mut self.indicators {
            let points = indicator.recalculate(candles);
            bridge::set_line_series(id, &points);
        }
    }

    /// Reset all indicators
    pub fn reset(&mut self) {
        for indicator in self.indicators.values_mut() {
            indicator.reset();
        }
    }
}

impl Default for IndicatorEngine {
    fn default() -> Self {
        Self::new()
    }
}
