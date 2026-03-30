//! # loom-core
//!
//! Core types for trading systems: Timeframes, Events, Symbols, and Market Data.
//!
//! ## Features
//!
//! - `std` (default): Enable standard library support
//! - `serde`: Enable serialization/deserialization
//! - `chrono`: Enable chrono datetime integration
//!
//! ## Example
//!
//! ```rust
//! use loom_core::{Timeframe, Candle, Symbol, Event};
//!
//! // Parse timeframe from string
//! let tf = Timeframe::from_str("1h").unwrap();
//! assert_eq!(tf.as_seconds(), 3600);
//!
//! // Round timestamp to bucket start
//! let bucket = tf.bucket_start(1700000000);
//!
//! // Create a symbol with metadata
//! let btc = Symbol::crypto("BTCUSDT", "BTC", "USDT")
//!     .with_tick_size(0.01)
//!     .with_lot_size(0.001);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod timeframe;
pub mod candle;
pub mod event;
pub mod symbol;
pub mod session;
pub mod bucket;

pub use timeframe::Timeframe;
pub use candle::{Candle, OHLCV, Bar};
pub use event::{Event, EventType, Trade, Quote, CandleEvent};
pub use symbol::{Symbol, AssetClass, SymbolInfo};
pub use session::{TradingSession, MarketCalendar, SessionState};
pub use bucket::TimeBucket;

/// Timestamp in milliseconds since Unix epoch
pub type Timestamp = i64;

/// Price type (using f64 for flexibility, can be wrapped for precision)
pub type Price = f64;

/// Volume type
pub type Volume = f64;

/// Quantity type (for orders/positions)
pub type Quantity = f64;
