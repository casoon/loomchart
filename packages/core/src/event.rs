//! Market data events for streaming.
//!
//! Defines event types for real-time data handling.

use crate::{Candle, Price, Volume, Timestamp, Timeframe};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Event type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EventType {
    /// Individual trade execution
    Trade,
    /// Bid/Ask quote update
    Quote,
    /// Partial (in-progress) candle update
    CandlePartial,
    /// Final (closed) candle
    CandleFinal,
    /// Heartbeat/keepalive
    Heartbeat,
    /// Session state change
    SessionChange,
    /// Error/warning
    Error,
}

/// Trade event (individual execution)
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Trade {
    /// Timestamp (milliseconds)
    pub time: Timestamp,
    /// Trade price
    pub price: Price,
    /// Trade size/quantity
    pub size: Volume,
    /// Trade ID (exchange-specific)
    pub id: u64,
    /// Is buyer the maker
    pub is_buyer_maker: bool,
}

impl Trade {
    #[inline]
    pub const fn new(time: Timestamp, price: Price, size: Volume) -> Self {
        Self {
            time,
            price,
            size,
            id: 0,
            is_buyer_maker: false,
        }
    }

    #[inline]
    pub const fn with_id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    #[inline]
    pub const fn with_side(mut self, is_buyer_maker: bool) -> Self {
        self.is_buyer_maker = is_buyer_maker;
        self
    }

    /// Trade value (price * size)
    #[inline]
    pub fn value(&self) -> f64 {
        self.price * self.size
    }
}

/// Quote event (bid/ask)
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Quote {
    /// Timestamp (milliseconds)
    pub time: Timestamp,
    /// Best bid price
    pub bid: Price,
    /// Best bid size
    pub bid_size: Volume,
    /// Best ask price
    pub ask: Price,
    /// Best ask size
    pub ask_size: Volume,
}

impl Quote {
    #[inline]
    pub const fn new(time: Timestamp, bid: Price, bid_size: Volume, ask: Price, ask_size: Volume) -> Self {
        Self { time, bid, bid_size, ask, ask_size }
    }

    /// Mid price: (bid + ask) / 2
    #[inline]
    pub fn mid(&self) -> Price {
        (self.bid + self.ask) / 2.0
    }

    /// Spread: ask - bid
    #[inline]
    pub fn spread(&self) -> Price {
        self.ask - self.bid
    }

    /// Spread in basis points
    #[inline]
    pub fn spread_bps(&self) -> f64 {
        if self.mid() == 0.0 {
            0.0
        } else {
            (self.spread() / self.mid()) * 10000.0
        }
    }

    /// Imbalance ratio: (bid_size - ask_size) / (bid_size + ask_size)
    #[inline]
    pub fn imbalance(&self) -> f64 {
        let total = self.bid_size + self.ask_size;
        if total == 0.0 {
            0.0
        } else {
            (self.bid_size - self.ask_size) / total
        }
    }
}

/// Candle event (partial or final)
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CandleEvent {
    /// The candle data
    pub candle: Candle,
    /// Timeframe of this candle
    pub timeframe: Timeframe,
    /// Is this a final (closed) candle
    pub is_final: bool,
}

impl CandleEvent {
    #[inline]
    pub const fn new(candle: Candle, timeframe: Timeframe, is_final: bool) -> Self {
        Self { candle, timeframe, is_final }
    }

    #[inline]
    pub const fn partial(candle: Candle, timeframe: Timeframe) -> Self {
        Self::new(candle, timeframe, false)
    }

    #[inline]
    pub const fn final_(candle: Candle, timeframe: Timeframe) -> Self {
        Self::new(candle, timeframe, true)
    }
}

/// Unified market event
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Event {
    /// Trade execution
    Trade(Trade),
    /// Quote update
    Quote(Quote),
    /// Candle update (partial or final)
    Candle(CandleEvent),
    /// Heartbeat with timestamp
    Heartbeat(Timestamp),
    /// Error message
    Error(String),
}

impl Event {
    /// Get event type
    pub fn event_type(&self) -> EventType {
        match self {
            Self::Trade(_) => EventType::Trade,
            Self::Quote(_) => EventType::Quote,
            Self::Candle(c) if c.is_final => EventType::CandleFinal,
            Self::Candle(_) => EventType::CandlePartial,
            Self::Heartbeat(_) => EventType::Heartbeat,
            Self::Error(_) => EventType::Error,
        }
    }

    /// Get timestamp if available
    pub fn timestamp(&self) -> Option<Timestamp> {
        match self {
            Self::Trade(t) => Some(t.time),
            Self::Quote(q) => Some(q.time),
            Self::Candle(c) => Some(c.candle.time),
            Self::Heartbeat(ts) => Some(*ts),
            Self::Error(_) => None,
        }
    }

    /// Get price if available
    pub fn price(&self) -> Option<Price> {
        match self {
            Self::Trade(t) => Some(t.price),
            Self::Quote(q) => Some(q.mid()),
            Self::Candle(c) => Some(c.candle.close),
            _ => None,
        }
    }
}

/// Event with symbol context
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymbolEvent {
    /// Symbol identifier
    pub symbol: String,
    /// The event
    pub event: Event,
    /// Sequence number (for ordering/dedup)
    pub sequence: u64,
}

impl SymbolEvent {
    pub fn new(symbol: impl Into<String>, event: Event, sequence: u64) -> Self {
        Self {
            symbol: symbol.into(),
            event,
            sequence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_spread() {
        let q = Quote::new(0, 100.0, 10.0, 101.0, 20.0);
        assert_eq!(q.spread(), 1.0);
        assert_eq!(q.mid(), 100.5);
    }

    #[test]
    fn test_quote_imbalance() {
        let q = Quote::new(0, 100.0, 30.0, 101.0, 10.0);
        assert_eq!(q.imbalance(), 0.5); // (30 - 10) / 40
    }
}
