//! Core types for technical analysis
//!
//! These types are designed to be simple, efficient, and `no_std` compatible.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single OHLCV (Open, High, Low, Close, Volume) candle.
///
/// This is the fundamental data structure for candlestick-based analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ohlcv {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl Ohlcv {
    /// Create a new OHLCV candle.
    #[inline]
    pub const fn new(open: f64, high: f64, low: f64, close: f64, volume: f64) -> Self {
        Self { open, high, low, close, volume }
    }

    /// Create an OHLCV with zero volume.
    #[inline]
    pub const fn from_ohlc(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self::new(open, high, low, close, 0.0)
    }

    /// The typical price: (high + low + close) / 3
    ///
    /// Used in many indicators like MFI, CCI, and VWAP.
    #[inline]
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// The median price: (high + low) / 2
    #[inline]
    pub fn median_price(&self) -> f64 {
        (self.high + self.low) / 2.0
    }

    /// The weighted close: (high + low + close + close) / 4
    #[inline]
    pub fn weighted_close(&self) -> f64 {
        (self.high + self.low + self.close + self.close) / 4.0
    }

    /// The candle range: high - low
    #[inline]
    pub fn range(&self) -> f64 {
        self.high - self.low
    }

    /// The candle body: close - open (positive = bullish, negative = bearish)
    #[inline]
    pub fn body(&self) -> f64 {
        self.close - self.open
    }

    /// Is this a bullish candle? (close >= open)
    #[inline]
    pub fn is_bullish(&self) -> bool {
        self.close >= self.open
    }

    /// Is this a bearish candle? (close < open)
    #[inline]
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Upper shadow/wick length
    #[inline]
    pub fn upper_shadow(&self) -> f64 {
        if self.is_bullish() {
            self.high - self.close
        } else {
            self.high - self.open
        }
    }

    /// Lower shadow/wick length
    #[inline]
    pub fn lower_shadow(&self) -> f64 {
        if self.is_bullish() {
            self.open - self.low
        } else {
            self.close - self.low
        }
    }
}

impl Default for Ohlcv {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, 0.0)
    }
}

/// Trait for types that can provide OHLCV data.
///
/// This allows indicators to work with various candle representations.
pub trait AsOhlcv {
    fn as_ohlcv(&self) -> Ohlcv;
}

impl AsOhlcv for Ohlcv {
    #[inline]
    fn as_ohlcv(&self) -> Ohlcv {
        *self
    }
}

impl AsOhlcv for (f64, f64, f64, f64, f64) {
    #[inline]
    fn as_ohlcv(&self) -> Ohlcv {
        Ohlcv::new(self.0, self.1, self.2, self.3, self.4)
    }
}

impl AsOhlcv for (f64, f64, f64, f64) {
    #[inline]
    fn as_ohlcv(&self) -> Ohlcv {
        Ohlcv::from_ohlc(self.0, self.1, self.2, self.3)
    }
}

/// Result of an indicator calculation that can be a single value or multiple values.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IndicatorValue {
    /// Single value (e.g., RSI, EMA)
    Single(f64),
    /// Two values (e.g., Stochastic %K and %D)
    Dual(f64, f64),
    /// Three values (e.g., MACD line, signal, histogram)
    Triple(f64, f64, f64),
    /// Bands (e.g., Bollinger: middle, upper, lower)
    Bands { middle: f64, upper: f64, lower: f64 },
}

impl IndicatorValue {
    /// Get the primary value (first value).
    #[inline]
    pub fn value(&self) -> f64 {
        match self {
            Self::Single(v) => *v,
            Self::Dual(v, _) => *v,
            Self::Triple(v, _, _) => *v,
            Self::Bands { middle, .. } => *middle,
        }
    }

    /// Convert to f64 (returns primary value).
    #[inline]
    pub fn as_f64(&self) -> f64 {
        self.value()
    }
}

impl From<f64> for IndicatorValue {
    #[inline]
    fn from(v: f64) -> Self {
        Self::Single(v)
    }
}

impl From<(f64, f64)> for IndicatorValue {
    #[inline]
    fn from((a, b): (f64, f64)) -> Self {
        Self::Dual(a, b)
    }
}

impl From<(f64, f64, f64)> for IndicatorValue {
    #[inline]
    fn from((a, b, c): (f64, f64, f64)) -> Self {
        Self::Triple(a, b, c)
    }
}

/// A fixed-size ring buffer for efficient rolling window calculations.
///
/// This is more efficient than Vec for fixed-size windows as it avoids
/// memory allocations and shifts.
#[derive(Clone)]
pub struct RingBuffer<const N: usize> {
    buffer: [f64; N],
    head: usize,
    count: usize,
}

impl<const N: usize> RingBuffer<N> {
    /// Create a new empty ring buffer.
    pub const fn new() -> Self {
        Self {
            buffer: [0.0; N],
            head: 0,
            count: 0,
        }
    }

    /// Push a value into the buffer.
    #[inline]
    pub fn push(&mut self, value: f64) {
        self.buffer[self.head] = value;
        self.head = (self.head + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }

    /// Get the number of values in the buffer.
    #[inline]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Is the buffer empty?
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Is the buffer full?
    #[inline]
    pub const fn is_full(&self) -> bool {
        self.count == N
    }

    /// Get the capacity.
    #[inline]
    pub const fn capacity(&self) -> usize {
        N
    }

    /// Get the oldest value in the buffer.
    #[inline]
    pub fn oldest(&self) -> Option<f64> {
        if self.is_empty() {
            None
        } else if self.is_full() {
            Some(self.buffer[self.head])
        } else {
            Some(self.buffer[0])
        }
    }

    /// Get the newest (most recent) value in the buffer.
    #[inline]
    pub fn newest(&self) -> Option<f64> {
        if self.is_empty() {
            None
        } else {
            let idx = if self.head == 0 { N - 1 } else { self.head - 1 };
            Some(self.buffer[idx])
        }
    }

    /// Iterate over values from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        let start = if self.is_full() { self.head } else { 0 };
        (0..self.count).map(move |i| self.buffer[(start + i) % N])
    }

    /// Sum of all values in the buffer.
    #[inline]
    pub fn sum(&self) -> f64 {
        self.iter().sum()
    }

    /// Clear the buffer.
    #[inline]
    pub fn clear(&mut self) {
        self.head = 0;
        self.count = 0;
    }
}

impl<const N: usize> Default for RingBuffer<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ohlcv_typical_price() {
        let candle = Ohlcv::new(100.0, 110.0, 90.0, 105.0, 1000.0);
        // (110 + 90 + 105) / 3 = 101.666...
        assert!((candle.typical_price() - 101.666666).abs() < 0.001);
    }

    #[test]
    fn test_ohlcv_bullish_bearish() {
        let bullish = Ohlcv::from_ohlc(100.0, 110.0, 95.0, 108.0);
        let bearish = Ohlcv::from_ohlc(100.0, 105.0, 90.0, 92.0);

        assert!(bullish.is_bullish());
        assert!(!bullish.is_bearish());
        assert!(bearish.is_bearish());
        assert!(!bearish.is_bullish());
    }

    #[test]
    fn test_ring_buffer() {
        let mut buf: RingBuffer<3> = RingBuffer::new();

        buf.push(1.0);
        buf.push(2.0);
        assert_eq!(buf.len(), 2);
        assert!(!buf.is_full());

        buf.push(3.0);
        assert!(buf.is_full());
        assert_eq!(buf.oldest(), Some(1.0));
        assert_eq!(buf.newest(), Some(3.0));

        buf.push(4.0);
        assert_eq!(buf.oldest(), Some(2.0)); // 1.0 was dropped
        assert_eq!(buf.newest(), Some(4.0));

        let values: Vec<f64> = buf.iter().collect();
        assert_eq!(values, vec![2.0, 3.0, 4.0]);
    }
}
