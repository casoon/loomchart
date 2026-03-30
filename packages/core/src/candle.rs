//! Candle/Bar data structures.
//!
//! OHLCV representation with various utilities.

use crate::{Price, Volume, Timestamp};
use core::cmp::Ordering;

/// OHLCV trait for any type that represents price data
pub trait OHLCV {
    fn open(&self) -> Price;
    fn high(&self) -> Price;
    fn low(&self) -> Price;
    fn close(&self) -> Price;
    fn volume(&self) -> Volume;

    /// True range (max of: high-low, |high-prev_close|, |low-prev_close|)
    #[inline]
    fn true_range(&self, prev_close: Price) -> Price {
        let hl = self.high() - self.low();
        let hc = (self.high() - prev_close).abs();
        let lc = (self.low() - prev_close).abs();
        hl.max(hc).max(lc)
    }

    /// Typical price: (H + L + C) / 3
    #[inline]
    fn typical_price(&self) -> Price {
        (self.high() + self.low() + self.close()) / 3.0
    }

    /// Median price: (H + L) / 2
    #[inline]
    fn median_price(&self) -> Price {
        (self.high() + self.low()) / 2.0
    }

    /// Weighted close: (H + L + C + C) / 4
    #[inline]
    fn weighted_close(&self) -> Price {
        (self.high() + self.low() + self.close() * 2.0) / 4.0
    }

    /// Body size (absolute)
    #[inline]
    fn body(&self) -> Price {
        (self.close() - self.open()).abs()
    }

    /// Full range (high - low)
    #[inline]
    fn range(&self) -> Price {
        self.high() - self.low()
    }

    /// Upper shadow/wick
    #[inline]
    fn upper_shadow(&self) -> Price {
        self.high() - self.open().max(self.close())
    }

    /// Lower shadow/wick
    #[inline]
    fn lower_shadow(&self) -> Price {
        self.open().min(self.close()) - self.low()
    }

    /// Is bullish (close >= open)
    #[inline]
    fn is_bullish(&self) -> bool {
        self.close() >= self.open()
    }

    /// Is bearish (close < open)
    #[inline]
    fn is_bearish(&self) -> bool {
        self.close() < self.open()
    }

    /// Body to range ratio (0.0 to 1.0)
    #[inline]
    fn body_ratio(&self) -> Price {
        let range = self.range();
        if range == 0.0 {
            0.0
        } else {
            self.body() / range
        }
    }
}

/// Standard candle with timestamp
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Candle {
    /// Timestamp (milliseconds since Unix epoch)
    pub time: Timestamp,
    /// Open price
    pub open: Price,
    /// High price
    pub high: Price,
    /// Low price
    pub low: Price,
    /// Close price
    pub close: Price,
    /// Volume
    pub volume: Volume,
}

impl Candle {
    /// Create a new candle
    #[inline]
    pub const fn new(
        time: Timestamp,
        open: Price,
        high: Price,
        low: Price,
        close: Price,
        volume: Volume,
    ) -> Self {
        Self { time, open, high, low, close, volume }
    }

    /// Create from OHLCV tuple
    #[inline]
    pub const fn from_ohlcv(time: Timestamp, ohlcv: (Price, Price, Price, Price, Volume)) -> Self {
        Self {
            time,
            open: ohlcv.0,
            high: ohlcv.1,
            low: ohlcv.2,
            close: ohlcv.3,
            volume: ohlcv.4,
        }
    }

    /// Create an empty/placeholder candle
    #[inline]
    pub const fn empty(time: Timestamp, price: Price) -> Self {
        Self {
            time,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 0.0,
        }
    }

    /// Update candle with a new tick price and volume
    #[inline]
    pub fn update(&mut self, price: Price, volume: Volume) {
        self.high = self.high.max(price);
        self.low = self.low.min(price);
        self.close = price;
        self.volume += volume;
    }

    /// Merge another candle into this one (for aggregation)
    #[inline]
    pub fn merge(&mut self, other: &Candle) {
        self.high = self.high.max(other.high);
        self.low = self.low.min(other.low);
        self.close = other.close;
        self.volume += other.volume;
    }

    /// Check if candle data is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.high >= self.low
            && self.high >= self.open
            && self.high >= self.close
            && self.low <= self.open
            && self.low <= self.close
            && self.volume >= 0.0
    }
}

impl OHLCV for Candle {
    #[inline] fn open(&self) -> Price { self.open }
    #[inline] fn high(&self) -> Price { self.high }
    #[inline] fn low(&self) -> Price { self.low }
    #[inline] fn close(&self) -> Price { self.close }
    #[inline] fn volume(&self) -> Volume { self.volume }
}

impl Default for Candle {
    fn default() -> Self {
        Self::new(0, 0.0, 0.0, 0.0, 0.0, 0.0)
    }
}

/// Lightweight bar without timestamp (for rolling windows)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bar {
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Volume,
}

impl Bar {
    #[inline]
    pub const fn new(open: Price, high: Price, low: Price, close: Price, volume: Volume) -> Self {
        Self { open, high, low, close, volume }
    }

    #[inline]
    pub const fn from_candle(candle: &Candle) -> Self {
        Self {
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
            volume: candle.volume,
        }
    }
}

impl OHLCV for Bar {
    #[inline] fn open(&self) -> Price { self.open }
    #[inline] fn high(&self) -> Price { self.high }
    #[inline] fn low(&self) -> Price { self.low }
    #[inline] fn close(&self) -> Price { self.close }
    #[inline] fn volume(&self) -> Volume { self.volume }
}

impl From<Candle> for Bar {
    fn from(c: Candle) -> Self {
        Bar::from_candle(&c)
    }
}

/// Candle with additional trading info
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TradingCandle {
    /// Base candle
    pub candle: Candle,
    /// Number of trades
    pub trades: u64,
    /// Buy volume
    pub buy_volume: Volume,
    /// Sell volume
    pub sell_volume: Volume,
    /// VWAP for this candle
    pub vwap: Price,
}

impl TradingCandle {
    #[inline]
    pub fn new(candle: Candle) -> Self {
        Self {
            candle,
            trades: 0,
            buy_volume: 0.0,
            sell_volume: 0.0,
            vwap: candle.typical_price(),
        }
    }

    /// Volume delta (buy - sell)
    #[inline]
    pub fn volume_delta(&self) -> Volume {
        self.buy_volume - self.sell_volume
    }
}

impl OHLCV for TradingCandle {
    #[inline] fn open(&self) -> Price { self.candle.open }
    #[inline] fn high(&self) -> Price { self.candle.high }
    #[inline] fn low(&self) -> Price { self.candle.low }
    #[inline] fn close(&self) -> Price { self.candle.close }
    #[inline] fn volume(&self) -> Volume { self.candle.volume }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_basics() {
        let c = Candle::new(1000, 100.0, 110.0, 95.0, 105.0, 1000.0);
        assert!(c.is_bullish());
        assert_eq!(c.range(), 15.0);
        assert_eq!(c.body(), 5.0);
        assert_eq!(c.upper_shadow(), 5.0);  // 110 - 105
        assert_eq!(c.lower_shadow(), 5.0);  // 100 - 95
    }

    #[test]
    fn test_candle_update() {
        let mut c = Candle::empty(1000, 100.0);
        c.update(105.0, 10.0);
        c.update(95.0, 20.0);
        c.update(102.0, 30.0);

        assert_eq!(c.open, 100.0);
        assert_eq!(c.high, 105.0);
        assert_eq!(c.low, 95.0);
        assert_eq!(c.close, 102.0);
        assert_eq!(c.volume, 60.0);
    }

    #[test]
    fn test_typical_price() {
        let c = Candle::new(0, 100.0, 110.0, 90.0, 100.0, 0.0);
        assert_eq!(c.typical_price(), 100.0); // (110 + 90 + 100) / 3
    }
}
