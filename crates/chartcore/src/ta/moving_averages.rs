// Moving Average Calculations

/// Simple Moving Average
pub fn sma(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if period == 0 {
        return vec![None; source.len()];
    }

    for i in 0..source.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let sum: f64 = source[start..=i].iter().sum();
            result.push(Some(sum / period as f64));
        }
    }

    result
}

/// Exponential Moving Average
pub fn ema(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if source.len() < period {
        return vec![None; source.len()];
    }

    let k = 2.0 / (period as f64 + 1.0);

    // Initial SMA
    let mut sum = 0.0;
    for i in 0..period {
        sum += source[i];
        result.push(None);
    }

    let mut ema_value = sum / period as f64;
    result[period - 1] = Some(ema_value);

    // EMA for subsequent values
    for i in period..source.len() {
        ema_value = source[i] * k + ema_value * (1.0 - k);
        result.push(Some(ema_value));
    }

    result
}

/// Weighted Moving Average
pub fn wma(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if period == 0 {
        return vec![None; source.len()];
    }

    let denominator = (period * (period + 1)) / 2;

    for i in 0..source.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let mut sum = 0.0;
            for j in 0..period {
                sum += source[i - j] * (period - j) as f64;
            }
            result.push(Some(sum / denominator as f64));
        }
    }

    result
}

/// Smoothed Moving Average (Wilder's - used in RSI, ATR)
pub fn rma(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if source.len() < period {
        return vec![None; source.len()];
    }

    // Initial SMA
    let mut sum = 0.0;
    for i in 0..period {
        sum += source[i];
        result.push(None);
    }

    let mut rma_value = sum / period as f64;
    result[period - 1] = Some(rma_value);

    // Wilder's smoothing
    for i in period..source.len() {
        rma_value = (rma_value * (period as f64 - 1.0) + source[i]) / period as f64;
        result.push(Some(rma_value));
    }

    result
}

/// SMMA (Smoothed Moving Average) - alias for RMA
pub fn smma(source: &[f64], period: usize) -> Vec<Option<f64>> {
    rma(source, period)
}

/// Double Exponential Moving Average
pub fn dema(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let ema1 = ema(source, period);

    // Convert to f64 for second EMA (replace None with 0)
    let ema1_values: Vec<f64> = ema1.iter().map(|&v| v.unwrap_or(0.0)).collect();
    let ema2 = ema(&ema1_values, period);

    ema1.iter()
        .zip(ema2.iter())
        .map(|(&e1, &e2)| match (e1, e2) {
            (Some(v1), Some(v2)) => Some(2.0 * v1 - v2),
            _ => None,
        })
        .collect()
}

/// Triple Exponential Moving Average
pub fn tema(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let ema1 = ema(source, period);
    let ema1_values: Vec<f64> = ema1.iter().map(|&v| v.unwrap_or(0.0)).collect();

    let ema2 = ema(&ema1_values, period);
    let ema2_values: Vec<f64> = ema2.iter().map(|&v| v.unwrap_or(0.0)).collect();

    let ema3 = ema(&ema2_values, period);

    ema1.iter()
        .zip(ema2.iter())
        .zip(ema3.iter())
        .map(|((&e1, &e2), &e3)| match (e1, e2, e3) {
            (Some(v1), Some(v2), Some(v3)) => Some(3.0 * v1 - 3.0 * v2 + v3),
            _ => None,
        })
        .collect()
}

/// Hull Moving Average
pub fn hma(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let half_period = period / 2;
    let sqrt_period = (period as f64).sqrt() as usize;

    let wma1 = wma(source, half_period);
    let wma2 = wma(source, period);

    // Calculate difference: 2 * WMA(n/2) - WMA(n)
    let diff: Vec<f64> = wma1
        .iter()
        .zip(wma2.iter())
        .map(|(&w1, &w2)| match (w1, w2) {
            (Some(v1), Some(v2)) => 2.0 * v1 - v2,
            _ => 0.0,
        })
        .collect();

    wma(&diff, sqrt_period)
}

/// Volume Weighted Moving Average
pub fn vwma(source: &[f64], volume: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let mut sum_pv = 0.0;
            let mut sum_v = 0.0;

            for j in 0..period {
                sum_pv += source[i - j] * volume[i - j];
                sum_v += volume[i - j];
            }

            if sum_v > 0.0 {
                result.push(Some(sum_pv / sum_v));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Linear Regression Moving Average (Least Squares MA)
pub fn lsma(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut sum_xy = 0.0;
            let mut sum_x2 = 0.0;

            for j in 0..period {
                let x = j as f64;
                let y = source[i - (period - 1 - j)];
                sum_x += x;
                sum_y += y;
                sum_xy += x * y;
                sum_x2 += x * x;
            }

            let n = period as f64;
            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
            let intercept = (sum_y - slope * sum_x) / n;

            // Return value at the end of the period
            result.push(Some(slope * (period as f64 - 1.0) + intercept));
        }
    }

    result
}

/// Arnaud Legoux Moving Average
/// offset: controls tradeoff between smoothness (closer to 1) and responsiveness (closer to 0)
/// sigma: standard deviation for Gaussian sharpness
pub fn alma(source: &[f64], period: usize, offset: f64, sigma: f64) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if period == 0 {
        return vec![None; source.len()];
    }

    let m = offset * (period as f64 - 1.0);
    let s = period as f64 / sigma;

    // Precompute weights
    let mut weights = Vec::with_capacity(period);
    let mut weight_sum = 0.0;

    for i in 0..period {
        let w = (-((i as f64 - m) * (i as f64 - m)) / (2.0 * s * s)).exp();
        weights.push(w);
        weight_sum += w;
    }

    // Normalize weights
    for w in &mut weights {
        *w /= weight_sum;
    }

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let mut sum = 0.0;
            for j in 0..period {
                sum += source[i - (period - 1 - j)] * weights[j];
            }
            result.push(Some(sum));
        }
    }

    result
}

/// McGinley Dynamic
pub fn mcginley_dynamic(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if source.is_empty() {
        return result;
    }

    // Start with first value
    let mut md = source[0];
    result.push(Some(md));

    let k = period as f64;

    for i in 1..source.len() {
        let price = source[i];
        // MD = MD[1] + (Price - MD[1]) / (K * (Price / MD[1])^4)
        if md != 0.0 {
            let ratio = price / md;
            let divisor = k * ratio.powi(4);
            if divisor != 0.0 {
                md = md + (price - md) / divisor;
            }
        } else {
            md = price;
        }
        result.push(Some(md));
    }

    result
}

/// Jurik Moving Average (JMA)
/// phase: -100 to +100, controls overshooting
/// power: smoothness (1 = max smooth, higher = more responsive)
pub fn jma(source: &[f64], period: usize, phase: f64, power: f64) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if source.is_empty() || period == 0 {
        return vec![None; source.len()];
    }

    // JMA parameters
    let phase_ratio = if phase < -100.0 {
        0.5
    } else if phase > 100.0 {
        2.5
    } else {
        phase / 100.0 + 1.5
    };

    let beta = 0.45 * (period as f64 - 1.0) / (0.45 * (period as f64 - 1.0) + 2.0);
    let alpha = beta.powf(power);

    let mut e0 = 0.0;
    let mut e1 = 0.0;
    let mut e2 = 0.0;
    let mut jma_value = source[0];

    for i in 0..source.len() {
        let price = source[i];

        e0 = (1.0 - alpha) * price + alpha * e0;
        e1 = (price - e0) * (1.0 - beta) + beta * e1;
        e2 = (e0 + phase_ratio * e1 - jma_value) * (1.0 - alpha).powi(2) + alpha.powi(2) * e2;
        jma_value = e2 + jma_value;

        if i < period - 1 {
            result.push(None);
        } else {
            result.push(Some(jma_value));
        }
    }

    result
}

/// TRIX - Triple Exponential Average Rate of Change
pub fn trix(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let ema1 = ema(source, period);
    let ema1_vals: Vec<f64> = ema1.iter().map(|v| v.unwrap_or(0.0)).collect();

    let ema2 = ema(&ema1_vals, period);
    let ema2_vals: Vec<f64> = ema2.iter().map(|v| v.unwrap_or(0.0)).collect();

    let ema3 = ema(&ema2_vals, period);

    let mut result = Vec::with_capacity(source.len());
    for i in 0..source.len() {
        if i < 1 {
            result.push(None);
        } else {
            match (ema3[i], ema3[i - 1]) {
                (Some(curr), Some(prev)) if prev != 0.0 => {
                    result.push(Some(((curr - prev) / prev) * 100.0));
                }
                _ => result.push(None),
            }
        }
    }

    result
}

/// Kaufman Adaptive Moving Average (KAMA)
pub fn kama(
    source: &[f64],
    period: usize,
    fast_period: usize,
    slow_period: usize,
) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if source.len() < period {
        return vec![None; source.len()];
    }

    let fast_sc = 2.0 / (fast_period as f64 + 1.0);
    let slow_sc = 2.0 / (slow_period as f64 + 1.0);

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else if i == period - 1 {
            result.push(Some(source[i]));
        } else {
            // Efficiency Ratio
            let change = (source[i] - source[i - period]).abs();
            let mut volatility = 0.0;
            for j in 1..=period {
                volatility += (source[i - j + 1] - source[i - j]).abs();
            }

            let er = if volatility != 0.0 {
                change / volatility
            } else {
                0.0
            };

            // Smoothing Constant
            let sc = (er * (fast_sc - slow_sc) + slow_sc).powi(2);

            // KAMA
            let prev_kama = result[i - 1].unwrap_or(source[i]);
            let kama_val = prev_kama + sc * (source[i] - prev_kama);
            result.push(Some(kama_val));
        }
    }

    result
}

/// Zero-Lag EMA (ZLEMA)
pub fn zlema(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let lag = (period - 1) / 2;

    // Create lag-adjusted data
    let mut adjusted = Vec::with_capacity(source.len());
    for i in 0..source.len() {
        if i >= lag {
            adjusted.push(2.0 * source[i] - source[i - lag]);
        } else {
            adjusted.push(source[i]);
        }
    }

    ema(&adjusted, period)
}

/// Fractal Adaptive Moving Average (FRAMA)
pub fn frama(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(close.len());

    if close.len() < period || period < 2 {
        return vec![None; close.len()];
    }

    let half = period / 2;

    for i in 0..close.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            // Calculate N1 (first half)
            let mut highest1 = f64::NEG_INFINITY;
            let mut lowest1 = f64::INFINITY;
            for j in (i - period + 1)..=(i - half) {
                highest1 = highest1.max(high[j]);
                lowest1 = lowest1.min(low[j]);
            }
            let n1 = (highest1 - lowest1) / half as f64;

            // Calculate N2 (second half)
            let mut highest2 = f64::NEG_INFINITY;
            let mut lowest2 = f64::INFINITY;
            for j in (i - half + 1)..=i {
                highest2 = highest2.max(high[j]);
                lowest2 = lowest2.min(low[j]);
            }
            let n2 = (highest2 - lowest2) / half as f64;

            // Calculate N3 (full period)
            let mut highest3 = f64::NEG_INFINITY;
            let mut lowest3 = f64::INFINITY;
            for j in (i - period + 1)..=i {
                highest3 = highest3.max(high[j]);
                lowest3 = lowest3.min(low[j]);
            }
            let n3 = (highest3 - lowest3) / period as f64;

            // Calculate dimension
            let d = if n1 > 0.0 && n2 > 0.0 && n3 > 0.0 {
                ((n1 + n2).ln() - n3.ln()) / 2.0_f64.ln()
            } else {
                1.0
            };

            // Calculate alpha
            let alpha = (-4.6 * (d - 1.0)).exp();
            let alpha = alpha.clamp(0.01, 1.0);

            // Calculate FRAMA
            if i == period - 1 {
                result.push(Some(close[i]));
            } else {
                let prev = result[i - 1].unwrap_or(close[i]);
                result.push(Some(alpha * close[i] + (1.0 - alpha) * prev));
            }
        }
    }

    result
}

/// T3 Moving Average (Tilson)
pub fn t3(source: &[f64], period: usize, volume_factor: f64) -> Vec<Option<f64>> {
    let ema1 = ema(source, period);
    let e1: Vec<f64> = ema1.iter().map(|v| v.unwrap_or(0.0)).collect();

    let ema2 = ema(&e1, period);
    let e2: Vec<f64> = ema2.iter().map(|v| v.unwrap_or(0.0)).collect();

    let ema3 = ema(&e2, period);
    let e3: Vec<f64> = ema3.iter().map(|v| v.unwrap_or(0.0)).collect();

    let ema4 = ema(&e3, period);
    let e4: Vec<f64> = ema4.iter().map(|v| v.unwrap_or(0.0)).collect();

    let ema5 = ema(&e4, period);
    let e5: Vec<f64> = ema5.iter().map(|v| v.unwrap_or(0.0)).collect();

    let ema6 = ema(&e5, period);

    let c1 = -volume_factor.powi(3);
    let c2 = 3.0 * volume_factor.powi(2) + 3.0 * volume_factor.powi(3);
    let c3 = -6.0 * volume_factor.powi(2) - 3.0 * volume_factor - 3.0 * volume_factor.powi(3);
    let c4 = 1.0 + 3.0 * volume_factor + volume_factor.powi(3) + 3.0 * volume_factor.powi(2);

    ema3.iter()
        .zip(ema4.iter())
        .zip(ema5.iter())
        .zip(ema6.iter())
        .map(
            |(((e3_opt, e4_opt), e5_opt), e6_opt)| match (e3_opt, e4_opt, e5_opt, e6_opt) {
                (Some(e3_v), Some(e4_v), Some(e5_v), Some(e6_v)) => {
                    Some(c1 * e6_v + c2 * e5_v + c3 * e4_v + c4 * e3_v)
                }
                _ => None,
            },
        )
        .collect()
}

/// Variable Index Dynamic Average (VIDYA)
pub fn vidya(source: &[f64], period: usize, cmo_period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if source.len() < cmo_period.max(period) {
        return vec![None; source.len()];
    }

    let alpha = 2.0 / (period as f64 + 1.0);

    for i in 0..source.len() {
        if i < cmo_period {
            result.push(None);
        } else {
            // Calculate CMO (Chande Momentum Oscillator)
            let mut sum_up = 0.0;
            let mut sum_down = 0.0;

            for j in (i - cmo_period + 1)..=i {
                let diff = source[j] - source[j - 1];
                if diff > 0.0 {
                    sum_up += diff;
                } else {
                    sum_down += diff.abs();
                }
            }

            let cmo = if sum_up + sum_down != 0.0 {
                (sum_up - sum_down) / (sum_up + sum_down)
            } else {
                0.0
            };

            // VIDYA calculation
            if i == cmo_period {
                result.push(Some(source[i]));
            } else {
                let prev = result[i - 1].unwrap_or(source[i]);
                let vidya_val = alpha * cmo.abs() * source[i] + (1.0 - alpha * cmo.abs()) * prev;
                result.push(Some(vidya_val));
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&data, 3);

        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(2.0)); // (1+2+3)/3
        assert_eq!(result[3], Some(3.0)); // (2+3+4)/3
        assert_eq!(result[4], Some(4.0)); // (3+4+5)/3
    }

    #[test]
    fn test_ema() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ema(&data, 3);

        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert!(result[2].is_some());
        assert!(result[3].is_some());
        assert!(result[4].is_some());
    }

    #[test]
    fn test_alma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = alma(&data, 5, 0.85, 6.0);

        assert_eq!(result[0], None);
        assert_eq!(result[3], None);
        assert!(result[4].is_some());
        assert!(result[9].is_some());
    }

    #[test]
    fn test_jma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = jma(&data, 5, 0.0, 2.0);

        assert_eq!(result[0], None);
        assert_eq!(result[3], None);
        assert!(result[4].is_some());
        assert!(result[9].is_some());
    }

    #[test]
    fn test_hma() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = hma(&data, 5);

        // HMA should have values after warmup period
        assert!(result.iter().skip(4).any(|v| v.is_some()));
    }
}
