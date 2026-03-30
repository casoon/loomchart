//! Trend-following indicators
//!
//! Moving averages, channels, and other trend indicators.

mod advanced_ma;
mod adx;
mod aroon;
mod beta;
mod donchian;
mod ema;
mod fibonacci;
mod ichimoku;
mod keltner;
mod macd;
mod parabolic_sar;
mod pivot_points;
mod sma;
mod supertrend;

// Basic moving averages
pub use ema::{Dema, Ema, Tema};
pub use sma::Sma;

// Advanced moving averages
pub use advanced_ma::{Alma, Hma, Zlema};

// MACD
pub use macd::{Macd, MacdOutput};

// Directional indicators
pub use adx::{Adx, AdxOutput};
pub use aroon::{Aroon, AroonOutput};
pub use parabolic_sar::{ParabolicSar, SarOutput};

// Channels
pub use donchian::{DonchianChannel, DonchianOutput};
pub use keltner::{KeltnerChannel, KeltnerOutput};

// Ichimoku
pub use ichimoku::{Ichimoku, IchimokuOutput};

// Supertrend
pub use supertrend::{Supertrend, SupertrendOutput};

// Support/Resistance
pub use fibonacci::{Fibonacci, FibonacciOutput};
pub use pivot_points::{PivotPoints, PivotPointsOutput, PivotType};

// Statistical
pub use beta::Beta;
