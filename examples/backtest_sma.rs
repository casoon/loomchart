//! SMA Crossover Backtest Example.
//!
//! Run with: cargo run --release --example backtest_sma

use loom_core::{Candle, Timeframe};
use loom_backtest::{
    Backtest, BacktestConfig, Strategy, StrategyContext,
    FillModel, Report,
};
use loom_indicators::prelude::*;

/// SMA Crossover Strategy with configurable periods
struct SmaCrossoverStrategy {
    fast_period: usize,
    slow_period: usize,
    position_size: f64,
    fast_sma: Sma,
    slow_sma: Sma,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
}

impl SmaCrossoverStrategy {
    fn new(fast_period: usize, slow_period: usize, position_size: f64) -> Self {
        Self {
            fast_period,
            slow_period,
            position_size,
            fast_sma: Sma::new(fast_period),
            slow_sma: Sma::new(slow_period),
            prev_fast: None,
            prev_slow: None,
        }
    }
}

impl Strategy for SmaCrossoverStrategy {
    fn name(&self) -> &str {
        "SMA Crossover"
    }

    fn parameters(&self) -> Vec<(String, String)> {
        vec![
            ("fast_period".into(), self.fast_period.to_string()),
            ("slow_period".into(), self.slow_period.to_string()),
            ("position_size".into(), self.position_size.to_string()),
        ]
    }

    fn on_candle(&mut self, ctx: &mut StrategyContext, candle: &Candle) {
        let fast = self.fast_sma.next(candle.close);
        let slow = self.slow_sma.next(candle.close);

        if let (Some(f), Some(s), Some(pf), Some(ps)) =
            (fast, slow, self.prev_fast, self.prev_slow)
        {
            // Golden Cross: fast crosses above slow
            if pf <= ps && f > s && !ctx.has_position() {
                ctx.buy(self.position_size);
            }
            // Death Cross: fast crosses below slow
            else if pf >= ps && f < s && ctx.has_position() {
                ctx.close_position();
            }
        }

        self.prev_fast = fast;
        self.prev_slow = slow;
    }
}

/// Generate synthetic trending data
fn generate_data(count: usize) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(count);
    let mut price = 100.0;

    for i in 0..count {
        // Add trend and cycles
        let trend = 0.02 * (1.0 + (i as f64 * 0.01).sin());
        let cycle = (i as f64 * 0.1).sin() * 2.0;
        let noise = (i as f64 * 0.7).cos() * 0.5;

        let open = price;
        let change = trend + cycle * 0.1 + noise * 0.1;
        let close = (price + change).max(1.0);
        let high = open.max(close) + (change.abs() + 0.5);
        let low = open.min(close) - (change.abs() + 0.3);
        let volume = 1000.0 + (i as f64 * 0.2).sin().abs() * 500.0;

        candles.push(Candle::new(
            (i as i64) * 3600000, // 1 hour intervals
            open,
            high,
            low.max(0.1),
            close,
            volume,
        ));

        price = close;
    }

    candles
}

fn main() {
    println!("=== SMA Crossover Backtest ===\n");

    // Generate 1 year of hourly data
    let candles = generate_data(8760);

    println!("Data: {} candles", candles.len());
    println!("Period: {} hours ({:.0} days)\n",
        candles.len(),
        candles.len() as f64 / 24.0
    );

    // Run backtest
    let mut strategy = SmaCrossoverStrategy::new(10, 30, 100.0);

    let result = Backtest::new("BTCUSDT", Timeframe::H1)
        .initial_capital(10000.0)
        .fill_model(FillModel::realistic())
        .data(candles)
        .run(&mut strategy);

    // Generate report
    let report = Report::new(result, strategy.name())
        .with_params(strategy.parameters());

    println!("{}", report.to_text());

    // Optionally save CSV files
    // std::fs::write("trades.csv", report.trades_to_csv()).ok();
    // std::fs::write("equity.csv", report.equity_to_csv()).ok();
}
