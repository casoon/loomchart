//! Chande Momentum Oscillator (CMO)

use crate::indicators::{Next, Period, Reset, Current};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Chande Momentum Oscillator (CMO)
///
/// Developed by Tushar Chande, CMO is similar to RSI but uses raw
/// momentum values instead of smoothed averages.
///
/// Formula: CMO = 100 * (Sum of Up Changes - Sum of Down Changes) / (Sum of Up + Sum of Down)
///
/// Values range from -100 to +100:
/// - > +50: Overbought
/// - < -50: Oversold
/// - Crosses zero line: Momentum shift
///
/// Default period: 14
#[derive(Clone)]
pub struct Cmo {
    period: usize,
    up_buffer: Vec<f64>,
    down_buffer: Vec<f64>,
    index: usize,
    count: usize,
    up_sum: f64,
    down_sum: f64,
    prev_value: Option<f64>,
    current: Option<f64>,
}

impl Cmo {
    pub fn new(period: usize) -> Self {
        assert!(period > 0, "Period must be greater than 0");
        Self {
            period,
            up_buffer: vec![0.0; period],
            down_buffer: vec![0.0; period],
            index: 0,
            count: 0,
            up_sum: 0.0,
            down_sum: 0.0,
            prev_value: None,
            current: None,
        }
    }
}

impl Next<f64> for Cmo {
    type Output = f64;

    fn next(&mut self, value: f64) -> Option<f64> {
        let result = if let Some(prev) = self.prev_value {
            let change = value - prev;
            let (up, down) = if change > 0.0 {
                (change, 0.0)
            } else {
                (0.0, -change)
            };

            // Remove old values from sum if buffer is full
            if self.count >= self.period {
                self.up_sum -= self.up_buffer[self.index];
                self.down_sum -= self.down_buffer[self.index];
            }

            // Store new values
            self.up_buffer[self.index] = up;
            self.down_buffer[self.index] = down;
            self.up_sum += up;
            self.down_sum += down;

            self.index = (self.index + 1) % self.period;
            if self.count < self.period {
                self.count += 1;
            }

            if self.count >= self.period {
                let total = self.up_sum + self.down_sum;
                let cmo = if total > 0.0 {
                    100.0 * (self.up_sum - self.down_sum) / total
                } else {
                    0.0
                };
                self.current = Some(cmo);
                Some(cmo)
            } else {
                None
            }
        } else {
            None
        };

        self.prev_value = Some(value);
        result
    }
}

impl Reset for Cmo {
    fn reset(&mut self) {
        self.up_buffer.fill(0.0);
        self.down_buffer.fill(0.0);
        self.index = 0;
        self.count = 0;
        self.up_sum = 0.0;
        self.down_sum = 0.0;
        self.prev_value = None;
        self.current = None;
    }
}

impl Period for Cmo {
    fn period(&self) -> usize {
        self.period
    }
}

impl Current for Cmo {
    type Output = f64;

    fn current(&self) -> Option<f64> {
        self.current
    }
}

impl Default for Cmo {
    fn default() -> Self {
        Self::new(14)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmo_uptrend() {
        let mut cmo = Cmo::new(5);

        // All upward movement
        for i in 0..10 {
            cmo.next(100.0 + i as f64);
        }

        let value = cmo.current().unwrap();
        assert!((value - 100.0).abs() < 0.01); // All up = +100
    }

    #[test]
    fn test_cmo_downtrend() {
        let mut cmo = Cmo::new(5);

        // All downward movement
        for i in 0..10 {
            cmo.next(100.0 - i as f64);
        }

        let value = cmo.current().unwrap();
        assert!((value - (-100.0)).abs() < 0.01); // All down = -100
    }

    #[test]
    fn test_cmo_reset() {
        let mut cmo = Cmo::default();

        for i in 0..20 {
            cmo.next(100.0 + (i % 5) as f64);
        }

        cmo.reset();
        assert!(cmo.current().is_none());
    }
}
