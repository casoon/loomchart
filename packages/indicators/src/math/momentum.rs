//! Momentum calculations
//!
//! Rate of Change, Momentum, Gains/Losses, and other momentum functions.

use libm::fabs;

/// Calculate the Rate of Change (ROC) as a percentage.
///
/// ROC = ((current - previous) / previous) * 100
///
/// # Example
/// ```
/// use loom_indicators::math::roc;
/// let change = roc(110.0, 100.0);
/// assert!((change - 10.0).abs() < 1e-10); // 10% increase
/// ```
#[inline]
pub fn roc(current: f64, previous: f64) -> f64 {
    if fabs(previous) < f64::EPSILON {
        return 0.0;
    }
    (current - previous) / previous * 100.0
}

/// Calculate the simple momentum (price difference).
///
/// Momentum = current - previous
#[inline]
pub fn momentum(current: f64, previous: f64) -> f64 {
    current - previous
}

/// Calculate the gain (positive change) from a price move.
///
/// Returns the change if positive, otherwise 0.
///
/// Used in RSI and similar indicators.
///
/// # Example
/// ```
/// use loom_indicators::math::gain;
/// assert!((gain(110.0, 100.0) - 10.0).abs() < 1e-10);
/// assert!((gain(90.0, 100.0) - 0.0).abs() < 1e-10);
/// ```
#[inline]
pub fn gain(current: f64, previous: f64) -> f64 {
    let change = current - previous;
    if change > 0.0 {
        change
    } else {
        0.0
    }
}

/// Calculate the loss (negative change) from a price move.
///
/// Returns the absolute value of change if negative, otherwise 0.
///
/// # Example
/// ```
/// use loom_indicators::math::loss;
/// assert!((loss(90.0, 100.0) - 10.0).abs() < 1e-10);
/// assert!((loss(110.0, 100.0) - 0.0).abs() < 1e-10);
/// ```
#[inline]
pub fn loss(current: f64, previous: f64) -> f64 {
    let change = current - previous;
    if change < 0.0 {
        -change
    } else {
        0.0
    }
}

/// Calculate the Relative Strength from average gain and average loss.
///
/// RS = avg_gain / avg_loss
///
/// Used in RSI calculation.
#[inline]
pub fn relative_strength(avg_gain: f64, avg_loss: f64) -> f64 {
    if fabs(avg_loss) < f64::EPSILON {
        return 100.0; // All gains, no losses
    }
    avg_gain / avg_loss
}

/// Calculate RSI from Relative Strength.
///
/// RSI = 100 - (100 / (1 + RS))
#[inline]
pub fn rsi_from_rs(rs: f64) -> f64 {
    100.0 - (100.0 / (1.0 + rs))
}

/// Calculate RSI directly from average gain and average loss.
///
/// This is a convenience function combining relative_strength and rsi_from_rs.
#[inline]
pub fn rsi(avg_gain: f64, avg_loss: f64) -> f64 {
    if fabs(avg_loss) < f64::EPSILON {
        return 100.0; // All gains
    }
    if fabs(avg_gain) < f64::EPSILON {
        return 0.0; // All losses
    }
    rsi_from_rs(relative_strength(avg_gain, avg_loss))
}

/// Calculate the Money Flow Multiplier.
///
/// MFM = ((close - low) - (high - close)) / (high - low)
///     = (2 * close - low - high) / (high - low)
///
/// Returns a value between -1 and 1.
#[inline]
pub fn money_flow_multiplier(high: f64, low: f64, close: f64) -> f64 {
    let range = high - low;
    if fabs(range) < f64::EPSILON {
        return 0.0;
    }
    ((close - low) - (high - close)) / range
}

/// Calculate the Money Flow Volume.
///
/// MFV = MFM * volume
#[inline]
pub fn money_flow_volume(high: f64, low: f64, close: f64, volume: f64) -> f64 {
    money_flow_multiplier(high, low, close) * volume
}

/// Calculate the Typical Price.
///
/// TP = (high + low + close) / 3
#[inline]
pub fn typical_price(high: f64, low: f64, close: f64) -> f64 {
    (high + low + close) / 3.0
}

/// Calculate the Raw Money Flow.
///
/// Raw MF = Typical Price * Volume
#[inline]
pub fn raw_money_flow(high: f64, low: f64, close: f64, volume: f64) -> f64 {
    typical_price(high, low, close) * volume
}

/// Determine if money flow is positive or negative.
///
/// Positive if current typical price > previous typical price.
#[inline]
pub fn is_positive_money_flow(current_tp: f64, previous_tp: f64) -> bool {
    current_tp > previous_tp
}

/// Calculate the Money Flow Index (MFI) from positive and negative money flow sums.
///
/// MFI = 100 - (100 / (1 + money_ratio))
/// where money_ratio = positive_mf / negative_mf
#[inline]
pub fn mfi(positive_mf: f64, negative_mf: f64) -> f64 {
    if fabs(negative_mf) < f64::EPSILON {
        return 100.0;
    }
    if fabs(positive_mf) < f64::EPSILON {
        return 0.0;
    }
    let ratio = positive_mf / negative_mf;
    100.0 - (100.0 / (1.0 + ratio))
}

/// Calculate the Commodity Channel Index (CCI) value.
///
/// CCI = (TP - SMA(TP)) / (0.015 * Mean Deviation)
#[inline]
pub fn cci(typical_price: f64, sma_tp: f64, mean_deviation: f64) -> f64 {
    if fabs(mean_deviation) < f64::EPSILON {
        return 0.0;
    }
    (typical_price - sma_tp) / (0.015 * mean_deviation)
}

/// Calculate Mean Deviation (used in CCI).
///
/// Mean Deviation = sum(|x - mean|) / n
#[inline]
pub fn mean_deviation(values: &[f64], mean: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: f64 = values.iter().map(|&x| fabs(x - mean)).sum();
    sum / values.len() as f64
}

/// Calculate the Williams %R value.
///
/// %R = (Highest High - Close) / (Highest High - Lowest Low) * -100
///
/// Note: Williams %R ranges from -100 to 0 (not 0 to 100).
#[inline]
pub fn williams_r(close: f64, highest_high: f64, lowest_low: f64) -> f64 {
    let range = highest_high - lowest_low;
    if fabs(range) < f64::EPSILON {
        return -50.0;
    }
    (highest_high - close) / range * -100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roc() {
        assert!((roc(110.0, 100.0) - 10.0).abs() < 1e-10);
        assert!((roc(90.0, 100.0) - (-10.0)).abs() < 1e-10);
    }

    #[test]
    fn test_gain_loss() {
        // Gain
        assert!((gain(110.0, 100.0) - 10.0).abs() < 1e-10);
        assert!((gain(90.0, 100.0) - 0.0).abs() < 1e-10);

        // Loss
        assert!((loss(90.0, 100.0) - 10.0).abs() < 1e-10);
        assert!((loss(110.0, 100.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_rsi() {
        // Equal gains and losses -> RSI = 50
        assert!((rsi(5.0, 5.0) - 50.0).abs() < 1e-10);

        // All gains, no losses -> RSI = 100
        assert!((rsi(5.0, 0.0) - 100.0).abs() < 1e-10);

        // All losses, no gains -> RSI = 0
        assert!((rsi(0.0, 5.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_money_flow_multiplier() {
        // Close at high -> +1
        assert!((money_flow_multiplier(110.0, 100.0, 110.0) - 1.0).abs() < 1e-10);

        // Close at low -> -1
        assert!((money_flow_multiplier(110.0, 100.0, 100.0) - (-1.0)).abs() < 1e-10);

        // Close at midpoint -> 0
        assert!((money_flow_multiplier(110.0, 100.0, 105.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_williams_r() {
        // At highest high -> %R = 0
        assert!((williams_r(100.0, 100.0, 80.0) - 0.0).abs() < 1e-10);

        // At lowest low -> %R = -100
        assert!((williams_r(80.0, 100.0, 80.0) - (-100.0)).abs() < 1e-10);

        // At midpoint -> %R = -50
        assert!((williams_r(90.0, 100.0, 80.0) - (-50.0)).abs() < 1e-10);
    }
}
