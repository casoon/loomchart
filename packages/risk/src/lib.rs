//! # loom-risk
//!
//! Risk management and position sizing for trading systems.
//!
//! ## Features
//!
//! - **Position Sizing**: Fixed fractional, volatility targeting, ATR-based
//! - **Kill Switch**: Max daily loss, drawdown limits, cool-down periods
//! - **Account Model**: Balance, margin, leverage tracking
//! - **Portfolio Limits**: Max exposure, correlated exposure limits
//!
//! ## Example
//!
//! ```rust
//! use loom_risk::{PositionSizer, FixedFractional, Account, KillSwitch};
//!
//! let mut account = Account::new(10000.0, "USD");
//! let sizer = FixedFractional::new(0.02); // 2% risk per trade
//!
//! let position_size = sizer.calculate_size(
//!     account.balance,
//!     100.0,  // entry price
//!     95.0,   // stop loss
//! );
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod sizing;
pub mod kill_switch;
pub mod account;
pub mod limits;

pub use sizing::{
    PositionSizer, FixedFractional, VolatilityTarget, AtrBased,
    KellyCriterion, OptimalF,
};
pub use kill_switch::{KillSwitch, KillSwitchConfig, KillReason};
pub use account::{Account, Position, PositionSide};
pub use limits::{PortfolioLimits, ExposureLimit};
