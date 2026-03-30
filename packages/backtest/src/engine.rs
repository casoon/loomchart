//! Backtesting engine.

use loom_core::{Candle, Timeframe, Timestamp};
use crate::strategy::{Strategy, StrategyContext};
use crate::order::{Order, OrderStatus, Side};
use crate::fill::{FillModel, Fill};
use crate::metrics::{Metrics, EquityPoint, Trade};
use crate::position::PositionSide;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, boxed::Box};

/// Backtest configuration
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub initial_capital: f64,
    pub risk_free_rate: f64,
    pub fill_model: FillModel,
    pub allow_shorting: bool,
    pub margin_requirement: f64, // 1.0 = no margin, 0.5 = 2x leverage
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10000.0,
            risk_free_rate: 0.02, // 2% annual
            fill_model: FillModel::NextBarOpen,
            allow_shorting: true,
            margin_requirement: 1.0,
        }
    }
}

impl BacktestConfig {
    pub fn new(initial_capital: f64) -> Self {
        Self {
            initial_capital,
            ..Default::default()
        }
    }

    pub fn with_fill_model(mut self, fill_model: FillModel) -> Self {
        self.fill_model = fill_model;
        self
    }

    pub fn with_risk_free_rate(mut self, rate: f64) -> Self {
        self.risk_free_rate = rate;
        self
    }

    pub fn with_leverage(mut self, leverage: f64) -> Self {
        self.margin_requirement = 1.0 / leverage;
        self
    }

    pub fn no_shorting(mut self) -> Self {
        self.allow_shorting = false;
        self
    }
}

/// Backtest result
#[derive(Debug)]
pub struct BacktestResult {
    pub metrics: Metrics,
    pub equity_curve: Vec<EquityPoint>,
    pub trades: Vec<Trade>,
    pub final_equity: f64,
    pub total_bars: usize,
}

impl BacktestResult {
    pub fn summary(&self) -> String {
        self.metrics.summary()
    }
}

/// The backtesting engine
pub struct Backtest {
    config: BacktestConfig,
    candles: Vec<Candle>,
    symbol: String,
    timeframe: Timeframe,
}

impl Backtest {
    /// Create a new backtest
    pub fn new(symbol: &str, timeframe: Timeframe) -> Self {
        Self {
            config: BacktestConfig::default(),
            candles: Vec::new(),
            symbol: symbol.into(),
            timeframe,
        }
    }

    /// Set configuration
    pub fn config(mut self, config: BacktestConfig) -> Self {
        self.config = config;
        self
    }

    /// Set initial capital
    pub fn initial_capital(mut self, capital: f64) -> Self {
        self.config.initial_capital = capital;
        self
    }

    /// Set fill model
    pub fn fill_model(mut self, model: FillModel) -> Self {
        self.config.fill_model = model;
        self
    }

    /// Set candle data
    pub fn data(mut self, candles: Vec<Candle>) -> Self {
        self.candles = candles;
        self
    }

    /// Run the backtest
    pub fn run<S: Strategy>(mut self, strategy: &mut S) -> BacktestResult {
        let mut ctx = StrategyContext::new(
            &self.symbol,
            self.timeframe,
            self.config.initial_capital,
        );

        let mut equity_curve = Vec::new();
        let mut trades: Vec<Trade> = Vec::new();
        let mut open_trade: Option<OpenTrade> = None;

        // Initialize strategy
        strategy.on_start(&mut ctx);

        // Main loop
        for i in 0..self.candles.len() {
            let candle = &self.candles[i];
            let next_candle = self.candles.get(i + 1);

            // Update context
            ctx.update(candle);

            // Process pending orders
            let pending_orders = ctx.take_pending_orders();
            for mut order in pending_orders {
                // Validate order
                if !self.config.allow_shorting && order.side == Side::Sell && !ctx.has_position() {
                    order.status = OrderStatus::Rejected;
                    strategy.on_order_rejected(&mut ctx, &order, "Shorting not allowed");
                    continue;
                }

                // Try to fill
                if let Some(fill) = self.config.fill_model.fill(&order, candle, next_candle) {
                    order.status = OrderStatus::Filled;
                    order.filled_quantity = fill.quantity;

                    // Track trade
                    let pos = ctx.positions.get_mut(&self.symbol);
                    let was_flat = pos.is_flat();

                    // Apply fill to position
                    pos.apply_fill(&fill, order.side);

                    // Trade tracking
                    if was_flat && !pos.is_flat() {
                        // Opened new position
                        open_trade = Some(OpenTrade {
                            symbol: self.symbol.clone(),
                            side: if order.side == Side::Buy { "long".into() } else { "short".into() },
                            quantity: fill.quantity,
                            entry_price: fill.price,
                            entry_time: fill.timestamp,
                            commission: fill.commission,
                            slippage: fill.slippage,
                        });
                    } else if !was_flat && pos.is_flat() {
                        // Closed position
                        if let Some(ot) = open_trade.take() {
                            let pnl = if ot.side == "long" {
                                (fill.price - ot.entry_price) * ot.quantity
                            } else {
                                (ot.entry_price - fill.price) * ot.quantity
                            } - ot.commission - fill.commission;

                            let return_pct = if ot.side == "long" {
                                (fill.price - ot.entry_price) / ot.entry_price * 100.0
                            } else {
                                (ot.entry_price - fill.price) / ot.entry_price * 100.0
                            };

                            trades.push(Trade {
                                symbol: ot.symbol,
                                side: ot.side,
                                quantity: ot.quantity,
                                entry_price: ot.entry_price,
                                exit_price: fill.price,
                                entry_time: ot.entry_time,
                                exit_time: fill.timestamp,
                                pnl,
                                return_pct,
                                commission: ot.commission + fill.commission,
                                slippage: ot.slippage + fill.slippage,
                            });
                        }
                    }

                    strategy.on_fill(&mut ctx, &fill);
                }
            }

            // Call strategy
            strategy.on_candle(&mut ctx, candle);

            // Record equity
            equity_curve.push(EquityPoint {
                timestamp: candle.time,
                equity: ctx.equity,
                cash: ctx.equity - ctx.positions.total_unrealized_pnl(),
                positions_value: ctx.positions.total_unrealized_pnl(),
            });
        }

        // Close any open position at the end
        if let Some(ot) = open_trade.take() {
            if let Some(last_candle) = self.candles.last() {
                let exit_price = last_candle.close;
                let pnl = if ot.side == "long" {
                    (exit_price - ot.entry_price) * ot.quantity
                } else {
                    (ot.entry_price - exit_price) * ot.quantity
                } - ot.commission;

                let return_pct = if ot.side == "long" {
                    (exit_price - ot.entry_price) / ot.entry_price * 100.0
                } else {
                    (ot.entry_price - exit_price) / ot.entry_price * 100.0
                };

                trades.push(Trade {
                    symbol: ot.symbol,
                    side: ot.side,
                    quantity: ot.quantity,
                    entry_price: ot.entry_price,
                    exit_price,
                    entry_time: ot.entry_time,
                    exit_time: last_candle.time,
                    pnl,
                    return_pct,
                    commission: ot.commission,
                    slippage: ot.slippage,
                });
            }
        }

        // Finalize strategy
        strategy.on_end(&mut ctx);

        // Calculate metrics
        let metrics = Metrics::calculate(
            &equity_curve,
            &trades,
            self.config.initial_capital,
            self.config.risk_free_rate,
        );

        BacktestResult {
            metrics,
            final_equity: ctx.equity,
            total_bars: self.candles.len(),
            equity_curve,
            trades,
        }
    }
}

/// Internal tracking for open trades
struct OpenTrade {
    symbol: String,
    side: String,
    quantity: f64,
    entry_price: f64,
    entry_time: Timestamp,
    commission: f64,
    slippage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::SmaCrossover;

    fn generate_trending_data(count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);
        let mut price = 100.0;

        for i in 0..count {
            let trend = 0.1 * (1.0 + (i as f64 * 0.1).sin() * 0.3);
            let open = price;
            let close = price + trend;
            let high = close + trend * 0.3;
            let low = open - trend * 0.2;

            candles.push(Candle::new(
                (i as i64) * 3600000,
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

    #[test]
    fn test_backtest_sma_crossover() {
        let candles = generate_trending_data(200);

        let mut strategy = SmaCrossover::new(10, 30, 100.0);

        let result = Backtest::new("BTCUSDT", Timeframe::H1)
            .initial_capital(10000.0)
            .fill_model(FillModel::NextBarOpen)
            .data(candles)
            .run(&mut strategy);

        assert!(result.total_bars == 200);
        assert!(!result.trades.is_empty());
        println!("{}", result.summary());
    }

    #[test]
    fn test_backtest_with_realistic_fills() {
        let candles = generate_trending_data(100);

        let mut strategy = SmaCrossover::new(5, 15, 10.0);

        let result = Backtest::new("BTCUSDT", Timeframe::H1)
            .initial_capital(10000.0)
            .fill_model(FillModel::realistic())
            .data(candles)
            .run(&mut strategy);

        // With realistic fills, we should see commission costs
        assert!(result.metrics.total_commission > 0.0 || result.trades.is_empty());
    }
}
