//! Account and position management.
//!
//! Track balances, positions, and margin.

use loom_core::{Price, Quantity, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Position side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PositionSide {
    Long,
    Short,
    Flat,
}

impl PositionSide {
    pub fn is_long(&self) -> bool {
        matches!(self, Self::Long)
    }

    pub fn is_short(&self) -> bool {
        matches!(self, Self::Short)
    }

    pub fn is_flat(&self) -> bool {
        matches!(self, Self::Flat)
    }
}

/// A trading position
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Position {
    /// Symbol
    pub symbol: String,
    /// Position side
    pub side: PositionSide,
    /// Position size (quantity)
    pub size: Quantity,
    /// Average entry price
    pub entry_price: Price,
    /// Current market price
    pub market_price: Price,
    /// Unrealized P&L
    pub unrealized_pnl: f64,
    /// Realized P&L (from partial closes)
    pub realized_pnl: f64,
    /// Entry timestamp
    pub opened_at: Timestamp,
    /// Stop loss price
    pub stop_loss: Option<Price>,
    /// Take profit price
    pub take_profit: Option<Price>,
    /// Margin used
    pub margin_used: f64,
}

impl Position {
    /// Create a new position
    pub fn new(
        symbol: impl Into<String>,
        side: PositionSide,
        size: Quantity,
        entry_price: Price,
        opened_at: Timestamp,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            side,
            size,
            entry_price,
            market_price: entry_price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            opened_at,
            stop_loss: None,
            take_profit: None,
            margin_used: 0.0,
        }
    }

    /// Update market price and recalculate P&L
    pub fn update_price(&mut self, price: Price) {
        self.market_price = price;
        self.unrealized_pnl = self.calculate_pnl(price);
    }

    /// Calculate P&L at a given price
    pub fn calculate_pnl(&self, price: Price) -> f64 {
        let price_diff = price - self.entry_price;
        let multiplier = match self.side {
            PositionSide::Long => 1.0,
            PositionSide::Short => -1.0,
            PositionSide::Flat => 0.0,
        };
        price_diff * self.size * multiplier
    }

    /// Check if stop loss is hit
    pub fn is_stop_hit(&self) -> bool {
        if let Some(stop) = self.stop_loss {
            match self.side {
                PositionSide::Long => self.market_price <= stop,
                PositionSide::Short => self.market_price >= stop,
                PositionSide::Flat => false,
            }
        } else {
            false
        }
    }

    /// Check if take profit is hit
    pub fn is_target_hit(&self) -> bool {
        if let Some(target) = self.take_profit {
            match self.side {
                PositionSide::Long => self.market_price >= target,
                PositionSide::Short => self.market_price <= target,
                PositionSide::Flat => false,
            }
        } else {
            false
        }
    }

    /// Notional value
    pub fn notional(&self) -> f64 {
        self.size * self.market_price
    }

    /// Risk per position (distance to stop)
    pub fn risk(&self) -> Option<f64> {
        self.stop_loss.map(|stop| {
            (self.entry_price - stop).abs() * self.size
        })
    }

    /// R-multiple (current P&L / initial risk)
    pub fn r_multiple(&self) -> Option<f64> {
        self.risk().map(|risk| {
            if risk > 0.0 {
                self.unrealized_pnl / risk
            } else {
                0.0
            }
        })
    }
}

/// Trading account
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Account {
    /// Account balance (cash)
    pub balance: f64,
    /// Account equity (balance + unrealized P&L)
    pub equity: f64,
    /// Used margin
    pub margin_used: f64,
    /// Available margin
    pub margin_available: f64,
    /// Account currency
    pub currency: String,
    /// Maximum leverage allowed
    pub max_leverage: f64,
    /// Current positions
    pub positions: Vec<Position>,
    /// Total realized P&L
    pub total_realized_pnl: f64,
    /// Total trades
    pub total_trades: u64,
    /// Winning trades
    pub winning_trades: u64,
    /// Peak equity (for drawdown)
    pub peak_equity: f64,
}

impl Account {
    /// Create a new account
    pub fn new(balance: f64, currency: impl Into<String>) -> Self {
        Self {
            balance,
            equity: balance,
            margin_used: 0.0,
            margin_available: balance,
            currency: currency.into(),
            max_leverage: 1.0,
            positions: Vec::new(),
            total_realized_pnl: 0.0,
            total_trades: 0,
            winning_trades: 0,
            peak_equity: balance,
        }
    }

    /// Set maximum leverage
    pub fn with_leverage(mut self, leverage: f64) -> Self {
        self.max_leverage = leverage;
        self.margin_available = self.balance * leverage;
        self
    }

    /// Update equity based on current positions
    pub fn update_equity(&mut self) {
        let unrealized: f64 = self.positions.iter().map(|p| p.unrealized_pnl).sum();
        self.equity = self.balance + unrealized;

        if self.equity > self.peak_equity {
            self.peak_equity = self.equity;
        }

        self.margin_used = self.positions.iter().map(|p| p.margin_used).sum();
        self.margin_available = (self.equity * self.max_leverage) - self.margin_used;
    }

    /// Open a new position
    pub fn open_position(&mut self, position: Position) -> Result<(), &'static str> {
        let required_margin = position.notional() / self.max_leverage;

        if required_margin > self.margin_available {
            return Err("Insufficient margin");
        }

        let mut pos = position;
        pos.margin_used = required_margin;

        self.positions.push(pos);
        self.update_equity();
        self.total_trades += 1;

        Ok(())
    }

    /// Close a position by symbol
    pub fn close_position(&mut self, symbol: &str, price: Price) -> Option<f64> {
        if let Some(idx) = self.positions.iter().position(|p| p.symbol == symbol) {
            let mut pos = self.positions.remove(idx);
            pos.update_price(price);

            let pnl = pos.unrealized_pnl + pos.realized_pnl;
            self.balance += pnl;
            self.total_realized_pnl += pnl;

            if pnl > 0.0 {
                self.winning_trades += 1;
            }

            self.update_equity();
            Some(pnl)
        } else {
            None
        }
    }

    /// Update all position prices
    pub fn update_prices(&mut self, prices: &[(String, Price)]) {
        for (symbol, price) in prices {
            if let Some(pos) = self.positions.iter_mut().find(|p| &p.symbol == symbol) {
                pos.update_price(*price);
            }
        }
        self.update_equity();
    }

    /// Get position by symbol
    pub fn position(&self, symbol: &str) -> Option<&Position> {
        self.positions.iter().find(|p| p.symbol == symbol)
    }

    /// Check if we have an open position for symbol
    pub fn has_position(&self, symbol: &str) -> bool {
        self.positions.iter().any(|p| p.symbol == symbol)
    }

    /// Total exposure (sum of notional values)
    pub fn total_exposure(&self) -> f64 {
        self.positions.iter().map(|p| p.notional()).sum()
    }

    /// Current drawdown from peak
    pub fn drawdown(&self) -> f64 {
        if self.peak_equity > 0.0 {
            (self.peak_equity - self.equity) / self.peak_equity
        } else {
            0.0
        }
    }

    /// Win rate
    pub fn win_rate(&self) -> f64 {
        if self.total_trades > 0 {
            self.winning_trades as f64 / self.total_trades as f64
        } else {
            0.0
        }
    }

    /// Current leverage ratio
    pub fn leverage_ratio(&self) -> f64 {
        if self.equity > 0.0 {
            self.total_exposure() / self.equity
        } else {
            0.0
        }
    }

    /// Margin level (equity / margin used)
    pub fn margin_level(&self) -> f64 {
        if self.margin_used > 0.0 {
            self.equity / self.margin_used
        } else {
            f64::INFINITY
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_pnl() {
        let mut pos = Position::new("BTCUSDT", PositionSide::Long, 1.0, 50000.0, 0);
        pos.update_price(51000.0);

        assert_eq!(pos.unrealized_pnl, 1000.0);
    }

    #[test]
    fn test_position_short_pnl() {
        let mut pos = Position::new("BTCUSDT", PositionSide::Short, 1.0, 50000.0, 0);
        pos.update_price(49000.0);

        assert_eq!(pos.unrealized_pnl, 1000.0); // Profit on short
    }

    #[test]
    fn test_account_operations() {
        let mut account = Account::new(10000.0, "USD").with_leverage(10.0);

        let pos = Position::new("BTCUSDT", PositionSide::Long, 0.1, 50000.0, 0);
        account.open_position(pos).unwrap();

        assert_eq!(account.positions.len(), 1);
        assert!(account.margin_used > 0.0);

        // Close at profit
        let pnl = account.close_position("BTCUSDT", 52000.0).unwrap();
        assert_eq!(pnl, 200.0); // 0.1 * 2000

        assert_eq!(account.positions.len(), 0);
        assert_eq!(account.balance, 10200.0);
    }

    #[test]
    fn test_stop_loss_check() {
        let mut pos = Position::new("BTCUSDT", PositionSide::Long, 1.0, 50000.0, 0);
        pos.stop_loss = Some(48000.0);

        pos.update_price(49000.0);
        assert!(!pos.is_stop_hit());

        pos.update_price(47000.0);
        assert!(pos.is_stop_hit());
    }
}
