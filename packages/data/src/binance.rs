//! Binance data provider.
//!
//! Fetches historical and realtime data from Binance.

use loom_core::{Candle, Timeframe, Timestamp};
use crate::{DataProvider, DataResult, DataError, TimeRange};
use std::collections::HashMap;

/// Binance REST API provider
pub struct BinanceProvider {
    base_url: String,
    client: reqwest::Client,
}

impl BinanceProvider {
    /// Create a new Binance provider (uses public API)
    pub fn new() -> Self {
        Self {
            base_url: "https://api.binance.com".into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create with custom base URL (for testnet)
    pub fn with_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    fn timeframe_to_interval(tf: Timeframe) -> &'static str {
        match tf {
            Timeframe::S1 => "1s",
            Timeframe::M1 => "1m",
            Timeframe::M3 => "3m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::M30 => "30m",
            Timeframe::H1 => "1h",
            Timeframe::H2 => "2h",
            Timeframe::H4 => "4h",
            Timeframe::H6 => "6h",
            Timeframe::H8 => "8h",
            Timeframe::H12 => "12h",
            Timeframe::D1 => "1d",
            Timeframe::D3 => "3d",
            Timeframe::W1 => "1w",
            Timeframe::MN1 => "1M",
        }
    }

    /// Fetch candles from REST API
    pub async fn fetch_candles(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        range: TimeRange,
    ) -> DataResult<Vec<Candle>> {
        let interval = Self::timeframe_to_interval(timeframe);
        let limit = range.limit.unwrap_or(1000).min(1000);

        let mut url = format!(
            "{}/api/v3/klines?symbol={}&interval={}&limit={}",
            self.base_url, symbol, interval, limit
        );

        if let Some(start) = range.start {
            url.push_str(&format!("&startTime={}", start));
        }
        if let Some(end) = range.end {
            url.push_str(&format!("&endTime={}", end));
        }

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| DataError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(DataError::Network(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        let data: Vec<Vec<serde_json::Value>> = response
            .json()
            .await
            .map_err(|e| DataError::Parse(e.to_string()))?;

        let candles = data
            .into_iter()
            .filter_map(|kline| Self::parse_kline(&kline))
            .collect();

        Ok(candles)
    }

    fn parse_kline(kline: &[serde_json::Value]) -> Option<Candle> {
        if kline.len() < 6 {
            return None;
        }

        let time = kline[0].as_i64()?;
        let open: f64 = kline[1].as_str()?.parse().ok()?;
        let high: f64 = kline[2].as_str()?.parse().ok()?;
        let low: f64 = kline[3].as_str()?.parse().ok()?;
        let close: f64 = kline[4].as_str()?.parse().ok()?;
        let volume: f64 = kline[5].as_str()?.parse().ok()?;

        Some(Candle::new(time, open, high, low, close, volume))
    }

    /// Fetch available trading pairs
    pub async fn fetch_symbols(&self) -> DataResult<Vec<String>> {
        let url = format!("{}/api/v3/exchangeInfo", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| DataError::Network(e.to_string()))?;

        #[derive(serde::Deserialize)]
        struct ExchangeInfo {
            symbols: Vec<SymbolInfo>,
        }

        #[derive(serde::Deserialize)]
        struct SymbolInfo {
            symbol: String,
            status: String,
        }

        let info: ExchangeInfo = response
            .json()
            .await
            .map_err(|e| DataError::Parse(e.to_string()))?;

        let symbols = info.symbols
            .into_iter()
            .filter(|s| s.status == "TRADING")
            .map(|s| s.symbol)
            .collect();

        Ok(symbols)
    }

    /// Get 24hr ticker for a symbol
    pub async fn get_ticker(&self, symbol: &str) -> DataResult<Ticker> {
        let url = format!(
            "{}/api/v3/ticker/24hr?symbol={}",
            self.base_url, symbol
        );

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| DataError::Network(e.to_string()))?;

        let ticker: Ticker = response
            .json()
            .await
            .map_err(|e| DataError::Parse(e.to_string()))?;

        Ok(ticker)
    }
}

impl Default for BinanceProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// 24hr ticker data
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub last_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
}

/// Free data sources information
pub mod free_sources {
    //! Information about free historical data sources.
    //!
    //! # Available Free Sources
    //!
    //! ## Binance Public Data
    //! - URL: https://data.binance.vision/
    //! - Format: CSV (gzipped)
    //! - Data: Spot, Futures, Options
    //! - Timeframes: All
    //! - History: Full history for all pairs
    //!
    //! ## Yahoo Finance
    //! - Stocks, ETFs, Indices, Crypto
    //! - Daily data free, intraday limited
    //!
    //! ## CryptoCompare
    //! - URL: https://min-api.cryptocompare.com/
    //! - Free tier: 100,000 calls/month
    //! - Crypto OHLCV data
    //!
    //! ## Polygon.io
    //! - Free tier: 5 API calls/min
    //! - Stocks, Options, Forex, Crypto
    //!
    //! ## Alpha Vantage
    //! - Free tier: 5 calls/min, 500/day
    //! - Stocks, Forex, Crypto
    //!
    //! ## Quandl (Nasdaq Data Link)
    //! - Various free datasets
    //! - Stocks, Futures, Economic data

    /// Binance historical data download URL
    pub const BINANCE_DATA_URL: &str = "https://data.binance.vision/";

    /// Download historical Binance data
    ///
    /// Example:
    /// ```text
    /// https://data.binance.vision/data/spot/monthly/klines/BTCUSDT/1h/BTCUSDT-1h-2023-01.zip
    /// ```
    pub fn binance_klines_url(
        symbol: &str,
        interval: &str,
        year: u32,
        month: u32,
    ) -> String {
        format!(
            "{}data/spot/monthly/klines/{}/{}/{}-{}-{:04}-{:02}.zip",
            BINANCE_DATA_URL, symbol, interval, symbol, interval, year, month
        )
    }

    /// CryptoCompare API base
    pub const CRYPTOCOMPARE_URL: &str = "https://min-api.cryptocompare.com/data/v2/";

    /// Yahoo Finance chart API
    pub const YAHOO_CHART_URL: &str = "https://query1.finance.yahoo.com/v8/finance/chart/";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeframe_mapping() {
        assert_eq!(BinanceProvider::timeframe_to_interval(Timeframe::M1), "1m");
        assert_eq!(BinanceProvider::timeframe_to_interval(Timeframe::H1), "1h");
        assert_eq!(BinanceProvider::timeframe_to_interval(Timeframe::D1), "1d");
    }

    #[test]
    fn test_binance_klines_url() {
        let url = free_sources::binance_klines_url("BTCUSDT", "1h", 2023, 6);
        assert!(url.contains("BTCUSDT"));
        assert!(url.contains("2023-06"));
    }
}
