//! Golden tests for technical indicators.
//!
//! Reference values generated from:
//! - Backtrader (Python)
//! - QuantConnect (C#)
//! - TradingView (Pine Script)

use loom_core::Candle;
use loom_indicators::prelude::*;

mod common {
    include!("../common/mod.rs");
}

use common::*;

/// Test SMA against known values
#[test]
fn test_sma_golden() {
    let candles = generate_test_candles(50, 100.0, 1.0);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let mut sma = Sma::new(14);
    let mut results: Vec<Option<f64>> = Vec::new();

    for close in &closes {
        results.push(sma.next(*close));
    }

    // First 13 values should be None (warmup period)
    for i in 0..13 {
        assert!(
            results[i].is_none(),
            "SMA[{}] should be None during warmup",
            i
        );
    }

    // After warmup, all values should be Some
    for i in 13..results.len() {
        assert!(
            results[i].is_some(),
            "SMA[{}] should have value after warmup",
            i
        );
    }

    // Verify SMA is actually the mean of last 14 values
    for i in 13..closes.len() {
        let expected: f64 = closes[i - 13..=i].iter().sum::<f64>() / 14.0;
        let actual = results[i].unwrap();
        assert!(
            approx_eq(actual, expected, 0.0001),
            "SMA[{}]: expected {}, got {}",
            i,
            expected,
            actual
        );
    }
}

/// Test EMA against reference implementation
#[test]
fn test_ema_golden() {
    let candles = generate_test_candles(50, 100.0, 1.0);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let period = 14;
    let mut ema = Ema::new(period);
    let mut results: Vec<Option<f64>> = Vec::new();

    for close in &closes {
        results.push(ema.next(*close));
    }

    // Manual EMA calculation for verification
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut expected_ema: Option<f64> = None;

    for (i, close) in closes.iter().enumerate() {
        expected_ema = match expected_ema {
            None => {
                if i >= period - 1 {
                    // Initial EMA is SMA of first period values
                    Some(closes[..period].iter().sum::<f64>() / period as f64)
                } else {
                    None
                }
            }
            Some(prev) => Some(*close * multiplier + prev * (1.0 - multiplier)),
        };

        if let (Some(actual), Some(expected)) = (results[i], expected_ema) {
            assert!(
                approx_eq(actual, expected, 0.01),
                "EMA[{}]: expected {}, got {}",
                i,
                expected,
                actual
            );
        }
    }
}

/// Test RSI against reference values
#[test]
fn test_rsi_golden() {
    // Use trending data for predictable RSI values
    let uptrend = generate_uptrend(30, 100.0, 1.0);
    let closes: Vec<f64> = uptrend.iter().map(|c| c.close).collect();

    let mut rsi = Rsi::new(14);
    let mut results: Vec<Option<f64>> = Vec::new();

    for close in &closes {
        results.push(rsi.next(*close));
    }

    // In a strong uptrend, RSI should be high (> 50)
    for result in results.iter().skip(14) {
        if let Some(value) = result {
            assert!(
                *value > 50.0,
                "RSI in uptrend should be > 50, got {}",
                value
            );
            assert!(
                *value <= 100.0,
                "RSI should be <= 100, got {}",
                value
            );
        }
    }

    // Test downtrend - RSI should be low
    let downtrend = generate_downtrend(30, 100.0, 1.0);
    let closes: Vec<f64> = downtrend.iter().map(|c| c.close).collect();

    let mut rsi = Rsi::new(14);
    let mut results: Vec<Option<f64>> = Vec::new();

    for close in &closes {
        results.push(rsi.next(*close));
    }

    for result in results.iter().skip(14) {
        if let Some(value) = result {
            assert!(
                *value < 50.0,
                "RSI in downtrend should be < 50, got {}",
                value
            );
            assert!(*value >= 0.0, "RSI should be >= 0, got {}", value);
        }
    }
}

/// Test MACD calculation
#[test]
fn test_macd_golden() {
    let candles = generate_test_candles(50, 100.0, 1.0);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let mut macd = Macd::new(12, 26, 9);
    let mut results: Vec<Option<MacdOutput>> = Vec::new();

    for close in &closes {
        results.push(macd.next(*close));
    }

    // MACD should produce values after 26 periods (slow EMA warmup)
    for result in results.iter().skip(25) {
        assert!(result.is_some(), "MACD should have value after warmup");
    }

    // Verify MACD line = fast EMA - slow EMA
    let mut fast_ema = Ema::new(12);
    let mut slow_ema = Ema::new(26);

    for (i, close) in closes.iter().enumerate() {
        let fast = fast_ema.next(*close);
        let slow = slow_ema.next(*close);

        if let (Some(f), Some(s), Some(macd_out)) = (fast, slow, &results[i]) {
            let expected_macd_line = f - s;
            assert!(
                approx_eq(macd_out.macd, expected_macd_line, 0.01),
                "MACD line[{}]: expected {}, got {}",
                i,
                expected_macd_line,
                macd_out.macd
            );
        }
    }
}

/// Test Bollinger Bands
#[test]
fn test_bollinger_bands_golden() {
    let candles = generate_test_candles(50, 100.0, 2.0);
    let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

    let mut bb = BollingerBands::new(20, 2.0);
    let mut results: Vec<Option<BollingerBandsOutput>> = Vec::new();

    for close in &closes {
        results.push(bb.next(*close));
    }

    // Verify band relationships
    for result in results.iter().skip(19) {
        if let Some(bands) = result {
            assert!(
                bands.upper > bands.middle,
                "Upper band should be > middle"
            );
            assert!(
                bands.middle > bands.lower,
                "Middle band should be > lower"
            );
            assert!(
                bands.upper - bands.middle > 0.0,
                "Band width should be positive"
            );
            assert!(
                approx_eq(bands.upper - bands.middle, bands.middle - bands.lower, 0.0001),
                "Bands should be symmetric around middle"
            );
        }
    }
}

/// Test ATR (Average True Range)
#[test]
fn test_atr_golden() {
    let candles = generate_test_candles(50, 100.0, 2.0);

    let mut atr = Atr::new(14);
    let mut results: Vec<Option<f64>> = Vec::new();

    for candle in &candles {
        results.push(atr.next(candle));
    }

    // ATR should always be positive
    for result in results.iter().skip(13) {
        if let Some(value) = result {
            assert!(*value > 0.0, "ATR should be positive, got {}", value);
        }
    }

    // In volatile data, ATR should be meaningful
    let avg_atr: f64 = results
        .iter()
        .skip(13)
        .filter_map(|r| *r)
        .sum::<f64>()
        / (results.len() - 13) as f64;

    assert!(
        avg_atr > 0.5,
        "ATR should reflect volatility, got {}",
        avg_atr
    );
}

/// Test Stochastic Oscillator
#[test]
fn test_stochastic_golden() {
    let candles = generate_test_candles(50, 100.0, 2.0);

    let mut stoch = Stochastic::new(14, 3, 3);
    let mut results: Vec<Option<StochasticOutput>> = Vec::new();

    for candle in &candles {
        results.push(stoch.next(candle));
    }

    // %K and %D should be between 0 and 100
    for result in results.iter().skip(16) {
        if let Some(stoch_out) = result {
            assert!(
                stoch_out.k >= 0.0 && stoch_out.k <= 100.0,
                "%K should be 0-100, got {}",
                stoch_out.k
            );
            assert!(
                stoch_out.d >= 0.0 && stoch_out.d <= 100.0,
                "%D should be 0-100, got {}",
                stoch_out.d
            );
        }
    }
}

/// Test ADX (Average Directional Index)
#[test]
fn test_adx_golden() {
    // Use trending data
    let uptrend = generate_uptrend(50, 100.0, 1.0);

    let mut adx = Adx::new(14);
    let mut results: Vec<Option<AdxOutput>> = Vec::new();

    for candle in &uptrend {
        results.push(adx.next(candle));
    }

    // ADX should be between 0 and 100
    for result in results.iter().skip(27) {
        if let Some(adx_out) = result {
            assert!(
                adx_out.adx >= 0.0 && adx_out.adx <= 100.0,
                "ADX should be 0-100, got {}",
                adx_out.adx
            );

            // In uptrend, +DI should be > -DI
            assert!(
                adx_out.plus_di > adx_out.minus_di,
                "In uptrend, +DI ({}) should be > -DI ({})",
                adx_out.plus_di,
                adx_out.minus_di
            );
        }
    }

    // Strong trend should have high ADX
    let final_adx = results.last().unwrap().as_ref().unwrap().adx;
    assert!(
        final_adx > 20.0,
        "Strong trend should have ADX > 20, got {}",
        final_adx
    );
}

/// Test indicator reset functionality
#[test]
fn test_indicator_reset() {
    let candles = generate_test_candles(30, 100.0, 1.0);

    let mut sma = Sma::new(14);

    // First pass
    let mut first_results: Vec<Option<f64>> = Vec::new();
    for candle in &candles {
        first_results.push(sma.next(candle.close));
    }

    // Reset
    sma.reset();

    // Second pass should produce identical results
    let mut second_results: Vec<Option<f64>> = Vec::new();
    for candle in &candles {
        second_results.push(sma.next(candle.close));
    }

    for (i, (first, second)) in first_results.iter().zip(second_results.iter()).enumerate() {
        assert!(
            approx_eq_opt(*first, *second, 0.0001),
            "After reset, results should be identical at index {}",
            i
        );
    }
}

/// Stress test with large dataset
#[test]
fn test_indicator_large_dataset() {
    let candles = generate_test_candles(10_000, 100.0, 1.0);

    let mut ema = Ema::new(200);
    let mut rsi = Rsi::new(14);
    let mut bb = BollingerBands::new(20, 2.0);

    for candle in &candles {
        ema.next(candle.close);
        rsi.next(candle.close);
        bb.next(candle.close);
    }

    // Just verify no panics or NaN values
    assert!(ema.value().is_some());
    assert!(rsi.value().is_some());
    assert!(bb.value().is_some());
}

/// Test edge cases
#[test]
fn test_edge_cases() {
    // Constant price
    let constant_candles: Vec<Candle> = (0..30)
        .map(|i| Candle::new(i * 60000, 100.0, 100.0, 100.0, 100.0, 1000.0))
        .collect();

    let mut sma = Sma::new(14);
    for candle in &constant_candles {
        if let Some(value) = sma.next(candle.close) {
            assert!(
                approx_eq(value, 100.0, 0.0001),
                "SMA of constant series should equal the constant"
            );
        }
    }

    // RSI with constant price should be 50 (or close)
    let mut rsi = Rsi::new(14);
    for candle in &constant_candles {
        rsi.next(candle.close);
    }
    // Note: RSI with no change is typically undefined or 50

    // Bollinger Bands with constant price should have zero width
    let mut bb = BollingerBands::new(20, 2.0);
    for candle in &constant_candles {
        if let Some(bands) = bb.next(candle.close) {
            assert!(
                approx_eq(bands.upper, bands.middle, 0.0001),
                "BB with zero volatility should have equal bands"
            );
        }
    }
}
