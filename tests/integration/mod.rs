//! Integration tests for the Loom trading platform.
//!
//! These tests verify that all crates work correctly together.

use loom_core::{Candle, Timeframe, Symbol};
use loom_indicators::prelude::*;
use loom_signals::{cross_over, cross_under, WyckoffAnalyzer, SmcAnalyzer};
use loom_risk::{FixedFractional, PositionSizer, KillSwitch, KillSwitchConfig};

mod common {
    include!("../common/mod.rs");
}

use common::*;

/// Test full workflow: data -> indicators -> signals -> risk
#[test]
fn test_full_workflow() {
    // Generate test data
    let candles = generate_test_candles(100, 100.0, 2.0);

    // Apply indicators
    let mut ema_fast = Ema::new(9);
    let mut ema_slow = Ema::new(21);
    let mut rsi = Rsi::new(14);

    let mut fast_values = Vec::new();
    let mut slow_values = Vec::new();
    let mut rsi_values = Vec::new();

    for candle in &candles {
        if let Some(f) = ema_fast.next(candle.close) {
            fast_values.push(f);
        }
        if let Some(s) = ema_slow.next(candle.close) {
            slow_values.push(s);
        }
        if let Some(r) = rsi.next(candle.close) {
            rsi_values.push(r);
        }
    }

    // Verify we got values
    assert!(!fast_values.is_empty());
    assert!(!slow_values.is_empty());
    assert!(!rsi_values.is_empty());

    // Check for crossovers
    if fast_values.len() >= 2 && slow_values.len() >= 2 {
        let _crossover = cross_over(&fast_values, &slow_values);
        let _crossunder = cross_under(&fast_values, &slow_values);
    }

    // Risk management
    let sizer = FixedFractional::new(0.02); // 2% risk
    let account = 10000.0;
    let entry = 100.0;
    let stop = 95.0;

    let size = sizer.calculate_size(account, entry, stop);
    assert!(size > 0.0);
    assert!(size * (entry - stop) <= account * 0.02 * 1.01); // Within 2% + tolerance
}

/// Test Wyckoff analysis
#[test]
fn test_wyckoff_integration() {
    let candles = generate_range(50, 100.0, 10.0);

    let analyzer = WyckoffAnalyzer::new();
    let springs = analyzer.detect_springs(&candles);
    let upthrusts = analyzer.detect_upthrusts(&candles);

    // In ranging data, we might find some patterns
    // The important thing is that it doesn't panic
    let _ = springs.len();
    let _ = upthrusts.len();
}

/// Test SMC analysis
#[test]
fn test_smc_integration() {
    let candles = generate_test_candles(100, 100.0, 3.0);

    let analyzer = SmcAnalyzer::new();
    let order_blocks = analyzer.find_order_blocks(&candles);
    let fvgs = analyzer.find_fvgs(&candles);
    let liquidity = analyzer.find_liquidity_zones(&candles);

    // Verify no panics and reasonable output
    let _ = order_blocks.len();
    let _ = fvgs.len();
    let _ = liquidity.len();
}

/// Test kill switch
#[test]
fn test_kill_switch_integration() {
    let config = KillSwitchConfig::conservative();
    let mut kill_switch = KillSwitch::new(config, 10000.0);

    // Simulate some losses
    let now = chrono::Utc::now().timestamp_millis();
    kill_switch.on_trade(-100.0, now);
    kill_switch.on_trade(-150.0, now + 1000);
    kill_switch.on_trade(-200.0, now + 2000);

    // Check if trading is still allowed
    let can_trade = kill_switch.can_trade();

    // After 3 consecutive losses of $450 total (4.5% of 10k),
    // conservative config should still allow trading
    // but we're testing the mechanism works
    let _ = can_trade;

    // More losses should trigger kill switch
    kill_switch.on_trade(-300.0, now + 3000);
    kill_switch.on_trade(-400.0, now + 4000);

    // At this point, we've lost $1150 (11.5%), which exceeds daily limit (5%)
    assert!(!kill_switch.can_trade() || kill_switch.reason().is_some());
}

/// Test symbol metadata
#[test]
fn test_symbol_integration() {
    let btc = Symbol::crypto("BTCUSDT", "BTC", "USDT")
        .with_tick_size(0.01)
        .with_lot_size(0.001)
        .with_min_notional(10.0);

    assert_eq!(btc.symbol, "BTCUSDT");
    assert_eq!(btc.tick_size, 0.01);
    assert_eq!(btc.lot_size, 0.001);

    // Test price rounding
    let price = 50000.123456;
    let rounded = btc.round_price(price);
    assert!((rounded - 50000.12).abs() < 0.001);

    // Test quantity rounding
    let qty = 0.123456;
    let rounded_qty = btc.round_quantity(qty);
    assert!((rounded_qty - 0.123).abs() < 0.0001);
}

/// Test timeframe operations
#[test]
fn test_timeframe_integration() {
    let tf = Timeframe::H1;

    // Test bucket alignment
    let timestamp = 1700000000000i64; // Some arbitrary timestamp
    let bucket_start = tf.bucket_start(timestamp);

    // Bucket start should be <= original timestamp
    assert!(bucket_start <= timestamp);

    // Bucket start should be aligned to hour
    assert_eq!(bucket_start % (3600 * 1000), 0);

    // Next bucket should be 1 hour later
    let next_bucket = tf.next_bucket_start(timestamp);
    assert_eq!(next_bucket - bucket_start, 3600 * 1000);
}

/// Test multiple indicators together
#[test]
fn test_indicator_combination() {
    let candles = generate_uptrend(50, 100.0, 1.0);

    let mut ema = Ema::new(14);
    let mut rsi = Rsi::new(14);
    let mut bb = BollingerBands::new(20, 2.0);
    let mut atr = Atr::new(14);
    let mut macd = Macd::new(12, 26, 9);

    let mut signals = Vec::new();

    for candle in &candles {
        let ema_val = ema.next(candle.close);
        let rsi_val = rsi.next(candle.close);
        let bb_val = bb.next(candle.close);
        let atr_val = atr.next(candle);
        let macd_val = macd.next(candle.close);

        // Generate signal when conditions align
        if let (Some(e), Some(r), Some(m)) = (ema_val, rsi_val, macd_val) {
            if candle.close > e && r < 70.0 && m.histogram > 0.0 {
                signals.push(candle.time);
            }
        }
    }

    // In an uptrend, we should get some buy signals
    assert!(!signals.is_empty(), "Should generate signals in uptrend");
}
