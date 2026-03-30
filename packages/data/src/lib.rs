//! Loom Data - Data providers for the Loom trading platform.
//!
//! This crate provides a unified interface for fetching historical and
//! realtime market data from various sources.
//!
//! # Supported Providers
//!
//! - **CSV**: Load historical data from CSV files
//! - **Binance**: Fetch from Binance REST API and WebSocket
//! - **Mock**: Generate synthetic data for testing
//!
//! # Example
//!
//! ```rust,ignore
//! use loom_data::{CsvProvider, DataProvider, TimeRange};
//! use loom_core::Timeframe;
//!
//! let provider = CsvProvider::new("data/btcusdt_1h.csv");
//! let candles = provider.get_candles("BTCUSDT", Timeframe::H1, TimeRange::all())?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod provider;
pub mod range;
pub mod mock;

#[cfg(feature = "csv")]
pub mod csv_provider;

#[cfg(feature = "binance")]
pub mod binance;

pub use provider::{DataProvider, DataError, DataResult};
pub use range::TimeRange;
pub use mock::MockProvider;

#[cfg(feature = "csv")]
pub use csv_provider::CsvProvider;

#[cfg(feature = "binance")]
pub use binance::BinanceProvider;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::provider::{DataProvider, DataError, DataResult};
    pub use crate::range::TimeRange;
    pub use crate::mock::MockProvider;

    #[cfg(feature = "csv")]
    pub use crate::csv_provider::CsvProvider;

    #[cfg(feature = "binance")]
    pub use crate::binance::BinanceProvider;
}
