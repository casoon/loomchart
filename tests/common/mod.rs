//! Common test utilities.

use loom_core::{Candle, Timestamp};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Load candles from CSV file
///
/// Expected format: timestamp,open,high,low,close,volume
pub fn load_candles_csv(path: impl AsRef<Path>) -> Vec<Candle> {
    let file = File::open(path).expect("Failed to open CSV file");
    let reader = BufReader::new(file);
    let mut candles = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        if i == 0 {
            continue; // Skip header
        }

        let line = line.expect("Failed to read line");
        let parts: Vec<&str> = line.split(',').collect();

        if parts.len() >= 6 {
            let candle = Candle::new(
                parts[0].parse().unwrap_or(0),
                parts[1].parse().unwrap_or(0.0),
                parts[2].parse().unwrap_or(0.0),
                parts[3].parse().unwrap_or(0.0),
                parts[4].parse().unwrap_or(0.0),
                parts[5].parse().unwrap_or(0.0),
            );
            candles.push(candle);
        }
    }

    candles
}

/// Load expected values from CSV (for golden tests)
pub fn load_expected_values(path: impl AsRef<Path>) -> Vec<Option<f64>> {
    let file = File::open(path).expect("Failed to open expected values file");
    let reader = BufReader::new(file);
    let mut values = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        if i == 0 {
            continue;
        }

        let line = line.expect("Failed to read line");
        let value = if line.trim().is_empty() || line.trim() == "NaN" {
            None
        } else {
            line.trim().parse().ok()
        };
        values.push(value);
    }

    values
}

/// Generate synthetic candle data for testing
pub fn generate_test_candles(count: usize, base_price: f64, volatility: f64) -> Vec<Candle> {
    use std::f64::consts::PI;

    let mut candles = Vec::with_capacity(count);
    let mut price = base_price;

    for i in 0..count {
        // Add trend and noise
        let trend = (i as f64 * 0.01).sin() * base_price * 0.1;
        let noise = ((i as f64 * 0.5).sin() + (i as f64 * 1.3).cos()) * volatility;

        let open = price;
        let close = price + trend + noise;
        let high = open.max(close) + volatility.abs() * 0.5;
        let low = open.min(close) - volatility.abs() * 0.5;
        let volume = 1000.0 + (i as f64 * 0.1).sin().abs() * 500.0;

        let time = (i as i64) * 60000; // 1 minute candles

        candles.push(Candle::new(time, open, high, low, close, volume));
        price = close;
    }

    candles
}

/// Generate trending candle data
pub fn generate_uptrend(count: usize, base_price: f64, strength: f64) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(count);
    let mut price = base_price;

    for i in 0..count {
        let move_size = strength * (1.0 + (i as f64 * 0.2).sin() * 0.5);
        let open = price;
        let close = price + move_size;
        let high = close + move_size * 0.3;
        let low = open - move_size * 0.2;
        let volume = 1000.0;

        candles.push(Candle::new(
            (i as i64) * 60000,
            open,
            high,
            low,
            close,
            volume,
        ));
        price = close;
    }

    candles
}

/// Generate downtrend candle data
pub fn generate_downtrend(count: usize, base_price: f64, strength: f64) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(count);
    let mut price = base_price;

    for i in 0..count {
        let move_size = strength * (1.0 + (i as f64 * 0.2).sin() * 0.5);
        let open = price;
        let close = price - move_size;
        let high = open + move_size * 0.2;
        let low = close - move_size * 0.3;
        let volume = 1000.0;

        candles.push(Candle::new(
            (i as i64) * 60000,
            open,
            high,
            low,
            close,
            volume,
        ));
        price = close;
    }

    candles
}

/// Generate sideways/ranging candle data
pub fn generate_range(count: usize, base_price: f64, range_size: f64) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(count);
    let mut price = base_price;

    for i in 0..count {
        let direction = if i % 3 == 0 { 1.0 } else if i % 3 == 1 { -1.0 } else { 0.0 };
        let move_size = range_size * 0.3 * direction;

        let open = price;
        let mut close = price + move_size;

        // Keep in range
        if close > base_price + range_size / 2.0 {
            close = base_price + range_size / 2.0 - range_size * 0.1;
        } else if close < base_price - range_size / 2.0 {
            close = base_price - range_size / 2.0 + range_size * 0.1;
        }

        let high = open.max(close) + range_size * 0.1;
        let low = open.min(close) - range_size * 0.1;

        candles.push(Candle::new(
            (i as i64) * 60000,
            open,
            high,
            low,
            close,
            1000.0,
        ));
        price = close;
    }

    candles
}

/// Compare floating point values with tolerance
pub fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
    if a.is_nan() && b.is_nan() {
        return true;
    }
    (a - b).abs() <= tolerance
}

/// Compare optional values
pub fn approx_eq_opt(a: Option<f64>, b: Option<f64>, tolerance: f64) -> bool {
    match (a, b) {
        (Some(x), Some(y)) => approx_eq(x, y, tolerance),
        (None, None) => true,
        _ => false,
    }
}

/// Assert indicator values match expected with detailed output
pub fn assert_indicator_values(
    name: &str,
    actual: &[Option<f64>],
    expected: &[Option<f64>],
    tolerance: f64,
) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{}: Length mismatch: actual={}, expected={}",
        name,
        actual.len(),
        expected.len()
    );

    let mut mismatches = Vec::new();

    for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
        if !approx_eq_opt(*a, *e, tolerance) {
            mismatches.push((i, *a, *e));
        }
    }

    if !mismatches.is_empty() {
        println!("\n{} MISMATCHES:", name);
        for (i, a, e) in mismatches.iter().take(10) {
            println!("  [{}] actual={:?}, expected={:?}", i, a, e);
        }
        if mismatches.len() > 10 {
            println!("  ... and {} more", mismatches.len() - 10);
        }
        panic!(
            "{}: {} values out of {} did not match (tolerance={})",
            name,
            mismatches.len(),
            actual.len(),
            tolerance
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_test_candles() {
        let candles = generate_test_candles(100, 100.0, 1.0);
        assert_eq!(candles.len(), 100);

        for candle in &candles {
            assert!(candle.high >= candle.low);
            assert!(candle.high >= candle.open);
            assert!(candle.high >= candle.close);
            assert!(candle.low <= candle.open);
            assert!(candle.low <= candle.close);
        }
    }

    #[test]
    fn test_approx_eq() {
        assert!(approx_eq(1.0, 1.0001, 0.001));
        assert!(!approx_eq(1.0, 1.01, 0.001));
        assert!(approx_eq(f64::NAN, f64::NAN, 0.001));
    }
}
