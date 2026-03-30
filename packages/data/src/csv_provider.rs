//! CSV data provider.

use std::path::Path;
use std::fs::File;
use std::io::{BufRead, BufReader};
use loom_core::{Candle, Timeframe};
use crate::{DataProvider, DataResult, DataError, TimeRange};

/// CSV file data provider
///
/// Supports common CSV formats:
/// - timestamp,open,high,low,close,volume (default)
/// - date,open,high,low,close,volume (with date parsing)
/// - TradingView export format
/// - Binance historical data format
pub struct CsvProvider {
    path: String,
    candles: Vec<Candle>,
    symbol: String,
    timeframe: Timeframe,
}

impl CsvProvider {
    /// Create a new CSV provider from file path
    pub fn new(path: impl AsRef<Path>) -> DataResult<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let candles = Self::load_file(&path_str)?;

        // Try to infer symbol from filename
        let symbol = path.as_ref()
            .file_stem()
            .map(|s| s.to_string_lossy().to_uppercase())
            .unwrap_or_else(|| "UNKNOWN".into());

        // Infer timeframe from candle intervals
        let timeframe = Self::infer_timeframe(&candles);

        Ok(Self {
            path: path_str,
            candles,
            symbol,
            timeframe,
        })
    }

    /// Create with explicit symbol and timeframe
    pub fn with_metadata(
        path: impl AsRef<Path>,
        symbol: &str,
        timeframe: Timeframe,
    ) -> DataResult<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        let candles = Self::load_file(&path_str)?;

        Ok(Self {
            path: path_str,
            candles,
            symbol: symbol.into(),
            timeframe,
        })
    }

    fn load_file(path: &str) -> DataResult<Vec<Candle>> {
        let file = File::open(path)
            .map_err(|e| DataError::Io(e.to_string()))?;
        let reader = BufReader::new(file);
        let mut candles = Vec::new();

        for (i, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| DataError::Io(e.to_string()))?;

            // Skip header
            if i == 0 && Self::is_header(&line) {
                continue;
            }

            if let Some(candle) = Self::parse_line(&line) {
                candles.push(candle);
            }
        }

        if candles.is_empty() {
            return Err(DataError::NoData);
        }

        // Sort by timestamp
        candles.sort_by_key(|c| c.time);

        Ok(candles)
    }

    fn is_header(line: &str) -> bool {
        let lower = line.to_lowercase();
        lower.contains("timestamp") ||
        lower.contains("date") ||
        lower.contains("open") ||
        lower.contains("time")
    }

    fn parse_line(line: &str) -> Option<Candle> {
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() < 5 {
            return None;
        }

        // Try different timestamp formats
        let timestamp = Self::parse_timestamp(parts[0])?;
        let open: f64 = parts[1].trim().parse().ok()?;
        let high: f64 = parts[2].trim().parse().ok()?;
        let low: f64 = parts[3].trim().parse().ok()?;
        let close: f64 = parts[4].trim().parse().ok()?;
        let volume: f64 = parts.get(5)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0.0);

        Some(Candle::new(timestamp, open, high, low, close, volume))
    }

    fn parse_timestamp(s: &str) -> Option<i64> {
        let s = s.trim();

        // Unix milliseconds
        if let Ok(ts) = s.parse::<i64>() {
            if ts > 1_000_000_000_000 {
                return Some(ts); // Already in milliseconds
            } else {
                return Some(ts * 1000); // Convert seconds to ms
            }
        }

        // ISO 8601 date/datetime
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
            return Some(dt.timestamp_millis());
        }

        // Common date formats
        let formats = [
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d",
            "%Y/%m/%d %H:%M:%S",
            "%Y/%m/%d",
            "%d-%m-%Y %H:%M:%S",
            "%d/%m/%Y %H:%M:%S",
        ];

        for fmt in formats {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, fmt) {
                return Some(dt.and_utc().timestamp_millis());
            }
            if let Ok(d) = chrono::NaiveDate::parse_from_str(s, fmt) {
                return Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp_millis());
            }
        }

        None
    }

    fn infer_timeframe(candles: &[Candle]) -> Timeframe {
        if candles.len() < 2 {
            return Timeframe::D1;
        }

        // Calculate average interval
        let intervals: Vec<i64> = candles.windows(2)
            .map(|w| w[1].time - w[0].time)
            .collect();

        let avg_interval = intervals.iter().sum::<i64>() / intervals.len() as i64;

        // Match to closest timeframe (in milliseconds)
        match avg_interval {
            0..=90_000 => Timeframe::M1,           // ~1 min
            90_001..=450_000 => Timeframe::M5,     // ~5 min
            450_001..=1_350_000 => Timeframe::M15, // ~15 min
            1_350_001..=2_700_000 => Timeframe::M30, // ~30 min
            2_700_001..=5_400_000 => Timeframe::H1,  // ~1 hour
            5_400_001..=18_000_000 => Timeframe::H4, // ~4 hours
            18_000_001..=129_600_000 => Timeframe::D1, // ~1 day
            _ => Timeframe::W1,                    // Weekly or longer
        }
    }
}

impl DataProvider for CsvProvider {
    fn get_candles(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        range: TimeRange,
    ) -> DataResult<Vec<Candle>> {
        if symbol != self.symbol {
            return Err(DataError::InvalidSymbol(symbol.into()));
        }

        if timeframe != self.timeframe {
            // TODO: Implement timeframe conversion
            return Err(DataError::InvalidTimeframe(timeframe));
        }

        Ok(range.filter(self.candles.clone()))
    }

    fn symbols(&self) -> DataResult<Vec<String>> {
        Ok(vec![self.symbol.clone()])
    }

    fn timeframes(&self, _symbol: &str) -> DataResult<Vec<Timeframe>> {
        Ok(vec![self.timeframe])
    }
}

/// Builder for creating CSV files (useful for tests)
pub struct CsvBuilder {
    candles: Vec<Candle>,
}

impl CsvBuilder {
    pub fn new() -> Self {
        Self { candles: Vec::new() }
    }

    pub fn add_candle(mut self, candle: Candle) -> Self {
        self.candles.push(candle);
        self
    }

    pub fn add_candles(mut self, candles: impl IntoIterator<Item = Candle>) -> Self {
        self.candles.extend(candles);
        self
    }

    pub fn write(self, path: impl AsRef<Path>) -> std::io::Result<()> {
        use std::io::Write;

        let mut file = File::create(path)?;
        writeln!(file, "timestamp,open,high,low,close,volume")?;

        for candle in &self.candles {
            writeln!(
                file,
                "{},{},{},{},{},{}",
                candle.time, candle.open, candle.high, candle.low, candle.close, candle.volume
            )?;
        }

        Ok(())
    }
}

impl Default for CsvBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_parsing() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "timestamp,open,high,low,close,volume").unwrap();
        writeln!(file, "1700000000000,100,105,98,103,1000").unwrap();
        writeln!(file, "1700003600000,103,108,102,106,1200").unwrap();

        let provider = CsvProvider::new(file.path()).unwrap();
        let candles = provider.get_candles(
            &provider.symbol,
            provider.timeframe,
            TimeRange::all()
        ).unwrap();

        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].open, 100.0);
        assert_eq!(candles[1].close, 106.0);
    }
}
