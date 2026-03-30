//! Position sizing algorithms.
//!
//! Various methods for calculating optimal position sizes.

use loom_core::Price;

/// Position sizing trait
pub trait PositionSizer {
    /// Calculate position size
    ///
    /// # Arguments
    /// * `balance` - Account balance
    /// * `entry` - Entry price
    /// * `stop` - Stop loss price
    ///
    /// # Returns
    /// Position size (quantity to trade)
    fn calculate_size(&self, balance: f64, entry: Price, stop: Price) -> f64;

    /// Get name of the sizing method
    fn name(&self) -> &'static str;
}

/// Fixed fractional position sizing (most common)
///
/// Risks a fixed percentage of account on each trade.
///
/// Formula: Size = (Balance * RiskPercent) / RiskPerUnit
/// Where RiskPerUnit = |Entry - Stop|
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FixedFractional {
    /// Risk percentage per trade (0.01 = 1%)
    pub risk_percent: f64,
    /// Maximum position size (0 = unlimited)
    pub max_size: f64,
}

impl FixedFractional {
    pub fn new(risk_percent: f64) -> Self {
        Self {
            risk_percent: risk_percent.clamp(0.001, 1.0),
            max_size: 0.0,
        }
    }

    pub fn with_max_size(mut self, max: f64) -> Self {
        self.max_size = max;
        self
    }

    /// Common presets
    pub fn conservative() -> Self {
        Self::new(0.005) // 0.5%
    }

    pub fn moderate() -> Self {
        Self::new(0.01) // 1%
    }

    pub fn aggressive() -> Self {
        Self::new(0.02) // 2%
    }
}

impl PositionSizer for FixedFractional {
    fn calculate_size(&self, balance: f64, entry: Price, stop: Price) -> f64 {
        let risk_per_unit = (entry - stop).abs();
        if risk_per_unit == 0.0 {
            return 0.0;
        }

        let risk_amount = balance * self.risk_percent;
        let mut size = risk_amount / risk_per_unit;

        if self.max_size > 0.0 {
            size = size.min(self.max_size);
        }

        size
    }

    fn name(&self) -> &'static str {
        "Fixed Fractional"
    }
}

/// Volatility targeting position sizing
///
/// Adjusts position size to target a specific portfolio volatility.
/// Uses ATR or standard deviation as volatility measure.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VolatilityTarget {
    /// Target annualized volatility (0.15 = 15%)
    pub target_volatility: f64,
    /// Trading days per year for annualization
    pub trading_days: f64,
}

impl VolatilityTarget {
    pub fn new(target_volatility: f64) -> Self {
        Self {
            target_volatility,
            trading_days: 252.0,
        }
    }

    /// Calculate position size based on asset volatility
    ///
    /// # Arguments
    /// * `balance` - Account balance
    /// * `price` - Current price
    /// * `volatility` - Daily volatility (as decimal, e.g., 0.02 = 2%)
    pub fn calculate_with_volatility(&self, balance: f64, price: Price, daily_volatility: f64) -> f64 {
        if daily_volatility == 0.0 || price == 0.0 {
            return 0.0;
        }

        // Annualize volatility
        let annual_vol = daily_volatility * (self.trading_days).sqrt();

        // Target dollar volatility
        let target_dollar_vol = balance * self.target_volatility;

        // Daily dollar volatility per unit
        let daily_dollar_vol_per_unit = price * daily_volatility;

        // Annualized per unit
        let annual_dollar_vol_per_unit = daily_dollar_vol_per_unit * (self.trading_days).sqrt();

        // Position size
        target_dollar_vol / annual_dollar_vol_per_unit
    }
}

impl PositionSizer for VolatilityTarget {
    fn calculate_size(&self, balance: f64, entry: Price, stop: Price) -> f64 {
        // Estimate volatility from stop distance
        let risk_per_unit = (entry - stop).abs();
        let daily_volatility = risk_per_unit / entry; // Rough approximation

        self.calculate_with_volatility(balance, entry, daily_volatility)
    }

    fn name(&self) -> &'static str {
        "Volatility Target"
    }
}

/// ATR-based position sizing
///
/// Uses Average True Range to determine position size.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AtrBased {
    /// Risk percentage per trade
    pub risk_percent: f64,
    /// ATR multiplier for stop distance
    pub atr_multiplier: f64,
}

impl AtrBased {
    pub fn new(risk_percent: f64, atr_multiplier: f64) -> Self {
        Self {
            risk_percent,
            atr_multiplier,
        }
    }

    /// Calculate size using ATR value
    pub fn calculate_with_atr(&self, balance: f64, price: Price, atr: f64) -> f64 {
        if atr == 0.0 {
            return 0.0;
        }

        let stop_distance = atr * self.atr_multiplier;
        let risk_amount = balance * self.risk_percent;

        risk_amount / stop_distance
    }
}

impl PositionSizer for AtrBased {
    fn calculate_size(&self, balance: f64, entry: Price, stop: Price) -> f64 {
        let stop_distance = (entry - stop).abs();
        let implied_atr = stop_distance / self.atr_multiplier;
        self.calculate_with_atr(balance, entry, implied_atr)
    }

    fn name(&self) -> &'static str {
        "ATR Based"
    }
}

/// Kelly Criterion position sizing
///
/// Optimal sizing based on win rate and average win/loss ratio.
/// Formula: f* = (bp - q) / b
/// Where: b = odds (avg win / avg loss), p = win probability, q = 1 - p
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KellyCriterion {
    /// Win probability (0.0 to 1.0)
    pub win_rate: f64,
    /// Average win / average loss ratio
    pub profit_ratio: f64,
    /// Fraction of full Kelly to use (0.25 = quarter Kelly)
    pub kelly_fraction: f64,
    /// Maximum allocation
    pub max_allocation: f64,
}

impl KellyCriterion {
    pub fn new(win_rate: f64, profit_ratio: f64) -> Self {
        Self {
            win_rate: win_rate.clamp(0.0, 1.0),
            profit_ratio: profit_ratio.max(0.0),
            kelly_fraction: 0.5, // Half Kelly is common
            max_allocation: 0.25, // Max 25% of account
        }
    }

    pub fn with_fraction(mut self, fraction: f64) -> Self {
        self.kelly_fraction = fraction.clamp(0.0, 1.0);
        self
    }

    /// Calculate Kelly percentage
    pub fn kelly_percent(&self) -> f64 {
        let b = self.profit_ratio;
        let p = self.win_rate;
        let q = 1.0 - p;

        let kelly = (b * p - q) / b;

        // Apply fraction and clamp
        (kelly * self.kelly_fraction).clamp(0.0, self.max_allocation)
    }

    /// Calculate optimal allocation
    pub fn optimal_allocation(&self, balance: f64) -> f64 {
        balance * self.kelly_percent()
    }
}

impl PositionSizer for KellyCriterion {
    fn calculate_size(&self, balance: f64, entry: Price, stop: Price) -> f64 {
        let risk_per_unit = (entry - stop).abs();
        if risk_per_unit == 0.0 {
            return 0.0;
        }

        let allocation = self.optimal_allocation(balance);
        allocation / entry // Convert to position size
    }

    fn name(&self) -> &'static str {
        "Kelly Criterion"
    }
}

/// Optimal F position sizing (Ralph Vince)
///
/// Similar to Kelly but uses historical trade data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptimalF {
    /// Historical trade P&Ls
    trades: Vec<f64>,
    /// Calculated optimal f
    optimal_f: f64,
}

impl OptimalF {
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
            optimal_f: 0.0,
        }
    }

    /// Add a trade result
    pub fn add_trade(&mut self, pnl: f64) {
        self.trades.push(pnl);
        self.recalculate();
    }

    /// Calculate optimal f from trades
    fn recalculate(&mut self) {
        if self.trades.is_empty() {
            self.optimal_f = 0.0;
            return;
        }

        // Find largest loss
        let largest_loss = self.trades
            .iter()
            .filter(|&&x| x < 0.0)
            .fold(0.0_f64, |acc, &x| acc.min(x))
            .abs();

        if largest_loss == 0.0 {
            self.optimal_f = 0.25; // Default if no losses
            return;
        }

        // Search for optimal f that maximizes TWR (Terminal Wealth Relative)
        let mut best_f = 0.01;
        let mut best_twr = 0.0;

        for f_pct in 1..=100 {
            let f = f_pct as f64 / 100.0;
            let twr = self.calculate_twr(f, largest_loss);

            if twr > best_twr {
                best_twr = twr;
                best_f = f;
            }
        }

        self.optimal_f = best_f * 0.5; // Use half for safety
    }

    fn calculate_twr(&self, f: f64, largest_loss: f64) -> f64 {
        let mut twr = 1.0;

        for &trade in &self.trades {
            let hpp = f * (trade / largest_loss);
            if 1.0 + hpp <= 0.0 {
                return 0.0; // Blew up
            }
            twr *= 1.0 + hpp;
        }

        twr
    }

    /// Get current optimal f value
    pub fn optimal_f(&self) -> f64 {
        self.optimal_f
    }
}

impl Default for OptimalF {
    fn default() -> Self {
        Self::new()
    }
}

impl PositionSizer for OptimalF {
    fn calculate_size(&self, balance: f64, entry: Price, stop: Price) -> f64 {
        let risk_per_unit = (entry - stop).abs();
        if risk_per_unit == 0.0 || self.optimal_f == 0.0 {
            return 0.0;
        }

        let risk_amount = balance * self.optimal_f;
        risk_amount / risk_per_unit
    }

    fn name(&self) -> &'static str {
        "Optimal F"
    }
}

/// Position size calculator with multiple methods
pub struct SizeCalculator {
    pub method: Box<dyn PositionSizer>,
}

impl SizeCalculator {
    pub fn fixed_fractional(risk_percent: f64) -> Self {
        Self {
            method: Box::new(FixedFractional::new(risk_percent)),
        }
    }

    pub fn volatility_target(target: f64) -> Self {
        Self {
            method: Box::new(VolatilityTarget::new(target)),
        }
    }

    pub fn atr_based(risk_percent: f64, atr_mult: f64) -> Self {
        Self {
            method: Box::new(AtrBased::new(risk_percent, atr_mult)),
        }
    }

    pub fn kelly(win_rate: f64, profit_ratio: f64) -> Self {
        Self {
            method: Box::new(KellyCriterion::new(win_rate, profit_ratio)),
        }
    }

    pub fn calculate(&self, balance: f64, entry: Price, stop: Price) -> f64 {
        self.method.calculate_size(balance, entry, stop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixed_fractional() {
        let sizer = FixedFractional::new(0.02); // 2% risk
        let size = sizer.calculate_size(10000.0, 100.0, 95.0);

        // Risk = 10000 * 0.02 = 200
        // Risk per unit = 100 - 95 = 5
        // Size = 200 / 5 = 40
        assert!((size - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_kelly_criterion() {
        let kelly = KellyCriterion::new(0.6, 1.5); // 60% win rate, 1.5:1 R:R
        let pct = kelly.kelly_percent();

        // Full Kelly: (1.5 * 0.6 - 0.4) / 1.5 = 0.333...
        // Half Kelly: 0.1666...
        assert!(pct > 0.15 && pct < 0.20);
    }

    #[test]
    fn test_atr_based() {
        let sizer = AtrBased::new(0.02, 2.0);
        let size = sizer.calculate_with_atr(10000.0, 100.0, 5.0);

        // Stop distance = 5 * 2 = 10
        // Risk = 10000 * 0.02 = 200
        // Size = 200 / 10 = 20
        assert!((size - 20.0).abs() < 0.001);
    }
}
