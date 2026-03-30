//! Common types for the trading UI

use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "apiUrl")]
    #[allow(dead_code)]
    pub api_url: String,
    #[serde(rename = "wsUrl")]
    pub ws_url: String,
}

/// Timeframe enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "lowercase")]
pub enum Timeframe {
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "15m")]
    M15,
    #[serde(rename = "30m")]
    M30,
    #[serde(rename = "1h")]
    H1,
    #[serde(rename = "4h")]
    H4,
    #[serde(rename = "1d")]
    D1,
    #[serde(rename = "1w")]
    W1,
}
    #[allow(dead_code)]

impl Timeframe {
    pub fn as_str(&self) -> &'static str {
        match self {
            Timeframe::M1 => "1m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::M30 => "30m",
            Timeframe::H1 => "1h",
            Timeframe::H4 => "4h",
            Timeframe::D1 => "1d",
            Timeframe::W1 => "1w",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "1m" => Some(Timeframe::M1),
            "5m" => Some(Timeframe::M5),
            "15m" => Some(Timeframe::M15),
            "30m" => Some(Timeframe::M30),
            "1h" => Some(Timeframe::H1),
            "4h" => Some(Timeframe::H4),
            "1d" => Some(Timeframe::D1),
            "1w" => Some(Timeframe::W1),
            _ => None,
        }
    }
}

/// OHLCV Candle - matches Phoenix Candle.to_map format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub source: String,
    pub symbol: String,
    pub tf: String,
    pub ts: String, // ISO 8601 timestamp
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
    pub is_final: bool, // snake_case to match Phoenix
}

/// Chart-compatible candle format
#[derive(Debug, Clone, Serialize)]
pub struct ChartCandle {
    pub time: i64, // unix timestamp in seconds
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
}

impl From<&Candle> for ChartCandle {
    fn from(c: &Candle) -> Self {
        // Parse ISO timestamp to unix seconds
        let time = chrono_to_unix(&c.ts).unwrap_or(0);
        ChartCandle {
            time,
            open: c.o,
            high: c.h,
            low: c.l,
            close: c.c,
            volume: Some(c.v),
        }
    }
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    /// Connected to WebSocket, awaiting initial data snapshot
    Syncing,
    Connected,
    Reconnecting,
    Error,
}

impl std::fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ConnectionStatus::Disconnected => "disconnected",
            ConnectionStatus::Connecting => "connecting",
            ConnectionStatus::Syncing => "syncing",
            ConnectionStatus::Connected => "connected",
            ConnectionStatus::Reconnecting => "reconnecting",
            ConnectionStatus::Error => "error",
        };
        write!(f, "{}", s)
    }
}

#[allow(dead_code)]
/// WebSocket message types
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "candle_snapshot")]
    CandleSnapshot {
        candles: Vec<Candle>,
        server_time: String,
    },
    #[serde(rename = "candle_update")]
    CandleUpdate(Candle),
    #[serde(rename = "candle_final")]
    CandleFinal(Candle),
    #[serde(rename = "indicator_update")]
    IndicatorUpdate {
        instance_id: String,
        ts: String,
        value: Option<f64>,
        values: Option<serde_json::Value>,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

/// Indicator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorConfig {
    pub id: String,
    pub name: String,
    pub params: serde_json::Value,
    pub enabled: bool,
}

// Helper to parse ISO timestamp to unix seconds
fn chrono_to_unix(ts: &str) -> Option<i64> {
    // Simple ISO 8601 parser (YYYY-MM-DDTHH:MM:SSZ)
    // For a full implementation, use chrono crate
    if ts.len() < 19 {
        return None;
    }

    // Extract components
    let year: i32 = ts[0..4].parse().ok()?;
    let month: u32 = ts[5..7].parse().ok()?;
    let day: u32 = ts[8..10].parse().ok()?;
    let hour: u32 = ts[11..13].parse().ok()?;
    let min: u32 = ts[14..16].parse().ok()?;
    let sec: u32 = ts[17..19].parse().ok()?;

    // Calculate unix timestamp (simplified, doesn't account for leap years properly)
    let days_since_epoch = days_from_epoch(year, month, day);
    let seconds =
        (days_since_epoch as i64) * 86400 + (hour as i64) * 3600 + (min as i64) * 60 + (sec as i64);
    Some(seconds)
}

fn days_from_epoch(year: i32, month: u32, day: u32) -> i32 {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * (m as i32 - 3) + 2) / 5 + day as i32 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}
