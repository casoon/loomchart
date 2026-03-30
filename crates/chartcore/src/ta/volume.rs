// Volume Indicators

use super::{ema, sma};

/// On Balance Volume
pub fn obv(close: &[f64], volume: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(close.len());

    if close.is_empty() {
        return result;
    }

    result.push(0.0);

    let mut cum_obv = 0.0;
    for i in 1..close.len() {
        if close[i] > close[i - 1] {
            cum_obv += volume[i];
        } else if close[i] < close[i - 1] {
            cum_obv -= volume[i];
        }
        result.push(cum_obv);
    }

    result
}

/// Accumulation/Distribution Line
pub fn ad_line(high: &[f64], low: &[f64], close: &[f64], volume: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(close.len());

    let mut cum_ad = 0.0;
    for i in 0..close.len() {
        let hl_range = high[i] - low[i];
        let mfm = if hl_range > 0.0 {
            ((close[i] - low[i]) - (high[i] - close[i])) / hl_range
        } else {
            0.0
        };
        let mfv = mfm * volume[i];
        cum_ad += mfv;
        result.push(cum_ad);
    }

    result
}

/// Chaikin Money Flow
pub fn chaikin_mf(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    period: usize,
) -> Vec<Option<f64>> {
    let len = close.len();
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let mut mfv_sum = 0.0;
            let mut vol_sum = 0.0;

            for j in start..=i {
                let hl_range = high[j] - low[j];
                let mfm = if hl_range > 0.0 {
                    ((close[j] - low[j]) - (high[j] - close[j])) / hl_range
                } else {
                    0.0
                };
                mfv_sum += mfm * volume[j];
                vol_sum += volume[j];
            }

            if vol_sum > 0.0 {
                result.push(Some(mfv_sum / vol_sum));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Chaikin Oscillator
pub fn chaikin_oscillator(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    fast_period: usize,
    slow_period: usize,
) -> Vec<Option<f64>> {
    let ad = ad_line(high, low, close, volume);
    let fast_ema = ema(&ad, fast_period);
    let slow_ema = ema(&ad, slow_period);

    fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(&f, &s)| match (f, s) {
            (Some(fv), Some(sv)) => Some(fv - sv),
            _ => None,
        })
        .collect()
}

/// Money Flow Index
pub fn mfi(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    period: usize,
) -> Vec<Option<f64>> {
    let len = close.len();
    let mut result = Vec::with_capacity(len);

    // Calculate typical price
    let typical: Vec<f64> = (0..len)
        .map(|i| (high[i] + low[i] + close[i]) / 3.0)
        .collect();

    // Calculate raw money flow
    let money_flow: Vec<f64> = (0..len).map(|i| typical[i] * volume[i]).collect();

    for i in 0..len {
        if i < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let mut positive_mf = 0.0;
            let mut negative_mf = 0.0;

            for j in start..=i {
                if typical[j] > typical[j - 1] {
                    positive_mf += money_flow[j];
                } else if typical[j] < typical[j - 1] {
                    negative_mf += money_flow[j];
                }
            }

            if negative_mf > 0.0 {
                let mfr = positive_mf / negative_mf;
                result.push(Some(100.0 - (100.0 / (1.0 + mfr))));
            } else if positive_mf > 0.0 {
                result.push(Some(100.0));
            } else {
                result.push(Some(50.0));
            }
        }
    }

    result
}

/// Price Volume Trend
pub fn pvt(close: &[f64], volume: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(close.len());

    if close.is_empty() {
        return result;
    }

    result.push(0.0);

    let mut cum_pvt = 0.0;
    for i in 1..close.len() {
        if close[i - 1] != 0.0 {
            let pct_change = (close[i] - close[i - 1]) / close[i - 1];
            cum_pvt += pct_change * volume[i];
        }
        result.push(cum_pvt);
    }

    result
}

/// Volume Oscillator
pub fn volume_oscillator(
    volume: &[f64],
    short_period: usize,
    long_period: usize,
) -> Vec<Option<f64>> {
    let short_ma = sma(volume, short_period);
    let long_ma = sma(volume, long_period);

    short_ma
        .iter()
        .zip(long_ma.iter())
        .map(|(&s, &l)| match (s, l) {
            (Some(sv), Some(lv)) if lv != 0.0 => Some((sv - lv) / lv * 100.0),
            _ => None,
        })
        .collect()
}

/// Klinger Volume Oscillator
pub fn klinger_oscillator(
    high: &[f64],
    low: &[f64],
    close: &[f64],
    volume: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    let len = close.len();

    // Calculate HLC3
    let hlc3: Vec<f64> = (0..len)
        .map(|i| (high[i] + low[i] + close[i]) / 3.0)
        .collect();

    // Calculate signed volume
    let mut signed_volume = vec![0.0; len];
    for i in 1..len {
        if hlc3[i] >= hlc3[i - 1] {
            signed_volume[i] = volume[i];
        } else {
            signed_volume[i] = -volume[i];
        }
    }
    signed_volume[0] = volume[0];

    // Calculate EMAs
    let fast_ema = ema(&signed_volume, fast_period);
    let slow_ema = ema(&signed_volume, slow_period);

    // KVO = Fast EMA - Slow EMA
    let mut kvo = Vec::with_capacity(len);
    for i in 0..len {
        match (fast_ema[i], slow_ema[i]) {
            (Some(f), Some(s)) => kvo.push(f - s),
            _ => kvo.push(0.0),
        }
    }

    // Signal line
    let signal = ema(&kvo, signal_period);

    let kvo_result: Vec<Option<f64>> = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(&f, &s)| match (f, s) {
            (Some(fv), Some(sv)) => Some(fv - sv),
            _ => None,
        })
        .collect();

    (kvo_result, signal)
}

/// Net Volume
pub fn net_volume(close: &[f64], volume: &[f64]) -> Vec<f64> {
    close
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if i == 0 {
                0.0
            } else if c > close[i - 1] {
                volume[i]
            } else if c < close[i - 1] {
                -volume[i]
            } else {
                0.0
            }
        })
        .collect()
}

/// Cumulative Volume Delta (simplified - needs tick data for accurate calculation)
pub fn cumulative_volume_delta(
    _open: &[f64],
    close: &[f64],
    high: &[f64],
    low: &[f64],
    volume: &[f64],
) -> Vec<f64> {
    let mut result = Vec::with_capacity(close.len());

    let mut cum_delta = 0.0;
    for i in 0..close.len() {
        // Approximate buy/sell volume based on candle position
        let range = high[i] - low[i];
        let buy_ratio = if range > 0.0 {
            (close[i] - low[i]) / range
        } else {
            0.5
        };

        let buy_vol = volume[i] * buy_ratio;
        let sell_vol = volume[i] * (1.0 - buy_ratio);
        let delta = buy_vol - sell_vol;

        cum_delta += delta;
        result.push(cum_delta);
    }

    result
}

/// Volume Delta (per bar)
pub fn volume_delta(
    _open: &[f64],
    close: &[f64],
    high: &[f64],
    low: &[f64],
    volume: &[f64],
) -> Vec<f64> {
    close
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let range = high[i] - low[i];
            let buy_ratio = if range > 0.0 {
                (close[i] - low[i]) / range
            } else {
                0.5
            };

            let buy_vol = volume[i] * buy_ratio;
            let sell_vol = volume[i] * (1.0 - buy_ratio);
            buy_vol - sell_vol
        })
        .collect()
}

/// Elder Force Index
pub fn elder_force_index(close: &[f64], volume: &[f64], period: usize) -> Vec<Option<f64>> {
    let len = close.len();
    let mut force = vec![0.0; len];

    for i in 1..len {
        force[i] = (close[i] - close[i - 1]) * volume[i];
    }

    ema(&force, period)
}

/// Ease of Movement
pub fn ease_of_movement(
    high: &[f64],
    low: &[f64],
    volume: &[f64],
    period: usize,
) -> Vec<Option<f64>> {
    let len = high.len();
    let mut emv = vec![0.0; len];

    for i in 1..len {
        let distance = ((high[i] + low[i]) / 2.0) - ((high[i - 1] + low[i - 1]) / 2.0);
        let box_ratio = (volume[i] / 10000.0) / (high[i] - low[i]).max(0.0001);
        emv[i] = distance / box_ratio;
    }

    sma(&emv, period)
}

/// Relative Volume at Time (simplified - assumes regular intervals)
pub fn relative_volume(volume: &[f64], lookback_periods: usize, period: usize) -> Vec<Option<f64>> {
    let len = volume.len();
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        if i < lookback_periods * period {
            result.push(None);
        } else {
            // Calculate average volume at same "time" in previous periods
            let mut sum = 0.0;
            let mut count = 0;

            for p in 1..=lookback_periods {
                let idx = i - (p * period);
                if idx < len {
                    sum += volume[idx];
                    count += 1;
                }
            }

            if count > 0 {
                let avg = sum / count as f64;
                if avg > 0.0 {
                    result.push(Some(volume[i] / avg));
                } else {
                    result.push(None);
                }
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Negative Volume Index
pub fn nvi(close: &[f64], volume: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(close.len());

    if close.is_empty() {
        return result;
    }

    result.push(1000.0); // Starting value

    for i in 1..close.len() {
        let prev_nvi = result[i - 1];
        if volume[i] < volume[i - 1] {
            let pct_change = (close[i] - close[i - 1]) / close[i - 1];
            result.push(prev_nvi * (1.0 + pct_change));
        } else {
            result.push(prev_nvi);
        }
    }

    result
}

/// Positive Volume Index
pub fn pvi(close: &[f64], volume: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(close.len());

    if close.is_empty() {
        return result;
    }

    result.push(1000.0); // Starting value

    for i in 1..close.len() {
        let prev_pvi = result[i - 1];
        if volume[i] > volume[i - 1] {
            let pct_change = (close[i] - close[i - 1]) / close[i - 1];
            result.push(prev_pvi * (1.0 + pct_change));
        } else {
            result.push(prev_pvi);
        }
    }

    result
}

/// Volume Weighted Average Price (VWAP)
pub fn vwap(high: &[f64], low: &[f64], close: &[f64], volume: &[f64]) -> Vec<f64> {
    let mut result = Vec::with_capacity(close.len());

    let mut cum_tpv = 0.0;
    let mut cum_vol = 0.0;

    for i in 0..close.len() {
        let typical_price = (high[i] + low[i] + close[i]) / 3.0;
        cum_tpv += typical_price * volume[i];
        cum_vol += volume[i];

        if cum_vol > 0.0 {
            result.push(cum_tpv / cum_vol);
        } else {
            result.push(typical_price);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obv() {
        let close = vec![10.0, 11.0, 10.5, 11.5, 11.0];
        let volume = vec![100.0, 150.0, 120.0, 200.0, 180.0];

        let result = obv(&close, &volume);

        assert_eq!(result[0], 0.0);
        assert_eq!(result[1], 150.0); // Up day
        assert_eq!(result[2], 30.0); // Down day: 150 - 120
        assert_eq!(result[3], 230.0); // Up day: 30 + 200
    }

    #[test]
    fn test_mfi() {
        let high = vec![50.0, 51.0, 52.0, 53.0, 54.0, 53.0, 52.0, 51.0, 50.0, 49.0];
        let low = vec![48.0, 49.0, 50.0, 51.0, 52.0, 51.0, 50.0, 49.0, 48.0, 47.0];
        let close = vec![49.0, 50.0, 51.0, 52.0, 53.0, 52.0, 51.0, 50.0, 49.0, 48.0];
        let volume = vec![1000.0; 10];

        let result = mfi(&high, &low, &close, &volume, 5);

        assert!(result[5].is_some());
        let mfi_val = result[5].unwrap();
        assert!(mfi_val >= 0.0 && mfi_val <= 100.0);
    }

    #[test]
    fn test_chaikin_mf() {
        let high = vec![50.0, 51.0, 52.0, 53.0, 54.0];
        let low = vec![48.0, 49.0, 50.0, 51.0, 52.0];
        let close = vec![49.0, 50.0, 51.0, 52.0, 53.0];
        let volume = vec![1000.0; 5];

        let result = chaikin_mf(&high, &low, &close, &volume, 3);

        assert!(result[2].is_some());
        let cmf = result[2].unwrap();
        assert!(cmf >= -1.0 && cmf <= 1.0);
    }
}
