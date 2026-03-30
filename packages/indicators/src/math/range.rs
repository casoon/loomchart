//! Range-based calculations
//!
//! True Range, Highest High, Lowest Low, and other range functions.

use libm::fabs;

/// Calculate the True Range for a single candle.
///
/// True Range = max(high - low, |high - prev_close|, |low - prev_close|)
///
/// True Range accounts for gaps between candles.
///
/// # Example
/// ```
/// use loom_indicators::math::true_range;
/// let tr = true_range(105.0, 100.0, 102.0);
/// assert!((tr - 5.0).abs() < 1e-10); // high - low = 105 - 100 = 5
/// ```
#[inline]
pub fn true_range(high: f64, low: f64, prev_close: f64) -> f64 {
    let hl = high - low;
    let hc = fabs(high - prev_close);
    let lc = fabs(low - prev_close);

    if hl >= hc && hl >= lc {
        hl
    } else if hc >= hl && hc >= lc {
        hc
    } else {
        lc
    }
}

/// Calculate the True Range for the first candle (no previous close).
///
/// For the first candle, True Range = High - Low.
#[inline]
pub fn true_range_first(high: f64, low: f64) -> f64 {
    high - low
}

/// Find the highest value in a slice.
#[inline]
pub fn highest(values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, |a, b| if b > a { b } else { a })
}

/// Find the lowest value in a slice.
#[inline]
pub fn lowest(values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .fold(f64::INFINITY, |a, b| if b < a { b } else { a })
}

/// Find the highest value and its index.
#[inline]
pub fn highest_with_index(values: &[f64]) -> (f64, usize) {
    if values.is_empty() {
        return (f64::NEG_INFINITY, 0);
    }

    let mut max_val = values[0];
    let mut max_idx = 0;

    for (i, &v) in values.iter().enumerate().skip(1) {
        if v > max_val {
            max_val = v;
            max_idx = i;
        }
    }

    (max_val, max_idx)
}

/// Find the lowest value and its index.
#[inline]
pub fn lowest_with_index(values: &[f64]) -> (f64, usize) {
    if values.is_empty() {
        return (f64::INFINITY, 0);
    }

    let mut min_val = values[0];
    let mut min_idx = 0;

    for (i, &v) in values.iter().enumerate().skip(1) {
        if v < min_val {
            min_val = v;
            min_idx = i;
        }
    }

    (min_val, min_idx)
}

/// Calculate the price range (high - low) of a period.
#[inline]
pub fn range(high: f64, low: f64) -> f64 {
    high - low
}

/// Calculate the price range as percentage of low.
#[inline]
pub fn range_percent(high: f64, low: f64) -> f64 {
    if fabs(low) < f64::EPSILON {
        return 0.0;
    }
    (high - low) / low * 100.0
}

/// Calculate the midpoint of a range.
#[inline]
pub fn midpoint(high: f64, low: f64) -> f64 {
    (high + low) / 2.0
}

/// Calculate the Highest High over a slice of highs.
///
/// This is commonly used in indicators like Stochastic and Donchian Channels.
#[inline]
pub fn highest_high(highs: &[f64]) -> f64 {
    highest(highs)
}

/// Calculate the Lowest Low over a slice of lows.
#[inline]
pub fn lowest_low(lows: &[f64]) -> f64 {
    lowest(lows)
}

/// Calculate where the current value falls within a range (0-100).
///
/// Used in Stochastic calculations: %K = (close - lowest_low) / (highest_high - lowest_low) * 100
#[inline]
pub fn percent_in_range(value: f64, low: f64, high: f64) -> f64 {
    let range = high - low;
    if fabs(range) < f64::EPSILON {
        return 50.0; // Midpoint if no range
    }
    (value - low) / range * 100.0
}

/// Calculate the Average True Range (ATR) style smoothing.
///
/// ATR uses Wilder's smoothing: ATR = ((prev_atr * (n-1)) + tr) / n
#[inline]
pub fn atr_smooth(prev_atr: f64, true_range: f64, period: usize) -> f64 {
    let p = period as f64;
    (prev_atr * (p - 1.0) + true_range) / p
}

/// Calculate Plus and Minus Directional Movement.
///
/// +DM = high - prev_high (if positive and > -DM, else 0)
/// -DM = prev_low - low (if positive and > +DM, else 0)
///
/// Returns (plus_dm, minus_dm).
#[inline]
pub fn directional_movement(high: f64, low: f64, prev_high: f64, prev_low: f64) -> (f64, f64) {
    let up_move = high - prev_high;
    let down_move = prev_low - low;

    let plus_dm = if up_move > down_move && up_move > 0.0 {
        up_move
    } else {
        0.0
    };

    let minus_dm = if down_move > up_move && down_move > 0.0 {
        down_move
    } else {
        0.0
    };

    (plus_dm, minus_dm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_range() {
        // Normal case: high - low is largest
        assert!((true_range(110.0, 100.0, 105.0) - 10.0).abs() < 1e-10);

        // Gap up: |high - prev_close| is largest
        assert!((true_range(115.0, 112.0, 100.0) - 15.0).abs() < 1e-10);

        // Gap down: |low - prev_close| is largest
        assert!((true_range(88.0, 85.0, 100.0) - 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_highest_lowest() {
        let values = [5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
        assert!((highest(&values) - 9.0).abs() < 1e-10);
        assert!((lowest(&values) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_percent_in_range() {
        // At low = 0%, at high = 100%, at midpoint = 50%
        assert!((percent_in_range(100.0, 100.0, 200.0) - 0.0).abs() < 1e-10);
        assert!((percent_in_range(200.0, 100.0, 200.0) - 100.0).abs() < 1e-10);
        assert!((percent_in_range(150.0, 100.0, 200.0) - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_directional_movement() {
        // Up move is stronger
        let (plus, minus) = directional_movement(110.0, 98.0, 105.0, 100.0);
        assert!((plus - 5.0).abs() < 1e-10);
        assert!((minus - 0.0).abs() < 1e-10);

        // Down move is stronger
        let (plus, minus) = directional_movement(103.0, 90.0, 105.0, 100.0);
        assert!((plus - 0.0).abs() < 1e-10);
        assert!((minus - 10.0).abs() < 1e-10);
    }
}
