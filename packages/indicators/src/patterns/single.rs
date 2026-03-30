//! Single candlestick patterns

use crate::types::Ohlcv;
use libm::fabs;

/// Tolerance for considering values "equal"
const TOLERANCE: f64 = 0.001;

/// Check if a candle is a Doji (open ≈ close).
///
/// Doji signals indecision between bulls and bears.
/// The smaller the body relative to the range, the stronger the doji.
#[inline]
pub fn is_doji(candle: &Ohlcv) -> bool {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return true; // Very small range = essentially a doji
    }

    // Body is less than 10% of range
    body / range < 0.1
}

/// Calculate the strength of a doji pattern (0.0 to 1.0).
#[inline]
pub fn doji_strength(candle: &Ohlcv) -> f64 {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return 1.0;
    }

    1.0 - (body / range).min(1.0)
}

/// Check if a candle is a Hammer (bullish reversal).
///
/// Characteristics:
/// - Small body at the top
/// - Long lower shadow (at least 2x the body)
/// - Little or no upper shadow
#[inline]
pub fn is_hammer(candle: &Ohlcv) -> bool {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return false;
    }

    let upper_shadow = candle.high - candle.open.max(candle.close);
    let lower_shadow = candle.open.min(candle.close) - candle.low;

    // Body in upper third
    let body_position = (candle.open.min(candle.close) - candle.low) / range;

    // Long lower shadow, small upper shadow, body near top
    lower_shadow > body * 2.0 &&
    upper_shadow < body * 0.5 &&
    body_position > 0.6
}

/// Calculate the strength of a hammer pattern.
#[inline]
pub fn hammer_strength(candle: &Ohlcv) -> f64 {
    let body = fabs(candle.close - candle.open);
    let lower_shadow = candle.open.min(candle.close) - candle.low;

    if body < TOLERANCE {
        return 0.7; // Doji-like hammer
    }

    // Strength based on shadow-to-body ratio
    (lower_shadow / body / 4.0).min(1.0)
}

/// Check if a candle is an Inverted Hammer (bullish reversal).
///
/// Like a hammer but inverted - long upper shadow, small body at bottom.
#[inline]
pub fn is_inverted_hammer(candle: &Ohlcv) -> bool {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return false;
    }

    let upper_shadow = candle.high - candle.open.max(candle.close);
    let lower_shadow = candle.open.min(candle.close) - candle.low;

    // Body in lower third
    let body_position = (candle.open.max(candle.close) - candle.low) / range;

    // Long upper shadow, small lower shadow, body near bottom
    upper_shadow > body * 2.0 &&
    lower_shadow < body * 0.5 &&
    body_position < 0.4
}

/// Check if a candle is a Shooting Star (bearish reversal).
///
/// Same shape as inverted hammer but appears in uptrends.
/// Small body at bottom, long upper shadow.
#[inline]
pub fn is_shooting_star(candle: &Ohlcv) -> bool {
    // Structure is same as inverted hammer
    // Context (uptrend) should be checked separately
    is_inverted_hammer(candle)
}

/// Check if a candle is a Hanging Man (bearish reversal).
///
/// Same shape as hammer but appears in uptrends.
#[inline]
pub fn is_hanging_man(candle: &Ohlcv) -> bool {
    // Structure is same as hammer
    // Context (uptrend) should be checked separately
    is_hammer(candle)
}

/// Check if a candle is a Marubozu (strong momentum).
///
/// A candle with no shadows (or very small shadows).
/// Bullish marubozu: Close = High, Open = Low
/// Bearish marubozu: Open = High, Close = Low
#[inline]
pub fn is_marubozu(candle: &Ohlcv) -> bool {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return false;
    }

    let upper_shadow = candle.high - candle.open.max(candle.close);
    let lower_shadow = candle.open.min(candle.close) - candle.low;

    // Body is at least 95% of range
    body / range > 0.95 ||
    (upper_shadow / range < 0.03 && lower_shadow / range < 0.03)
}

/// Check if a candle is a Spinning Top (indecision).
///
/// Small body with roughly equal upper and lower shadows.
#[inline]
pub fn is_spinning_top(candle: &Ohlcv) -> bool {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return false;
    }

    let upper_shadow = candle.high - candle.open.max(candle.close);
    let lower_shadow = candle.open.min(candle.close) - candle.low;

    // Small body (less than 30% of range)
    let small_body = body / range < 0.3;

    // Shadows are roughly equal (ratio between 0.5 and 2.0)
    let shadow_ratio = if lower_shadow > TOLERANCE {
        upper_shadow / lower_shadow
    } else {
        0.0
    };

    let balanced_shadows = shadow_ratio > 0.5 && shadow_ratio < 2.0;

    small_body && balanced_shadows
}

/// Check if a candle is a Dragonfly Doji.
///
/// Doji with long lower shadow, no upper shadow.
/// Open, close, and high are at the same level.
#[inline]
pub fn is_dragonfly_doji(candle: &Ohlcv) -> bool {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return false;
    }

    let upper_shadow = candle.high - candle.open.max(candle.close);
    let lower_shadow = candle.open.min(candle.close) - candle.low;

    // Very small body
    let doji = body / range < 0.1;

    // Long lower shadow, almost no upper shadow
    doji && lower_shadow > range * 0.6 && upper_shadow < range * 0.1
}

/// Check if a candle is a Gravestone Doji.
///
/// Doji with long upper shadow, no lower shadow.
/// Open, close, and low are at the same level.
#[inline]
pub fn is_gravestone_doji(candle: &Ohlcv) -> bool {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return false;
    }

    let upper_shadow = candle.high - candle.open.max(candle.close);
    let lower_shadow = candle.open.min(candle.close) - candle.low;

    // Very small body
    let doji = body / range < 0.1;

    // Long upper shadow, almost no lower shadow
    doji && upper_shadow > range * 0.6 && lower_shadow < range * 0.1
}

/// Check if a candle is a Long-Legged Doji.
///
/// Doji with long shadows on both sides.
#[inline]
pub fn is_long_legged_doji(candle: &Ohlcv) -> bool {
    let body = fabs(candle.close - candle.open);
    let range = candle.high - candle.low;

    if range < TOLERANCE {
        return false;
    }

    let upper_shadow = candle.high - candle.open.max(candle.close);
    let lower_shadow = candle.open.min(candle.close) - candle.low;

    // Very small body (doji)
    let doji = body / range < 0.1;

    // Both shadows are significant
    doji && upper_shadow > range * 0.3 && lower_shadow > range * 0.3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(open: f64, high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(open, high, low, close, 1000.0)
    }

    #[test]
    fn test_doji() {
        // Perfect doji
        let doji = make_candle(100.0, 105.0, 95.0, 100.0);
        assert!(is_doji(&doji));

        // Not a doji (big body)
        let not_doji = make_candle(95.0, 105.0, 95.0, 105.0);
        assert!(!is_doji(&not_doji));
    }

    #[test]
    fn test_hammer() {
        // Hammer: small body at top, long lower shadow
        let hammer = make_candle(99.0, 100.0, 90.0, 100.0);
        assert!(is_hammer(&hammer));

        // Not a hammer (body at bottom)
        let not_hammer = make_candle(90.0, 100.0, 90.0, 91.0);
        assert!(!is_hammer(&not_hammer));
    }

    #[test]
    fn test_marubozu() {
        // Bullish marubozu
        let marubozu = make_candle(100.0, 110.0, 100.0, 110.0);
        assert!(is_marubozu(&marubozu));

        // Not marubozu (has shadows)
        let not_marubozu = make_candle(100.0, 115.0, 95.0, 110.0);
        assert!(!is_marubozu(&not_marubozu));
    }

    #[test]
    fn test_spinning_top() {
        // Spinning top: small body, balanced shadows
        let spinning = make_candle(99.0, 105.0, 95.0, 101.0);
        assert!(is_spinning_top(&spinning));
    }
}
