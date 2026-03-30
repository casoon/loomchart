//! Additional Moving Averages
//!
//! Hull Moving Average, Arnaud Legoux MA, and other advanced MAs.

use crate::indicators::trend::Ema;
use crate::indicators::{Current, Next, Period, Reset};
use crate::math::average::wma;
use crate::types::Ohlcv;
use libm::sqrt;

/// Hull Moving Average (HMA)
///
/// HMA reduces lag while maintaining smoothness using weighted moving averages.
///
/// HMA = WMA(2 * WMA(n/2) - WMA(n), sqrt(n))
///
/// Created by Alan Hull.
#[derive(Clone)]
pub struct Hma {
    period: usize,
    sqrt_period: usize,
    half_buffer: Vec<f64>,
    full_buffer: Vec<f64>,
    result_buffer: Vec<f64>,
    half_idx: usize,
    full_idx: usize,
    result_idx: usize,
    half_count: usize,
    full_count: usize,
    result_count: usize,
    current: Option<f64>,
}

impl Hma {
    pub fn new(period: usize) -> Self {
        assert!(period >= 2, "Period must be at least 2");
        let half_period = period / 2;
        let sqrt_period = (sqrt(period as f64) as usize).max(1);

        Self {
            period,
            sqrt_period,
            half_buffer: vec![0.0; half_period],
            full_buffer: vec![0.0; period],
            result_buffer: vec![0.0; sqrt_period],
            half_idx: 0,
            full_idx: 0,
            result_idx: 0,
            half_count: 0,
            full_count: 0,
            result_count: 0,
            current: None,
        }
    }
}

impl Next<f64> for Hma {
    type Output = f64;

    fn next(&mut self, value: f64) -> Option<f64> {
        let half_period = self.period / 2;

        // Update half buffer
        self.half_buffer[self.half_idx] = value;
        self.half_idx = (self.half_idx + 1) % half_period;
        if self.half_count < half_period {
            self.half_count += 1;
        }

        // Update full buffer
        self.full_buffer[self.full_idx] = value;
        self.full_idx = (self.full_idx + 1) % self.period;
        if self.full_count < self.period {
            self.full_count += 1;
        }

        // Need full period for WMAs
        if self.full_count < self.period {
            return None;
        }

        // Calculate WMAs
        let wma_half = wma(&self.half_buffer);
        let wma_full = wma(&self.full_buffer);

        // Raw HMA value: 2 * WMA(n/2) - WMA(n)
        let raw = 2.0 * wma_half - wma_full;

        // Update result buffer
        self.result_buffer[self.result_idx] = raw;
        self.result_idx = (self.result_idx + 1) % self.sqrt_period;
        if self.result_count < self.sqrt_period {
            self.result_count += 1;
        }

        if self.result_count >= self.sqrt_period {
            let hma = wma(&self.result_buffer);
            self.current = Some(hma);
            Some(hma)
        } else {
            None
        }
    }
}

impl Next<&Ohlcv> for Hma {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.next(candle.close)
    }
}

impl Reset for Hma {
    fn reset(&mut self) {
        self.half_buffer.fill(0.0);
        self.full_buffer.fill(0.0);
        self.result_buffer.fill(0.0);
        self.half_idx = 0;
        self.full_idx = 0;
        self.result_idx = 0;
        self.half_count = 0;
        self.full_count = 0;
        self.result_count = 0;
        self.current = None;
    }
}

impl Period for Hma {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Hma {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

/// Arnaud Legoux Moving Average (ALMA)
///
/// Uses a Gaussian distribution to weight the moving average,
/// reducing noise while responding to price changes.
///
/// Parameters:
/// - period: lookback period
/// - offset: controls the Gaussian center (0.0 to 1.0, default 0.85)
/// - sigma: controls the Gaussian width (default 6.0)
#[derive(Clone)]
pub struct Alma {
    #[allow(dead_code)]
    period: usize,
    #[allow(dead_code)]
    offset: f64,
    #[allow(dead_code)]
    sigma: f64,
    weights: Vec<f64>,
    buffer: Vec<f64>,
    index: usize,
    count: usize,
    current: Option<f64>,
}

impl Alma {
    pub fn new(period: usize, offset: f64, sigma: f64) -> Self {
        assert!(period > 0, "Period must be greater than 0");

        // Pre-calculate Gaussian weights
        let m = offset * (period as f64 - 1.0);
        let s = period as f64 / sigma;
        let s2 = 2.0 * s * s;

        let mut weights = Vec::with_capacity(period);
        let mut weight_sum = 0.0;

        for i in 0..period {
            let w = libm::exp(-((i as f64 - m) * (i as f64 - m)) / s2);
            weights.push(w);
            weight_sum += w;
        }

        // Normalize weights
        for w in &mut weights {
            *w /= weight_sum;
        }

        Self {
            period,
            offset,
            sigma,
            weights,
            buffer: vec![0.0; period],
            index: 0,
            count: 0,
            current: None,
        }
    }

    pub fn default_params(period: usize) -> Self {
        Self::new(period, 0.85, 6.0)
    }
}

impl Next<f64> for Alma {
    type Output = f64;

    fn next(&mut self, value: f64) -> Option<f64> {
        self.buffer[self.index] = value;
        self.index = (self.index + 1) % self.period;

        if self.count < self.period {
            self.count += 1;
        }

        if self.count >= self.period {
            let mut alma = 0.0;
            let start = self.index; // oldest value

            for i in 0..self.period {
                let buf_idx = (start + i) % self.period;
                alma += self.buffer[buf_idx] * self.weights[i];
            }

            self.current = Some(alma);
            Some(alma)
        } else {
            None
        }
    }
}

impl Next<&Ohlcv> for Alma {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.next(candle.close)
    }
}

impl Reset for Alma {
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.current = None;
    }
}

impl Period for Alma {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Alma {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

/// Zero-Lag Exponential Moving Average (ZLEMA)
///
/// Reduces lag by adding a momentum term.
/// ZLEMA = EMA(Price + (Price - Price[lag]))
#[derive(Clone)]
pub struct Zlema {
    period: usize,
    lag: usize,
    buffer: Vec<f64>,
    index: usize,
    count: usize,
    ema: Ema,
    current: Option<f64>,
}

impl Zlema {
    pub fn new(period: usize) -> Self {
        let lag = (period - 1) / 2;
        Self {
            period,
            lag,
            buffer: vec![0.0; lag + 1],
            index: 0,
            count: 0,
            ema: Ema::new(period),
            current: None,
        }
    }
}

impl Next<f64> for Zlema {
    type Output = f64;

    fn next(&mut self, value: f64) -> Option<f64> {
        // Get lagged value
        let lag_idx = (self.index + 1) % (self.lag + 1);
        let lagged = if self.count > self.lag {
            self.buffer[lag_idx]
        } else {
            value
        };

        // Store current value
        self.buffer[self.index] = value;
        self.index = (self.index + 1) % (self.lag + 1);
        self.count += 1;

        // Calculate ZLEMA input: price + (price - lagged_price)
        let zlema_input = value + (value - lagged);

        if let Some(zlema) = self.ema.next(zlema_input) {
            self.current = Some(zlema);
            Some(zlema)
        } else {
            None
        }
    }
}

impl Next<&Ohlcv> for Zlema {
    type Output = f64;

    fn next(&mut self, candle: &Ohlcv) -> Option<f64> {
        self.next(candle.close)
    }
}

impl Reset for Zlema {
    fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.ema.reset();
        self.current = None;
    }
}

impl Period for Zlema {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Zlema {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hma_basic() {
        let mut hma = Hma::new(9);

        // Feed some data
        for i in 0..20 {
            hma.next(100.0 + i as f64);
        }

        assert!(hma.current().is_some());
    }

    #[test]
    fn test_alma_basic() {
        let mut alma = Alma::default_params(9);

        for i in 0..15 {
            alma.next(100.0 + i as f64);
        }

        assert!(alma.current().is_some());
    }

    #[test]
    fn test_zlema_basic() {
        let mut zlema = Zlema::new(10);

        for i in 0..15 {
            zlema.next(100.0 + i as f64);
        }

        assert!(zlema.current().is_some());
    }
}
