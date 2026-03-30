// Technical Analysis Calculation Library (Rust)
//
// High-performance calculation primitives for indicators.
// Used by both built-in plugins and external WASM plugins.

pub mod momentum;
pub mod moving_averages;
pub mod statistics;
pub mod trend;
pub mod volatility;
pub mod volume;

pub use momentum::*;
pub use moving_averages::*;
pub use statistics::*;
pub use trend::*;
pub use volatility::*;
pub use volume::*;

/// Helper: Convert Vec<Option<f64>> to Vec<f64> (filter None)
pub fn compact(values: &[Option<f64>]) -> Vec<f64> {
    values.iter().filter_map(|&v| v).collect()
}

/// Helper: Convert Vec<f64> to Vec<Option<f64>>
pub fn optional(values: Vec<f64>, start_index: usize) -> Vec<Option<f64>> {
    let mut result = vec![None; start_index];
    result.extend(values.into_iter().map(Some));
    result
}

/// Helper: Get typical price (HLC3)
pub fn typical_price(high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
    high.iter()
        .zip(low.iter())
        .zip(close.iter())
        .map(|((&h, &l), &c)| (h + l + c) / 3.0)
        .collect()
}

/// Helper: Get median price (HL2)
pub fn median_price(high: &[f64], low: &[f64]) -> Vec<f64> {
    high.iter()
        .zip(low.iter())
        .map(|(&h, &l)| (h + l) / 2.0)
        .collect()
}

/// Helper: Get weighted close (HLCC4)
pub fn weighted_close(high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
    high.iter()
        .zip(low.iter())
        .zip(close.iter())
        .map(|((&h, &l), &c)| (h + l + c + c) / 4.0)
        .collect()
}

/// Helper: Get OHLC4
pub fn ohlc4(open: &[f64], high: &[f64], low: &[f64], close: &[f64]) -> Vec<f64> {
    open.iter()
        .zip(high.iter())
        .zip(low.iter())
        .zip(close.iter())
        .map(|(((&o, &h), &l), &c)| (o + h + l + c) / 4.0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typical_price() {
        let high = vec![50.0, 51.0];
        let low = vec![48.0, 49.0];
        let close = vec![49.0, 50.0];

        let tp = typical_price(&high, &low, &close);

        assert_eq!(tp[0], 49.0); // (50+48+49)/3
        assert_eq!(tp[1], 50.0); // (51+49+50)/3
    }

    #[test]
    fn test_median_price() {
        let high = vec![50.0, 52.0];
        let low = vec![48.0, 48.0];

        let mp = median_price(&high, &low);

        assert_eq!(mp[0], 49.0);
        assert_eq!(mp[1], 50.0);
    }
}
