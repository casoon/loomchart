//! Strategy trait and context.

use loom_core::{Candle, Timeframe, Timestamp, Price};
use crate::order::{Order, OrderId, Side};
use crate::position::{Position, PositionManager};
use crate::fill::Fill;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, collections::VecDeque};
#[cfg(feature = "std")]
use std::collections::VecDeque;

/// Strategy trait - implement this to create trading strategies
pub trait Strategy {
    /// Called once at the start of backtest
    fn on_start(&mut self, _ctx: &mut StrategyContext) {}

    /// Called for each new candle
    fn on_candle(&mut self, ctx: &mut StrategyContext, candle: &Candle);

    /// Called when an order is filled
    fn on_fill(&mut self, _ctx: &mut StrategyContext, _fill: &Fill) {}

    /// Called when an order is rejected
    fn on_order_rejected(&mut self, _ctx: &mut StrategyContext, _order: &Order, _reason: &str) {}

    /// Called once at the end of backtest
    fn on_end(&mut self, _ctx: &mut StrategyContext) {}

    /// Strategy name
    fn name(&self) -> &str {
        "Unnamed Strategy"
    }

    /// Strategy parameters as key-value pairs
    fn parameters(&self) -> Vec<(String, String)> {
        Vec::new()
    }
}

/// Context provided to strategies during backtest
pub struct StrategyContext {
    /// Current symbol being traded
    pub symbol: String,
    /// Current timeframe
    pub timeframe: Timeframe,
    /// Current timestamp
    pub current_time: Timestamp,
    /// Current candle index
    pub bar_index: usize,
    /// Position manager
    pub positions: PositionManager,
    /// Account equity
    pub equity: f64,
    /// Initial capital
    pub initial_capital: f64,
    /// Pending orders
    pending_orders: Vec<Order>,
    /// Order ID counter
    next_order_id: u64,
    /// Historical candles (for lookback)
    candle_history: VecDeque<Candle>,
    /// Max history size
    max_history: usize,
    /// Custom data storage
    #[cfg(feature = "std")]
    pub data: std::collections::HashMap<String, f64>,
    #[cfg(not(feature = "std"))]
    pub data: alloc::collections::BTreeMap<String, f64>,
}

impl StrategyContext {
    pub fn new(symbol: &str, timeframe: Timeframe, initial_capital: f64) -> Self {
        Self {
            symbol: symbol.into(),
            timeframe,
            current_time: 0,
            bar_index: 0,
            positions: PositionManager::new(),
            equity: initial_capital,
            initial_capital,
            pending_orders: Vec::new(),
            next_order_id: 1,
            candle_history: VecDeque::new(),
            max_history: 500,
            #[cfg(feature = "std")]
            data: std::collections::HashMap::new(),
            #[cfg(not(feature = "std"))]
            data: alloc::collections::BTreeMap::new(),
        }
    }

    /// Set max history size
    pub fn with_history_size(mut self, size: usize) -> Self {
        self.max_history = size;
        self
    }

    /// Update context with new candle
    pub fn update(&mut self, candle: &Candle) {
        self.current_time = candle.time;
        self.bar_index += 1;

        // Update position prices
        self.positions.get_mut(&self.symbol).update_price(candle.close, candle.time);

        // Update equity
        self.equity = self.initial_capital +
            self.positions.total_realized_pnl() +
            self.positions.total_unrealized_pnl();

        // Add to history
        self.candle_history.push_back(candle.clone());
        if self.candle_history.len() > self.max_history {
            self.candle_history.pop_front();
        }
    }

    /// Submit an order
    pub fn submit_order(&mut self, mut order: Order) -> OrderId {
        order.id = Some(OrderId(self.next_order_id));
        order.symbol = self.symbol.clone();
        order.created_at = Some(self.current_time);
        self.next_order_id += 1;

        let id = order.id.unwrap();
        self.pending_orders.push(order);
        id
    }

    /// Submit a market buy order
    pub fn buy(&mut self, quantity: f64) -> OrderId {
        self.submit_order(Order::market(&self.symbol, Side::Buy, quantity))
    }

    /// Submit a market sell order
    pub fn sell(&mut self, quantity: f64) -> OrderId {
        self.submit_order(Order::market(&self.symbol, Side::Sell, quantity))
    }

    /// Submit a limit buy order
    pub fn buy_limit(&mut self, quantity: f64, price: Price) -> OrderId {
        self.submit_order(Order::limit(&self.symbol, Side::Buy, quantity, price))
    }

    /// Submit a limit sell order
    pub fn sell_limit(&mut self, quantity: f64, price: Price) -> OrderId {
        self.submit_order(Order::limit(&self.symbol, Side::Sell, quantity, price))
    }

    /// Submit a stop loss order
    pub fn stop_loss(&mut self, quantity: f64, stop_price: Price) -> OrderId {
        self.submit_order(Order::stop(&self.symbol, Side::Sell, quantity, stop_price).reduce_only())
    }

    /// Submit a take profit order
    pub fn take_profit(&mut self, quantity: f64, price: Price) -> OrderId {
        self.submit_order(Order::take_profit(&self.symbol, Side::Sell, quantity, price).reduce_only())
    }

    /// Close entire position
    pub fn close_position(&mut self) -> Option<OrderId> {
        let pos = self.positions.get(&self.symbol)?;
        if pos.is_flat() {
            return None;
        }

        let quantity = pos.quantity;
        let side = if pos.is_long() { Side::Sell } else { Side::Buy };

        Some(self.submit_order(Order::market(&self.symbol, side, quantity).reduce_only()))
    }

    /// Cancel an order
    pub fn cancel_order(&mut self, order_id: OrderId) -> bool {
        if let Some(pos) = self.pending_orders.iter().position(|o| o.id == Some(order_id)) {
            self.pending_orders.remove(pos);
            true
        } else {
            false
        }
    }

    /// Cancel all pending orders
    pub fn cancel_all(&mut self) {
        self.pending_orders.clear();
    }

    /// Get current position
    pub fn position(&self) -> Option<&Position> {
        self.positions.get(&self.symbol).filter(|p| !p.is_flat())
    }

    /// Check if we have a position
    pub fn has_position(&self) -> bool {
        self.positions.has_position(&self.symbol)
    }

    /// Get position quantity (positive for long, negative for short)
    pub fn position_quantity(&self) -> f64 {
        self.positions.get(&self.symbol)
            .map(|p| if p.is_long() { p.quantity } else if p.is_short() { -p.quantity } else { 0.0 })
            .unwrap_or(0.0)
    }

    /// Take pending orders for processing
    pub fn take_pending_orders(&mut self) -> Vec<Order> {
        core::mem::take(&mut self.pending_orders)
    }

    /// Get historical candle (0 = current, 1 = previous, etc.)
    pub fn candle(&self, lookback: usize) -> Option<&Candle> {
        if lookback >= self.candle_history.len() {
            return None;
        }
        self.candle_history.get(self.candle_history.len() - 1 - lookback)
    }

    /// Get close prices
    pub fn closes(&self, count: usize) -> Vec<f64> {
        self.candle_history.iter()
            .rev()
            .take(count)
            .map(|c| c.close)
            .collect()
    }

    /// Get high prices
    pub fn highs(&self, count: usize) -> Vec<f64> {
        self.candle_history.iter()
            .rev()
            .take(count)
            .map(|c| c.high)
            .collect()
    }

    /// Get low prices
    pub fn lows(&self, count: usize) -> Vec<f64> {
        self.candle_history.iter()
            .rev()
            .take(count)
            .map(|c| c.low)
            .collect()
    }

    /// Get volumes
    pub fn volumes(&self, count: usize) -> Vec<f64> {
        self.candle_history.iter()
            .rev()
            .take(count)
            .map(|c| c.volume)
            .collect()
    }

    /// Current price (last close)
    pub fn price(&self) -> f64 {
        self.candle(0).map(|c| c.close).unwrap_or(0.0)
    }

    /// Return percentage
    pub fn return_pct(&self) -> f64 {
        if self.initial_capital == 0.0 {
            return 0.0;
        }
        (self.equity - self.initial_capital) / self.initial_capital * 100.0
    }
}

/// A simple moving average crossover strategy
pub struct SmaCrossover {
    fast_period: usize,
    slow_period: usize,
    position_size: f64,
}

impl SmaCrossover {
    pub fn new(fast_period: usize, slow_period: usize, position_size: f64) -> Self {
        Self { fast_period, slow_period, position_size }
    }

    fn sma(prices: &[f64]) -> f64 {
        if prices.is_empty() {
            return 0.0;
        }
        prices.iter().sum::<f64>() / prices.len() as f64
    }
}

impl Strategy for SmaCrossover {
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

    fn on_candle(&mut self, ctx: &mut StrategyContext, _candle: &Candle) {
        if ctx.bar_index < self.slow_period {
            return;
        }

        let fast_prices = ctx.closes(self.fast_period);
        let slow_prices = ctx.closes(self.slow_period);

        let fast_ma = Self::sma(&fast_prices);
        let slow_ma = Self::sma(&slow_prices);

        // Get previous MAs
        let prev_closes = ctx.closes(self.slow_period + 1);
        if prev_closes.len() < self.slow_period + 1 {
            return;
        }

        let prev_fast_prices: Vec<_> = prev_closes[1..self.fast_period+1].to_vec();
        let prev_slow_prices: Vec<_> = prev_closes[1..self.slow_period+1].to_vec();

        let prev_fast_ma = Self::sma(&prev_fast_prices);
        let prev_slow_ma = Self::sma(&prev_slow_prices);

        // Crossover detection
        let crossed_above = prev_fast_ma <= prev_slow_ma && fast_ma > slow_ma;
        let crossed_below = prev_fast_ma >= prev_slow_ma && fast_ma < slow_ma;

        if crossed_above && !ctx.has_position() {
            ctx.buy(self.position_size);
        } else if crossed_below && ctx.has_position() {
            ctx.close_position();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_context() {
        let mut ctx = StrategyContext::new("BTCUSDT", Timeframe::H1, 10000.0);

        let candle = Candle::new(1700000000000, 100.0, 105.0, 98.0, 103.0, 1000.0);
        ctx.update(&candle);

        assert_eq!(ctx.bar_index, 1);
        assert_eq!(ctx.price(), 103.0);
    }

    #[test]
    fn test_order_submission() {
        let mut ctx = StrategyContext::new("BTCUSDT", Timeframe::H1, 10000.0);

        let id = ctx.buy(1.0);
        assert_eq!(id.0, 1);

        let orders = ctx.take_pending_orders();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].quantity, 1.0);
    }
}
