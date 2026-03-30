//! Kill switch for risk management.
//!
//! Automatic trading halt when limits are exceeded.

use loom_core::Timestamp;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Reason for kill switch activation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum KillReason {
    /// Maximum daily loss reached
    MaxDailyLoss,
    /// Maximum drawdown reached
    MaxDrawdown,
    /// Maximum consecutive losses
    ConsecutiveLosses,
    /// Maximum trades per day
    MaxTrades,
    /// Manual kill
    Manual,
    /// System error
    SystemError,
    /// Unusual market conditions
    MarketCondition,
}

/// Kill switch configuration
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KillSwitchConfig {
    /// Maximum daily loss (as fraction of starting balance, e.g., 0.05 = 5%)
    pub max_daily_loss: f64,
    /// Maximum drawdown from peak (e.g., 0.10 = 10%)
    pub max_drawdown: f64,
    /// Maximum consecutive losing trades
    pub max_consecutive_losses: u32,
    /// Maximum trades per day
    pub max_trades_per_day: u32,
    /// Cool-down period in seconds after trigger
    pub cooldown_seconds: u64,
    /// Allow manual override
    pub allow_override: bool,
}

impl Default for KillSwitchConfig {
    fn default() -> Self {
        Self {
            max_daily_loss: 0.05,        // 5% daily loss
            max_drawdown: 0.10,          // 10% max drawdown
            max_consecutive_losses: 5,    // 5 consecutive losses
            max_trades_per_day: 50,      // 50 trades per day
            cooldown_seconds: 3600,      // 1 hour cooldown
            allow_override: false,
        }
    }
}

impl KillSwitchConfig {
    pub fn conservative() -> Self {
        Self {
            max_daily_loss: 0.02,
            max_drawdown: 0.05,
            max_consecutive_losses: 3,
            max_trades_per_day: 20,
            cooldown_seconds: 7200,
            allow_override: false,
        }
    }

    pub fn moderate() -> Self {
        Self::default()
    }

    pub fn aggressive() -> Self {
        Self {
            max_daily_loss: 0.10,
            max_drawdown: 0.20,
            max_consecutive_losses: 10,
            max_trades_per_day: 100,
            cooldown_seconds: 1800,
            allow_override: true,
        }
    }
}

/// Kill switch state
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KillSwitch {
    config: KillSwitchConfig,
    /// Is the kill switch active
    active: bool,
    /// Reason for activation
    reason: Option<KillReason>,
    /// Time of activation
    activated_at: Option<Timestamp>,
    /// Starting balance for the day
    day_start_balance: f64,
    /// Peak balance (for drawdown calculation)
    peak_balance: f64,
    /// Current balance
    current_balance: f64,
    /// Daily P&L
    daily_pnl: f64,
    /// Number of trades today
    trades_today: u32,
    /// Consecutive losses
    consecutive_losses: u32,
    /// Trade history for the day
    trade_results: Vec<f64>,
}

impl KillSwitch {
    pub fn new(config: KillSwitchConfig, starting_balance: f64) -> Self {
        Self {
            config,
            active: false,
            reason: None,
            activated_at: None,
            day_start_balance: starting_balance,
            peak_balance: starting_balance,
            current_balance: starting_balance,
            daily_pnl: 0.0,
            trades_today: 0,
            consecutive_losses: 0,
            trade_results: Vec::new(),
        }
    }

    /// Check if trading is allowed
    pub fn can_trade(&self) -> bool {
        !self.active
    }

    /// Check if kill switch is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Get reason for activation
    pub fn reason(&self) -> Option<KillReason> {
        self.reason
    }

    /// Update with a new trade result
    pub fn on_trade(&mut self, pnl: f64, timestamp: Timestamp) -> bool {
        self.trades_today += 1;
        self.daily_pnl += pnl;
        self.current_balance += pnl;
        self.trade_results.push(pnl);

        // Update peak balance
        if self.current_balance > self.peak_balance {
            self.peak_balance = self.current_balance;
        }

        // Update consecutive losses
        if pnl < 0.0 {
            self.consecutive_losses += 1;
        } else if pnl > 0.0 {
            self.consecutive_losses = 0;
        }

        // Check all conditions
        self.check_conditions(timestamp)
    }

    /// Check all kill switch conditions
    fn check_conditions(&mut self, timestamp: Timestamp) -> bool {
        // Check daily loss
        let daily_loss_pct = -self.daily_pnl / self.day_start_balance;
        if daily_loss_pct >= self.config.max_daily_loss {
            self.activate(KillReason::MaxDailyLoss, timestamp);
            return true;
        }

        // Check drawdown
        let drawdown = (self.peak_balance - self.current_balance) / self.peak_balance;
        if drawdown >= self.config.max_drawdown {
            self.activate(KillReason::MaxDrawdown, timestamp);
            return true;
        }

        // Check consecutive losses
        if self.consecutive_losses >= self.config.max_consecutive_losses {
            self.activate(KillReason::ConsecutiveLosses, timestamp);
            return true;
        }

        // Check max trades
        if self.trades_today >= self.config.max_trades_per_day {
            self.activate(KillReason::MaxTrades, timestamp);
            return true;
        }

        false
    }

    /// Activate the kill switch
    fn activate(&mut self, reason: KillReason, timestamp: Timestamp) {
        self.active = true;
        self.reason = Some(reason);
        self.activated_at = Some(timestamp);
    }

    /// Manually activate kill switch
    pub fn manual_kill(&mut self, timestamp: Timestamp) {
        self.activate(KillReason::Manual, timestamp);
    }

    /// Check if cooldown has passed and deactivate
    pub fn check_cooldown(&mut self, current_time: Timestamp) -> bool {
        if !self.active {
            return false;
        }

        if let Some(activated) = self.activated_at {
            let elapsed = (current_time - activated) as u64 / 1000; // Convert to seconds
            if elapsed >= self.config.cooldown_seconds {
                self.deactivate();
                return true;
            }
        }

        false
    }

    /// Deactivate kill switch (use with caution)
    pub fn deactivate(&mut self) {
        if self.config.allow_override || self.reason == Some(KillReason::Manual) {
            self.active = false;
            self.reason = None;
            self.activated_at = None;
        }
    }

    /// Reset for a new trading day
    pub fn new_day(&mut self, starting_balance: f64) {
        self.day_start_balance = starting_balance;
        self.daily_pnl = 0.0;
        self.trades_today = 0;
        self.trade_results.clear();

        // Only reset if not in drawdown kill
        if self.reason != Some(KillReason::MaxDrawdown) {
            self.active = false;
            self.reason = None;
            self.activated_at = None;
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> KillSwitchStats {
        let daily_loss_pct = if self.day_start_balance > 0.0 {
            -self.daily_pnl / self.day_start_balance
        } else {
            0.0
        };

        let drawdown = if self.peak_balance > 0.0 {
            (self.peak_balance - self.current_balance) / self.peak_balance
        } else {
            0.0
        };

        KillSwitchStats {
            daily_pnl: self.daily_pnl,
            daily_loss_pct,
            drawdown,
            trades_today: self.trades_today,
            consecutive_losses: self.consecutive_losses,
            is_active: self.active,
            reason: self.reason,
            distance_to_daily_limit: self.config.max_daily_loss - daily_loss_pct,
            distance_to_drawdown_limit: self.config.max_drawdown - drawdown,
        }
    }
}

/// Kill switch statistics
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KillSwitchStats {
    pub daily_pnl: f64,
    pub daily_loss_pct: f64,
    pub drawdown: f64,
    pub trades_today: u32,
    pub consecutive_losses: u32,
    pub is_active: bool,
    pub reason: Option<KillReason>,
    pub distance_to_daily_limit: f64,
    pub distance_to_drawdown_limit: f64,
}

impl KillSwitchStats {
    /// Get warning level (0-3: none, low, medium, high)
    pub fn warning_level(&self) -> u8 {
        let daily_danger = 1.0 - (self.distance_to_daily_limit / 0.05).clamp(0.0, 1.0);
        let dd_danger = 1.0 - (self.distance_to_drawdown_limit / 0.10).clamp(0.0, 1.0);

        let max_danger = daily_danger.max(dd_danger);

        if max_danger < 0.5 {
            0 // None
        } else if max_danger < 0.75 {
            1 // Low
        } else if max_danger < 0.9 {
            2 // Medium
        } else {
            3 // High
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daily_loss_kill() {
        let config = KillSwitchConfig {
            max_daily_loss: 0.05,
            ..Default::default()
        };
        let mut kill = KillSwitch::new(config, 10000.0);

        // Lose 4% - should be fine
        assert!(!kill.on_trade(-400.0, 1000));
        assert!(kill.can_trade());

        // Lose another 2% - should trigger (total 6%)
        assert!(kill.on_trade(-200.0, 2000));
        assert!(!kill.can_trade());
        assert_eq!(kill.reason(), Some(KillReason::MaxDailyLoss));
    }

    #[test]
    fn test_consecutive_losses() {
        let config = KillSwitchConfig {
            max_consecutive_losses: 3,
            max_daily_loss: 1.0, // High to not trigger
            ..Default::default()
        };
        let mut kill = KillSwitch::new(config, 10000.0);

        kill.on_trade(-100.0, 1000);
        kill.on_trade(-100.0, 2000);
        assert!(kill.can_trade());

        kill.on_trade(-100.0, 3000); // 3rd loss
        assert!(!kill.can_trade());
        assert_eq!(kill.reason(), Some(KillReason::ConsecutiveLosses));
    }

    #[test]
    fn test_winning_resets_consecutive() {
        let config = KillSwitchConfig {
            max_consecutive_losses: 3,
            max_daily_loss: 1.0,
            ..Default::default()
        };
        let mut kill = KillSwitch::new(config, 10000.0);

        kill.on_trade(-100.0, 1000);
        kill.on_trade(-100.0, 2000);
        kill.on_trade(200.0, 3000); // Win resets counter
        kill.on_trade(-100.0, 4000);
        kill.on_trade(-100.0, 5000);

        assert!(kill.can_trade()); // Only 2 consecutive losses
    }
}
