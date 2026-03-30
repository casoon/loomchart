// Core types for chartcore-indicators

use serde::{Deserialize, Serialize};

/// OHLCV Candle - minimal type for indicator calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub time: i64,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
}

impl Candle {
    pub fn new(time: i64, o: f64, h: f64, l: f64, c: f64, v: f64) -> Self {
        Self {
            time,
            o,
            h,
            l,
            c,
            v,
        }
    }
}
