// Volatility Calculations

use super::{ema, highest, lowest, rma, sma, stdev_simple};

/// True Range
pub fn tr(high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(high.len());

    for i in 0..high.len() {
        if i == 0 {
            result.push(high[i] - low[i]);
        } else {
            let hl = high[i] - low[i];
            let hc = (high[i] - close[i - 1]).abs();
            let lc = (low[i] - close[i - 1]).abs();
            result.push(hl.max(hc).max(lc));
        }
    }

    result
}

/// Average True Range
pub fn atr(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let true_range = tr(high, low, close);
    rma(&true_range, period)
}

/// Average True Range with different smoothing methods
pub fn atr_smoothed(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
    smoothing: &str,
) -> Vec<Option<f64>> {
    let true_range = tr(high, low, close);

    match smoothing {
        "SMA" => sma(&true_range, period),
        "EMA" => ema(&true_range, period),
        _ => rma(&true_range, period), // RMA is default (Wilder's)
    }
}

/// Bollinger Bands
/// Returns (basis, upper, lower)
pub fn bollinger_bands(
    source: &[f64],
    period: usize,
    mult: f64,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let basis = sma(source, period);
    let std = stdev_simple(source, period);

    let upper: Vec<Option<f64>> = basis
        .iter()
        .zip(std.iter())
        .map(|(&b, &s)| match (b, s) {
            (Some(bv), Some(sv)) => Some(bv + mult * sv),
            _ => None,
        })
        .collect();

    let lower: Vec<Option<f64>> = basis
        .iter()
        .zip(std.iter())
        .map(|(&b, &s)| match (b, s) {
            (Some(bv), Some(sv)) => Some(bv - mult * sv),
            _ => None,
        })
        .collect();

    (basis, upper, lower)
}

/// Bollinger Bands %B
pub fn bb_percent_b(source: &[f64], period: usize, mult: f64) -> Vec<Option<f64>> {
    let (_basis, upper, lower) = bollinger_bands(source, period, mult);

    source
        .iter()
        .enumerate()
        .map(|(i, &price)| match (upper[i], lower[i]) {
            (Some(u), Some(l)) if u != l => Some((price - l) / (u - l)),
            _ => None,
        })
        .collect()
}

/// Bollinger Bands Bandwidth
pub fn bb_bandwidth(source: &[f64], period: usize, mult: f64) -> Vec<Option<f64>> {
    let (basis, upper, lower) = bollinger_bands(source, period, mult);

    basis
        .iter()
        .zip(upper.iter())
        .zip(lower.iter())
        .map(|((&b, &u), &l)| match (b, u, l) {
            (Some(bv), Some(uv), Some(lv)) if bv != 0.0 => Some((uv - lv) / bv * 100.0),
            _ => None,
        })
        .collect()
}

/// Keltner Channels
/// Returns (basis, upper, lower)
pub fn keltner_channels(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
    atr_period: usize,
    mult: f64,
    use_ema: bool,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let basis = if use_ema {
        ema(close, period)
    } else {
        sma(close, period)
    };

    let atr_values = atr(high, low, close, atr_period);

    let upper: Vec<Option<f64>> = basis
        .iter()
        .zip(atr_values.iter())
        .map(|(&b, &a)| match (b, a) {
            (Some(bv), Some(av)) => Some(bv + mult * av),
            _ => None,
        })
        .collect();

    let lower: Vec<Option<f64>> = basis
        .iter()
        .zip(atr_values.iter())
        .map(|(&b, &a)| match (b, a) {
            (Some(bv), Some(av)) => Some(bv - mult * av),
            _ => None,
        })
        .collect();

    (basis, upper, lower)
}

/// Donchian Channels
/// Returns (upper, middle, lower)
pub fn donchian_channels(
    high: &[f64],
    low: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let upper = highest(high, period);
    let lower = lowest(low, period);

    let middle: Vec<Option<f64>> = upper
        .iter()
        .zip(lower.iter())
        .map(|(&u, &l)| match (u, l) {
            (Some(uv), Some(lv)) => Some((uv + lv) / 2.0),
            _ => None,
        })
        .collect();

    (upper, middle, lower)
}

/// Average Daily Range
pub fn adr(high: &[f64], low: &[f64], period: usize) -> Vec<Option<f64>> {
    let daily_range: Vec<f64> = high.iter().zip(low.iter()).map(|(&h, &l)| h - l).collect();

    sma(&daily_range, period)
}

/// Average Daily Range Percent
pub fn adr_percent(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(close.len());

    for i in 0..close.len() {
        if i < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let mut sum = 0.0;
            for j in start..=i {
                let range = high[j] - low[j];
                let mid = (high[j] + low[j]) / 2.0;
                if mid != 0.0 {
                    sum += (range / mid) * 100.0;
                }
            }
            result.push(Some(sum / period as f64));
        }
    }

    result
}

/// Historical Volatility (annualized)
pub fn historical_volatility(
    close: &[f64],
    period: usize,
    annual_periods: f64,
) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(close.len());

    // Calculate log returns
    let mut log_returns = Vec::with_capacity(close.len());
    log_returns.push(0.0);

    for i in 1..close.len() {
        if close[i - 1] > 0.0 && close[i] > 0.0 {
            log_returns.push((close[i] / close[i - 1]).ln());
        } else {
            log_returns.push(0.0);
        }
    }

    // Calculate rolling standard deviation of log returns
    for i in 0..close.len() {
        if i < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let slice = &log_returns[start..=i];
            let mean: f64 = slice.iter().sum::<f64>() / period as f64;
            let variance: f64 =
                slice.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / period as f64;
            let std_dev = variance.sqrt();

            // Annualize
            result.push(Some(std_dev * annual_periods.sqrt() * 100.0));
        }
    }

    result
}

/// Choppiness Index
pub fn choppiness_index(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(close.len());
    let true_range = tr(high, low, close);

    for i in 0..close.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            // Sum of TR
            let atr_sum: f64 = true_range[start..=i].iter().sum();

            // Highest high - Lowest low
            let hh = high[start..=i]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let ll = low[start..=i]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);

            let range = hh - ll;

            if range > 0.0 && atr_sum > 0.0 {
                let chop = 100.0 * (atr_sum / range).ln() / (period as f64).ln();
                result.push(Some(chop));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Envelope (Percentage bands around MA)
/// Returns (upper, basis, lower)
pub fn envelope(
    source: &[f64],
    period: usize,
    percent: f64,
    use_ema: bool,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let basis = if use_ema {
        ema(source, period)
    } else {
        sma(source, period)
    };

    let mult = percent / 100.0;

    let upper: Vec<Option<f64>> = basis.iter().map(|&b| b.map(|v| v * (1.0 + mult))).collect();

    let lower: Vec<Option<f64>> = basis.iter().map(|&b| b.map(|v| v * (1.0 - mult))).collect();

    (upper, basis, lower)
}

/// BBTrend - Bollinger Bands Trend
pub fn bbtrend(
    source: &[f64],
    short_period: usize,
    long_period: usize,
    mult: f64,
) -> Vec<Option<f64>> {
    let (_, upper_short, lower_short) = bollinger_bands(source, short_period, mult);
    let (_, upper_long, lower_long) = bollinger_bands(source, long_period, mult);

    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        match (upper_short[i], lower_short[i], upper_long[i], lower_long[i]) {
            (Some(us), Some(ls), Some(ul), Some(ll)) => {
                let short_width = us - ls;
                let long_width = ul - ll;

                if long_width != 0.0 {
                    result.push(Some((short_width - long_width) / long_width * 100.0));
                } else {
                    result.push(None);
                }
            }
            _ => result.push(None),
        }
    }

    result
}

/// Ulcer Index - Measures downside volatility
pub fn ulcer_index(close: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(close.len());

    for i in 0..close.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            // Find highest close in period
            let highest_close = close[start..=i]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);

            // Calculate sum of squared percent drawdowns
            let mut sum_sq = 0.0;
            for j in start..=i {
                let pct_drawdown = if highest_close > 0.0 {
                    (close[j] - highest_close) / highest_close * 100.0
                } else {
                    0.0
                };
                sum_sq += pct_drawdown.powi(2);
            }

            result.push(Some((sum_sq / period as f64).sqrt()));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tr() {
        let high = vec![48.0, 48.5, 49.0, 49.5, 50.0];
        let low = vec![46.0, 46.5, 47.0, 47.5, 48.0];
        let close = vec![47.0, 47.5, 48.0, 48.5, 49.0];

        let result = tr(&high, &low, &close);

        assert_eq!(result[0], 2.0); // First bar: high - low
        assert!(result[1] > 0.0);
    }

    #[test]
    fn test_atr() {
        let high = vec![48.0, 48.5, 49.0, 49.5, 50.0, 50.5, 51.0];
        let low = vec![46.0, 46.5, 47.0, 47.5, 48.0, 48.5, 49.0];
        let close = vec![47.0, 47.5, 48.0, 48.5, 49.0, 49.5, 50.0];

        let result = atr(&high, &low, &close, 3);

        assert!(result[0].is_none());
        assert!(result[1].is_none());
        assert!(result[2].is_some());
    }

    #[test]
    fn test_bollinger_bands() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let (basis, upper, lower) = bollinger_bands(&data, 5, 2.0);

        assert!(basis[4].is_some());
        assert!(upper[4].is_some());
        assert!(lower[4].is_some());

        // Upper should be > basis > lower
        if let (Some(b), Some(u), Some(l)) = (basis[4], upper[4], lower[4]) {
            assert!(u > b);
            assert!(b > l);
        }
    }

    #[test]
    fn test_donchian() {
        let high = vec![5.0, 6.0, 7.0, 8.0, 9.0];
        let low = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let (upper, middle, lower) = donchian_channels(&high, &low, 3);

        assert!(upper[2].is_some());
        assert_eq!(upper[2], Some(7.0)); // Highest high of first 3
        assert_eq!(lower[2], Some(1.0)); // Lowest low of first 3
        assert_eq!(middle[2], Some(4.0)); // Average
    }
}
