//! Symbol and instrument definitions.
//!
//! Metadata for tradable instruments.

use crate::{Price, Quantity};
use core::fmt;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Asset class enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetClass {
    /// Spot cryptocurrency
    Crypto,
    /// Crypto perpetual futures
    CryptoPerpetual,
    /// Crypto dated futures
    CryptoFutures,
    /// Forex (spot)
    Forex,
    /// Stock/Equity
    Stock,
    /// Stock index
    Index,
    /// Commodity
    Commodity,
    /// CFD
    Cfd,
    /// Options
    Option,
    /// Futures
    Futures,
    /// Bond
    Bond,
}

impl AssetClass {
    /// Is this a leveraged product
    pub const fn is_leveraged(&self) -> bool {
        matches!(
            self,
            Self::CryptoPerpetual | Self::CryptoFutures | Self::Cfd | Self::Futures | Self::Option
        )
    }

    /// Is this crypto-related
    pub const fn is_crypto(&self) -> bool {
        matches!(self, Self::Crypto | Self::CryptoPerpetual | Self::CryptoFutures)
    }

    /// Typical trading hours (24/7 or session-based)
    pub const fn is_24_7(&self) -> bool {
        matches!(self, Self::Crypto | Self::CryptoPerpetual | Self::CryptoFutures | Self::Forex)
    }
}

/// Symbol information and metadata
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Symbol {
    /// Symbol identifier (e.g., "BTCUSDT", "EURUSD")
    pub id: String,
    /// Base currency/asset (e.g., "BTC", "EUR")
    pub base: String,
    /// Quote currency (e.g., "USDT", "USD")
    pub quote: String,
    /// Asset class
    pub asset_class: AssetClass,
    /// Tick size (minimum price increment)
    pub tick_size: Price,
    /// Lot size (minimum quantity increment)
    pub lot_size: Quantity,
    /// Minimum order quantity
    pub min_quantity: Quantity,
    /// Maximum order quantity (0 = unlimited)
    pub max_quantity: Quantity,
    /// Contract multiplier (for futures/options)
    pub contract_multiplier: f64,
    /// Price precision (decimal places)
    pub price_precision: u8,
    /// Quantity precision (decimal places)
    pub quantity_precision: u8,
    /// Is actively trading
    pub is_active: bool,
}

impl Symbol {
    /// Create a new symbol with defaults
    pub fn new(id: impl Into<String>, base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            base: base.into(),
            quote: quote.into(),
            asset_class: AssetClass::Crypto,
            tick_size: 0.01,
            lot_size: 0.001,
            min_quantity: 0.001,
            max_quantity: 0.0,
            contract_multiplier: 1.0,
            price_precision: 2,
            quantity_precision: 3,
            is_active: true,
        }
    }

    /// Create a crypto spot symbol
    pub fn crypto(id: impl Into<String>, base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self::new(id, base, quote).with_asset_class(AssetClass::Crypto)
    }

    /// Create a forex pair
    pub fn forex(id: impl Into<String>, base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self::new(id, base, quote)
            .with_asset_class(AssetClass::Forex)
            .with_tick_size(0.00001)
            .with_lot_size(1000.0)
            .with_price_precision(5)
    }

    /// Create a stock symbol
    pub fn stock(id: impl Into<String>, currency: impl Into<String>) -> Self {
        let id_str: String = id.into();
        Self::new(id_str.clone(), id_str, currency)
            .with_asset_class(AssetClass::Stock)
            .with_lot_size(1.0)
            .with_quantity_precision(0)
    }

    /// Builder: set asset class
    pub fn with_asset_class(mut self, class: AssetClass) -> Self {
        self.asset_class = class;
        self
    }

    /// Builder: set tick size
    pub fn with_tick_size(mut self, tick: Price) -> Self {
        self.tick_size = tick;
        self
    }

    /// Builder: set lot size
    pub fn with_lot_size(mut self, lot: Quantity) -> Self {
        self.lot_size = lot;
        self
    }

    /// Builder: set min quantity
    pub fn with_min_quantity(mut self, min: Quantity) -> Self {
        self.min_quantity = min;
        self
    }

    /// Builder: set max quantity
    pub fn with_max_quantity(mut self, max: Quantity) -> Self {
        self.max_quantity = max;
        self
    }

    /// Builder: set contract multiplier
    pub fn with_multiplier(mut self, mult: f64) -> Self {
        self.contract_multiplier = mult;
        self
    }

    /// Builder: set price precision
    pub fn with_price_precision(mut self, prec: u8) -> Self {
        self.price_precision = prec;
        self
    }

    /// Builder: set quantity precision
    pub fn with_quantity_precision(mut self, prec: u8) -> Self {
        self.quantity_precision = prec;
        self
    }

    /// Round price to tick size
    #[inline]
    pub fn round_price(&self, price: Price) -> Price {
        (price / self.tick_size).round() * self.tick_size
    }

    /// Round quantity to lot size
    #[inline]
    pub fn round_quantity(&self, qty: Quantity) -> Quantity {
        (qty / self.lot_size).floor() * self.lot_size
    }

    /// Calculate pip value (for forex)
    #[inline]
    pub fn pip_value(&self, lot_size: Quantity) -> Price {
        self.tick_size * lot_size * self.contract_multiplier
    }

    /// Calculate notional value
    #[inline]
    pub fn notional(&self, price: Price, quantity: Quantity) -> f64 {
        price * quantity * self.contract_multiplier
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Extended symbol info (for UI/display)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymbolInfo {
    /// The symbol
    pub symbol: Symbol,
    /// Human-readable name
    pub name: String,
    /// Exchange/venue
    pub exchange: String,
    /// Description
    pub description: String,
    /// Logo URL (optional)
    pub logo_url: Option<String>,
}

impl SymbolInfo {
    pub fn new(symbol: Symbol, name: impl Into<String>, exchange: impl Into<String>) -> Self {
        Self {
            symbol,
            name: name.into(),
            exchange: exchange.into(),
            description: String::new(),
            logo_url: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_price() {
        let btc = Symbol::crypto("BTCUSDT", "BTC", "USDT").with_tick_size(0.01);
        assert_eq!(btc.round_price(42123.456), 42123.46);
    }

    #[test]
    fn test_round_quantity() {
        let btc = Symbol::crypto("BTCUSDT", "BTC", "USDT").with_lot_size(0.001);
        assert_eq!(btc.round_quantity(0.12345), 0.123);
    }
}
