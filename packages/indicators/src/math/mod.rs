//! Mathematical functions for technical analysis
//!
//! This module contains pure, stateless mathematical functions that can be
//! composed to build indicators. All functions are `no_std` compatible.
//!
//! # Categories
//!
//! - **Average**: Moving average calculations (SMA, EMA, WMA)
//! - **Stats**: Statistical functions (variance, stddev, correlation)
//! - **Range**: Price range functions (true range, highest, lowest)
//! - **Momentum**: Momentum calculations (ROC, gain/loss)
//! - **Performance**: Trading metrics (Sharpe, Sortino, drawdown, beta)

pub mod average;
pub mod momentum;
pub mod range;
pub mod stats;
pub mod performance;

// Re-export commonly used functions at module level
pub use average::{ema_multiplier, ema_next, sma, wma};
pub use momentum::{gain, loss, roc};
pub use range::{highest, lowest, true_range};
pub use stats::{mean, stddev, variance};
pub use performance::{sharpe_ratio, sortino_ratio, max_drawdown, beta, hurst_exponent};
