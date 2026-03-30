//! Portfolio limits and exposure management.
//!
//! Control overall portfolio risk and exposure.

use loom_core::Price;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::account::{Account, Position};

/// Exposure limit type
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExposureLimit {
    /// Absolute dollar amount
    Absolute(f64),
    /// Percentage of equity
    Percentage(f64),
    /// Multiple of equity (leverage)
    LeverageMultiple(f64),
}

impl ExposureLimit {
    /// Calculate absolute limit from account equity
    pub fn to_absolute(&self, equity: f64) -> f64 {
        match self {
            Self::Absolute(v) => *v,
            Self::Percentage(p) => equity * p,
            Self::LeverageMultiple(m) => equity * m,
        }
    }
}

/// Portfolio limits configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PortfolioLimits {
    /// Maximum total exposure
    pub max_total_exposure: ExposureLimit,
    /// Maximum exposure per symbol
    pub max_symbol_exposure: ExposureLimit,
    /// Maximum exposure per sector/group
    pub max_sector_exposure: ExposureLimit,
    /// Maximum number of positions
    pub max_positions: u32,
    /// Maximum correlated exposure (symbols moving together)
    pub max_correlated_exposure: ExposureLimit,
    /// Maximum long exposure
    pub max_long_exposure: ExposureLimit,
    /// Maximum short exposure
    pub max_short_exposure: ExposureLimit,
    /// Minimum diversification (number of uncorrelated positions)
    pub min_diversification: u32,
}

impl Default for PortfolioLimits {
    fn default() -> Self {
        Self {
            max_total_exposure: ExposureLimit::LeverageMultiple(2.0),
            max_symbol_exposure: ExposureLimit::Percentage(0.20),
            max_sector_exposure: ExposureLimit::Percentage(0.40),
            max_positions: 20,
            max_correlated_exposure: ExposureLimit::Percentage(0.50),
            max_long_exposure: ExposureLimit::LeverageMultiple(2.0),
            max_short_exposure: ExposureLimit::LeverageMultiple(1.0),
            min_diversification: 3,
        }
    }
}

impl PortfolioLimits {
    /// Conservative limits
    pub fn conservative() -> Self {
        Self {
            max_total_exposure: ExposureLimit::Percentage(0.80),
            max_symbol_exposure: ExposureLimit::Percentage(0.10),
            max_sector_exposure: ExposureLimit::Percentage(0.25),
            max_positions: 15,
            max_correlated_exposure: ExposureLimit::Percentage(0.30),
            max_long_exposure: ExposureLimit::Percentage(0.80),
            max_short_exposure: ExposureLimit::Percentage(0.20),
            min_diversification: 5,
        }
    }

    /// Aggressive limits
    pub fn aggressive() -> Self {
        Self {
            max_total_exposure: ExposureLimit::LeverageMultiple(5.0),
            max_symbol_exposure: ExposureLimit::Percentage(0.50),
            max_sector_exposure: ExposureLimit::Percentage(0.75),
            max_positions: 30,
            max_correlated_exposure: ExposureLimit::Percentage(0.75),
            max_long_exposure: ExposureLimit::LeverageMultiple(5.0),
            max_short_exposure: ExposureLimit::LeverageMultiple(3.0),
            min_diversification: 2,
        }
    }

    /// Check if a new position would violate limits
    pub fn can_open_position(
        &self,
        account: &Account,
        position: &Position,
        sector: Option<&str>,
        current_sector_exposure: f64,
    ) -> Result<(), LimitViolation> {
        let equity = account.equity;
        let new_exposure = position.notional();

        // Check total exposure
        let current_total = account.total_exposure();
        let max_total = self.max_total_exposure.to_absolute(equity);
        if current_total + new_exposure > max_total {
            return Err(LimitViolation::TotalExposure {
                current: current_total,
                new: new_exposure,
                limit: max_total,
            });
        }

        // Check symbol exposure
        let max_symbol = self.max_symbol_exposure.to_absolute(equity);
        if new_exposure > max_symbol {
            return Err(LimitViolation::SymbolExposure {
                symbol: position.symbol.clone(),
                exposure: new_exposure,
                limit: max_symbol,
            });
        }

        // Check sector exposure
        if sector.is_some() {
            let max_sector = self.max_sector_exposure.to_absolute(equity);
            if current_sector_exposure + new_exposure > max_sector {
                return Err(LimitViolation::SectorExposure {
                    sector: sector.unwrap_or("unknown").to_string(),
                    exposure: current_sector_exposure + new_exposure,
                    limit: max_sector,
                });
            }
        }

        // Check position count
        if account.positions.len() >= self.max_positions as usize {
            return Err(LimitViolation::PositionCount {
                current: account.positions.len() as u32,
                limit: self.max_positions,
            });
        }

        // Check directional exposure
        let (long_exp, short_exp) = account.positions.iter().fold((0.0, 0.0), |(l, s), p| {
            if p.side.is_long() {
                (l + p.notional(), s)
            } else if p.side.is_short() {
                (l, s + p.notional())
            } else {
                (l, s)
            }
        });

        let max_long = self.max_long_exposure.to_absolute(equity);
        let max_short = self.max_short_exposure.to_absolute(equity);

        if position.side.is_long() && long_exp + new_exposure > max_long {
            return Err(LimitViolation::LongExposure {
                exposure: long_exp + new_exposure,
                limit: max_long,
            });
        }

        if position.side.is_short() && short_exp + new_exposure > max_short {
            return Err(LimitViolation::ShortExposure {
                exposure: short_exp + new_exposure,
                limit: max_short,
            });
        }

        Ok(())
    }

    /// Calculate remaining capacity for a new position
    pub fn remaining_capacity(&self, account: &Account) -> RemainingCapacity {
        let equity = account.equity;
        let current_total = account.total_exposure();
        let max_total = self.max_total_exposure.to_absolute(equity);

        let (long_exp, short_exp) = account.positions.iter().fold((0.0, 0.0), |(l, s), p| {
            if p.side.is_long() {
                (l + p.notional(), s)
            } else {
                (l, s + p.notional())
            }
        });

        RemainingCapacity {
            total: (max_total - current_total).max(0.0),
            per_symbol: self.max_symbol_exposure.to_absolute(equity),
            long: (self.max_long_exposure.to_absolute(equity) - long_exp).max(0.0),
            short: (self.max_short_exposure.to_absolute(equity) - short_exp).max(0.0),
            positions: (self.max_positions as usize).saturating_sub(account.positions.len()) as u32,
        }
    }
}

/// Limit violation type
#[derive(Debug, Clone)]
pub enum LimitViolation {
    TotalExposure {
        current: f64,
        new: f64,
        limit: f64,
    },
    SymbolExposure {
        symbol: String,
        exposure: f64,
        limit: f64,
    },
    SectorExposure {
        sector: String,
        exposure: f64,
        limit: f64,
    },
    PositionCount {
        current: u32,
        limit: u32,
    },
    LongExposure {
        exposure: f64,
        limit: f64,
    },
    ShortExposure {
        exposure: f64,
        limit: f64,
    },
    CorrelatedExposure {
        exposure: f64,
        limit: f64,
    },
}

impl LimitViolation {
    pub fn message(&self) -> String {
        match self {
            Self::TotalExposure { current, new, limit } => {
                format!(
                    "Total exposure would exceed limit: {:.2} + {:.2} > {:.2}",
                    current, new, limit
                )
            }
            Self::SymbolExposure { symbol, exposure, limit } => {
                format!(
                    "Symbol {} exposure {:.2} exceeds limit {:.2}",
                    symbol, exposure, limit
                )
            }
            Self::SectorExposure { sector, exposure, limit } => {
                format!(
                    "Sector {} exposure {:.2} exceeds limit {:.2}",
                    sector, exposure, limit
                )
            }
            Self::PositionCount { current, limit } => {
                format!("Position count {} at limit {}", current, limit)
            }
            Self::LongExposure { exposure, limit } => {
                format!("Long exposure {:.2} exceeds limit {:.2}", exposure, limit)
            }
            Self::ShortExposure { exposure, limit } => {
                format!("Short exposure {:.2} exceeds limit {:.2}", exposure, limit)
            }
            Self::CorrelatedExposure { exposure, limit } => {
                format!(
                    "Correlated exposure {:.2} exceeds limit {:.2}",
                    exposure, limit
                )
            }
        }
    }
}

/// Remaining capacity for new positions
#[derive(Debug, Clone, Copy)]
pub struct RemainingCapacity {
    /// Total remaining exposure
    pub total: f64,
    /// Per-symbol limit
    pub per_symbol: f64,
    /// Remaining long capacity
    pub long: f64,
    /// Remaining short capacity
    pub short: f64,
    /// Remaining position slots
    pub positions: u32,
}

impl RemainingCapacity {
    /// Maximum position size given current capacity
    pub fn max_position_size(&self, is_long: bool) -> f64 {
        let directional = if is_long { self.long } else { self.short };
        self.total.min(self.per_symbol).min(directional)
    }
}

/// Correlation matrix for correlated exposure limits
pub struct CorrelationMatrix {
    symbols: Vec<String>,
    correlations: Vec<Vec<f64>>,
}

impl CorrelationMatrix {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            correlations: Vec::new(),
        }
    }

    /// Add a symbol
    pub fn add_symbol(&mut self, symbol: impl Into<String>) {
        let sym = symbol.into();
        if !self.symbols.contains(&sym) {
            self.symbols.push(sym);
            // Expand matrix
            let n = self.symbols.len();
            for row in &mut self.correlations {
                row.push(0.0);
            }
            self.correlations.push(vec![0.0; n]);
            // Self-correlation is 1
            self.correlations[n - 1][n - 1] = 1.0;
        }
    }

    /// Set correlation between two symbols
    pub fn set_correlation(&mut self, sym1: &str, sym2: &str, correlation: f64) {
        let corr = correlation.clamp(-1.0, 1.0);

        if let (Some(i), Some(j)) = (
            self.symbols.iter().position(|s| s == sym1),
            self.symbols.iter().position(|s| s == sym2),
        ) {
            self.correlations[i][j] = corr;
            self.correlations[j][i] = corr;
        }
    }

    /// Get correlation between two symbols
    pub fn get_correlation(&self, sym1: &str, sym2: &str) -> Option<f64> {
        let i = self.symbols.iter().position(|s| s == sym1)?;
        let j = self.symbols.iter().position(|s| s == sym2)?;
        Some(self.correlations[i][j])
    }

    /// Calculate total correlated exposure
    pub fn correlated_exposure(&self, positions: &[Position], threshold: f64) -> f64 {
        let mut total = 0.0;

        for (i, pos1) in positions.iter().enumerate() {
            for pos2 in positions.iter().skip(i + 1) {
                if let Some(corr) = self.get_correlation(&pos1.symbol, &pos2.symbol) {
                    if corr.abs() >= threshold {
                        // Highly correlated - count as single exposure
                        total += pos1.notional().min(pos2.notional()) * corr.abs();
                    }
                }
            }
        }

        total
    }
}

impl Default for CorrelationMatrix {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::PositionSide;

    #[test]
    fn test_exposure_limit() {
        let limit = ExposureLimit::Percentage(0.20);
        assert_eq!(limit.to_absolute(10000.0), 2000.0);

        let limit = ExposureLimit::LeverageMultiple(2.0);
        assert_eq!(limit.to_absolute(10000.0), 20000.0);
    }

    #[test]
    fn test_can_open_position() {
        let limits = PortfolioLimits::default();
        let account = Account::new(10000.0, "USD");
        let position = Position::new("BTCUSDT", PositionSide::Long, 0.1, 50000.0, 0);

        // Should succeed - well within limits
        assert!(limits.can_open_position(&account, &position, None, 0.0).is_ok());
    }

    #[test]
    fn test_remaining_capacity() {
        let limits = PortfolioLimits::default();
        let account = Account::new(10000.0, "USD").with_leverage(2.0);

        let capacity = limits.remaining_capacity(&account);
        assert_eq!(capacity.total, 20000.0); // 2x leverage
        assert_eq!(capacity.positions, 20);
    }
}
