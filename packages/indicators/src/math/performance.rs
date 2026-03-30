//! Performance and risk metrics
//!
//! Functions for calculating trading performance metrics like Sharpe ratio,
//! Sortino ratio, maximum drawdown, and other risk-adjusted returns.

use libm::sqrt;
use super::stats::{mean, sample_stddev};

/// Calculate returns from a series of prices.
///
/// Returns a vector of (price[i] - price[i-1]) / price[i-1] for each consecutive pair.
#[inline]
pub fn returns(prices: &[f64]) -> Vec<f64> {
    if prices.len() < 2 {
        return Vec::new();
    }

    prices.windows(2)
        .map(|w| {
            if w[0] != 0.0 {
                (w[1] - w[0]) / w[0]
            } else {
                0.0
            }
        })
        .collect()
}

/// Calculate log returns from a series of prices.
///
/// Returns ln(price[i] / price[i-1]) for each consecutive pair.
#[inline]
pub fn log_returns(prices: &[f64]) -> Vec<f64> {
    if prices.len() < 2 {
        return Vec::new();
    }

    prices.windows(2)
        .map(|w| {
            if w[0] > 0.0 && w[1] > 0.0 {
                libm::log(w[1] / w[0])
            } else {
                0.0
            }
        })
        .collect()
}

/// Calculate the Sharpe Ratio.
///
/// Sharpe = (Mean Return - Risk-Free Rate) / StdDev of Returns
///
/// - `returns`: Series of periodic returns
/// - `risk_free_rate`: Risk-free rate for the same period (e.g., daily rate for daily returns)
///
/// A higher Sharpe ratio indicates better risk-adjusted returns.
/// - > 1.0: Good
/// - > 2.0: Very good
/// - > 3.0: Excellent
#[inline]
pub fn sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let excess_returns: Vec<f64> = returns.iter().map(|r| r - risk_free_rate).collect();
    let mean_excess = mean(&excess_returns);
    let std = sample_stddev(&excess_returns);

    if std.abs() < f64::EPSILON {
        return 0.0;
    }

    mean_excess / std
}

/// Calculate the annualized Sharpe Ratio.
///
/// - `returns`: Series of periodic returns
/// - `risk_free_rate`: Risk-free rate (annualized)
/// - `periods_per_year`: Number of periods per year (252 for daily, 52 for weekly, 12 for monthly)
#[inline]
pub fn sharpe_ratio_annualized(returns: &[f64], risk_free_rate: f64, periods_per_year: f64) -> f64 {
    if returns.is_empty() || periods_per_year <= 0.0 {
        return 0.0;
    }

    // Convert annualized risk-free rate to periodic
    let periodic_rf = risk_free_rate / periods_per_year;

    let sharpe = sharpe_ratio(returns, periodic_rf);

    // Annualize the Sharpe ratio
    sharpe * sqrt(periods_per_year)
}

/// Calculate the Sortino Ratio.
///
/// Like Sharpe, but only penalizes downside volatility.
/// Sortino = (Mean Return - Target) / Downside Deviation
///
/// - `returns`: Series of periodic returns
/// - `target_return`: Minimum acceptable return (often 0 or risk-free rate)
#[inline]
pub fn sortino_ratio(returns: &[f64], target_return: f64) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean_return = mean(returns);

    // Calculate downside deviation (only negative deviations from target)
    let downside_returns: Vec<f64> = returns
        .iter()
        .filter(|&&r| r < target_return)
        .map(|&r| (r - target_return) * (r - target_return))
        .collect();

    if downside_returns.is_empty() {
        // No downside returns - return very high value if positive mean
        return if mean_return > target_return { f64::MAX } else { 0.0 };
    }

    let downside_deviation = sqrt(mean(&downside_returns));

    if downside_deviation.abs() < f64::EPSILON {
        return 0.0;
    }

    (mean_return - target_return) / downside_deviation
}

/// Calculate the maximum drawdown from a series of prices or equity values.
///
/// Maximum drawdown measures the largest peak-to-trough decline.
/// Returns a value between 0 and 1 (e.g., 0.25 = 25% drawdown).
#[inline]
pub fn max_drawdown(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let mut max_value = values[0];
    let mut max_dd = 0.0;

    for &value in &values[1..] {
        if value > max_value {
            max_value = value;
        } else if max_value > 0.0 {
            let dd = (max_value - value) / max_value;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }

    max_dd
}

/// Calculate the maximum drawdown and its duration (number of periods).
///
/// Returns (max_drawdown, peak_index, trough_index).
#[inline]
pub fn max_drawdown_with_duration(values: &[f64]) -> (f64, usize, usize) {
    if values.len() < 2 {
        return (0.0, 0, 0);
    }

    let mut max_value = values[0];
    let mut max_value_idx = 0;
    let mut max_dd = 0.0;
    let mut max_dd_peak_idx = 0;
    let mut max_dd_trough_idx = 0;

    for (i, &value) in values.iter().enumerate().skip(1) {
        if value > max_value {
            max_value = value;
            max_value_idx = i;
        } else if max_value > 0.0 {
            let dd = (max_value - value) / max_value;
            if dd > max_dd {
                max_dd = dd;
                max_dd_peak_idx = max_value_idx;
                max_dd_trough_idx = i;
            }
        }
    }

    (max_dd, max_dd_peak_idx, max_dd_trough_idx)
}

/// Calculate the Calmar Ratio.
///
/// Calmar = Annualized Return / Maximum Drawdown
///
/// Measures the risk-adjusted return relative to the worst drawdown.
#[inline]
pub fn calmar_ratio(returns: &[f64], max_dd: f64) -> f64 {
    if max_dd.abs() < f64::EPSILON {
        return 0.0;
    }

    let total_return: f64 = returns.iter().map(|r| 1.0 + r).product::<f64>() - 1.0;

    total_return / max_dd
}

/// Calculate Beta (systematic risk relative to a benchmark).
///
/// Beta = Covariance(asset, market) / Variance(market)
///
/// - Beta = 1: Moves with the market
/// - Beta > 1: More volatile than market
/// - Beta < 1: Less volatile than market
/// - Beta < 0: Moves opposite to market
#[inline]
pub fn beta(asset_returns: &[f64], market_returns: &[f64]) -> f64 {
    if asset_returns.len() != market_returns.len() || asset_returns.is_empty() {
        return 0.0;
    }

    let mean_asset = mean(asset_returns);
    let mean_market = mean(market_returns);

    let mut covariance = 0.0;
    let mut market_variance = 0.0;

    for (&asset, &market) in asset_returns.iter().zip(market_returns.iter()) {
        let da = asset - mean_asset;
        let dm = market - mean_market;
        covariance += da * dm;
        market_variance += dm * dm;
    }

    let n = asset_returns.len() as f64;
    covariance /= n;
    market_variance /= n;

    if market_variance.abs() < f64::EPSILON {
        return 0.0;
    }

    covariance / market_variance
}

/// Calculate Alpha (excess return over benchmark).
///
/// Alpha = Asset Return - (Risk-Free + Beta * (Market Return - Risk-Free))
///
/// A positive alpha indicates outperformance.
#[inline]
pub fn alpha(
    asset_return: f64,
    market_return: f64,
    risk_free_rate: f64,
    beta: f64,
) -> f64 {
    asset_return - (risk_free_rate + beta * (market_return - risk_free_rate))
}

/// Calculate the Information Ratio.
///
/// IR = (Asset Return - Benchmark Return) / Tracking Error
///
/// Measures the consistency of excess returns over a benchmark.
#[inline]
pub fn information_ratio(asset_returns: &[f64], benchmark_returns: &[f64]) -> f64 {
    if asset_returns.len() != benchmark_returns.len() || asset_returns.is_empty() {
        return 0.0;
    }

    let excess: Vec<f64> = asset_returns
        .iter()
        .zip(benchmark_returns.iter())
        .map(|(&a, &b)| a - b)
        .collect();

    let mean_excess = mean(&excess);
    let tracking_error = sample_stddev(&excess);

    if tracking_error.abs() < f64::EPSILON {
        return 0.0;
    }

    mean_excess / tracking_error
}

/// Estimate the Hurst Exponent using R/S analysis.
///
/// The Hurst exponent measures the long-term memory of a time series.
///
/// - H = 0.5: Random walk (no memory)
/// - H > 0.5: Trending/persistent (up follows up)
/// - H < 0.5: Mean-reverting (up follows down)
///
/// This is a simplified estimation using R/S analysis.
#[inline]
pub fn hurst_exponent(values: &[f64]) -> f64 {
    if values.len() < 20 {
        return 0.5; // Not enough data, assume random walk
    }

    let n = values.len();

    // Calculate returns
    let rets = returns(values);
    if rets.is_empty() {
        return 0.5;
    }

    // We'll use a single R/S calculation for simplicity
    // A full implementation would use multiple window sizes
    let mean_ret = mean(&rets);

    // Calculate cumulative deviations from mean
    let mut cum_devs = vec![0.0; rets.len()];
    let mut sum = 0.0;
    for (i, &r) in rets.iter().enumerate() {
        sum += r - mean_ret;
        cum_devs[i] = sum;
    }

    // Range
    let max_dev = cum_devs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_dev = cum_devs.iter().cloned().fold(f64::INFINITY, f64::min);
    let range = max_dev - min_dev;

    // Standard deviation
    let std = sample_stddev(&rets);
    if std.abs() < f64::EPSILON {
        return 0.5;
    }

    // R/S ratio
    let rs = range / std;

    // H ≈ log(R/S) / log(n)
    // For a more accurate estimate, we'd use multiple window sizes and regression
    let h = libm::log(rs) / libm::log(n as f64);

    // Clamp to valid range [0, 1]
    h.max(0.0).min(1.0)
}

/// Calculate the profit factor.
///
/// Profit Factor = Gross Profit / Gross Loss
///
/// - > 1: Profitable
/// - < 1: Unprofitable
/// - > 2: Generally considered good
#[inline]
pub fn profit_factor(returns: &[f64]) -> f64 {
    let gross_profit: f64 = returns.iter().filter(|&&r| r > 0.0).sum();
    let gross_loss: f64 = returns.iter().filter(|&&r| r < 0.0).map(|r| -r).sum();

    if gross_loss.abs() < f64::EPSILON {
        return if gross_profit > 0.0 { f64::MAX } else { 0.0 };
    }

    gross_profit / gross_loss
}

/// Calculate the win rate (percentage of positive returns).
#[inline]
pub fn win_rate(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let wins = returns.iter().filter(|&&r| r > 0.0).count();
    wins as f64 / returns.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_returns() {
        let prices = [100.0, 110.0, 99.0, 121.0];
        let rets = returns(&prices);
        assert_eq!(rets.len(), 3);
        assert!((rets[0] - 0.1).abs() < 0.0001); // 10% up
        assert!((rets[1] - (-0.1)).abs() < 0.0001); // 10% down
    }

    #[test]
    fn test_max_drawdown() {
        // Peak at 100, drops to 50, then recovers to 80
        let values = [80.0, 100.0, 90.0, 50.0, 60.0, 80.0];
        let dd = max_drawdown(&values);
        assert!((dd - 0.5).abs() < 0.0001); // 50% drawdown
    }

    #[test]
    fn test_sharpe_ratio() {
        // Consistent positive returns should have high Sharpe
        let returns = [0.01, 0.012, 0.008, 0.011, 0.009];
        let sharpe = sharpe_ratio(&returns, 0.0);
        assert!(sharpe > 0.0);
    }

    #[test]
    fn test_beta() {
        // Perfect correlation with market = beta of 1
        let asset = [0.01, 0.02, -0.01, 0.015, -0.005];
        let market = [0.01, 0.02, -0.01, 0.015, -0.005];
        let b = beta(&asset, &market);
        assert!((b - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_profit_factor() {
        let returns = [0.10, -0.05, 0.08, -0.02, 0.06];
        // Gross profit = 0.24, Gross loss = 0.07
        let pf = profit_factor(&returns);
        assert!((pf - (0.24 / 0.07)).abs() < 0.0001);
    }

    #[test]
    fn test_win_rate() {
        let returns = [0.10, -0.05, 0.08, -0.02, 0.06];
        let wr = win_rate(&returns);
        assert!((wr - 0.6).abs() < 0.0001); // 3 out of 5 = 60%
    }
}
