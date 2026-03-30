//! Trading rules and signal composition.
//!
//! Provides building blocks for creating trading strategies.

use loom_core::{Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, boxed::Box};

/// Signal type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalType {
    /// Buy/Long signal
    Buy,
    /// Sell/Short signal
    Sell,
    /// Exit long position
    ExitLong,
    /// Exit short position
    ExitShort,
    /// No signal
    None,
}

impl SignalType {
    pub fn is_entry(&self) -> bool {
        matches!(self, Self::Buy | Self::Sell)
    }

    pub fn is_exit(&self) -> bool {
        matches!(self, Self::ExitLong | Self::ExitShort)
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// Signal strength
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SignalStrength {
    /// Weak signal (single confluence)
    Weak,
    /// Moderate signal (2-3 confluences)
    Moderate,
    /// Strong signal (4+ confluences)
    Strong,
    /// Very strong signal (multiple timeframes align)
    VeryStrong,
}

/// A trading signal
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Signal {
    /// Signal type
    pub signal_type: SignalType,
    /// Signal strength
    pub strength: SignalStrength,
    /// Price at signal
    pub price: Price,
    /// Timestamp
    pub time: Timestamp,
    /// Suggested stop loss
    pub stop_loss: Option<Price>,
    /// Suggested take profit
    pub take_profit: Option<Price>,
    /// Risk/reward ratio
    pub risk_reward: Option<f64>,
    /// Reason/description
    pub reason: String,
    /// Confluences that contributed
    pub confluences: Vec<String>,
}

impl Signal {
    pub fn buy(price: Price, time: Timestamp) -> Self {
        Self {
            signal_type: SignalType::Buy,
            strength: SignalStrength::Weak,
            price,
            time,
            stop_loss: None,
            take_profit: None,
            risk_reward: None,
            reason: String::new(),
            confluences: Vec::new(),
        }
    }

    pub fn sell(price: Price, time: Timestamp) -> Self {
        Self {
            signal_type: SignalType::Sell,
            strength: SignalStrength::Weak,
            price,
            time,
            stop_loss: None,
            take_profit: None,
            risk_reward: None,
            reason: String::new(),
            confluences: Vec::new(),
        }
    }

    pub fn with_stop_loss(mut self, stop: Price) -> Self {
        self.stop_loss = Some(stop);
        self.update_risk_reward();
        self
    }

    pub fn with_take_profit(mut self, target: Price) -> Self {
        self.take_profit = Some(target);
        self.update_risk_reward();
        self
    }

    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = reason.into();
        self
    }

    pub fn add_confluence(mut self, confluence: impl Into<String>) -> Self {
        self.confluences.push(confluence.into());
        self.update_strength();
        self
    }

    fn update_strength(&mut self) {
        self.strength = match self.confluences.len() {
            0..=1 => SignalStrength::Weak,
            2..=3 => SignalStrength::Moderate,
            4..=5 => SignalStrength::Strong,
            _ => SignalStrength::VeryStrong,
        };
    }

    fn update_risk_reward(&mut self) {
        if let (Some(sl), Some(tp)) = (self.stop_loss, self.take_profit) {
            let risk = (self.price - sl).abs();
            let reward = (tp - self.price).abs();
            if risk > 0.0 {
                self.risk_reward = Some(reward / risk);
            }
        }
    }
}

/// Condition for rules
#[derive(Debug, Clone)]
pub enum Condition {
    /// Value above threshold
    Above(f64),
    /// Value below threshold
    Below(f64),
    /// Value between range
    Between(f64, f64),
    /// Crossed above threshold
    CrossedAbove(f64),
    /// Crossed below threshold
    CrossedBelow(f64),
    /// Increasing for N periods
    Rising(usize),
    /// Decreasing for N periods
    Falling(usize),
    /// Custom condition
    Custom(Box<dyn Fn(f64, f64) -> bool + Send + Sync>),
}

impl Condition {
    /// Evaluate condition with current and previous values
    pub fn evaluate(&self, current: f64, previous: f64) -> bool {
        match self {
            Self::Above(threshold) => current > *threshold,
            Self::Below(threshold) => current < *threshold,
            Self::Between(low, high) => current >= *low && current <= *high,
            Self::CrossedAbove(threshold) => previous <= *threshold && current > *threshold,
            Self::CrossedBelow(threshold) => previous >= *threshold && current < *threshold,
            Self::Rising(_) => current > previous, // Simplified; full impl needs history
            Self::Falling(_) => current < previous,
            Self::Custom(f) => f(current, previous),
        }
    }
}

/// A trading rule
#[derive(Debug, Clone)]
pub struct Rule {
    /// Rule name
    pub name: String,
    /// Indicator name
    pub indicator: String,
    /// Condition to check
    pub condition: Condition,
    /// Signal to generate when condition is true
    pub signal: SignalType,
    /// Weight (for combining rules)
    pub weight: f64,
}

impl Rule {
    pub fn new(name: impl Into<String>, indicator: impl Into<String>, condition: Condition) -> Self {
        Self {
            name: name.into(),
            indicator: indicator.into(),
            condition,
            signal: SignalType::None,
            weight: 1.0,
        }
    }

    pub fn generates(mut self, signal: SignalType) -> Self {
        self.signal = signal;
        self
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Evaluate the rule
    pub fn evaluate(&self, current: f64, previous: f64) -> Option<SignalType> {
        if self.condition.evaluate(current, previous) {
            Some(self.signal)
        } else {
            None
        }
    }
}

/// Rule set for combining multiple rules
pub struct RuleSet {
    /// Rules
    pub rules: Vec<Rule>,
    /// Minimum confirmations needed
    pub min_confirmations: usize,
    /// Require all rules to agree
    pub require_all: bool,
}

impl RuleSet {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            min_confirmations: 1,
            require_all: false,
        }
    }

    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_min_confirmations(mut self, min: usize) -> Self {
        self.min_confirmations = min;
        self
    }

    pub fn require_all(mut self) -> Self {
        self.require_all = true;
        self
    }

    /// Evaluate all rules
    pub fn evaluate(&self, values: &[(String, f64, f64)]) -> Option<SignalType> {
        let mut buy_score = 0.0;
        let mut sell_score = 0.0;
        let mut buy_count = 0;
        let mut sell_count = 0;

        for rule in &self.rules {
            // Find matching indicator value
            if let Some((_, current, previous)) = values.iter().find(|(name, _, _)| name == &rule.indicator) {
                if let Some(signal) = rule.evaluate(*current, *previous) {
                    match signal {
                        SignalType::Buy => {
                            buy_score += rule.weight;
                            buy_count += 1;
                        }
                        SignalType::Sell => {
                            sell_score += rule.weight;
                            sell_count += 1;
                        }
                        _ => {}
                    }
                }
            }
        }

        if self.require_all {
            if buy_count == self.rules.len() {
                return Some(SignalType::Buy);
            }
            if sell_count == self.rules.len() {
                return Some(SignalType::Sell);
            }
            return None;
        }

        if buy_count >= self.min_confirmations && buy_score > sell_score {
            Some(SignalType::Buy)
        } else if sell_count >= self.min_confirmations && sell_score > buy_score {
            Some(SignalType::Sell)
        } else {
            None
        }
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Example rule builders
pub mod presets {
    use super::*;

    /// RSI overbought/oversold rule
    pub fn rsi_extremes() -> RuleSet {
        RuleSet::new()
            .add_rule(
                Rule::new("RSI Oversold", "rsi", Condition::CrossedAbove(30.0))
                    .generates(SignalType::Buy)
            )
            .add_rule(
                Rule::new("RSI Overbought", "rsi", Condition::CrossedBelow(70.0))
                    .generates(SignalType::Sell)
            )
    }

    /// Moving average crossover
    pub fn ma_crossover() -> RuleSet {
        RuleSet::new()
            .add_rule(
                Rule::new("Fast above Slow", "ma_cross", Condition::CrossedAbove(0.0))
                    .generates(SignalType::Buy)
            )
            .add_rule(
                Rule::new("Fast below Slow", "ma_cross", Condition::CrossedBelow(0.0))
                    .generates(SignalType::Sell)
            )
    }

    /// MACD signal line crossover
    pub fn macd_signal() -> RuleSet {
        RuleSet::new()
            .add_rule(
                Rule::new("MACD above Signal", "macd_hist", Condition::CrossedAbove(0.0))
                    .generates(SignalType::Buy)
            )
            .add_rule(
                Rule::new("MACD below Signal", "macd_hist", Condition::CrossedBelow(0.0))
                    .generates(SignalType::Sell)
            )
    }

    /// Trend following (price above/below MA)
    pub fn trend_filter() -> RuleSet {
        RuleSet::new()
            .add_rule(
                Rule::new("Bullish Trend", "price_vs_ma", Condition::Above(0.0))
                    .generates(SignalType::Buy)
                    .with_weight(0.5)
            )
            .add_rule(
                Rule::new("Bearish Trend", "price_vs_ma", Condition::Below(0.0))
                    .generates(SignalType::Sell)
                    .with_weight(0.5)
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_condition_evaluate() {
        let above = Condition::Above(50.0);
        assert!(above.evaluate(60.0, 40.0));
        assert!(!above.evaluate(40.0, 60.0));

        let crossed = Condition::CrossedAbove(50.0);
        assert!(crossed.evaluate(51.0, 49.0));
        assert!(!crossed.evaluate(51.0, 52.0));
    }

    #[test]
    fn test_rule_evaluate() {
        let rule = Rule::new("Test", "rsi", Condition::CrossedAbove(30.0))
            .generates(SignalType::Buy);

        assert_eq!(rule.evaluate(31.0, 29.0), Some(SignalType::Buy));
        assert_eq!(rule.evaluate(31.0, 31.0), None);
    }

    #[test]
    fn test_signal_risk_reward() {
        let signal = Signal::buy(100.0, 0)
            .with_stop_loss(95.0)
            .with_take_profit(115.0);

        assert_eq!(signal.risk_reward, Some(3.0)); // 15 reward / 5 risk
    }
}
