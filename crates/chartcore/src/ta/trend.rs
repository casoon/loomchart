// Trend Indicators

use super::{atr, highest, lowest, rma};

/// ADX (Average Directional Index)
/// Returns (adx, plus_di, minus_di)
pub fn adx(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    di_period: usize,
    adx_period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = high.len();
    let mut plus_dm = vec![0.0; len];
    let mut minus_dm = vec![0.0; len];

    // Calculate +DM and -DM
    for i in 1..len {
        let up_move = high[i] - high[i - 1];
        let down_move = low[i - 1] - low[i];

        if up_move > down_move && up_move > 0.0 {
            plus_dm[i] = up_move;
        }
        if down_move > up_move && down_move > 0.0 {
            minus_dm[i] = down_move;
        }
    }

    // Smooth DM and TR
    let smoothed_plus_dm = rma(&plus_dm, di_period);
    let smoothed_minus_dm = rma(&minus_dm, di_period);
    let smoothed_tr = atr(high, low, close, di_period);

    // Calculate +DI and -DI
    let mut plus_di = Vec::with_capacity(len);
    let mut minus_di = Vec::with_capacity(len);
    let mut dx = Vec::with_capacity(len);

    for i in 0..len {
        match (smoothed_plus_dm[i], smoothed_minus_dm[i], smoothed_tr[i]) {
            (Some(pdm), Some(mdm), Some(tr)) if tr > 0.0 => {
                let pdi = (pdm / tr) * 100.0;
                let mdi = (mdm / tr) * 100.0;
                plus_di.push(Some(pdi));
                minus_di.push(Some(mdi));

                let sum = pdi + mdi;
                if sum > 0.0 {
                    dx.push((pdi - mdi).abs() / sum * 100.0);
                } else {
                    dx.push(0.0);
                }
            }
            _ => {
                plus_di.push(None);
                minus_di.push(None);
                dx.push(0.0);
            }
        }
    }

    // Smooth DX to get ADX
    let adx_values = rma(&dx, adx_period);

    (adx_values, plus_di, minus_di)
}

/// DMI (Directional Movement Index)
/// Returns (plus_di, minus_di)
pub fn dmi(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let (_, plus_di, minus_di) = adx(high, low, close, period, period);
    (plus_di, minus_di)
}

/// Aroon Indicator
/// Returns (aroon_up, aroon_down, aroon_oscillator)
pub fn aroon(
    high: &[f64],
    low: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = high.len();
    let mut aroon_up = Vec::with_capacity(len);
    let mut aroon_down = Vec::with_capacity(len);
    let mut aroon_osc = Vec::with_capacity(len);

    for i in 0..len {
        if i < period {
            aroon_up.push(None);
            aroon_down.push(None);
            aroon_osc.push(None);
        } else {
            // Find bars since highest high
            let mut bars_since_high = 0;
            let mut highest_val = f64::NEG_INFINITY;
            for j in 0..=period {
                let idx = i - j;
                if high[idx] > highest_val {
                    highest_val = high[idx];
                    bars_since_high = j;
                }
            }

            // Find bars since lowest low
            let mut bars_since_low = 0;
            let mut lowest_val = f64::INFINITY;
            for j in 0..=period {
                let idx = i - j;
                if low[idx] < lowest_val {
                    lowest_val = low[idx];
                    bars_since_low = j;
                }
            }

            let up = ((period - bars_since_high) as f64 / period as f64) * 100.0;
            let down = ((period - bars_since_low) as f64 / period as f64) * 100.0;

            aroon_up.push(Some(up));
            aroon_down.push(Some(down));
            aroon_osc.push(Some(up - down));
        }
    }

    (aroon_up, aroon_down, aroon_osc)
}

/// Supertrend
/// Returns (supertrend, direction) where direction: -1 = bullish, 1 = bearish
pub fn supertrend(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    atr_period: usize,
    multiplier: f64,
) -> (Vec<Option<f64>>, Vec<Option<i8>>) {
    let len = high.len();
    let atr_values = atr(high, low, close, atr_period);

    let mut supertrend = Vec::with_capacity(len);
    let mut direction: Vec<Option<i8>> = Vec::with_capacity(len);

    let mut upper_band = vec![0.0; len];
    let mut lower_band = vec![0.0; len];
    let mut final_upper = vec![0.0; len];
    let mut final_lower = vec![0.0; len];

    for i in 0..len {
        let hl2 = (high[i] + low[i]) / 2.0;

        if let Some(atr_val) = atr_values[i] {
            upper_band[i] = hl2 + multiplier * atr_val;
            lower_band[i] = hl2 - multiplier * atr_val;
        } else {
            upper_band[i] = hl2;
            lower_band[i] = hl2;
        }

        if i == 0 {
            final_upper[i] = upper_band[i];
            final_lower[i] = lower_band[i];
            supertrend.push(None);
            direction.push(None);
        } else {
            // Final upper band
            final_upper[i] =
                if upper_band[i] < final_upper[i - 1] || close[i - 1] > final_upper[i - 1] {
                    upper_band[i]
                } else {
                    final_upper[i - 1]
                };

            // Final lower band
            final_lower[i] =
                if lower_band[i] > final_lower[i - 1] || close[i - 1] < final_lower[i - 1] {
                    lower_band[i]
                } else {
                    final_lower[i - 1]
                };

            // Determine direction and supertrend value
            let _prev_dir = direction[i - 1].unwrap_or(1);
            let prev_st = supertrend[i - 1].unwrap_or(final_upper[i]);

            let (new_dir, new_st) = if prev_st == final_upper[i - 1] {
                if close[i] > final_upper[i] {
                    (-1_i8, final_lower[i])
                } else {
                    (1_i8, final_upper[i])
                }
            } else {
                if close[i] < final_lower[i] {
                    (1_i8, final_upper[i])
                } else {
                    (-1_i8, final_lower[i])
                }
            };

            supertrend.push(Some(new_st));
            direction.push(Some(new_dir));
        }
    }

    (supertrend, direction)
}

/// Parabolic SAR
pub fn parabolic_sar(
    high: &[f64],
    low: &[f64],
    start_af: f64,
    max_af: f64,
    step_af: f64,
) -> Vec<Option<f64>> {
    let len = high.len();
    if len < 2 {
        return vec![None; len];
    }

    let mut sar = Vec::with_capacity(len);
    let mut is_uptrend = true;
    let mut af = start_af;
    let mut ep = low[0]; // Extreme point
    let mut sar_value = high[0];

    sar.push(None);

    for i in 1..len {
        // Calculate new SAR
        let prev_sar = sar_value;
        sar_value = prev_sar + af * (ep - prev_sar);

        if is_uptrend {
            // In uptrend, SAR should not be above prior two lows
            if i >= 2 {
                sar_value = sar_value.min(low[i - 1]).min(low[i - 2]);
            } else if i >= 1 {
                sar_value = sar_value.min(low[i - 1]);
            }

            // Check for reversal
            if low[i] < sar_value {
                is_uptrend = false;
                sar_value = ep;
                ep = low[i];
                af = start_af;
            } else {
                // Update EP if new high
                if high[i] > ep {
                    ep = high[i];
                    af = (af + step_af).min(max_af);
                }
            }
        } else {
            // In downtrend, SAR should not be below prior two highs
            if i >= 2 {
                sar_value = sar_value.max(high[i - 1]).max(high[i - 2]);
            } else if i >= 1 {
                sar_value = sar_value.max(high[i - 1]);
            }

            // Check for reversal
            if high[i] > sar_value {
                is_uptrend = true;
                sar_value = ep;
                ep = high[i];
                af = start_af;
            } else {
                // Update EP if new low
                if low[i] < ep {
                    ep = low[i];
                    af = (af + step_af).min(max_af);
                }
            }
        }

        sar.push(Some(sar_value));
    }

    sar
}

/// Ichimoku Cloud
/// Returns (tenkan, kijun, senkou_a, senkou_b, chikou)
pub fn ichimoku(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    displacement: usize,
) -> (
    Vec<Option<f64>>,
    Vec<Option<f64>>,
    Vec<Option<f64>>,
    Vec<Option<f64>>,
    Vec<Option<f64>>,
) {
    let len = high.len();

    // Donchian midline helper
    let donchian_mid = |h: &[f64], l: &[f64], period: usize| -> Vec<Option<f64>> {
        let hh = highest(h, period);
        let ll = lowest(l, period);
        hh.iter()
            .zip(ll.iter())
            .map(|(&h, &l)| match (h, l) {
                (Some(hv), Some(lv)) => Some((hv + lv) / 2.0),
                _ => None,
            })
            .collect()
    };

    let tenkan = donchian_mid(high, low, tenkan_period);
    let kijun = donchian_mid(high, low, kijun_period);

    // Senkou Span A = (Tenkan + Kijun) / 2, shifted forward
    let senkou_a_raw: Vec<Option<f64>> = tenkan
        .iter()
        .zip(kijun.iter())
        .map(|(&t, &k)| match (t, k) {
            (Some(tv), Some(kv)) => Some((tv + kv) / 2.0),
            _ => None,
        })
        .collect();

    // Senkou Span B = Donchian(52), shifted forward
    let senkou_b_raw = donchian_mid(high, low, senkou_b_period);

    // Shift Senkou spans forward by displacement
    let mut senkou_a = vec![None; len];
    let mut senkou_b = vec![None; len];

    for i in 0..len {
        let target = i + displacement - 1;
        if target < len {
            senkou_a[target] = senkou_a_raw[i];
            senkou_b[target] = senkou_b_raw[i];
        }
    }

    // Chikou Span = Close shifted backward
    let mut chikou = vec![None; len];
    for i in 0..len {
        if i + displacement - 1 < len {
            chikou[i] = Some(close[i + displacement - 1]);
        }
    }

    (tenkan, kijun, senkou_a, senkou_b, chikou)
}

/// Vortex Indicator
/// Returns (vi_plus, vi_minus)
pub fn vortex(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = high.len();
    let mut vi_plus = Vec::with_capacity(len);
    let mut vi_minus = Vec::with_capacity(len);

    for i in 0..len {
        if i < period {
            vi_plus.push(None);
            vi_minus.push(None);
        } else {
            let mut vm_plus = 0.0;
            let mut vm_minus = 0.0;
            let mut tr_sum = 0.0;

            for j in (i - period + 1)..=i {
                // Vortex Movement
                vm_plus += (high[j] - low[j - 1]).abs();
                vm_minus += (low[j] - high[j - 1]).abs();

                // True Range
                let hl = high[j] - low[j];
                let hc = (high[j] - close[j - 1]).abs();
                let lc = (low[j] - close[j - 1]).abs();
                tr_sum += hl.max(hc).max(lc);
            }

            if tr_sum > 0.0 {
                vi_plus.push(Some(vm_plus / tr_sum));
                vi_minus.push(Some(vm_minus / tr_sum));
            } else {
                vi_plus.push(None);
                vi_minus.push(None);
            }
        }
    }

    (vi_plus, vi_minus)
}

/// ZigZag indicator
/// Returns vector of (index, price, is_high) for pivots
pub fn zigzag(high: &[f64], low: &[f64], deviation: f64) -> Vec<(usize, f64, bool)> {
    let len = high.len();
    if len < 2 {
        return vec![];
    }

    let mut pivots = Vec::new();
    let dev_pct = deviation / 100.0;

    let mut last_pivot_idx = 0;
    let mut last_pivot_price = (high[0] + low[0]) / 2.0;
    let mut is_looking_for_high = true;

    for i in 1..len {
        if is_looking_for_high {
            if high[i] > last_pivot_price * (1.0 + dev_pct) {
                // Found a new high
                if !pivots.is_empty() {
                    // Update last pivot to be a low
                    pivots.push((last_pivot_idx, last_pivot_price, false));
                }
                last_pivot_idx = i;
                last_pivot_price = high[i];
                is_looking_for_high = false;
            } else if low[i] < last_pivot_price * (1.0 - dev_pct) {
                // Continue looking for low
                last_pivot_idx = i;
                last_pivot_price = low[i];
            }
        } else {
            if low[i] < last_pivot_price * (1.0 - dev_pct) {
                // Found a new low
                pivots.push((last_pivot_idx, last_pivot_price, true));
                last_pivot_idx = i;
                last_pivot_price = low[i];
                is_looking_for_high = true;
            } else if high[i] > last_pivot_price * (1.0 + dev_pct) {
                // Continue looking for high
                last_pivot_idx = i;
                last_pivot_price = high[i];
            }
        }
    }

    // Add last pivot
    pivots.push((last_pivot_idx, last_pivot_price, !is_looking_for_high));

    pivots
}

/// Trend Strength Index
pub fn trend_strength(close: &[f64], period: usize) -> Vec<Option<f64>> {
    let len = close.len();
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        if i < period {
            result.push(None);
        } else {
            let mut up_count = 0;
            let mut down_count = 0;

            for j in (i - period + 1)..=i {
                if close[j] > close[j - 1] {
                    up_count += 1;
                } else if close[j] < close[j - 1] {
                    down_count += 1;
                }
            }

            let total = up_count + down_count;
            if total > 0 {
                let strength = ((up_count as f64 - down_count as f64) / total as f64) * 100.0;
                result.push(Some(strength));
            } else {
                result.push(Some(0.0));
            }
        }
    }

    result
}

/// Chande Kroll Stop
/// Returns (stop_long, stop_short)
pub fn chande_kroll_stop(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    p: usize,
    q: usize,
    x: f64,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = high.len();
    let atr_values = atr(high, low, close, p);

    let mut first_high_stop = vec![0.0; len];
    let mut first_low_stop = vec![0.0; len];

    // First High/Low Stop
    for i in 0..len {
        if i < p - 1 {
            first_high_stop[i] = high[i];
            first_low_stop[i] = low[i];
        } else {
            let hh = highest(high, p)[i].unwrap_or(high[i]);
            let ll = lowest(low, p)[i].unwrap_or(low[i]);
            let atr_val = atr_values[i].unwrap_or(0.0);

            first_high_stop[i] = hh - x * atr_val;
            first_low_stop[i] = ll + x * atr_val;
        }
    }

    // Stop Long/Short (highest/lowest of first stops over q periods)
    let stop_long = highest(&first_high_stop, q);
    let stop_short = lowest(&first_low_stop, q);

    (stop_long, stop_short)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adx() {
        let high = vec![
            48.0, 48.5, 49.0, 49.5, 50.0, 50.5, 51.0, 51.5, 52.0, 52.5, 53.0, 53.5, 54.0, 54.5,
            55.0,
        ];
        let low = vec![
            46.0, 46.5, 47.0, 47.5, 48.0, 48.5, 49.0, 49.5, 50.0, 50.5, 51.0, 51.5, 52.0, 52.5,
            53.0,
        ];
        let close = vec![
            47.0, 47.5, 48.0, 48.5, 49.0, 49.5, 50.0, 50.5, 51.0, 51.5, 52.0, 52.5, 53.0, 53.5,
            54.0,
        ];

        let (adx, plus_di, minus_di) = adx(&high, &low, &close, 5, 5);

        // Should have values after warmup
        assert!(adx.iter().skip(5).any(|v| v.is_some()));
        assert!(plus_di.iter().skip(5).any(|v| v.is_some()));
        assert!(minus_di.iter().skip(5).any(|v| v.is_some()));
    }

    #[test]
    fn test_aroon() {
        let high = vec![5.0, 6.0, 7.0, 8.0, 9.0, 8.0, 7.0, 6.0, 5.0, 4.0];
        let low = vec![4.0, 5.0, 6.0, 7.0, 8.0, 7.0, 6.0, 5.0, 4.0, 3.0];

        let (up, down, osc) = aroon(&high, &low, 5);

        assert!(up[5].is_some());
        assert!(down[5].is_some());
        assert!(osc[5].is_some());
    }

    #[test]
    fn test_supertrend() {
        let high = vec![50.0, 51.0, 52.0, 53.0, 54.0, 53.0, 52.0, 51.0, 50.0, 49.0];
        let low = vec![48.0, 49.0, 50.0, 51.0, 52.0, 51.0, 50.0, 49.0, 48.0, 47.0];
        let close = vec![49.0, 50.0, 51.0, 52.0, 53.0, 52.0, 51.0, 50.0, 49.0, 48.0];

        let (st, dir) = supertrend(&high, &low, &close, 3, 2.0);

        assert!(st.iter().skip(3).any(|v| v.is_some()));
        assert!(dir.iter().skip(3).any(|v| v.is_some()));
    }
}
