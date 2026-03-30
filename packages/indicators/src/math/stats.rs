//! Statistical functions
//!
//! Variance, standard deviation, correlation, and other statistical measures.

use libm::sqrt;

/// Calculate the arithmetic mean of a slice.
///
/// This is equivalent to SMA but named for statistical context.
#[inline]
pub fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

/// Calculate the population variance.
///
/// Variance = sum((x - mean)^2) / n
#[inline]
pub fn variance(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let m = mean(values);
    let sum_sq: f64 = values.iter().map(|&x| (x - m) * (x - m)).sum();
    sum_sq / values.len() as f64
}

/// Calculate the sample variance.
///
/// Sample Variance = sum((x - mean)^2) / (n - 1)
#[inline]
pub fn sample_variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let m = mean(values);
    let sum_sq: f64 = values.iter().map(|&x| (x - m) * (x - m)).sum();
    sum_sq / (values.len() - 1) as f64
}

/// Calculate the population standard deviation.
///
/// StdDev = sqrt(variance)
#[inline]
pub fn stddev(values: &[f64]) -> f64 {
    sqrt(variance(values))
}

/// Calculate the sample standard deviation.
#[inline]
pub fn sample_stddev(values: &[f64]) -> f64 {
    sqrt(sample_variance(values))
}

/// Calculate variance given a known mean (optimization).
#[inline]
pub fn variance_with_mean(values: &[f64], m: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let sum_sq: f64 = values.iter().map(|&x| (x - m) * (x - m)).sum();
    sum_sq / values.len() as f64
}

/// Calculate standard deviation given a known mean (optimization).
#[inline]
pub fn stddev_with_mean(values: &[f64], m: f64) -> f64 {
    sqrt(variance_with_mean(values, m))
}

/// Calculate the coefficient of variation (CV).
///
/// CV = stddev / mean * 100
///
/// Useful for comparing volatility across different price ranges.
#[inline]
pub fn coefficient_of_variation(values: &[f64]) -> f64 {
    let m = mean(values);
    if m.abs() < f64::EPSILON {
        return 0.0;
    }
    stddev(values) / m * 100.0
}

/// Calculate the Pearson correlation coefficient between two series.
///
/// r = sum((x - mean_x)(y - mean_y)) / sqrt(sum((x - mean_x)^2) * sum((y - mean_y)^2))
///
/// Returns a value between -1 and 1.
#[inline]
pub fn correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let mean_x = mean(x);
    let mean_y = mean(y);

    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;
    let mut sum_y2 = 0.0;

    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        sum_xy += dx * dy;
        sum_x2 += dx * dx;
        sum_y2 += dy * dy;
    }

    let denominator = sqrt(sum_x2 * sum_y2);
    if denominator.abs() < f64::EPSILON {
        return 0.0;
    }

    sum_xy / denominator
}

/// Calculate the covariance between two series.
#[inline]
pub fn covariance(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let mean_x = mean(x);
    let mean_y = mean(y);

    let sum_xy: f64 = x
        .iter()
        .zip(y.iter())
        .map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y))
        .sum();

    sum_xy / x.len() as f64
}

/// Calculate the linear regression slope.
///
/// slope = covariance(x, y) / variance(x)
#[inline]
pub fn linear_regression_slope(y: &[f64]) -> f64 {
    if y.len() < 2 {
        return 0.0;
    }

    // x is just the indices: 0, 1, 2, ...
    let n = y.len() as f64;
    let mean_x = (n - 1.0) / 2.0; // mean of 0..n-1
    let mean_y = mean(y);

    let mut sum_xy = 0.0;
    let mut sum_x2 = 0.0;

    for (i, &yi) in y.iter().enumerate() {
        let x = i as f64;
        let dx = x - mean_x;
        sum_xy += dx * (yi - mean_y);
        sum_x2 += dx * dx;
    }

    if sum_x2.abs() < f64::EPSILON {
        return 0.0;
    }

    sum_xy / sum_x2
}

/// Calculate the linear regression value at the end of the series.
///
/// This is useful for trend analysis.
#[inline]
pub fn linear_regression_value(y: &[f64]) -> f64 {
    if y.len() < 2 {
        return *y.last().unwrap_or(&0.0);
    }

    let slope = linear_regression_slope(y);
    let mean_y = mean(y);
    let n = y.len() as f64;
    let mean_x = (n - 1.0) / 2.0;

    // y = slope * x + intercept
    // intercept = mean_y - slope * mean_x
    // at x = n - 1 (last point): y = slope * (n-1) + intercept
    let intercept = mean_y - slope * mean_x;
    slope * (n - 1.0) + intercept
}

/// Calculate the R-squared (coefficient of determination) for linear regression.
#[inline]
pub fn r_squared(y: &[f64]) -> f64 {
    if y.len() < 2 {
        return 0.0;
    }

    let slope = linear_regression_slope(y);
    let mean_y = mean(y);
    let n = y.len();
    let mean_x = (n - 1) as f64 / 2.0;
    let intercept = mean_y - slope * mean_x;

    let mut ss_res = 0.0; // residual sum of squares
    let mut ss_tot = 0.0; // total sum of squares

    for (i, &yi) in y.iter().enumerate() {
        let predicted = slope * i as f64 + intercept;
        ss_res += (yi - predicted) * (yi - predicted);
        ss_tot += (yi - mean_y) * (yi - mean_y);
    }

    if ss_tot.abs() < f64::EPSILON {
        return 0.0;
    }

    1.0 - (ss_res / ss_tot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mean() {
        assert!((mean(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_variance() {
        // Variance of [1, 2, 3, 4, 5] with mean 3:
        // ((1-3)^2 + (2-3)^2 + (3-3)^2 + (4-3)^2 + (5-3)^2) / 5
        // = (4 + 1 + 0 + 1 + 4) / 5 = 10 / 5 = 2
        assert!((variance(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_stddev() {
        // StdDev = sqrt(2) ≈ 1.414
        assert!((stddev(&[1.0, 2.0, 3.0, 4.0, 5.0]) - 1.414213).abs() < 0.0001);
    }

    #[test]
    fn test_correlation() {
        // Perfect positive correlation
        let x = [1.0, 2.0, 3.0, 4.0, 5.0];
        let y = [2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((correlation(&x, &y) - 1.0).abs() < 1e-10);

        // Perfect negative correlation
        let y_neg = [10.0, 8.0, 6.0, 4.0, 2.0];
        assert!((correlation(&x, &y_neg) - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_linear_regression_slope() {
        // y = 2x + 1 -> [1, 3, 5, 7, 9]
        let y = [1.0, 3.0, 5.0, 7.0, 9.0];
        assert!((linear_regression_slope(&y) - 2.0).abs() < 1e-10);
    }
}
