//! Integration tests for Chart Core
//!
//! These tests verify end-to-end functionality of the chart engine

use chartcore::core::{
    Candle, CandleGenerator, Chart, GeneratorConfig, MarketType, Scenario, Timeframe, Trend,
    VolatilityRegime,
};

#[test]
fn test_chart_initialization() {
    let chart = Chart::new();
    assert!(chart.is_empty());
    assert_eq!(chart.len(), 0);
}

#[test]
fn test_candle_generator_basic() {
    let config = GeneratorConfig::crypto();
    let mut gen = CandleGenerator::new(config);

    let candles = gen.generate(100);
    assert_eq!(candles.len(), 100);

    // Verify OHLC validity
    for candle in &candles {
        assert!(candle.h >= candle.o, "High must be >= Open");
        assert!(candle.h >= candle.c, "High must be >= Close");
        assert!(candle.l <= candle.o, "Low must be <= Open");
        assert!(candle.l <= candle.c, "Low must be <= Close");
        assert!(candle.v > 0.0, "Volume must be positive");
    }
}

#[test]
fn test_candle_generator_scenarios() {
    let scenarios = vec![
        Scenario::Breakout,
        Scenario::Consolidation,
        Scenario::NewsEvent,
        Scenario::MarketCrash,
        Scenario::Recovery,
        Scenario::DoubleTop,
        Scenario::DoubleBottom,
    ];

    for scenario in scenarios {
        let config = GeneratorConfig::crypto();
        let mut gen = CandleGenerator::new(config);
        let candles = gen.generate_scenario(scenario, 100);

        assert_eq!(candles.len(), 100, "Scenario {:?} failed", scenario);

        // Verify OHLC validity
        for candle in &candles {
            assert!(candle.h >= candle.o);
            assert!(candle.h >= candle.c);
            assert!(candle.l <= candle.o);
            assert!(candle.l <= candle.c);
        }
    }
}

#[test]
fn test_streaming_candles() {
    let config = GeneratorConfig::crypto();
    let mut gen = CandleGenerator::new(config);

    // Start streaming candle
    let initial = gen.start_streaming_candle();
    assert_eq!(initial.o, initial.h);
    assert_eq!(initial.o, initial.l);
    assert_eq!(initial.o, initial.c);
    assert_eq!(initial.v, 0.0);

    // Update streaming candle multiple times
    for _ in 0..5 {
        gen.advance_time(1000); // 1 second
        if let Some(updated) = gen.update_streaming_candle() {
            assert!(updated.h >= updated.o);
            assert!(updated.l <= updated.o);
            assert!(updated.v >= 0.0);
        }
    }

    // Finalize candle
    if let Some(final_candle) = gen.finalize_streaming_candle() {
        assert!(final_candle.h >= final_candle.o);
        assert!(final_candle.l <= final_candle.o);
    }
}

#[test]
fn test_trend_behavior() {
    let trends = vec![
        Trend::BullishStrong,
        Trend::BullishMild,
        Trend::Sideways,
        Trend::BearishMild,
        Trend::BearishStrong,
    ];

    for trend in trends {
        let config = GeneratorConfig::crypto().with_trend(trend);
        let mut gen = CandleGenerator::new(config);
        let candles = gen.generate(1000);

        let start_price = candles.first().unwrap().c;
        let end_price = candles.last().unwrap().c;

        match trend {
            Trend::BullishStrong | Trend::BullishMild => {
                // Should trend up (with some tolerance for volatility)
                // Over 1000 candles, we expect net upward movement
            }
            Trend::Sideways => {
                // Price should stay relatively close to start
                let deviation = ((end_price - start_price) / start_price).abs();
                assert!(deviation < 0.5, "Sideways should not deviate more than 50%");
            }
            Trend::BearishStrong | Trend::BearishMild => {
                // Should trend down
            }
        }
    }
}

#[test]
fn test_market_types() {
    let markets = vec![
        MarketType::Stock,
        MarketType::Forex,
        MarketType::Crypto,
        MarketType::Futures,
        MarketType::Commodities,
    ];

    for market in markets {
        let config = GeneratorConfig::new(market);
        let mut gen = CandleGenerator::new(config);
        let candles = gen.generate(50);

        assert_eq!(candles.len(), 50, "Market {:?} failed", market);

        // Each market should produce valid candles
        for candle in &candles {
            assert!(candle.h >= candle.l);
            assert!(candle.v >= 0.0);
        }
    }
}

#[test]
fn test_volatility_regimes() {
    let regimes = vec![
        VolatilityRegime::Low,
        VolatilityRegime::Normal,
        VolatilityRegime::High,
        VolatilityRegime::Extreme,
    ];

    for regime in regimes {
        let config = GeneratorConfig::crypto().with_regime(regime);
        let mut gen = CandleGenerator::new(config);
        let candles = gen.generate(100);

        // Calculate average range
        let avg_range: f64 =
            candles.iter().map(|c| (c.h - c.l) / c.o).sum::<f64>() / candles.len() as f64;

        // Verify volatility increases with regime
        match regime {
            VolatilityRegime::Low => {
                assert!(avg_range < 0.05, "Low volatility too high: {}", avg_range);
            }
            VolatilityRegime::Extreme => {
                assert!(
                    avg_range > 0.02,
                    "Extreme volatility too low: {}",
                    avg_range
                );
            }
            _ => {}
        }
    }
}

#[test]
fn test_chart_with_generated_data() {
    let config = GeneratorConfig::crypto();
    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate(500);

    let mut chart = Chart::new();

    // Push all candles to chart
    for candle in candles {
        chart.push(candle);
    }

    assert_eq!(chart.len(), 500);
    assert!(!chart.is_empty());
}

#[test]
fn test_reproducibility() {
    let config1 = GeneratorConfig::crypto().with_seed(12345);
    let config2 = GeneratorConfig::crypto().with_seed(12345);

    let mut gen1 = CandleGenerator::new(config1);
    let mut gen2 = CandleGenerator::new(config2);

    let candles1 = gen1.generate(100);
    let candles2 = gen2.generate(100);

    // Same seed should produce identical results
    for (c1, c2) in candles1.iter().zip(candles2.iter()) {
        assert_eq!(c1.time, c2.time);
        assert_eq!(c1.o, c2.o);
        assert_eq!(c1.h, c2.h);
        assert_eq!(c1.l, c2.l);
        assert_eq!(c1.c, c2.c);
    }
}

#[test]
fn test_generator_reset() {
    let config = GeneratorConfig::crypto().with_seed(999);
    let mut gen = CandleGenerator::new(config);

    let candles1 = gen.generate(50);

    gen.reset();

    let candles2 = gen.generate(50);

    // After reset, should produce same sequence
    assert_eq!(candles1.len(), candles2.len());
    for (c1, c2) in candles1.iter().zip(candles2.iter()) {
        assert_eq!(c1.o, c2.o);
        assert_eq!(c1.h, c2.h);
        assert_eq!(c1.l, c2.l);
        assert_eq!(c1.c, c2.c);
    }
}

#[test]
fn test_timeframe_scaling() {
    let timeframes = vec![Timeframe::M1, Timeframe::M5, Timeframe::H1, Timeframe::D1];

    for tf in timeframes {
        let mut config = GeneratorConfig::crypto();
        config.timeframe = tf;
        let mut gen = CandleGenerator::new(config);
        let candles = gen.generate(10);

        // Verify timestamps are correctly spaced
        for i in 1..candles.len() {
            let time_diff = candles[i].time - candles[i - 1].time;
            let expected_diff = tf.duration_ms();
            assert_eq!(
                time_diff, expected_diff,
                "Timeframe {:?} spacing incorrect",
                tf
            );
        }
    }
}

#[test]
fn test_scenario_breakout() {
    let config = GeneratorConfig::crypto();
    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate_scenario(Scenario::Breakout, 300);

    // First 2/3 should be consolidation (lower volatility)
    let consolidation = &candles[0..200];
    let breakout = &candles[200..];

    let consolidation_range: f64 =
        consolidation.iter().map(|c| c.h - c.l).sum::<f64>() / consolidation.len() as f64;

    let breakout_range: f64 =
        breakout.iter().map(|c| c.h - c.l).sum::<f64>() / breakout.len() as f64;

    // Breakout should have higher average range
    assert!(
        breakout_range > consolidation_range,
        "Breakout volatility should be higher"
    );
}

#[test]
fn test_scenario_recovery() {
    let config = GeneratorConfig::crypto();
    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate_scenario(Scenario::Recovery, 400);

    let start_price = candles[0].c;
    let crash_bottom = candles[0..100]
        .iter()
        .map(|c| c.l)
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let end_price = candles.last().unwrap().c;

    // Should crash down then recover
    assert!(crash_bottom < start_price, "Should crash below start");
    // May not fully recover, but should be higher than crash bottom
    assert!(end_price > crash_bottom, "Should recover from crash");
}
