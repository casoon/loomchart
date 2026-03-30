//! Smart Money Concepts (SMC) implementation.
//!
//! Implements institutional trading concepts:
//! - Order Blocks (OB)
//! - Fair Value Gaps (FVG) / Imbalances
//! - Liquidity zones (buy-side/sell-side)
//! - Break of Structure (BOS)
//! - Change of Character (CHoCH)
//! - Premium/Discount zones

use loom_core::{Candle, OHLCV, Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Market structure type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MarketStructure {
    /// Bullish structure (higher highs, higher lows)
    Bullish,
    /// Bearish structure (lower highs, lower lows)
    Bearish,
    /// Ranging/consolidation
    Ranging,
}

/// Swing point (high or low)
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Swing {
    /// Bar index
    pub index: usize,
    /// Timestamp
    pub time: Timestamp,
    /// Price
    pub price: Price,
    /// Is this a swing high (true) or swing low (false)
    pub is_high: bool,
    /// Has this swing been broken
    pub broken: bool,
}

/// Order Block - Institutional supply/demand zone
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OrderBlock {
    /// Bar index where OB formed
    pub index: usize,
    /// Timestamp
    pub time: Timestamp,
    /// Top of the order block zone
    pub top: Price,
    /// Bottom of the order block zone
    pub bottom: Price,
    /// Is this a bullish OB (demand) or bearish OB (supply)
    pub is_bullish: bool,
    /// Has price returned to this OB (mitigated)
    pub mitigated: bool,
    /// Strength score (0-100)
    pub strength: f64,
}

impl OrderBlock {
    /// Check if price is within this order block
    pub fn contains(&self, price: Price) -> bool {
        price >= self.bottom && price <= self.top
    }

    /// Distance from current price (negative if price is below for bullish OB)
    pub fn distance_from(&self, price: Price) -> Price {
        if self.is_bullish {
            price - self.top // Negative when above OB
        } else {
            self.bottom - price // Negative when below OB
        }
    }

    /// Zone height
    pub fn height(&self) -> Price {
        self.top - self.bottom
    }
}

/// Fair Value Gap (FVG) - Price imbalance
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FairValueGap {
    /// Bar index of the middle candle
    pub index: usize,
    /// Timestamp
    pub time: Timestamp,
    /// Top of the gap
    pub top: Price,
    /// Bottom of the gap
    pub bottom: Price,
    /// Is this a bullish FVG (gap up) or bearish FVG (gap down)
    pub is_bullish: bool,
    /// Has price filled this gap
    pub filled: bool,
    /// Fill percentage (0-100)
    pub fill_percent: f64,
}

impl FairValueGap {
    /// Gap size
    pub fn size(&self) -> Price {
        self.top - self.bottom
    }

    /// Check if price is within gap
    pub fn contains(&self, price: Price) -> bool {
        price >= self.bottom && price <= self.top
    }

    /// 50% level of the gap (equilibrium)
    pub fn equilibrium(&self) -> Price {
        (self.top + self.bottom) / 2.0
    }
}

/// Liquidity zone (cluster of stops)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiquidityZone {
    /// Price level
    pub level: Price,
    /// Number of equal highs/lows
    pub touches: usize,
    /// Is this buy-side (above price) or sell-side (below price) liquidity
    pub is_buy_side: bool,
    /// Has this liquidity been taken
    pub swept: bool,
    /// Strength based on touches and time
    pub strength: f64,
}

/// Break of Structure
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BreakOfStructure {
    /// Bar index
    pub index: usize,
    /// Timestamp
    pub time: Timestamp,
    /// Broken swing level
    pub level: Price,
    /// Is this a bullish BOS (broke above) or bearish BOS (broke below)
    pub is_bullish: bool,
    /// Is this a Change of Character (first break against trend)
    pub is_choch: bool,
}

/// Smart Money Concepts analyzer
pub struct SmcAnalyzer {
    /// Swing detection lookback
    swing_lookback: usize,
    /// Equal high/low tolerance (percentage)
    equal_tolerance: f64,
    /// Minimum FVG size (percentage of price)
    min_fvg_size: f64,
}

impl SmcAnalyzer {
    pub fn new() -> Self {
        Self {
            swing_lookback: 5,
            equal_tolerance: 0.001, // 0.1%
            min_fvg_size: 0.001,    // 0.1%
        }
    }

    pub fn with_swing_lookback(mut self, lookback: usize) -> Self {
        self.swing_lookback = lookback;
        self
    }

    /// Find swing highs and lows
    pub fn find_swings(&self, candles: &[Candle]) -> Vec<Swing> {
        let mut swings = Vec::new();
        let lb = self.swing_lookback;

        if candles.len() < lb * 2 + 1 {
            return swings;
        }

        for i in lb..candles.len() - lb {
            // Check for swing high
            let is_swing_high = (0..lb).all(|j| candles[i].high > candles[i - j - 1].high)
                && (0..lb).all(|j| candles[i].high > candles[i + j + 1].high);

            // Check for swing low
            let is_swing_low = (0..lb).all(|j| candles[i].low < candles[i - j - 1].low)
                && (0..lb).all(|j| candles[i].low < candles[i + j + 1].low);

            if is_swing_high {
                swings.push(Swing {
                    index: i,
                    time: candles[i].time,
                    price: candles[i].high,
                    is_high: true,
                    broken: false,
                });
            }

            if is_swing_low {
                swings.push(Swing {
                    index: i,
                    time: candles[i].time,
                    price: candles[i].low,
                    is_high: false,
                    broken: false,
                });
            }
        }

        // Mark broken swings
        if let Some(last_candle) = candles.last() {
            for swing in &mut swings {
                if swing.is_high && last_candle.high > swing.price {
                    swing.broken = true;
                } else if !swing.is_high && last_candle.low < swing.price {
                    swing.broken = true;
                }
            }
        }

        swings
    }

    /// Determine market structure
    pub fn market_structure(&self, candles: &[Candle]) -> MarketStructure {
        let swings = self.find_swings(candles);

        // Need at least 4 swings for structure determination
        if swings.len() < 4 {
            return MarketStructure::Ranging;
        }

        // Get last 4 swing points
        let recent: Vec<_> = swings.iter().rev().take(4).collect();

        // Extract highs and lows
        let highs: Vec<_> = recent.iter().filter(|s| s.is_high).collect();
        let lows: Vec<_> = recent.iter().filter(|s| !s.is_high).collect();

        if highs.len() >= 2 && lows.len() >= 2 {
            let higher_highs = highs[0].price > highs[1].price;
            let higher_lows = lows[0].price > lows[1].price;
            let lower_highs = highs[0].price < highs[1].price;
            let lower_lows = lows[0].price < lows[1].price;

            if higher_highs && higher_lows {
                return MarketStructure::Bullish;
            }
            if lower_highs && lower_lows {
                return MarketStructure::Bearish;
            }
        }

        MarketStructure::Ranging
    }

    /// Find Fair Value Gaps (imbalances)
    pub fn find_fvgs(&self, candles: &[Candle]) -> Vec<FairValueGap> {
        let mut fvgs = Vec::new();

        if candles.len() < 3 {
            return fvgs;
        }

        for i in 1..candles.len() - 1 {
            let prev = &candles[i - 1];
            let curr = &candles[i];
            let next = &candles[i + 1];

            // Bullish FVG: Gap between candle 1 high and candle 3 low
            if next.low > prev.high {
                let gap_size = next.low - prev.high;
                if gap_size / curr.close > self.min_fvg_size {
                    fvgs.push(FairValueGap {
                        index: i,
                        time: curr.time,
                        top: next.low,
                        bottom: prev.high,
                        is_bullish: true,
                        filled: false,
                        fill_percent: 0.0,
                    });
                }
            }

            // Bearish FVG: Gap between candle 3 high and candle 1 low
            if prev.low > next.high {
                let gap_size = prev.low - next.high;
                if gap_size / curr.close > self.min_fvg_size {
                    fvgs.push(FairValueGap {
                        index: i,
                        time: curr.time,
                        top: prev.low,
                        bottom: next.high,
                        is_bullish: false,
                        filled: false,
                        fill_percent: 0.0,
                    });
                }
            }
        }

        // Check if FVGs have been filled
        if let Some(last) = candles.last() {
            for fvg in &mut fvgs {
                if fvg.is_bullish {
                    if last.low <= fvg.bottom {
                        fvg.filled = true;
                        fvg.fill_percent = 100.0;
                    } else if last.low < fvg.top {
                        fvg.fill_percent = (fvg.top - last.low) / fvg.size() * 100.0;
                    }
                } else {
                    if last.high >= fvg.top {
                        fvg.filled = true;
                        fvg.fill_percent = 100.0;
                    } else if last.high > fvg.bottom {
                        fvg.fill_percent = (last.high - fvg.bottom) / fvg.size() * 100.0;
                    }
                }
            }
        }

        fvgs
    }

    /// Find Order Blocks
    pub fn find_order_blocks(&self, candles: &[Candle]) -> Vec<OrderBlock> {
        let mut obs = Vec::new();

        if candles.len() < 3 {
            return obs;
        }

        for i in 0..candles.len() - 2 {
            let curr = &candles[i];
            let next = &candles[i + 1];
            let after = &candles[i + 2];

            // Bullish Order Block:
            // Last bearish candle before a strong bullish move
            if curr.is_bearish()
                && next.is_bullish()
                && after.is_bullish()
                && after.close > curr.high
            {
                let strength = self.calculate_ob_strength(curr, &candles[i + 1..]);
                obs.push(OrderBlock {
                    index: i,
                    time: curr.time,
                    top: curr.high,
                    bottom: curr.low,
                    is_bullish: true,
                    mitigated: false,
                    strength,
                });
            }

            // Bearish Order Block:
            // Last bullish candle before a strong bearish move
            if curr.is_bullish()
                && next.is_bearish()
                && after.is_bearish()
                && after.close < curr.low
            {
                let strength = self.calculate_ob_strength(curr, &candles[i + 1..]);
                obs.push(OrderBlock {
                    index: i,
                    time: curr.time,
                    top: curr.high,
                    bottom: curr.low,
                    is_bullish: false,
                    mitigated: false,
                    strength,
                });
            }
        }

        // Check mitigation
        if let Some(last) = candles.last() {
            for ob in &mut obs {
                if ob.is_bullish && last.low < ob.bottom {
                    ob.mitigated = true;
                } else if !ob.is_bullish && last.high > ob.top {
                    ob.mitigated = true;
                }
            }
        }

        obs
    }

    /// Find liquidity zones (equal highs/lows)
    pub fn find_liquidity_zones(&self, candles: &[Candle]) -> Vec<LiquidityZone> {
        let mut zones = Vec::new();
        let swings = self.find_swings(candles);

        // Group swing highs by similar price
        let swing_highs: Vec<_> = swings.iter().filter(|s| s.is_high).collect();
        let swing_lows: Vec<_> = swings.iter().filter(|s| !s.is_high).collect();

        // Find clusters of equal highs (buy-side liquidity)
        zones.extend(self.find_equal_levels(&swing_highs, true));

        // Find clusters of equal lows (sell-side liquidity)
        zones.extend(self.find_equal_levels(&swing_lows, false));

        zones
    }

    /// Detect Break of Structure
    pub fn find_bos(&self, candles: &[Candle]) -> Vec<BreakOfStructure> {
        let mut bos_list = Vec::new();
        let swings = self.find_swings(candles);

        if swings.len() < 2 {
            return bos_list;
        }

        let mut prev_structure = MarketStructure::Ranging;

        // Track last significant swing high and low
        let mut last_swing_high: Option<&Swing> = None;
        let mut last_swing_low: Option<&Swing> = None;

        for swing in &swings {
            if swing.is_high {
                last_swing_high = Some(swing);
            } else {
                last_swing_low = Some(swing);
            }
        }

        // Check recent candles for BOS
        for i in swings.len().saturating_sub(5)..swings.len() {
            let swing = &swings[i];

            if swing.broken {
                let is_choch = match prev_structure {
                    MarketStructure::Bullish => !swing.is_high, // Breaking low in uptrend
                    MarketStructure::Bearish => swing.is_high,  // Breaking high in downtrend
                    _ => false,
                };

                bos_list.push(BreakOfStructure {
                    index: swing.index,
                    time: swing.time,
                    level: swing.price,
                    is_bullish: swing.is_high,
                    is_choch,
                });

                // Update structure after break
                if swing.is_high {
                    prev_structure = MarketStructure::Bullish;
                } else {
                    prev_structure = MarketStructure::Bearish;
                }
            }
        }

        bos_list
    }

    /// Calculate premium/discount zones
    ///
    /// Returns (discount_top, equilibrium, premium_bottom)
    pub fn premium_discount_zones(&self, candles: &[Candle], lookback: usize) -> Option<(Price, Price, Price)> {
        if candles.len() < lookback {
            return None;
        }

        let slice = &candles[candles.len() - lookback..];
        let high = slice.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
        let low = slice.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);

        let range = high - low;
        let equilibrium = low + range * 0.5;
        let discount_top = low + range * 0.5;  // Below 50% = discount
        let premium_bottom = low + range * 0.5; // Above 50% = premium

        Some((discount_top, equilibrium, premium_bottom))
    }

    // Helper functions
    fn calculate_ob_strength(&self, ob_candle: &Candle, following: &[Candle]) -> f64 {
        let mut score = 0.0;

        // Larger body = stronger OB
        if ob_candle.body_ratio() > 0.7 {
            score += 30.0;
        } else if ob_candle.body_ratio() > 0.5 {
            score += 20.0;
        }

        // Strong move away from OB
        if following.len() >= 3 {
            let move_size = (following[2].close - ob_candle.close).abs() / ob_candle.close;
            score += (move_size * 1000.0).min(40.0);
        }

        // Not yet tested (fresher = stronger)
        score += 30.0;

        score.min(100.0)
    }

    fn find_equal_levels(&self, swings: &[&Swing], is_high: bool) -> Vec<LiquidityZone> {
        let mut zones = Vec::new();

        for i in 0..swings.len() {
            let mut touches = 1;
            let base_price = swings[i].price;

            for j in (i + 1)..swings.len() {
                let diff = (swings[j].price - base_price).abs() / base_price;
                if diff < self.equal_tolerance {
                    touches += 1;
                }
            }

            if touches >= 2 {
                zones.push(LiquidityZone {
                    level: base_price,
                    touches,
                    is_buy_side: is_high,
                    swept: swings[i].broken,
                    strength: (touches as f64 * 20.0).min(100.0),
                });
            }
        }

        // Deduplicate by merging similar levels
        zones.sort_by(|a, b| a.level.partial_cmp(&b.level).unwrap());
        zones.dedup_by(|a, b| {
            let diff = (a.level - b.level).abs() / b.level;
            if diff < self.equal_tolerance * 2.0 {
                b.touches = b.touches.max(a.touches);
                b.strength = b.strength.max(a.strength);
                true
            } else {
                false
            }
        });

        zones
    }
}

impl Default for SmcAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(o: f64, h: f64, l: f64, c: f64, time: i64) -> Candle {
        Candle::new(time, o, h, l, c, 1000.0)
    }

    #[test]
    fn test_fvg_detection() {
        let candles = vec![
            make_candle(100.0, 102.0, 99.0, 101.0, 1000),
            make_candle(101.0, 110.0, 100.0, 108.0, 2000), // Big bullish
            make_candle(108.0, 112.0, 107.0, 111.0, 3000), // Gap: 107 > 102
        ];

        let analyzer = SmcAnalyzer::new();
        let fvgs = analyzer.find_fvgs(&candles);

        assert_eq!(fvgs.len(), 1);
        assert!(fvgs[0].is_bullish);
        assert_eq!(fvgs[0].bottom, 102.0);
        assert_eq!(fvgs[0].top, 107.0);
    }

    #[test]
    fn test_market_structure() {
        // Create bullish structure
        let candles: Vec<Candle> = (0..20)
            .map(|i| {
                let base = 100.0 + (i as f64 * 2.0);
                make_candle(base, base + 3.0, base - 1.0, base + 2.0, i * 60000)
            })
            .collect();

        let analyzer = SmcAnalyzer::new().with_swing_lookback(3);
        let structure = analyzer.market_structure(&candles);

        assert_eq!(structure, MarketStructure::Bullish);
    }
}
