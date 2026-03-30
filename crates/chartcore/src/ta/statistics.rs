// Statistical Calculations

/// Standard Deviation (using provided mean values)
pub fn stdev(source: &[f64], mean: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if period == 0 {
        return vec![None; source.len()];
    }

    for i in 0..source.len() {
        if i + 1 < period || mean[i].is_none() {
            result.push(None);
        } else {
            let mean_val = mean[i].unwrap();
            let mut variance = 0.0;

            for j in 0..period {
                let diff = source[i - j] - mean_val;
                variance += diff * diff;
            }

            result.push(Some((variance / period as f64).sqrt()));
        }
    }

    result
}

/// Standard Deviation (calculates mean internally)
pub fn stdev_simple(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if period == 0 {
        return vec![None; source.len()];
    }

    for i in 0..source.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let mean: f64 = source[start..=i].iter().sum::<f64>() / period as f64;
            let variance: f64 = source[start..=i]
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>()
                / period as f64;

            result.push(Some(variance.sqrt()));
        }
    }

    result
}

/// Variance
pub fn variance(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if period == 0 {
        return vec![None; source.len()];
    }

    for i in 0..source.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let mean: f64 = source[start..=i].iter().sum::<f64>() / period as f64;
            let var: f64 = source[start..=i]
                .iter()
                .map(|&x| (x - mean).powi(2))
                .sum::<f64>()
                / period as f64;

            result.push(Some(var));
        }
    }

    result
}

/// Highest value in period
pub fn highest(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if period == 0 {
        return vec![None; source.len()];
    }

    for i in 0..source.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let max = source[start..=i]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            result.push(Some(max));
        }
    }

    result
}

/// Lowest value in period
pub fn lowest(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    if period == 0 {
        return vec![None; source.len()];
    }

    for i in 0..source.len() {
        if i + 1 < period {
            result.push(None);
        } else {
            let start = i + 1 - period;
            let min = source[start..=i]
                .iter()
                .copied()
                .fold(f64::INFINITY, f64::min);
            result.push(Some(min));
        }
    }

    result
}

/// Highest value bars ago
pub fn highest_bars(source: &[f64], period: usize) -> Vec<Option<usize>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let mut max_val = f64::NEG_INFINITY;
            let mut bars_ago = 0;

            for j in 0..period {
                if source[i - j] > max_val {
                    max_val = source[i - j];
                    bars_ago = j;
                }
            }
            result.push(Some(bars_ago));
        }
    }

    result
}

/// Lowest value bars ago
pub fn lowest_bars(source: &[f64], period: usize) -> Vec<Option<usize>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let mut min_val = f64::INFINITY;
            let mut bars_ago = 0;

            for j in 0..period {
                if source[i - j] < min_val {
                    min_val = source[i - j];
                    bars_ago = j;
                }
            }
            result.push(Some(bars_ago));
        }
    }

    result
}

/// Sum over period
pub fn sum(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let total: f64 = source[i - period + 1..=i].iter().sum();
            result.push(Some(total));
        }
    }

    result
}

/// Change (current - previous)
pub fn change(source: &[f64], length: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < length {
            result.push(None);
        } else {
            result.push(Some(source[i] - source[i - length]));
        }
    }

    result
}

/// Rate of Change (percentage)
pub fn roc(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period {
            result.push(None);
        } else {
            let prev = source[i - period];
            if prev != 0.0 {
                result.push(Some(((source[i] - prev) / prev) * 100.0));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Correlation Coefficient
pub fn correlation(series1: &[f64], series2: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(series1.len());

    for i in 0..series1.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut sum_xy = 0.0;
            let mut sum_x2 = 0.0;
            let mut sum_y2 = 0.0;

            for j in 0..period {
                let x = series1[i - j];
                let y = series2[i - j];
                sum_x += x;
                sum_y += y;
                sum_xy += x * y;
                sum_x2 += x * x;
                sum_y2 += y * y;
            }

            let n = period as f64;
            let numerator = n * sum_xy - sum_x * sum_y;
            let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();

            if denominator != 0.0 {
                result.push(Some(numerator / denominator));
            } else {
                result.push(None);
            }
        }
    }

    result
}

/// Covariance
pub fn covariance(series1: &[f64], series2: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(series1.len());

    for i in 0..series1.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let mean1: f64 = series1[i - period + 1..=i].iter().sum::<f64>() / period as f64;
            let mean2: f64 = series2[i - period + 1..=i].iter().sum::<f64>() / period as f64;

            let mut cov = 0.0;
            for j in 0..period {
                cov += (series1[i - j] - mean1) * (series2[i - j] - mean2);
            }

            result.push(Some(cov / period as f64));
        }
    }

    result
}

/// Linear Regression
pub fn linreg(source: &[f64], period: usize) -> Vec<Option<f64>> {
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

/// Linear Regression Slope
pub fn linreg_slope(source: &[f64], period: usize) -> Vec<Option<f64>> {
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

            result.push(Some(slope));
        }
    }

    result
}

/// Linear Regression Intercept
pub fn linreg_intercept(source: &[f64], period: usize) -> Vec<Option<f64>> {
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

            result.push(Some(intercept));
        }
    }

    result
}

/// Percentile Rank
pub fn percentile_rank(source: &[f64], period: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let current = source[i];
            let count_below = source[i - period + 1..=i]
                .iter()
                .filter(|&&x| x < current)
                .count();

            result.push(Some((count_below as f64 / period as f64) * 100.0));
        }
    }

    result
}

/// Percentile (value at given percentile)
pub fn percentile(source: &[f64], period: usize, pct: f64) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let mut window: Vec<f64> = source[i - period + 1..=i].to_vec();
            window.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let idx = ((pct / 100.0) * (period - 1) as f64) as usize;
            result.push(Some(window[idx.min(period - 1)]));
        }
    }

    result
}

/// Mode (most common value, approximated for continuous data)
pub fn mode(source: &[f64], period: usize, num_bins: usize) -> Vec<Option<f64>> {
    let mut result = Vec::with_capacity(source.len());

    for i in 0..source.len() {
        if i < period - 1 {
            result.push(None);
        } else {
            let window: Vec<f64> = source[i - period + 1..=i].to_vec();
            let min = window.iter().copied().fold(f64::INFINITY, f64::min);
            let max = window.iter().copied().fold(f64::NEG_INFINITY, f64::max);

            if (max - min).abs() < f64::EPSILON {
                result.push(Some(min));
                continue;
            }

            let bin_size = (max - min) / num_bins as f64;
            let mut bins = vec![0usize; num_bins];

            for &val in &window {
                let bin = ((val - min) / bin_size).floor() as usize;
                let bin = bin.min(num_bins - 1);
                bins[bin] += 1;
            }

            let max_bin = bins
                .iter()
                .enumerate()
                .max_by_key(|(_, &count)| count)
                .unwrap()
                .0;
            let mode_val = min + (max_bin as f64 + 0.5) * bin_size;
            result.push(Some(mode_val));
        }
    }

    result
}

/// Cross above (series1 crosses above series2)
pub fn cross_above(series1: &[f64], series2: &[f64]) -> Vec<bool> {
    let mut result = Vec::with_capacity(series1.len());

    for i in 0..series1.len() {
        if i == 0 {
            result.push(false);
        } else {
            let cross = series1[i] > series2[i] && series1[i - 1] <= series2[i - 1];
            result.push(cross);
        }
    }

    result
}

/// Cross below (series1 crosses below series2)
pub fn cross_below(series1: &[f64], series2: &[f64]) -> Vec<bool> {
    let mut result = Vec::with_capacity(series1.len());

    for i in 0..series1.len() {
        if i == 0 {
            result.push(false);
        } else {
            let cross = series1[i] < series2[i] && series1[i - 1] >= series2[i - 1];
            result.push(cross);
        }
    }

    result
}

/// Cross (either direction)
pub fn cross(series1: &[f64], series2: &[f64]) -> Vec<bool> {
    let above = cross_above(series1, series2);
    let below = cross_below(series1, series2);

    above
        .iter()
        .zip(below.iter())
        .map(|(&a, &b)| a || b)
        .collect()
}

/// Bars since condition was true
pub fn bars_since(condition: &[bool]) -> Vec<Option<usize>> {
    let mut result = Vec::with_capacity(condition.len());
    let mut last_true: Option<usize> = None;

    for (i, &cond) in condition.iter().enumerate() {
        if cond {
            last_true = Some(i);
            result.push(Some(0));
        } else if let Some(last) = last_true {
            result.push(Some(i - last));
        } else {
            result.push(None);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highest() {
        let data = vec![1.0, 5.0, 3.0, 4.0, 2.0];
        let result = highest(&data, 3);

        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(5.0));
        assert_eq!(result[3], Some(5.0));
        assert_eq!(result[4], Some(4.0));
    }

    #[test]
    fn test_lowest() {
        let data = vec![5.0, 1.0, 3.0, 4.0, 2.0];
        let result = lowest(&data, 3);

        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert_eq!(result[2], Some(1.0));
        assert_eq!(result[3], Some(1.0));
        assert_eq!(result[4], Some(2.0));
    }

    #[test]
    fn test_change() {
        let data = vec![10.0, 12.0, 15.0, 13.0, 18.0];
        let result = change(&data, 1);

        assert_eq!(result[0], None);
        assert_eq!(result[1], Some(2.0));
        assert_eq!(result[2], Some(3.0));
        assert_eq!(result[3], Some(-2.0));
        assert_eq!(result[4], Some(5.0));
    }

    #[test]
    fn test_stdev_simple() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let result = stdev_simple(&data, 3);

        assert!(result[2].is_some());
        // StdDev of [2,4,4] should be sqrt(2/3) ≈ 0.816
        let std = result[2].unwrap();
        assert!((std - 0.9428).abs() < 0.01);
    }

    #[test]
    fn test_cross_above() {
        let series1 = vec![1.0, 2.0, 3.0, 2.0, 3.0];
        let series2 = vec![2.0, 2.0, 2.0, 2.0, 2.0];

        let result = cross_above(&series1, &series2);

        assert!(!result[0]);
        assert!(!result[1]); // Not a cross, just touching
        assert!(result[2]); // Cross above
        assert!(!result[3]);
        assert!(result[4]); // Cross above again
    }
}
