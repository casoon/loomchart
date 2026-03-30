// Example: Using the Candle Generator
//
// Run with: cargo run --example generator

use chartcore::prelude::*;

fn main() {
    println!("🎲 Chartcore Candle Generator Examples\n");

    // Example 1: Crypto market (24/7, high volatility)
    println!("📊 Example 1: Crypto Bull Market");
    println!("═══════════════════════════════════");

    let config = GeneratorConfig::crypto()
        .with_trend(Trend::BullishStrong)
        .with_regime(VolatilityRegime::High)
        .with_seed(42);

    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate(10);

    print_candles("BTC/USD", &candles);

    // Example 2: Stock market (weekdays only, moderate volatility)
    println!("\n📊 Example 2: Stock Market (Sideways)");
    println!("═══════════════════════════════════");

    let config = GeneratorConfig::stock()
        .with_trend(Trend::Sideways)
        .with_regime(VolatilityRegime::Normal)
        .with_volatility(1.5)
        .with_seed(123);

    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate(10);

    print_candles("AAPL", &candles);

    // Example 3: Forex (24/5, low volatility)
    println!("\n📊 Example 3: Forex (Bearish Trend)");
    println!("═══════════════════════════════════");

    let config = GeneratorConfig::forex()
        .with_trend(Trend::BearishMild)
        .with_regime(VolatilityRegime::Low)
        .with_seed(456);

    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate(10);

    print_candles("EUR/USD", &candles);

    // Example 4: Generate large dataset for backtesting
    println!("\n📊 Example 4: Large Dataset for Backtesting");
    println!("═══════════════════════════════════");

    let config = GeneratorConfig::crypto()
        .with_trend(Trend::BullishMild)
        .with_seed(789);

    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate(1000);

    println!("Generated {} candles", candles.len());
    println!("Start price: ${:.2}", candles.first().unwrap().o);
    println!("End price:   ${:.2}", candles.last().unwrap().c);
    println!(
        "Change:      {:.2}%",
        (candles.last().unwrap().c - candles.first().unwrap().o) / candles.first().unwrap().o
            * 100.0
    );

    // Calculate some statistics
    let mut total_volume = 0.0;
    let mut bullish = 0;
    let mut bearish = 0;

    for candle in &candles {
        total_volume += candle.v;
        if candle.is_bullish() {
            bullish += 1;
        } else {
            bearish += 1;
        }
    }

    println!("Total volume: ${:.0}", total_volume);
    println!(
        "Bullish candles: {} ({:.1}%)",
        bullish,
        bullish as f64 / candles.len() as f64 * 100.0
    );
    println!(
        "Bearish candles: {} ({:.1}%)",
        bearish,
        bearish as f64 / candles.len() as f64 * 100.0
    );

    // Example 5: Extreme volatility (flash crash)
    println!("\n📊 Example 5: Flash Crash Simulation");
    println!("═══════════════════════════════════");

    let config = GeneratorConfig::stock()
        .with_trend(Trend::BearishStrong)
        .with_regime(VolatilityRegime::Extreme)
        .with_volatility(10.0)
        .with_seed(999);

    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate(5);

    print_candles("CRASH", &candles);

    // Example 6: Use with Chart and Indicators
    println!("\n📊 Example 6: Integration with Indicators");
    println!("═══════════════════════════════════");

    let config = GeneratorConfig::crypto().with_seed(1234);
    let mut gen = CandleGenerator::new(config);
    let candles = gen.generate(50);

    // Create chart
    let mut chart = Chart::new();
    for candle in &candles {
        chart.push(candle.clone());
    }

    // Calculate RSI
    let registry = IndicatorRegistry::default();
    let context =
        CalculationContext::new(chart.candles()).with_input("length", InputValue::Int(14));

    let result = registry.calculate("rsi", &context).unwrap();
    let rsi_values = &result.plots["rsi"];

    println!("Generated {} candles and calculated RSI", chart.len());
    println!("Last 5 RSI values:");
    for (i, value) in rsi_values.iter().rev().take(5).enumerate() {
        if let Some(rsi) = value {
            println!("  {} candles ago: RSI = {:.2}", i, rsi);
        }
    }
}

fn print_candles(symbol: &str, candles: &[Candle]) {
    println!("Symbol: {}", symbol);
    println!(
        "{:<6} {:<10} {:<10} {:<10} {:<10} {:<12} {:<6}",
        "#", "Open", "High", "Low", "Close", "Volume", "Type"
    );
    println!("{}", "─".repeat(70));

    for (i, candle) in candles.iter().enumerate() {
        let candle_type = if candle.is_bullish() { "🟢" } else { "🔴" };
        println!(
            "{:<6} ${:<9.2} ${:<9.2} ${:<9.2} ${:<9.2} ${:<11.0} {}",
            i + 1,
            candle.o,
            candle.h,
            candle.l,
            candle.c,
            candle.v,
            candle_type
        );
    }
}
