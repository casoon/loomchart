// Momentum Indicators

use super::{change, ema, rma, roc, sma, wma};

/// RSI (Relative Strength Index)
pub fn rsi(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if source.len() < period + 1 {
        return vec![None; source.len()];
    }

    // Calculate price changes
    let changes = change(source, 1);

    // Separate gains and losses
    let gains: Vec<f64> = changes
        .iter()
        .map(|&c| {
            if let Some(val) = c {
                if val > 0.0 {
                    val
                } else {
                    0.0
                }
            } else {
                0.0
            }
        })
        .collect();

    let losses: Vec<f64> = changes
        .iter()
        .map(|&c| {
            if let Some(val) = c {
                if val < 0.0 {
                    val.abs()
                } else {
                    0.0
                }
            } else {
                0.0
            }
        })
        .collect();

    // Apply RMA (Wilder's smoothing)
    let avg_gain = rma(&gains, period);
    let avg_loss = rma(&losses, period);

    // Calculate RSI
    for i in 0..source.len() {
        if let (Some(gain), Some(loss)) = (avg_gain[i], avg_loss[i]) {
            let rs = if loss == 0.0 { 100.0 } else { gain / loss };
            result.push(Some(100.0 - (100.0 / (1.0 + rs))));
        } else {
            result.push(None);
        }
    }

    result
}

/// MACD (Moving Average Convergence Divergence)
/// Returns (macd_line, signal_line, histogram)
pub fn macd(
    source: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let fast_ema = ema(source, fast_period);
    let slow_ema = ema(source, slow_period);

    // MACD Line = Fast EMA - Slow EMA
    let macd_line: Vec<Option<f64>> = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(&f, &s)| match (f, s) {
            (Some(fv), Some(sv)) => Some(fv - sv),
            _ => None,
        })
        .collect();

    // Convert to f64 for signal EMA
    let macd_values: Vec<f64> = macd_line.iter().map(|v| v.unwrap_or(0.0)).collect();
    let signal_line = ema(&macd_values, signal_period);

    // Histogram = MACD - Signal
    let histogram: Vec<Option<f64>> = macd_line
        .iter()
        .zip(signal_line.iter())
        .map(|(&m, &s)| match (m, s) {
            (Some(mv), Some(sv)) => Some(mv - sv),
            _ => None,
        })
        .collect();

    (macd_line, signal_line, histogram)
}

/// Stochastic %K
pub fn stochastic_k(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(close.len());

    for i in 0..close.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let highest = high[start..=i]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);

            let lowest = low[start..=i].iter().copied().fold(f64::INFINITY, f64::min);

            let range = highest - lowest;
            if range > 0.0 {
                let k = ((close[i] - lowest) / range) * 100.0;
                result.push(Some(k));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Stochastic Oscillator
/// Returns (%K, %D)
pub fn stochastic(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    k_period: usize,
    k_smooth: usize,
    d_period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let raw_k = stochastic_k(high, low, close, k_period);
    let k_values: Vec<f64> = raw_k.iter().map(|v| v.unwrap_or(0.0)).collect();

    let k = sma(&k_values, k_smooth);
    let k_for_d: Vec<f64> = k.iter().map(|v| v.unwrap_or(0.0)).collect();
    let d = sma(&k_for_d, d_period);

    (k, d)
}

/// Stochastic RSI
/// Returns (%K, %D)
pub fn stoch_rsi(
    source: &[f64],
    rsi_period: usize,
    stoch_period: usize,
    k_smooth: usize,
    d_smooth: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let rsi_values = rsi(source, rsi_period);
    let rsi_f64: Vec<f64> = rsi_values.iter().map(|v| v.unwrap_or(50.0)).collect();

    // Apply stochastic to RSI
    let len = source.len();
    let mut stoch_rsi = Vec::with_capacity(len);

    for i in 0..len {
        if i < stoch_period - 1 || rsi_values[i].is_none() {
            stoch_rsi.push(0.0);
        } else {
            let highest = rsi_f64[i - stoch_period + 1..=i]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let lowest = rsi_f64[i - stoch_period + 1..=i]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);

            let range = highest - lowest;
            if range > 0.0 {
                stoch_rsi.push(((rsi_f64[i] - lowest) / range) * 100.0);
            } else {
                stoch_rsi.push(50.0);
            }
        }
    }

    let k = sma(&stoch_rsi, k_smooth);
    let k_values: Vec<f64> = k.iter().map(|v| v.unwrap_or(0.0)).collect();
    let d = sma(&k_values, d_smooth);

    (k, d)
}

/// CCI (Commodity Channel Index)
pub fn cci(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(close.len());

    // Calculate typical price
    let typical: Vec<f64> = (0..close.len())
        .map(|i| (high[i] + low[i] + close[i]) / 3.0)
        .collect();

    for i in 0..close.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            // SMA of typical price
            let sma: f64 = typical[start..=i].iter().sum::<f64>() / period as f64;

            // Mean deviation
            let mean_dev: f64 = typical[start..=i]
                .iter()
                .map(|&tp| (tp - sma).abs())
                .sum::<f64>()
                / period as f64;

            if mean_dev != 0.0 {
                let cci = (typical[i] - sma) / (0.015 * mean_dev);
                result.push(Some(cci));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Williams %R
pub fn williams_r(high: &[f64], low: &[f64], close: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(close.len());

    for i in 0..close.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let highest = high[start..=i]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);

            let lowest = low[start..=i]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);

            let range = highest - lowest;
            if range > 0.0 {
                let wr = ((highest - close[i]) / range) * -100.0;
                result.push(Some(wr));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Momentum (Price change over period)
pub fn momentum(source: &[f64], period: usize) -> Vec<Option<f64>> {
    change(source, period)
}

/// Awesome Oscillator
pub fn awesome_oscillator(
    high: &[f64],
    low: &[f64],
    fast_period: usize,
    slow_period: usize,
) -> Vec<Option<f64>> {
    // Midpoint (HL2)
    let midpoint: Vec<f64> = high
        .iter()
        .zip(low.iter())
        .map(|(&h, &l)| (h + l) / 2.0)
        .collect();

    let fast_sma = sma(&midpoint, fast_period);
    let slow_sma = sma(&midpoint, slow_period);

    fast_sma
        .iter()
        .zip(slow_sma.iter())
        .map(|(&f, &s)| match (f, s) {
            (Some(fv), Some(sv)) => Some(fv - sv),
            _ => None,
        })
        .collect()
}

/// TSI (True Strength Index)
pub fn tsi(
    source: &[f64],
    long_period: usize,
    short_period: usize,
    signal_period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = source.len();

    // Price changes
    let mut pc = vec![0.0; len];
    let mut abs_pc = vec![0.0; len];

    for i in 1..len {
        pc[i] = source[i] - source[i - 1];
        abs_pc[i] = pc[i].abs();
    }

    // Double smooth price change
    let pc_ema1 = ema(&pc, long_period);
    let pc_ema1_vals: Vec<f64> = pc_ema1.iter().map(|v| v.unwrap_or(0.0)).collect();
    let double_smooth_pc = ema(&pc_ema1_vals, short_period);

    // Double smooth absolute price change
    let abs_pc_ema1 = ema(&abs_pc, long_period);
    let abs_pc_ema1_vals: Vec<f64> = abs_pc_ema1.iter().map(|v| v.unwrap_or(0.0)).collect();
    let double_smooth_abs_pc = ema(&abs_pc_ema1_vals, short_period);

    // TSI
    let mut tsi_values = Vec::with_capacity(len);
    for i in 0..len {
        match (double_smooth_pc[i], double_smooth_abs_pc[i]) {
            (Some(pc_val), Some(abs_val)) if abs_val != 0.0 => {
                tsi_values.push(Some((pc_val / abs_val) * 100.0));
            }
            _ => tsi_values.push(None),
        }
    }

    // Signal
    let tsi_f64: Vec<f64> = tsi_values.iter().map(|v| v.unwrap_or(0.0)).collect();
    let signal = ema(&tsi_f64, signal_period);

    (tsi_values, signal)
}

/// RVI (Relative Vigor Index)
/// Returns (rvi, signal)
pub fn rvi(
    open: &[f64],
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = close.len();

    // Numerator: close - open (smoothed)
    let mut num = vec![0.0; len];
    // Denominator: high - low (smoothed)
    let mut denom = vec![0.0; len];

    for i in 3..len {
        // Symmetrically weighted moving average
        num[i] = ((close[i] - open[i])
            + 2.0 * (close[i - 1] - open[i - 1])
            + 2.0 * (close[i - 2] - open[i - 2])
            + (close[i - 3] - open[i - 3]))
            / 6.0;
        denom[i] = ((high[i] - low[i])
            + 2.0 * (high[i - 1] - low[i - 1])
            + 2.0 * (high[i - 2] - low[i - 2])
            + (high[i - 3] - low[i - 3]))
            / 6.0;
    }

    // Sum over period
    let mut rvi_values = Vec::with_capacity(len);
    for i in 0..len {
        if i < period + 2 {
            rvi_values.push(None);
        } else {
            let start = i + 1 - period;
            let num_sum: f64 = num[start..=i].iter().sum();
            let denom_sum: f64 = denom[start..=i].iter().sum();

            if denom_sum != 0.0 {
                rvi_values.push(Some(num_sum / denom_sum));
            } else {
                rvi_values.push(None);
            }
        }
    }

    // Signal (symmetrically weighted MA of RVI)
    let mut signal = Vec::with_capacity(len);
    for i in 0..len {
        if i < 3 {
            signal.push(None);
        } else {
            match (
                rvi_values[i],
                rvi_values[i - 1],
                rvi_values[i - 2],
                rvi_values[i - 3],
            ) {
                (Some(r0), Some(r1), Some(r2), Some(r3)) => {
                    signal.push(Some((r0 + 2.0 * r1 + 2.0 * r2 + r3) / 6.0));
                }
                _ => signal.push(None),
            }
        }
    }

    (rvi_values, signal)
}

/// Chande Momentum Oscillator
pub fn chande_mo(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let mut sum_up = 0.0;
            let mut sum_down = 0.0;

            for j in start..=i {
                let diff = source[j] - source[j - 1];
                if diff > 0.0 {
                    sum_up += diff;
                } else {
                    sum_down += diff.abs();
                }
            }

            let total = sum_up + sum_down;
            if total != 0.0 {
                result.push(Some(((sum_up - sum_down) / total) * 100.0));
            } else {
                result.push(Some(0.0));
            }
        }
    }

    result
}

/// Coppock Curve
pub fn coppock_curve(
    source: &[f64],
    wma_period: usize,
    long_roc: usize,
    short_roc: usize,
) -> Vec<Option<f64>> {
    let long_roc_values = roc(source, long_roc);
    let short_roc_values = roc(source, short_roc);

    // Sum of ROCs
    let sum: Vec<f64> = long_roc_values
        .iter()
        .zip(short_roc_values.iter())
        .map(|(&l, &s)| l.unwrap_or(0.0) + s.unwrap_or(0.0))
        .collect();

    wma(&sum, wma_period)
}

/// Detrended Price Oscillator
pub fn dpo(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let ma = sma(source, period);
    let shift = period / 2 + 1;

    let mut result = Vec::with_capacity(source.len());
    for i in 0..source.len() {
        if i < shift || ma[i - shift].is_none() {
            result.push(None);
        } else {
            result.push(Some(source[i] - ma[i - shift].unwrap()));
        }
    }

    result
}

/// Price Oscillator
pub fn price_oscillator(
    source: &[f64],
    short_period: usize,
    long_period: usize,
    use_percent: bool,
) -> Vec<Option<f64>> {
    let short_ma = ema(source, short_period);
    let long_ma = ema(source, long_period);

    short_ma
        .iter()
        .zip(long_ma.iter())
        .map(|(&s, &l)| match (s, l) {
            (Some(sv), Some(lv)) => {
                if use_percent && lv != 0.0 {
                    Some((sv - lv) / lv * 100.0)
                } else {
                    Some(sv - lv)
                }
            }
            _ => None,
        })
        .collect()
}

/// SMI Ergodic Indicator
/// Returns (smi, signal)
pub fn smi_ergodic(
    source: &[f64],
    long_period: usize,
    short_period: usize,
    signal_period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    tsi(source, long_period, short_period, signal_period)
}

/// SMI Ergodic Oscillator (histogram)
pub fn smi_ergodic_oscillator(
    source: &[f64],
    long_period: usize,
    short_period: usize,
    signal_period: usize,
) -> Vec<Option<f64>> {
    let (smi, signal) = smi_ergodic(source, long_period, short_period, signal_period);

    smi.iter()
        .zip(signal.iter())
        .map(|(&s, &sig)| match (s, sig) {
            (Some(sv), Some(sigv)) => Some(sv - sigv),
            _ => None,
        })
        .collect()
}

/// Ultimate Oscillator
pub fn ultimate_oscillator(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    short_period: usize,
    medium_period: usize,
    long_period: usize,
) -> Vec<Option<f64>> {
    let len = close.len();
    let mut result = Vec::with_capacity(len);

    // Calculate BP (Buying Pressure) and TR (True Range)
    let mut bp = vec![0.0; len];
    let mut tr = vec![0.0; len];

    for i in 0..len {
        if i == 0 {
            bp[i] = close[i] - low[i];
            tr[i] = high[i] - low[i];
        } else {
            let true_low = low[i].min(close[i - 1]);
            let true_high = high[i].max(close[i - 1]);
            bp[i] = close[i] - true_low;
            tr[i] = true_high - true_low;
        }
    }

    for i in 0..len {
        if i < long_period - 1 {
            result.push(None);
        } else {
            let bp_short: f64 = bp[i - short_period + 1..=i].iter().sum();
            let tr_short: f64 = tr[i - short_period + 1..=i].iter().sum();

            let bp_medium: f64 = bp[i - medium_period + 1..=i].iter().sum();
            let tr_medium: f64 = tr[i - medium_period + 1..=i].iter().sum();

            let bp_long: f64 = bp[i - long_period + 1..=i].iter().sum();
            let tr_long: f64 = tr[i - long_period + 1..=i].iter().sum();

            if tr_short > 0.0 && tr_medium > 0.0 && tr_long > 0.0 {
                let avg1 = bp_short / tr_short;
                let avg2 = bp_medium / tr_medium;
                let avg3 = bp_long / tr_long;

                // Weighted average (4, 2, 1)
                let uo = 100.0 * (4.0 * avg1 + 2.0 * avg2 + avg3) / 7.0;
                result.push(Some(uo));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Balance of Power
pub fn bop(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
    open.iter()
        .zip(high.iter())
        .zip(low.iter())
        .zip(close.iter())
        .map(|(((&o, &h), &l), &c)| {
            let range = h - l;
            if range > 0.0 {
                (c - o) / range
            } else {
                0.0
            }
        })
        .collect()
}

/// Bull Bear Power
/// Returns (bull_power, bear_power)
pub fn bull_bear_power(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let ema_values = ema(close, period);

    let bull: Vec<Option<f64>> = ema_values
        .iter()
        .enumerate()
        .map(|(i, &e)| e.map(|ev| high[i] - ev))
        .collect();

    let bear: Vec<Option<f64>> = ema_values
        .iter()
        .enumerate()
        .map(|(i, &e)| e.map(|ev| low[i] - ev))
        .collect();

    (bull, bear)
}

/// Fisher Transform
pub fn fisher_transform(
    high: &[f64],
    low: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = high.len();
    let mut fisher = Vec::with_capacity(len);
    let mut trigger = Vec::with_capacity(len);

    let hl2: Vec<f64> = high
        .iter()
        .zip(low.iter())
        .map(|(&h, &l)| (h + l) / 2.0)
        .collect();

    let mut value = 0.0;
    let mut prev_fisher = 0.0;

    for i in 0..len {
        if i + 1 < period {
            fisher.push(None);
            trigger.push(None);
        } else {
            let start = i + 1 - period;
            let hh = hl2[start..=i]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let ll = hl2[start..=i]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);

            let range = hh - ll;
            if range > 0.0 {
                let raw = 2.0 * ((hl2[i] - ll) / range - 0.5);
                value = 0.66 * raw.clamp(-0.999, 0.999) + 0.67 * value;
                value = value.clamp(-0.999, 0.999);

                let fish = 0.5 * ((1.0 + value) / (1.0 - value)).ln() + 0.5 * prev_fisher;
                fisher.push(Some(fish));
                trigger.push(Some(prev_fisher));
                prev_fisher = fish;
            } else {
                fisher.push(Some(prev_fisher));
                trigger.push(Some(prev_fisher));
            }
        }
    }

    (fisher, trigger)
}

/// Williams Alligator
/// Returns (jaw, teeth, lips)
pub fn williams_alligator(
    high: &[f64],
    low: &[f64],
    jaw_period: usize,
    jaw_offset: usize,
    teeth_period: usize,
    teeth_offset: usize,
    lips_period: usize,
    lips_offset: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = high.len();

    // Median price (HL2)
    let hl2: Vec<f64> = high
        .iter()
        .zip(low.iter())
        .map(|(&h, &l)| (h + l) / 2.0)
        .collect();

    // SMMA (same as RMA/Wilder's)
    let jaw_raw = rma(&hl2, jaw_period);
    let teeth_raw = rma(&hl2, teeth_period);
    let lips_raw = rma(&hl2, lips_period);

    // Apply offsets (shift forward)
    let mut jaw = vec![None; len];
    let mut teeth = vec![None; len];
    let mut lips = vec![None; len];

    for i in 0..len {
        if i + jaw_offset < len {
            jaw[i + jaw_offset] = jaw_raw[i];
        }
        if i + teeth_offset < len {
            teeth[i + teeth_offset] = teeth_raw[i];
        }
        if i + lips_offset < len {
            lips[i + lips_offset] = lips_raw[i];
        }
    }

    (jaw, teeth, lips)
}

/// Woodies CCI
/// Returns (cci, turbo_cci)
pub fn woodies_cci(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    cci_period: usize,
    turbo_period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let cci_values = cci(high, low, close, cci_period);
    let turbo_cci = cci(high, low, close, turbo_period);

    (cci_values, turbo_cci)
}

/// Median (Middle value over period)
pub fn median(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let mut window: Vec<f64> = source[start..=i].to_vec();
            window.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let mid = period / 2;
            if period % 2 == 0 {
                result.push(Some((window[mid - 1] + window[mid]) / 2.0));
            } else {
                result.push(Some(window[mid]));
            }
        }
    }

    result
}

/// RCI (Rank Correlation Index)
pub fn rci(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let len = source.len();
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let window: Vec<f64> = source[start..=i].to_vec();

            // Create price ranks (sorted by value)
            let mut indexed: Vec<(usize, f64)> = window.iter().copied().enumerate().collect();
            indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            let mut price_ranks = vec![0usize; period];
            for (rank, (orig_idx, _)) in indexed.iter().enumerate() {
                price_ranks[*orig_idx] = rank + 1;
            }

            // Time ranks (most recent = highest)
            // d = sum of (price_rank - time_rank)^2
            let mut d_sum = 0.0;
            for j in 0..period {
                let time_rank = j + 1;
                let diff = price_ranks[j] as f64 - time_rank as f64;
                d_sum += diff * diff;
            }

            let n = period as f64;
            let rci_val = (1.0 - (6.0 * d_sum) / (n * (n * n - 1.0))) * 100.0;
            result.push(Some(rci_val));
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi() {
        let data = vec![
            44.0, 44.25, 44.5, 43.75, 44.0, 44.5, 45.0, 45.5, 45.0, 46.0, 46.5, 46.0, 45.5, 46.0,
            46.5,
        ];
        let result = rsi(&data, 14);

        for (i, &val) in result.iter().enumerate() {
            if let Some(rsi_val) = val {
                assert!(
                    rsi_val >= 0.0 && rsi_val <= 100.0,
                    "RSI at index {} = {}",
                    i,
                    rsi_val
                );
            }
        }
    }

    #[test]
    fn test_macd() {
        let data: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64) * 0.5).collect();
        let (macd_line, signal, histogram) = macd(&data, 12, 26, 9);

        assert!(macd_line.iter().skip(26).any(|v| v.is_some()));
        assert!(signal.iter().skip(35).any(|v| v.is_some()));
        assert!(histogram.iter().skip(35).any(|v| v.is_some()));
    }

    #[test]
    fn test_stochastic() {
        let high = vec![48.0, 48.5, 49.0, 49.5, 50.0];
        let low = vec![46.0, 46.5, 47.0, 47.5, 48.0];
        let close = vec![47.0, 47.5, 48.0, 48.5, 49.0];

        let (k, d) = stochastic(&high, &low, &close, 3, 1, 3);

        assert!(k[2].is_some());
        if let Some(k_val) = k[2] {
            assert!(k_val >= 0.0 && k_val <= 100.0);
        }
    }
}
