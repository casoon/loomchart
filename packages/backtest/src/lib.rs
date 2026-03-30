//! Loom Backtest - Backtesting engine for the Loom trading platform.
//!
//! This crate provides a complete backtesting framework with:
//!
//! - Deterministic event replay
//! - Multiple fill models (instant, realistic, slippage)
//! - Portfolio and position tracking
//! - Performance metrics (Sharpe, Sortino, max drawdown, etc.)
//! - Strategy trait for easy strategy development
//!
//! # Example
//!
//! ```rust,ignore
//! use loom_backtest::{Backtest, Strategy, StrategyContext, Order, Side};
//! use loom_core::Candle;
//!
//! struct SimpleMA {
//!     period: usize,
//!     prices: Vec<f64>,
//! }
//!
//! impl Strategy for SimpleMA {
//!     fn on_candle(&mut self, ctx: &mut StrategyContext, candle: &Candle) {
//!         self.prices.push(candle.close);
//!
//!         if self.prices.len() >= self.period {
//!             let ma: f64 = self.prices.iter().rev()
//!                 .take(self.period).sum::<f64>() / self.period as f64;
//!
//!             if candle.close > ma && !ctx.has_position() {
//!                 ctx.submit_order(Order::market(Side::Buy, 1.0));
//!             } else if candle.close < ma && ctx.has_position() {
//!                 ctx.submit_order(Order::market(Side::Sell, 1.0));
//!             }
//!         }
//!     }
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod engine;
pub mod strategy;
pub mod order;
pub mod fill;
pub mod position;
pub mod metrics;
pub mod report;

pub use engine::{Backtest, BacktestConfig, BacktestResult};
pub use strategy::{Strategy, StrategyContext};
pub use order::{Order, OrderType, Side, OrderId, OrderStatus};
pub use fill::{FillModel, Fill};
pub use position::{Position, PositionManager};
pub use metrics::Metrics;
pub use report::Report;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::engine::{Backtest, BacktestConfig, BacktestResult};
    pub use crate::strategy::{Strategy, StrategyContext};
    pub use crate::order::{Order, OrderType, Side};
    pub use crate::fill::FillModel;
    pub use crate::metrics::Metrics;
}
