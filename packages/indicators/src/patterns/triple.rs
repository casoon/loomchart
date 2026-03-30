//! Triple candlestick patterns

use crate::types::Ohlcv;
use libm::fabs;
use super::single::is_doji;

/// Check if three candles form a Morning Star pattern.
///
/// A bullish reversal pattern:
/// 1. First candle: Large bearish
/// 2. Second candle: Small body (star) that gaps below first
/// 3. Third candle: Large bullish that closes above first's midpoint
#[inline]
pub fn is_morning_star(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    // First must be bearish with significant body
    if first.is_bullish() || fabs(first.close - first.open) < (first.high - first.low) * 0.3 {
        return false;
    }

    // Second must have small body
    let second_body = fabs(second.close - second.open);
    let second_range = second.high - second.low;
    if second_range > 0.0 && second_body / second_range > 0.3 {
        return false;
    }

    // Third must be bullish with significant body
    if !third.is_bullish() || fabs(third.close - third.open) < (third.high - third.low) * 0.3 {
        return false;
    }

    // Gap: second's high should be below or near first's close
    let gap_down = second.high <= first.close;

    // Third closes above first's midpoint
    let first_midpoint = (first.open + first.close) / 2.0;
    let strong_close = third.close > first_midpoint;

    gap_down && strong_close
}

/// Check if three candles form an Evening Star pattern.
///
/// A bearish reversal pattern (opposite of morning star):
/// 1. First candle: Large bullish
/// 2. Second candle: Small body (star) that gaps above first
/// 3. Third candle: Large bearish that closes below first's midpoint
#[inline]
pub fn is_evening_star(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    // First must be bullish with significant body
    if !first.is_bullish() || fabs(first.close - first.open) < (first.high - first.low) * 0.3 {
        return false;
    }

    // Second must have small body
    let second_body = fabs(second.close - second.open);
    let second_range = second.high - second.low;
    if second_range > 0.0 && second_body / second_range > 0.3 {
        return false;
    }

    // Third must be bearish with significant body
    if third.is_bullish() || fabs(third.close - third.open) < (third.high - third.low) * 0.3 {
        return false;
    }

    // Gap: second's low should be above or near first's close
    let gap_up = second.low >= first.close;

    // Third closes below first's midpoint
    let first_midpoint = (first.open + first.close) / 2.0;
    let strong_close = third.close < first_midpoint;

    gap_up && strong_close
}

/// Check if three candles form a Morning Doji Star pattern.
///
/// Like morning star but the middle candle is specifically a doji.
#[inline]
pub fn is_morning_doji_star(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    is_doji(second) && is_morning_star(first, second, third)
}

/// Check if three candles form an Evening Doji Star pattern.
///
/// Like evening star but the middle candle is specifically a doji.
#[inline]
pub fn is_evening_doji_star(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    is_doji(second) && is_evening_star(first, second, third)
}

/// Check if three candles form a Three White Soldiers pattern.
///
/// A strong bullish continuation pattern:
/// - Three consecutive long bullish candles
/// - Each opens within the prior candle's body
/// - Each closes near its high
#[inline]
pub fn is_three_white_soldiers(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    // All three must be bullish
    if !first.is_bullish() || !second.is_bullish() || !third.is_bullish() {
        return false;
    }

    // Each candle should have a significant body (not small)
    let first_body_ratio = (first.close - first.open) / (first.high - first.low);
    let second_body_ratio = (second.close - second.open) / (second.high - second.low);
    let third_body_ratio = (third.close - third.open) / (third.high - third.low);

    if first_body_ratio < 0.6 || second_body_ratio < 0.6 || third_body_ratio < 0.6 {
        return false;
    }

    // Each opens within prior body and closes higher
    let second_opens_in_first = second.open >= first.open && second.open <= first.close;
    let third_opens_in_second = third.open >= second.open && third.open <= second.close;

    // Progressive higher closes
    let progressive_closes = third.close > second.close && second.close > first.close;

    second_opens_in_first && third_opens_in_second && progressive_closes
}

/// Check if three candles form a Three Black Crows pattern.
///
/// A strong bearish continuation pattern (opposite of three white soldiers):
/// - Three consecutive long bearish candles
/// - Each opens within the prior candle's body
/// - Each closes near its low
#[inline]
pub fn is_three_black_crows(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    // All three must be bearish
    if first.is_bullish() || second.is_bullish() || third.is_bullish() {
        return false;
    }

    // Each candle should have a significant body
    let first_body_ratio = (first.open - first.close) / (first.high - first.low);
    let second_body_ratio = (second.open - second.close) / (second.high - second.low);
    let third_body_ratio = (third.open - third.close) / (third.high - third.low);

    if first_body_ratio < 0.6 || second_body_ratio < 0.6 || third_body_ratio < 0.6 {
        return false;
    }

    // Each opens within prior body and closes lower
    let second_opens_in_first = second.open <= first.open && second.open >= first.close;
    let third_opens_in_second = third.open <= second.open && third.open >= second.close;

    // Progressive lower closes
    let progressive_closes = third.close < second.close && second.close < first.close;

    second_opens_in_first && third_opens_in_second && progressive_closes
}

/// Check if three candles form a Three Inside Up pattern.
///
/// A bullish reversal pattern:
/// 1. First candle: Large bearish
/// 2. Second candle: Bullish harami (inside first)
/// 3. Third candle: Bullish that closes above first's open
#[inline]
pub fn is_three_inside_up(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    use super::double::is_bullish_harami;

    // First two form bullish harami
    if !is_bullish_harami(first, second) {
        return false;
    }

    // Third is bullish and closes above first's open
    third.is_bullish() && third.close > first.open
}

/// Check if three candles form a Three Inside Down pattern.
///
/// A bearish reversal pattern:
/// 1. First candle: Large bullish
/// 2. Second candle: Bearish harami (inside first)
/// 3. Third candle: Bearish that closes below first's open
#[inline]
pub fn is_three_inside_down(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    use super::double::is_bearish_harami;

    // First two form bearish harami
    if !is_bearish_harami(first, second) {
        return false;
    }

    // Third is bearish and closes below first's open
    !third.is_bullish() && third.close < first.open
}

/// Check if three candles form a Three Outside Up pattern.
///
/// A bullish reversal pattern:
/// 1. First candle: Bearish
/// 2. Second candle: Bullish engulfing first
/// 3. Third candle: Bullish that closes above second
#[inline]
pub fn is_three_outside_up(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    use super::double::is_bullish_engulfing;

    // First two form bullish engulfing
    if !is_bullish_engulfing(first, second) {
        return false;
    }

    // Third is bullish and closes above second
    third.is_bullish() && third.close > second.close
}

/// Check if three candles form a Three Outside Down pattern.
///
/// A bearish reversal pattern:
/// 1. First candle: Bullish
/// 2. Second candle: Bearish engulfing first
/// 3. Third candle: Bearish that closes below second
#[inline]
pub fn is_three_outside_down(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    use super::double::is_bearish_engulfing;

    // First two form bearish engulfing
    if !is_bearish_engulfing(first, second) {
        return false;
    }

    // Third is bearish and closes below second
    !third.is_bullish() && third.close < second.close
}

/// Check if three candles form an Abandoned Baby (bullish).
///
/// A strong reversal pattern with gaps:
/// 1. First candle: Bearish
/// 2. Second candle: Doji that gaps below first (island)
/// 3. Third candle: Bullish that gaps above second
#[inline]
pub fn is_abandoned_baby_bullish(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    // First is bearish
    if first.is_bullish() {
        return false;
    }

    // Second is doji
    if !is_doji(second) {
        return false;
    }

    // Third is bullish
    if !third.is_bullish() {
        return false;
    }

    // Gap down between first and second
    let gap_down = second.high < first.low;

    // Gap up between second and third
    let gap_up = third.low > second.high;

    gap_down && gap_up
}

/// Check if three candles form an Abandoned Baby (bearish).
///
/// A strong reversal pattern with gaps (opposite of bullish version).
#[inline]
pub fn is_abandoned_baby_bearish(first: &Ohlcv, second: &Ohlcv, third: &Ohlcv) -> bool {
    // First is bullish
    if !first.is_bullish() {
        return false;
    }

    // Second is doji
    if !is_doji(second) {
        return false;
    }

    // Third is bearish
    if third.is_bullish() {
        return false;
    }

    // Gap up between first and second
    let gap_up = second.low > first.high;

    // Gap down between second and third
    let gap_down = third.high < second.low;

    gap_up && gap_down
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(open: f64, high: f64, low: f64, close: f64) -> Ohlcv {
        Ohlcv::new(open, high, low, close, 1000.0)
    }

    #[test]
    fn test_morning_star() {
        // Large bearish, small star, large bullish
        let first = make_candle(110.0, 111.0, 99.0, 100.0);   // Bearish
        let second = make_candle(98.0, 99.0, 97.0, 98.5);     // Small (star)
        let third = make_candle(99.0, 112.0, 98.0, 108.0);    // Bullish above midpoint
        assert!(is_morning_star(&first, &second, &third));
    }

    #[test]
    fn test_evening_star() {
        // Large bullish, small star gapping above, large bearish
        let first = make_candle(100.0, 111.0, 99.0, 110.0);   // Bullish (body=10, range=12)
        let second = make_candle(112.0, 114.0, 111.0, 112.5); // Small star (gaps above 110)
        let third = make_candle(111.0, 112.0, 98.0, 100.0);   // Bearish below midpoint (105)
        assert!(is_evening_star(&first, &second, &third));
    }

    #[test]
    fn test_three_white_soldiers() {
        // Three consecutive bullish candles
        let first = make_candle(100.0, 105.0, 99.0, 104.0);
        let second = make_candle(103.0, 109.0, 102.0, 108.0);
        let third = make_candle(107.0, 114.0, 106.0, 113.0);
        assert!(is_three_white_soldiers(&first, &second, &third));
    }

    #[test]
    fn test_three_black_crows() {
        // Three consecutive bearish candles
        let first = make_candle(110.0, 111.0, 105.0, 106.0);
        let second = make_candle(107.0, 108.0, 101.0, 102.0);
        let third = make_candle(103.0, 104.0, 97.0, 98.0);
        assert!(is_three_black_crows(&first, &second, &third));
    }
}
