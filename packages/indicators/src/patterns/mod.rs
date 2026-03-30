//! Candlestick Pattern Recognition
//!
//! Functions for detecting common candlestick patterns used in technical analysis.
//! Patterns are categorized as:
//!
//! - **Single candle**: Doji, Hammer, Shooting Star, Marubozu
//! - **Double candle**: Engulfing, Harami, Piercing, Dark Cloud
//! - **Triple candle**: Morning Star, Evening Star, Three White Soldiers

mod single;
mod double;
mod triple;

pub use single::*;
pub use double::*;
pub use triple::*;

use crate::types::Ohlcv;

/// Pattern signal direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternSignal {
    /// Bullish reversal or continuation
    Bullish,
    /// Bearish reversal or continuation
    Bearish,
    /// Neutral/indecision
    Neutral,
}

/// Pattern detection result
#[derive(Debug, Clone)]
pub struct PatternMatch {
    /// Pattern name
    pub name: &'static str,
    /// Signal direction
    pub signal: PatternSignal,
    /// Confidence/strength (0.0 to 1.0)
    pub strength: f64,
}

impl PatternMatch {
    pub fn new(name: &'static str, signal: PatternSignal, strength: f64) -> Self {
        Self { name, signal, strength }
    }
}

/// Detect all patterns in a candle sequence.
///
/// Returns a vector of all detected patterns.
pub fn detect_patterns(candles: &[Ohlcv]) -> Vec<PatternMatch> {
    let mut patterns = Vec::new();

    if candles.is_empty() {
        return patterns;
    }

    let last = &candles[candles.len() - 1];

    // Single candle patterns (on the last candle)
    if is_doji(last) {
        patterns.push(PatternMatch::new("Doji", PatternSignal::Neutral, doji_strength(last)));
    }
    if is_hammer(last) {
        patterns.push(PatternMatch::new("Hammer", PatternSignal::Bullish, hammer_strength(last)));
    }
    if is_inverted_hammer(last) {
        patterns.push(PatternMatch::new("Inverted Hammer", PatternSignal::Bullish, 0.6));
    }
    if is_shooting_star(last) {
        patterns.push(PatternMatch::new("Shooting Star", PatternSignal::Bearish, 0.7));
    }
    if is_marubozu(last) {
        let signal = if last.is_bullish() { PatternSignal::Bullish } else { PatternSignal::Bearish };
        patterns.push(PatternMatch::new("Marubozu", signal, 0.8));
    }
    if is_spinning_top(last) {
        patterns.push(PatternMatch::new("Spinning Top", PatternSignal::Neutral, 0.5));
    }

    // Double candle patterns (need at least 2 candles)
    if candles.len() >= 2 {
        let prev = &candles[candles.len() - 2];

        if is_bullish_engulfing(prev, last) {
            patterns.push(PatternMatch::new("Bullish Engulfing", PatternSignal::Bullish, engulfing_strength(prev, last)));
        }
        if is_bearish_engulfing(prev, last) {
            patterns.push(PatternMatch::new("Bearish Engulfing", PatternSignal::Bearish, engulfing_strength(prev, last)));
        }
        if is_bullish_harami(prev, last) {
            patterns.push(PatternMatch::new("Bullish Harami", PatternSignal::Bullish, 0.6));
        }
        if is_bearish_harami(prev, last) {
            patterns.push(PatternMatch::new("Bearish Harami", PatternSignal::Bearish, 0.6));
        }
        if is_piercing_line(prev, last) {
            patterns.push(PatternMatch::new("Piercing Line", PatternSignal::Bullish, 0.7));
        }
        if is_dark_cloud_cover(prev, last) {
            patterns.push(PatternMatch::new("Dark Cloud Cover", PatternSignal::Bearish, 0.7));
        }
        if is_tweezer_top(prev, last) {
            patterns.push(PatternMatch::new("Tweezer Top", PatternSignal::Bearish, 0.6));
        }
        if is_tweezer_bottom(prev, last) {
            patterns.push(PatternMatch::new("Tweezer Bottom", PatternSignal::Bullish, 0.6));
        }
    }

    // Triple candle patterns (need at least 3 candles)
    if candles.len() >= 3 {
        let first = &candles[candles.len() - 3];
        let second = &candles[candles.len() - 2];
        let third = &candles[candles.len() - 1];

        if is_morning_star(first, second, third) {
            patterns.push(PatternMatch::new("Morning Star", PatternSignal::Bullish, 0.8));
        }
        if is_evening_star(first, second, third) {
            patterns.push(PatternMatch::new("Evening Star", PatternSignal::Bearish, 0.8));
        }
        if is_three_white_soldiers(first, second, third) {
            patterns.push(PatternMatch::new("Three White Soldiers", PatternSignal::Bullish, 0.85));
        }
        if is_three_black_crows(first, second, third) {
            patterns.push(PatternMatch::new("Three Black Crows", PatternSignal::Bearish, 0.85));
        }
        if is_three_inside_up(first, second, third) {
            patterns.push(PatternMatch::new("Three Inside Up", PatternSignal::Bullish, 0.75));
        }
        if is_three_inside_down(first, second, third) {
            patterns.push(PatternMatch::new("Three Inside Down", PatternSignal::Bearish, 0.75));
        }
    }

    patterns
}
