//! Simple indicator usage example.
//!
//! Run with: cargo run --example simple_indicators

use loom_indicators::prelude::*;

fn main() {
    println!("=== Loom Indicators Example ===\n");

    // Sample price data
    let prices = vec![
        44.0, 44.25, 44.5, 43.75, 44.5, 44.25, 44.0, 43.75, 44.0, 44.25,
        44.75, 45.0, 45.5, 45.0, 44.5, 44.0, 43.75, 44.0, 44.25, 44.5,
    ];

    // Simple Moving Average
    println!("SMA (5 period):");
    let mut sma = Sma::new(5);
    for (i, price) in prices.iter().enumerate() {
        if let Some(value) = sma.next(*price) {
            println!("  [{:2}] Price: {:.2} -> SMA: {:.2}", i, price, value);
        }
    }

    println!("\nEMA (5 period):");
    let mut ema = Ema::new(5);
    for (i, price) in prices.iter().enumerate() {
        if let Some(value) = ema.next(*price) {
            println!("  [{:2}] Price: {:.2} -> EMA: {:.2}", i, price, value);
        }
    }

    // RSI
    println!("\nRSI (14 period):");
    let mut rsi = Rsi::new(14);
    for (i, price) in prices.iter().enumerate() {
        if let Some(value) = rsi.next(*price) {
            let signal = if value > 70.0 {
                "OVERBOUGHT"
            } else if value < 30.0 {
                "OVERSOLD"
            } else {
                ""
            };
            println!("  [{:2}] RSI: {:.2} {}", i, value, signal);
        }
    }

    // MACD
    println!("\nMACD (12, 26, 9):");
    let mut macd = Macd::new(12, 26, 9);
    for (i, price) in prices.iter().enumerate() {
        if let Some(output) = macd.next(*price) {
            let signal = if output.histogram > 0.0 { "BULLISH" } else { "BEARISH" };
            println!(
                "  [{:2}] MACD: {:.4}, Signal: {:.4}, Hist: {:.4} {}",
                i, output.macd, output.signal, output.histogram, signal
            );
        }
    }

    // Bollinger Bands
    println!("\nBollinger Bands (20, 2.0):");
    let mut bb = BollingerBands::new(20, 2.0);
    // Need more data for 20 period
    let extended_prices: Vec<f64> = (0..30)
        .map(|i| 44.0 + (i as f64 * 0.1).sin() * 2.0)
        .collect();

    for (i, price) in extended_prices.iter().enumerate() {
        if let Some(bands) = bb.next(*price) {
            println!(
                "  [{:2}] Price: {:.2} | Lower: {:.2} | Middle: {:.2} | Upper: {:.2}",
                i, price, bands.lower, bands.middle, bands.upper
            );
        }
    }

    println!("\n=== Done ===");
}
