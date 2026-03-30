//! Moving average calculations
//!
//! Pure functions for computing various types of moving averages.

use libm::fabs;

/// Calculate the Simple Moving Average (SMA) of a slice.
///
/// SMA = sum(values) / count
///
/// # Example
/// ```
/// use loom_indicators::math::sma;
/// let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
/// assert!((sma(&prices) - 3.0).abs() < 1e-10);
/// ```
#[inline]
pub fn sma(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

/// Calculate the SMA with a known sum (optimization for rolling calculations).
///
/// When maintaining a running sum, this avoids re-summing all values.
#[inline]
pub fn sma_from_sum(sum: f64, count: usize) -> f64 {
    if count == 0 {
        return 0.0;
    }
    sum / count as f64
}

/// Calculate the EMA multiplier (smoothing factor).
///
/// k = 2 / (period + 1)
///
/// # Example
/// ```
/// use loom_indicators::math::ema_multiplier;
/// let k = ema_multiplier(21);
/// assert!((k - 0.090909).abs() < 0.0001);
/// ```
#[inline]
pub fn ema_multiplier(period: usize) -> f64 {
    2.0 / (period as f64 + 1.0)
}

/// Calculate the next EMA value given the current price and previous EMA.
///
/// EMA = price * k + prev_ema * (1 - k)
///
/// This is the core building block for EMA-based indicators.
///
/// # Example
/// ```
/// use loom_indicators::math::{ema_multiplier, ema_next};
/// let k = ema_multiplier(10);
/// let prev_ema = 100.0;
/// let price = 105.0;
/// let new_ema = ema_next(price, prev_ema, k);
/// // EMA moves toward price
/// assert!(new_ema > prev_ema && new_ema < price);
/// ```
#[inline]
pub fn ema_next(price: f64, prev_ema: f64, multiplier: f64) -> f64 {
    price * multiplier + prev_ema * (1.0 - multiplier)
}

/// Calculate the Weighted Moving Average (WMA) of a slice.
///
/// WMA gives more weight to recent values:
/// WMA = (n*p_n + (n-1)*p_{n-1} + ... + 1*p_1) / (n + (n-1) + ... + 1)
///
/// # Example
/// ```
/// use loom_indicators::math::wma;
/// let prices = [1.0, 2.0, 3.0, 4.0, 5.0];
/// let result = wma(&prices);
/// // Later values weighted more heavily, so > SMA(3.0)
/// assert!(result > 3.0);
/// ```
#[inline]
pub fn wma(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let _n = values.len();
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for (i, &value) in values.iter().enumerate() {
        let weight = (i + 1) as f64;
        weighted_sum += value * weight;
        weight_total += weight;
    }

    weighted_sum / weight_total
}

/// Calculate the Smoothed Moving Average (SMMA / RMA).
///
/// SMMA is similar to EMA but with a different smoothing factor:
/// SMMA = (prev_smma * (period - 1) + price) / period
///
/// Used in RSI and other Wilder indicators.
#[inline]
pub fn smma_next(price: f64, prev_smma: f64, period: usize) -> f64 {
    let p = period as f64;
    (prev_smma * (p - 1.0) + price) / p
}

/// Calculate the Volume Weighted Average Price (VWAP) for a set of candles.
///
/// VWAP = sum(typical_price * volume) / sum(volume)
///
/// This is a batch calculation. For streaming VWAP, maintain cumulative sums.
#[inline]
pub fn vwap(typical_prices: &[f64], volumes: &[f64]) -> f64 {
    if typical_prices.is_empty() || volumes.is_empty() {
        return 0.0;
    }

    let mut tp_vol_sum = 0.0;
    let mut vol_sum = 0.0;

    for (&tp, &vol) in typical_prices.iter().zip(volumes.iter()) {
        tp_vol_sum += tp * vol;
        vol_sum += vol;
    }

    if fabs(vol_sum) < f64::EPSILON {
        return 0.0;
    }

    tp_vol_sum / vol_sum
}

/// Calculate Double EMA (DEMA).
///
/// DEMA = 2 * EMA - EMA(EMA)
///
/// Less lag than single EMA.
#[inline]
pub fn dema_next(price: f64, prev_ema: f64, prev_ema_ema: f64, multiplier: f64) -> (f64, f64, f64) {
    let new_ema = ema_next(price, prev_ema, multiplier);
    let new_ema_ema = ema_next(new_ema, prev_ema_ema, multiplier);
    let dema = 2.0 * new_ema - new_ema_ema;
    (dema, new_ema, new_ema_ema)
}

/// Calculate Triple EMA (TEMA).
///
/// TEMA = 3 * EMA - 3 * EMA(EMA) + EMA(EMA(EMA))
///
/// Even less lag than DEMA.
#[inline]
pub fn tema_next(
    price: f64,
    prev_ema1: f64,
    prev_ema2: f64,
    prev_ema3: f64,
    multiplier: f64,
) -> (f64, f64, f64, f64) {
    let ema1 = ema_next(price, prev_ema1, multiplier);
    let ema2 = ema_next(ema1, prev_ema2, multiplier);
    let ema3 = ema_next(ema2, prev_ema3, multiplier);
    let tema = 3.0 * ema1 - 3.0 * ema2 + ema3;
    (tema, ema1, ema2, ema3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        assert!((sma(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < 1e-10);
        assert!((sma(&[10.0]) - 10.0).abs() < 1e-10);
        assert!((sma(&[]) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_ema_multiplier() {
        // Period 9: k = 2/10 = 0.2
        assert!((ema_multiplier(9) - 0.2).abs() < 1e-10);
        // Period 21: k = 2/22 ≈ 0.0909
        assert!((ema_multiplier(21) - 0.090909).abs() < 0.0001);
    }

    #[test]
    fn test_ema_next() {
        let k = 0.2;
        let prev = 100.0;
        let price = 110.0;
        // EMA = 110 * 0.2 + 100 * 0.8 = 22 + 80 = 102
        assert!((ema_next(price, prev, k) - 102.0).abs() < 1e-10);
    }

    #[test]
    fn test_wma() {
        let prices = [1.0, 2.0, 3.0];
        // WMA = (1*1 + 2*2 + 3*3) / (1+2+3) = (1+4+9) / 6 = 14/6 ≈ 2.333
        assert!((wma(&prices) - 2.333333).abs() < 0.0001);
    }

    #[test]
    fn test_vwap() {
        let tp = [100.0, 102.0, 101.0];
        let vol = [1000.0, 2000.0, 1500.0];
        // VWAP = (100*1000 + 102*2000 + 101*1500) / (1000+2000+1500)
        //      = (100000 + 204000 + 151500) / 4500 = 455500 / 4500 ≈ 101.22
        assert!((vwap(&tp, &vol) - 101.222222).abs() < 0.001);
    }
}
