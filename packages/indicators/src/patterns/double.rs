//! Double candlestick patterns

use crate::types::Ohlcv;
use libm::fabs;

/// Tolerance for price comparisons
const TOLERANCE: f64 = 0.001;

/// Check if two candles form a Bullish Engulfing pattern.
///
/// A bullish engulfing occurs when a small bearish candle is followed
/// by a larger bullish candle that completely "engulfs" the first.
#[inline]
pub fn is_bullish_engulfing(first: &Ohlcv, second: &Ohlcv) -> bool {
    // First candle must be bearish
    if first.is_bullish() {
        return false;
    }

    // Second candle must be bullish
    if !second.is_bullish() {
        return false;
    }

    // Second body engulfs first body
    second.open < first.close && second.close > first.open
}

/// Check if two candles form a Bearish Engulfing pattern.
///
/// A bearish engulfing occurs when a small bullish candle is followed
/// by a larger bearish candle that completely "engulfs" the first.
#[inline]
pub fn is_bearish_engulfing(first: &Ohlcv, second: &Ohlcv) -> bool {
    // First candle must be bullish
    if !first.is_bullish() {
        return false;
    }

    // Second candle must be bearish
    if second.is_bullish() {
        return false;
    }

    // Second body engulfs first body
    second.open > first.close && second.close < first.open
}

/// Calculate the strength of an engulfing pattern.
#[inline]
pub fn engulfing_strength(first: &Ohlcv, second: &Ohlcv) -> f64 {
    let first_body = fabs(first.close - first.open);
    let second_body = fabs(second.close - second.open);

    if first_body < TOLERANCE {
        return 0.8;
    }

    // Strength based on how much bigger the second body is
    (second_body / first_body / 3.0).min(1.0)
}

/// Check if two candles form a Bullish Harami pattern.
///
/// A bullish harami is a small bullish candle contained within
/// the body of a larger preceding bearish candle.
#[inline]
pub fn is_bullish_harami(first: &Ohlcv, second: &Ohlcv) -> bool {
    // First candle must be bearish and larger
    if first.is_bullish() {
        return false;
    }

    // Second candle must be bullish
    if !second.is_bullish() {
        return false;
    }

    // Second body is contained within first body
    second.open > first.close &&
    second.close < first.open &&
    fabs(second.close - second.open) < fabs(first.close - first.open)
}

/// Check if two candles form a Bearish Harami pattern.
///
/// A bearish harami is a small bearish candle contained within
/// the body of a larger preceding bullish candle.
#[inline]
pub fn is_bearish_harami(first: &Ohlcv, second: &Ohlcv) -> bool {
    // First candle must be bullish and larger
    if !first.is_bullish() {
        return false;
    }

    // Second candle must be bearish
    if second.is_bullish() {
        return false;
    }

    // Second body is contained within first body
    second.open < first.close &&
    second.close > first.open &&
    fabs(second.close - second.open) < fabs(first.close - first.open)
}

/// Check if two candles form a Piercing Line pattern.
///
/// A bullish reversal pattern where:
/// 1. First candle is bearish
/// 2. Second candle opens below first's low
/// 3. Second candle closes above the midpoint of first's body
#[inline]
pub fn is_piercing_line(first: &Ohlcv, second: &Ohlcv) -> bool {
    // First candle must be bearish
    if first.is_bullish() {
        return false;
    }

    // Second candle must be bullish
    if !second.is_bullish() {
        return false;
    }

    let first_midpoint = (first.open + first.close) / 2.0;

    // Opens below first's low (or close for less strict)
    // Closes above first's midpoint but below first's open
    second.open < first.close &&
    second.close > first_midpoint &&
    second.close < first.open
}

/// Check if two candles form a Dark Cloud Cover pattern.
///
/// A bearish reversal pattern (opposite of piercing line):
/// 1. First candle is bullish
/// 2. Second candle opens above first's high
/// 3. Second candle closes below the midpoint of first's body
#[inline]
pub fn is_dark_cloud_cover(first: &Ohlcv, second: &Ohlcv) -> bool {
    // First candle must be bullish
    if !first.is_bullish() {
        return false;
    }

    // Second candle must be bearish
    if second.is_bullish() {
        return false;
    }

    let first_midpoint = (first.open + first.close) / 2.0;

    // Opens above first's close (or high for strict)
    // Closes below first's midpoint but above first's open
    second.open > first.close &&
    second.close < first_midpoint &&
    second.close > first.open
}

/// Check if two candles form a Tweezer Top pattern.
///
/// A bearish reversal where two candles have equal or nearly equal highs.
#[inline]
pub fn is_tweezer_top(first: &Ohlcv, second: &Ohlcv) -> bool {
    let range = (first.high - first.low).max(second.high - second.low);
    let tolerance = range * 0.02;

    // First should be bullish, second bearish (ideal)
    let direction_change = first.is_bullish() && !second.is_bullish();

    // Highs are approximately equal
    let equal_highs = fabs(first.high - second.high) < tolerance;

    direction_change && equal_highs
}

/// Check if two candles form a Tweezer Bottom pattern.
///
/// A bullish reversal where two candles have equal or nearly equal lows.
#[inline]
pub fn is_tweezer_bottom(first: &Ohlcv, second: &Ohlcv) -> bool {
    let range = (first.high - first.low).max(second.high - second.low);
    let tolerance = range * 0.02;

    // First should be bearish, second bullish (ideal)
    let direction_change = !first.is_bullish() && second.is_bullish();

    // Lows are approximately equal
    let equal_lows = fabs(first.low - second.low) < tolerance;

    direction_change && equal_lows
}

/// Check if two candles form a Kicking pattern (bullish).
///
/// Strong pattern: bearish marubozu followed by bullish marubozu with gap up.
#[inline]
pub fn is_bullish_kicking(first: &Ohlcv, second: &Ohlcv) -> bool {
    use super::single::is_marubozu;

    // Both must be marubozu
    if !is_marubozu(first) || !is_marubozu(second) {
        return false;
    }

    // First bearish, second bullish
    if first.is_bullish() || !second.is_bullish() {
        return false;
    }

    // Gap up (second opens above first's open)
    second.open > first.open
}

/// Check if two candles form a Kicking pattern (bearish).
///
/// Strong pattern: bullish marubozu followed by bearish marubozu with gap down.
#[inline]
pub fn is_bearish_kicking(first: &Ohlcv, second: &Ohlcv) -> bool {
    use super::single::is_marubozu;

    // Both must be marubozu
    if !is_marubozu(first) || !is_marubozu(second) {
        return false;
    }

    // First bullish, second bearish
    if !first.is_bullish() || second.is_bullish() {
        return false;
    }

    // Gap down (second opens below first's open)
    second.open < first.open
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(open: f64, high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(open, high, low, close, 1000.0)
    }

    #[test]
    fn test_bullish_engulfing() {
        // Small bearish followed by larger bullish
        let first = make_candle(102.0, 103.0, 100.0, 101.0);  // Bearish
        let second = make_candle(100.0, 105.0, 99.0, 104.0);  // Bullish engulfs
        assert!(is_bullish_engulfing(&first, &second));
    }

    #[test]
    fn test_bearish_engulfing() {
        // Small bullish followed by larger bearish
        let first = make_candle(101.0, 103.0, 100.0, 102.0);  // Bullish
        let second = make_candle(104.0, 105.0, 99.0, 100.0);  // Bearish engulfs
        assert!(is_bearish_engulfing(&first, &second));
    }

    #[test]
    fn test_bullish_harami() {
        // Large bearish followed by small bullish inside
        let first = make_candle(110.0, 111.0, 99.0, 100.0);   // Large bearish
        let second = make_candle(102.0, 106.0, 101.0, 105.0); // Small bullish inside
        assert!(is_bullish_harami(&first, &second));
    }

    #[test]
    fn test_piercing_line() {
        // Bearish followed by bullish that closes above midpoint
        let first = make_candle(110.0, 111.0, 99.0, 100.0);   // Bearish (mid = 105)
        let second = make_candle(98.0, 108.0, 97.0, 107.0);   // Opens below, closes above mid
        assert!(is_piercing_line(&first, &second));
    }

    #[test]
    fn test_tweezer_bottom() {
        // Two candles with equal lows
        let first = make_candle(105.0, 106.0, 100.0, 101.0);  // Bearish
        let second = make_candle(102.0, 107.0, 100.0, 106.0); // Bullish, same low
        assert!(is_tweezer_bottom(&first, &second));
    }
}
