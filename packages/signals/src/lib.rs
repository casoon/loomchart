//! # loom-signals
//!
//! Signal DSL and trading rules for technical analysis.
//!
//! ## Features
//!
//! - **Signal DSL**: `cross_over`, `cross_under`, `above`, `below`, `slope`
//! - **Wyckoff**: Accumulation/Distribution phases, Spring, Upthrust
//! - **Smart Money Concepts**: Order Blocks, Fair Value Gaps, Liquidity Sweeps
//! - **Elliott Waves**: Impulse and Corrective wave detection
//!
//! ## Example
//!
//! ```rust
//! use loom_signals::{cross_over, cross_under, above, SignalBuffer};
//!
//! let fast = [10.0, 11.0, 12.0, 11.5, 11.0];
//! let slow = [10.5, 10.8, 11.0, 11.2, 11.5];
//!
//! // Detect crossover
//! if cross_over(&fast, &slow) {
//!     println!("Bullish crossover!");
//! }
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod dsl;
pub mod wyckoff;
pub mod smc;
pub mod elliott;
pub mod rules;

// Re-exports
pub use dsl::{
    cross_over, cross_under, cross_over_value, cross_under_value,
    above, below, above_for, below_for,
    slope, acceleration, momentum,
    divergence, hidden_divergence,
    SignalBuffer, SignalState,
};

pub use wyckoff::{
    WyckoffPhase, WyckoffEvent, WyckoffAnalyzer,
    Spring, Upthrust, SignOfStrength, SignOfWeakness,
};

pub use smc::{
    OrderBlock, FairValueGap, LiquidityZone, BreakOfStructure,
    SmcAnalyzer, MarketStructure, Swing,
};

pub use elliott::{
    WaveType, WaveDegree, Wave, WaveCount,
    ElliottAnalyzer, ImpulsePattern, CorrectivePattern,
};

pub use rules::{
    Signal, SignalType, SignalStrength,
    Rule, RuleSet, Condition,
};
