//! Order types and management.

use loom_core::{Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Order ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

/// Order side
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn opposite(&self) -> Self {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }

    pub fn sign(&self) -> f64 {
        match self {
            Side::Buy => 1.0,
            Side::Sell => -1.0,
        }
    }
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    /// Execute at current market price
    Market,
    /// Execute at specified price or better
    Limit { price: Price },
    /// Trigger at stop price, then execute as market
    Stop { stop_price: Price },
    /// Trigger at stop price, then execute as limit
    StopLimit { stop_price: Price, limit_price: Price },
    /// Take profit order
    TakeProfit { price: Price },
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    /// Order is pending submission
    Pending,
    /// Order is submitted and waiting for fill
    Open,
    /// Order is partially filled
    PartiallyFilled,
    /// Order is completely filled
    Filled,
    /// Order was cancelled
    Cancelled,
    /// Order was rejected
    Rejected,
    /// Order expired
    Expired,
}

/// Time in force
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeInForce {
    /// Good until cancelled
    GTC,
    /// Immediate or cancel
    IOC,
    /// Fill or kill
    FOK,
    /// Good for day
    Day,
    /// Good until time
    GTD(Timestamp),
}

impl Default for TimeInForce {
    fn default() -> Self {
        Self::GTC
    }
}

/// A trading order
#[derive(Debug, Clone)]
pub struct Order {
    pub id: Option<OrderId>,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub status: OrderStatus,
    pub time_in_force: TimeInForce,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    /// Reduce-only flag (for futures)
    pub reduce_only: bool,
    /// Post-only flag (maker only)
    pub post_only: bool,
    /// Client order ID
    pub client_id: Option<String>,
}

impl Order {
    /// Create a new order
    pub fn new(symbol: &str, side: Side, order_type: OrderType, quantity: f64) -> Self {
        Self {
            id: None,
            symbol: symbol.into(),
            side,
            order_type,
            quantity,
            filled_quantity: 0.0,
            status: OrderStatus::Pending,
            time_in_force: TimeInForce::default(),
            created_at: None,
            updated_at: None,
            reduce_only: false,
            post_only: false,
            client_id: None,
        }
    }

    /// Create a market order
    pub fn market(symbol: &str, side: Side, quantity: f64) -> Self {
        Self::new(symbol, side, OrderType::Market, quantity)
    }

    /// Create a limit order
    pub fn limit(symbol: &str, side: Side, quantity: f64, price: Price) -> Self {
        Self::new(symbol, side, OrderType::Limit { price }, quantity)
    }

    /// Create a stop order
    pub fn stop(symbol: &str, side: Side, quantity: f64, stop_price: Price) -> Self {
        Self::new(symbol, side, OrderType::Stop { stop_price }, quantity)
    }

    /// Create a stop-limit order
    pub fn stop_limit(
        symbol: &str,
        side: Side,
        quantity: f64,
        stop_price: Price,
        limit_price: Price,
    ) -> Self {
        Self::new(symbol, side, OrderType::StopLimit { stop_price, limit_price }, quantity)
    }

    /// Create a take-profit order
    pub fn take_profit(symbol: &str, side: Side, quantity: f64, price: Price) -> Self {
        Self::new(symbol, side, OrderType::TakeProfit { price }, quantity)
    }

    /// Set time in force
    pub fn with_tif(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = tif;
        self
    }

    /// Set reduce-only
    pub fn reduce_only(mut self) -> Self {
        self.reduce_only = true;
        self
    }

    /// Set post-only
    pub fn post_only(mut self) -> Self {
        self.post_only = true;
        self
    }

    /// Set client ID
    pub fn with_client_id(mut self, id: &str) -> Self {
        self.client_id = Some(id.into());
        self
    }

    /// Check if order is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, OrderStatus::Open | OrderStatus::PartiallyFilled)
    }

    /// Check if order is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled | OrderStatus::Cancelled | OrderStatus::Rejected | OrderStatus::Expired
        )
    }

    /// Get remaining quantity
    pub fn remaining_quantity(&self) -> f64 {
        self.quantity - self.filled_quantity
    }

    /// Get fill percentage
    pub fn fill_percent(&self) -> f64 {
        if self.quantity == 0.0 {
            0.0
        } else {
            self.filled_quantity / self.quantity * 100.0
        }
    }
}

/// Bracket order (entry + stop loss + take profit)
#[derive(Debug, Clone)]
pub struct BracketOrder {
    pub entry: Order,
    pub stop_loss: Option<Order>,
    pub take_profit: Option<Order>,
}

impl BracketOrder {
    pub fn new(entry: Order) -> Self {
        Self {
            entry,
            stop_loss: None,
            take_profit: None,
        }
    }

    pub fn with_stop_loss(mut self, stop_price: Price) -> Self {
        let sl_side = self.entry.side.opposite();
        self.stop_loss = Some(Order::stop(
            &self.entry.symbol,
            sl_side,
            self.entry.quantity,
            stop_price,
        ).reduce_only());
        self
    }

    pub fn with_take_profit(mut self, price: Price) -> Self {
        let tp_side = self.entry.side.opposite();
        self.take_profit = Some(Order::take_profit(
            &self.entry.symbol,
            tp_side,
            self.entry.quantity,
            price,
        ).reduce_only());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let order = Order::market("BTCUSDT", Side::Buy, 1.0);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.quantity, 1.0);
        assert!(matches!(order.order_type, OrderType::Market));
    }

    #[test]
    fn test_bracket_order() {
        let entry = Order::limit("BTCUSDT", Side::Buy, 1.0, 50000.0);
        let bracket = BracketOrder::new(entry)
            .with_stop_loss(48000.0)
            .with_take_profit(55000.0);

        assert!(bracket.stop_loss.is_some());
        assert!(bracket.take_profit.is_some());

        let sl = bracket.stop_loss.unwrap();
        assert_eq!(sl.side, Side::Sell);
        assert!(sl.reduce_only);
    }
}
