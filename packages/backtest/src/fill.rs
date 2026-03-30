//! Fill models for order execution simulation.

use loom_core::{Candle, Price, Timestamp};
use crate::order::{Order, OrderType, Side};

/// A fill represents order execution
#[derive(Debug, Clone)]
pub struct Fill {
    pub price: Price,
    pub quantity: f64,
    pub commission: f64,
    pub slippage: f64,
    pub timestamp: Timestamp,
}

impl Fill {
    pub fn new(price: Price, quantity: f64, commission: f64, slippage: f64, timestamp: Timestamp) -> Self {
        Self { price, quantity, commission, slippage, timestamp }
    }

    /// Total cost including commission
    pub fn total_cost(&self) -> f64 {
        self.price * self.quantity + self.commission
    }

    /// Effective price after slippage
    pub fn effective_price(&self) -> f64 {
        self.price + self.slippage
    }
}

/// Fill model determines how orders are executed
#[derive(Debug, Clone)]
pub enum FillModel {
    /// Instant fill at specified price
    Instant,
    /// Fill at open of next bar
    NextBarOpen,
    /// Fill at close of current bar
    Close,
    /// Fill at VWAP of current bar
    Vwap,
    /// Realistic fill with slippage and partial fills
    Realistic {
        /// Slippage as percentage (0.001 = 0.1%)
        slippage_pct: f64,
        /// Commission per unit
        commission_per_unit: f64,
        /// Commission as percentage
        commission_pct: f64,
        /// Max fill percentage of bar volume
        max_volume_pct: f64,
    },
    /// Custom fill logic
    Custom(Box<dyn FillSimulator + Send + Sync>),
}

impl Default for FillModel {
    fn default() -> Self {
        Self::NextBarOpen
    }
}

/// Trait for custom fill simulation
pub trait FillSimulator {
    fn simulate_fill(&self, order: &Order, candle: &Candle) -> Option<Fill>;
}

impl FillModel {
    /// Create a realistic fill model with defaults
    pub fn realistic() -> Self {
        Self::Realistic {
            slippage_pct: 0.0001, // 0.01%
            commission_per_unit: 0.0,
            commission_pct: 0.001, // 0.1%
            max_volume_pct: 0.1,   // Max 10% of bar volume
        }
    }

    /// Simulate fill for an order
    pub fn fill(&self, order: &Order, current_bar: &Candle, next_bar: Option<&Candle>) -> Option<Fill> {
        match self {
            FillModel::Instant => self.fill_instant(order, current_bar),
            FillModel::NextBarOpen => next_bar.map(|bar| self.fill_at_price(order, bar.open, bar.time)),
            FillModel::Close => Some(self.fill_at_price(order, current_bar.close, current_bar.time)),
            FillModel::Vwap => Some(self.fill_at_vwap(order, current_bar)),
            FillModel::Realistic { slippage_pct, commission_per_unit, commission_pct, max_volume_pct } => {
                self.fill_realistic(order, current_bar, *slippage_pct, *commission_per_unit, *commission_pct, *max_volume_pct)
            }
            FillModel::Custom(simulator) => simulator.simulate_fill(order, current_bar),
        }
    }

    fn fill_instant(&self, order: &Order, candle: &Candle) -> Option<Fill> {
        let price = match order.order_type {
            OrderType::Market => candle.close,
            OrderType::Limit { price } => {
                // Check if limit price is reachable
                match order.side {
                    Side::Buy if candle.low <= price => price.min(candle.close),
                    Side::Sell if candle.high >= price => price.max(candle.close),
                    _ => return None,
                }
            }
            OrderType::Stop { stop_price } => {
                // Check if stop is triggered
                match order.side {
                    Side::Buy if candle.high >= stop_price => candle.close,
                    Side::Sell if candle.low <= stop_price => candle.close,
                    _ => return None,
                }
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                // Check if stop is triggered and limit is reachable
                match order.side {
                    Side::Buy if candle.high >= stop_price && candle.low <= limit_price => {
                        limit_price.min(candle.close)
                    }
                    Side::Sell if candle.low <= stop_price && candle.high >= limit_price => {
                        limit_price.max(candle.close)
                    }
                    _ => return None,
                }
            }
            OrderType::TakeProfit { price } => {
                match order.side {
                    Side::Buy if candle.low <= price => price,
                    Side::Sell if candle.high >= price => price,
                    _ => return None,
                }
            }
        };

        Some(Fill::new(price, order.quantity, 0.0, 0.0, candle.time))
    }

    fn fill_at_price(&self, order: &Order, price: Price, time: Timestamp) -> Fill {
        Fill::new(price, order.quantity, 0.0, 0.0, time)
    }

    fn fill_at_vwap(&self, order: &Order, candle: &Candle) -> Fill {
        // Approximate VWAP as typical price
        let vwap = (candle.high + candle.low + candle.close) / 3.0;
        Fill::new(vwap, order.quantity, 0.0, 0.0, candle.time)
    }

    fn fill_realistic(
        &self,
        order: &Order,
        candle: &Candle,
        slippage_pct: f64,
        commission_per_unit: f64,
        commission_pct: f64,
        max_volume_pct: f64,
    ) -> Option<Fill> {
        // Check order type validity
        let base_price = match order.order_type {
            OrderType::Market => candle.close,
            OrderType::Limit { price } => {
                match order.side {
                    Side::Buy if candle.low <= price => price.min(candle.close),
                    Side::Sell if candle.high >= price => price.max(candle.close),
                    _ => return None,
                }
            }
            OrderType::Stop { stop_price } => {
                match order.side {
                    Side::Buy if candle.high >= stop_price => stop_price.max(candle.close),
                    Side::Sell if candle.low <= stop_price => stop_price.min(candle.close),
                    _ => return None,
                }
            }
            _ => candle.close,
        };

        // Calculate slippage
        let slippage = match order.side {
            Side::Buy => base_price * slippage_pct,
            Side::Sell => -base_price * slippage_pct,
        };

        let fill_price = base_price + slippage;

        // Calculate quantity based on volume constraint
        let max_quantity = candle.volume * max_volume_pct;
        let fill_quantity = order.remaining_quantity().min(max_quantity);

        if fill_quantity <= 0.0 {
            return None;
        }

        // Calculate commission
        let commission = fill_quantity * commission_per_unit + fill_quantity * fill_price * commission_pct;

        Some(Fill::new(fill_price, fill_quantity, commission, slippage, candle.time))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_candle() -> Candle {
        Candle::new(1700000000000, 100.0, 105.0, 98.0, 103.0, 10000.0)
    }

    #[test]
    fn test_market_fill() {
        let model = FillModel::Instant;
        let order = Order::market("TEST", Side::Buy, 10.0);
        let candle = test_candle();

        let fill = model.fill(&order, &candle, None).unwrap();
        assert_eq!(fill.price, 103.0); // Close price
        assert_eq!(fill.quantity, 10.0);
    }

    #[test]
    fn test_limit_fill() {
        let model = FillModel::Instant;
        let candle = test_candle();

        // Buy limit below low - should not fill
        let order = Order::limit("TEST", Side::Buy, 10.0, 97.0);
        assert!(model.fill(&order, &candle, None).is_none());

        // Buy limit above low - should fill
        let order = Order::limit("TEST", Side::Buy, 10.0, 99.0);
        let fill = model.fill(&order, &candle, None).unwrap();
        assert_eq!(fill.price, 99.0);
    }

    #[test]
    fn test_realistic_fill() {
        let model = FillModel::Realistic {
            slippage_pct: 0.001,
            commission_per_unit: 0.0,
            commission_pct: 0.001,
            max_volume_pct: 0.1,
        };

        let order = Order::market("TEST", Side::Buy, 100.0);
        let candle = test_candle();

        let fill = model.fill(&order, &candle, None).unwrap();

        // Should have slippage
        assert!(fill.slippage > 0.0);
        // Should have commission
        assert!(fill.commission > 0.0);
        // Quantity limited by volume
        assert!(fill.quantity <= candle.volume * 0.1);
    }
}
