//! Position tracking and management.

use loom_core::{Price, Timestamp};
use crate::order::Side;
use crate::fill::Fill;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, collections::BTreeMap};
#[cfg(feature = "std")]
use std::collections::HashMap;

/// Position side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionSide {
    Long,
    Short,
    Flat,
}

/// A trading position
#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: PositionSide,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub commission_paid: f64,
    pub entry_time: Option<Timestamp>,
    pub last_update: Option<Timestamp>,
}

impl Position {
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.into(),
            side: PositionSide::Flat,
            quantity: 0.0,
            entry_price: 0.0,
            current_price: 0.0,
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            commission_paid: 0.0,
            entry_time: None,
            last_update: None,
        }
    }

    /// Apply a fill to the position
    pub fn apply_fill(&mut self, fill: &Fill, side: Side) {
        let trade_qty = match side {
            Side::Buy => fill.quantity,
            Side::Sell => -fill.quantity,
        };

        let old_qty = self.quantity;
        let new_qty = old_qty + trade_qty;

        // Update commission
        self.commission_paid += fill.commission;

        // Calculate PnL if reducing position
        if old_qty != 0.0 && old_qty.signum() != new_qty.signum() {
            // Closing or reversing position
            let close_qty = old_qty.abs().min(trade_qty.abs());
            let pnl = if old_qty > 0.0 {
                // Closing long
                (fill.price - self.entry_price) * close_qty
            } else {
                // Closing short
                (self.entry_price - fill.price) * close_qty
            };
            self.realized_pnl += pnl - fill.commission;
        }

        // Update position
        if new_qty.abs() < 1e-10 {
            // Position closed
            self.side = PositionSide::Flat;
            self.quantity = 0.0;
            self.entry_price = 0.0;
            self.unrealized_pnl = 0.0;
        } else if (old_qty == 0.0) || (old_qty.signum() != new_qty.signum()) {
            // New position or reversal
            self.side = if new_qty > 0.0 { PositionSide::Long } else { PositionSide::Short };
            self.quantity = new_qty.abs();
            self.entry_price = fill.price;
            self.entry_time = Some(fill.timestamp);
        } else {
            // Adding to position - average entry price
            let total_cost = self.entry_price * old_qty.abs() + fill.price * trade_qty.abs();
            self.quantity = new_qty.abs();
            self.entry_price = total_cost / self.quantity;
        }

        self.current_price = fill.price;
        self.last_update = Some(fill.timestamp);
        self.update_unrealized_pnl();
    }

    /// Update current price and unrealized PnL
    pub fn update_price(&mut self, price: Price, time: Timestamp) {
        self.current_price = price;
        self.last_update = Some(time);
        self.update_unrealized_pnl();
    }

    fn update_unrealized_pnl(&mut self) {
        if self.quantity == 0.0 {
            self.unrealized_pnl = 0.0;
            return;
        }

        self.unrealized_pnl = match self.side {
            PositionSide::Long => (self.current_price - self.entry_price) * self.quantity,
            PositionSide::Short => (self.entry_price - self.current_price) * self.quantity,
            PositionSide::Flat => 0.0,
        };
    }

    /// Total PnL (realized + unrealized)
    pub fn total_pnl(&self) -> f64 {
        self.realized_pnl + self.unrealized_pnl
    }

    /// Position value at current price
    pub fn market_value(&self) -> f64 {
        self.current_price * self.quantity
    }

    /// Position value at entry price
    pub fn cost_basis(&self) -> f64 {
        self.entry_price * self.quantity
    }

    /// Return percentage
    pub fn return_pct(&self) -> f64 {
        if self.entry_price == 0.0 {
            return 0.0;
        }
        match self.side {
            PositionSide::Long => (self.current_price - self.entry_price) / self.entry_price * 100.0,
            PositionSide::Short => (self.entry_price - self.current_price) / self.entry_price * 100.0,
            PositionSide::Flat => 0.0,
        }
    }

    /// Is this position flat?
    pub fn is_flat(&self) -> bool {
        self.side == PositionSide::Flat
    }

    /// Is this a long position?
    pub fn is_long(&self) -> bool {
        self.side == PositionSide::Long
    }

    /// Is this a short position?
    pub fn is_short(&self) -> bool {
        self.side == PositionSide::Short
    }
}

/// Manages multiple positions
pub struct PositionManager {
    #[cfg(feature = "std")]
    positions: HashMap<String, Position>,
    #[cfg(not(feature = "std"))]
    positions: BTreeMap<String, Position>,
}

impl PositionManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "std")]
            positions: HashMap::new(),
            #[cfg(not(feature = "std"))]
            positions: BTreeMap::new(),
        }
    }

    /// Get or create position for symbol
    pub fn get_mut(&mut self, symbol: &str) -> &mut Position {
        #[cfg(feature = "std")]
        {
            self.positions.entry(symbol.to_string())
                .or_insert_with(|| Position::new(symbol))
        }
        #[cfg(not(feature = "std"))]
        {
            if !self.positions.contains_key(symbol) {
                self.positions.insert(symbol.to_string(), Position::new(symbol));
            }
            self.positions.get_mut(symbol).unwrap()
        }
    }

    /// Get position if exists
    pub fn get(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// Check if any position exists
    pub fn has_position(&self, symbol: &str) -> bool {
        self.positions.get(symbol)
            .map(|p| !p.is_flat())
            .unwrap_or(false)
    }

    /// Get all positions
    pub fn all(&self) -> impl Iterator<Item = &Position> {
        self.positions.values()
    }

    /// Get open positions
    pub fn open(&self) -> impl Iterator<Item = &Position> {
        self.positions.values().filter(|p| !p.is_flat())
    }

    /// Total unrealized PnL
    pub fn total_unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl).sum()
    }

    /// Total realized PnL
    pub fn total_realized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.realized_pnl).sum()
    }

    /// Total commission paid
    pub fn total_commission(&self) -> f64 {
        self.positions.values().map(|p| p.commission_paid).sum()
    }

    /// Update all positions with new prices
    pub fn update_prices(&mut self, prices: &[(String, Price)], time: Timestamp) {
        for (symbol, price) in prices {
            if let Some(pos) = self.positions.get_mut(symbol) {
                pos.update_price(*price, time);
            }
        }
    }
}

impl Default for PositionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_long() {
        let mut pos = Position::new("BTCUSDT");

        // Open long
        let fill = Fill::new(50000.0, 1.0, 5.0, 0.0, 1700000000000);
        pos.apply_fill(&fill, Side::Buy);

        assert!(pos.is_long());
        assert_eq!(pos.quantity, 1.0);
        assert_eq!(pos.entry_price, 50000.0);

        // Price goes up
        pos.update_price(55000.0, 1700000001000);
        assert_eq!(pos.unrealized_pnl, 5000.0);

        // Close position
        let fill = Fill::new(55000.0, 1.0, 5.0, 0.0, 1700000002000);
        pos.apply_fill(&fill, Side::Sell);

        assert!(pos.is_flat());
        assert!(pos.realized_pnl > 0.0);
    }

    #[test]
    fn test_position_short() {
        let mut pos = Position::new("BTCUSDT");

        // Open short
        let fill = Fill::new(50000.0, 1.0, 5.0, 0.0, 1700000000000);
        pos.apply_fill(&fill, Side::Sell);

        assert!(pos.is_short());

        // Price goes down (profit for short)
        pos.update_price(45000.0, 1700000001000);
        assert_eq!(pos.unrealized_pnl, 5000.0);
    }

    #[test]
    fn test_position_averaging() {
        let mut pos = Position::new("BTCUSDT");

        // Buy at 100
        let fill = Fill::new(100.0, 10.0, 0.0, 0.0, 1700000000000);
        pos.apply_fill(&fill, Side::Buy);

        // Buy more at 110
        let fill = Fill::new(110.0, 10.0, 0.0, 0.0, 1700000001000);
        pos.apply_fill(&fill, Side::Buy);

        assert_eq!(pos.quantity, 20.0);
        assert_eq!(pos.entry_price, 105.0); // Average
    }
}
