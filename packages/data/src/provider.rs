//! Data provider trait and common types.

use loom_core::{Candle, Timeframe, Timestamp};
use crate::range::TimeRange;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, boxed::Box};

/// Data provider error
#[derive(Debug)]
pub enum DataError {
    /// Network error
    Network(String),
    /// Parse error
    Parse(String),
    /// Invalid symbol
    InvalidSymbol(String),
    /// Invalid timeframe
    InvalidTimeframe(Timeframe),
    /// Rate limit exceeded
    RateLimited,
    /// No data available
    NoData,
    /// IO error
    Io(String),
    /// Other error
    Other(String),
}

impl core::fmt::Display for DataError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DataError::Network(msg) => write!(f, "Network error: {}", msg),
            DataError::Parse(msg) => write!(f, "Parse error: {}", msg),
            DataError::InvalidSymbol(s) => write!(f, "Invalid symbol: {}", s),
            DataError::InvalidTimeframe(tf) => write!(f, "Invalid timeframe: {:?}", tf),
            DataError::RateLimited => write!(f, "Rate limit exceeded"),
            DataError::NoData => write!(f, "No data available"),
            DataError::Io(msg) => write!(f, "IO error: {}", msg),
            DataError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for DataError {}

/// Result type for data operations
pub type DataResult<T> = Result<T, DataError>;

/// Synchronous data provider trait
pub trait DataProvider {
    /// Get historical candles
    fn get_candles(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        range: TimeRange,
    ) -> DataResult<Vec<Candle>>;

    /// Get available symbols
    fn symbols(&self) -> DataResult<Vec<String>>;

    /// Get available timeframes for a symbol
    fn timeframes(&self, symbol: &str) -> DataResult<Vec<Timeframe>>;

    /// Get the latest candle
    fn latest_candle(&self, symbol: &str, timeframe: Timeframe) -> DataResult<Option<Candle>> {
        let candles = self.get_candles(
            symbol,
            timeframe,
            TimeRange::last_n(1),
        )?;
        Ok(candles.into_iter().last())
    }

    /// Get candle count
    fn candle_count(&self, symbol: &str, timeframe: Timeframe) -> DataResult<usize> {
        let candles = self.get_candles(symbol, timeframe, TimeRange::all())?;
        Ok(candles.len())
    }
}

/// Asynchronous data provider trait
#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait AsyncDataProvider: Send + Sync {
    /// Get historical candles
    async fn get_candles(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        range: TimeRange,
    ) -> DataResult<Vec<Candle>>;

    /// Get available symbols
    async fn symbols(&self) -> DataResult<Vec<String>>;

    /// Subscribe to realtime updates
    async fn subscribe(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        callback: Box<dyn Fn(Candle) + Send + Sync>,
    ) -> DataResult<SubscriptionHandle>;
}

/// Handle for a realtime subscription
#[cfg(feature = "async")]
pub struct SubscriptionHandle {
    pub id: u64,
    cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

#[cfg(feature = "async")]
impl SubscriptionHandle {
    pub fn new(id: u64, cancel_tx: tokio::sync::oneshot::Sender<()>) -> Self {
        Self { id, cancel_tx: Some(cancel_tx) }
    }

    pub fn cancel(mut self) {
        if let Some(tx) = self.cancel_tx.take() {
            let _ = tx.send(());
        }
    }
}
