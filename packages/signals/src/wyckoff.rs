//! Wyckoff Method implementation.
//!
//! Detects accumulation/distribution phases, springs, upthrusts,
//! and other Wyckoff events.
//!
//! ## Wyckoff Phases
//!
//! ### Accumulation
//! 1. PS (Preliminary Support) - First buying after downtrend
//! 2. SC (Selling Climax) - Wide spread, high volume capitulation
//! 3. AR (Automatic Rally) - Short covering rally
//! 4. ST (Secondary Test) - Tests SC low on lower volume
//! 5. Spring - False breakdown below support (trap)
//! 6. SOS (Sign of Strength) - Rally on increasing volume
//! 7. LPS (Last Point of Support) - Pullback before markup
//!
//! ### Distribution
//! 1. PSY (Preliminary Supply) - First selling after uptrend
//! 2. BC (Buying Climax) - Wide spread, high volume top
//! 3. AR (Automatic Reaction) - Profit taking decline
//! 4. ST (Secondary Test) - Tests BC high on lower volume
//! 5. Upthrust - False breakout above resistance (trap)
//! 6. SOW (Sign of Weakness) - Decline on increasing volume
//! 7. LPSY (Last Point of Supply) - Rally before markdown

use loom_core::{Candle, OHLCV, Price, Volume};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Wyckoff market phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WyckoffPhase {
    /// Markup phase (uptrend)
    Markup,
    /// Distribution phase (top)
    Distribution,
    /// Markdown phase (downtrend)
    Markdown,
    /// Accumulation phase (bottom)
    Accumulation,
    /// Unknown/transitional
    Unknown,
}

/// Wyckoff event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WyckoffEvent {
    /// Preliminary Support
    PreliminarySupport,
    /// Selling Climax
    SellingClimax,
    /// Automatic Rally
    AutomaticRally,
    /// Secondary Test of low
    SecondaryTestLow,
    /// Spring (false breakdown)
    Spring,
    /// Sign of Strength
    SignOfStrength,
    /// Last Point of Support
    LastPointOfSupport,
    /// Preliminary Supply
    PreliminarySupply,
    /// Buying Climax
    BuyingClimax,
    /// Automatic Reaction
    AutomaticReaction,
    /// Secondary Test of high
    SecondaryTestHigh,
    /// Upthrust (false breakout)
    Upthrust,
    /// Sign of Weakness
    SignOfWeakness,
    /// Last Point of Supply
    LastPointOfSupply,
}

impl WyckoffEvent {
    /// Is this a bullish event
    pub fn is_bullish(&self) -> bool {
        matches!(
            self,
            Self::Spring | Self::SignOfStrength | Self::LastPointOfSupport
        )
    }

    /// Is this a bearish event
    pub fn is_bearish(&self) -> bool {
        matches!(
            self,
            Self::Upthrust | Self::SignOfWeakness | Self::LastPointOfSupply
        )
    }
}

/// Spring detection result
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Spring {
    /// Bar index where spring occurred
    pub bar_index: usize,
    /// Price low of the spring
    pub low: Price,
    /// Support level that was broken
    pub support: Price,
    /// Volume at spring
    pub volume: Volume,
    /// Strength score (0-100)
    pub strength: f64,
}

/// Upthrust detection result
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Upthrust {
    /// Bar index where upthrust occurred
    pub bar_index: usize,
    /// Price high of the upthrust
    pub high: Price,
    /// Resistance level that was broken
    pub resistance: Price,
    /// Volume at upthrust
    pub volume: Volume,
    /// Strength score (0-100)
    pub strength: f64,
}

/// Sign of Strength
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignOfStrength {
    pub bar_index: usize,
    pub close: Price,
    pub volume_ratio: f64, // Volume relative to average
}

/// Sign of Weakness
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignOfWeakness {
    pub bar_index: usize,
    pub close: Price,
    pub volume_ratio: f64,
}

/// Wyckoff analyzer
pub struct WyckoffAnalyzer {
    /// Lookback period for ranges
    lookback: usize,
    /// Volume average period
    volume_period: usize,
    /// Threshold for volume spikes
    volume_spike_threshold: f64,
    /// Threshold for range breakout
    breakout_threshold: f64,
}

impl WyckoffAnalyzer {
    pub fn new() -> Self {
        Self {
            lookback: 50,
            volume_period: 20,
            volume_spike_threshold: 2.0,
            breakout_threshold: 0.02, // 2% beyond range
        }
    }

    pub fn with_lookback(mut self, lookback: usize) -> Self {
        self.lookback = lookback;
        self
    }

    pub fn with_volume_period(mut self, period: usize) -> Self {
        self.volume_period = period;
        self
    }

    /// Detect spring pattern
    ///
    /// A spring occurs when price breaks below support but quickly
    /// recovers and closes back above it, typically on lower volume.
    pub fn detect_spring(&self, candles: &[Candle]) -> Option<Spring> {
        if candles.len() < self.lookback + 3 {
            return None;
        }

        let len = candles.len();
        let current = &candles[len - 1];
        let prev = &candles[len - 2];

        // Find support level (lowest low in lookback, excluding last 3 bars)
        let support = candles[len - self.lookback..len - 3]
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min);

        // Calculate average volume
        let avg_volume = self.average_volume(&candles[len - self.volume_period - 1..len - 1]);

        // Spring criteria:
        // 1. Previous bar broke below support
        // 2. Current bar closes above support
        // 3. Volume is relatively low (not climactic)
        if prev.low < support
            && current.close > support
            && current.is_bullish()
            && prev.volume < avg_volume * self.volume_spike_threshold
        {
            let strength = self.calculate_spring_strength(prev, current, support, avg_volume);
            return Some(Spring {
                bar_index: len - 1,
                low: prev.low,
                support,
                volume: prev.volume,
                strength,
            });
        }

        None
    }

    /// Detect upthrust pattern
    ///
    /// An upthrust occurs when price breaks above resistance but quickly
    /// fails and closes back below it.
    pub fn detect_upthrust(&self, candles: &[Candle]) -> Option<Upthrust> {
        if candles.len() < self.lookback + 3 {
            return None;
        }

        let len = candles.len();
        let current = &candles[len - 1];
        let prev = &candles[len - 2];

        // Find resistance level
        let resistance = candles[len - self.lookback..len - 3]
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);

        let avg_volume = self.average_volume(&candles[len - self.volume_period - 1..len - 1]);

        // Upthrust criteria:
        // 1. Previous bar broke above resistance
        // 2. Current bar closes below resistance
        // 3. Volume is relatively low
        if prev.high > resistance
            && current.close < resistance
            && current.is_bearish()
            && prev.volume < avg_volume * self.volume_spike_threshold
        {
            let strength = self.calculate_upthrust_strength(prev, current, resistance, avg_volume);
            return Some(Upthrust {
                bar_index: len - 1,
                high: prev.high,
                resistance,
                volume: prev.volume,
                strength,
            });
        }

        None
    }

    /// Detect selling climax
    ///
    /// Wide spread, high volume bar with long lower shadow,
    /// followed by reversal.
    pub fn detect_selling_climax(&self, candles: &[Candle]) -> Option<usize> {
        if candles.len() < self.volume_period + 2 {
            return None;
        }

        let len = candles.len();
        let bar = &candles[len - 2];
        let next = &candles[len - 1];
        let avg_volume = self.average_volume(&candles[len - self.volume_period - 2..len - 2]);

        // Selling climax criteria:
        // 1. High volume (> 2x average)
        // 2. Wide spread (large range)
        // 3. Closes off lows (long lower shadow)
        // 4. Followed by bullish bar
        let avg_range = self.average_range(&candles[len - self.volume_period - 2..len - 2]);

        if bar.volume > avg_volume * 2.0
            && bar.range() > avg_range * 1.5
            && bar.lower_shadow() > bar.body()
            && next.is_bullish()
        {
            return Some(len - 2);
        }

        None
    }

    /// Detect buying climax
    pub fn detect_buying_climax(&self, candles: &[Candle]) -> Option<usize> {
        if candles.len() < self.volume_period + 2 {
            return None;
        }

        let len = candles.len();
        let bar = &candles[len - 2];
        let next = &candles[len - 1];
        let avg_volume = self.average_volume(&candles[len - self.volume_period - 2..len - 2]);
        let avg_range = self.average_range(&candles[len - self.volume_period - 2..len - 2]);

        if bar.volume > avg_volume * 2.0
            && bar.range() > avg_range * 1.5
            && bar.upper_shadow() > bar.body()
            && next.is_bearish()
        {
            return Some(len - 2);
        }

        None
    }

    /// Detect Sign of Strength (SOS)
    ///
    /// Rally with increasing volume, breaking previous high
    pub fn detect_sos(&self, candles: &[Candle]) -> Option<SignOfStrength> {
        if candles.len() < self.lookback {
            return None;
        }

        let len = candles.len();
        let current = &candles[len - 1];

        // Find previous swing high
        let prev_high = candles[len - self.lookback..len - 1]
            .iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);

        let avg_volume = self.average_volume(&candles[len - self.volume_period..]);

        // SOS: Close above previous high on increasing volume
        if current.close > prev_high
            && current.is_bullish()
            && current.volume > avg_volume * 1.2
        {
            return Some(SignOfStrength {
                bar_index: len - 1,
                close: current.close,
                volume_ratio: current.volume / avg_volume,
            });
        }

        None
    }

    /// Detect Sign of Weakness (SOW)
    pub fn detect_sow(&self, candles: &[Candle]) -> Option<SignOfWeakness> {
        if candles.len() < self.lookback {
            return None;
        }

        let len = candles.len();
        let current = &candles[len - 1];

        let prev_low = candles[len - self.lookback..len - 1]
            .iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min);

        let avg_volume = self.average_volume(&candles[len - self.volume_period..]);

        if current.close < prev_low
            && current.is_bearish()
            && current.volume > avg_volume * 1.2
        {
            return Some(SignOfWeakness {
                bar_index: len - 1,
                close: current.close,
                volume_ratio: current.volume / avg_volume,
            });
        }

        None
    }

    /// Analyze current Wyckoff phase
    pub fn analyze_phase(&self, candles: &[Candle]) -> WyckoffPhase {
        if candles.len() < self.lookback * 2 {
            return WyckoffPhase::Unknown;
        }

        let len = candles.len();

        // Calculate trend direction
        let first_half_avg = self.average_close(&candles[len - self.lookback * 2..len - self.lookback]);
        let second_half_avg = self.average_close(&candles[len - self.lookback..]);

        // Calculate range contraction/expansion
        let first_half_range = self.average_range(&candles[len - self.lookback * 2..len - self.lookback]);
        let second_half_range = self.average_range(&candles[len - self.lookback..]);

        let trend = second_half_avg - first_half_avg;
        let range_change = second_half_range / first_half_range;

        // Markup: Rising prices, expanding ranges
        if trend > 0.0 && range_change > 1.0 {
            return WyckoffPhase::Markup;
        }

        // Markdown: Falling prices, expanding ranges
        if trend < 0.0 && range_change > 1.0 {
            return WyckoffPhase::Markdown;
        }

        // Accumulation: Contracting ranges at lows
        if range_change < 0.8 && second_half_avg < first_half_avg * 1.02 {
            return WyckoffPhase::Accumulation;
        }

        // Distribution: Contracting ranges at highs
        if range_change < 0.8 && second_half_avg > first_half_avg * 0.98 {
            return WyckoffPhase::Distribution;
        }

        WyckoffPhase::Unknown
    }

    // Helper functions
    fn average_volume(&self, candles: &[Candle]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }
        candles.iter().map(|c| c.volume).sum::<f64>() / candles.len() as f64
    }

    fn average_range(&self, candles: &[Candle]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }
        candles.iter().map(|c| c.range()).sum::<f64>() / candles.len() as f64
    }

    fn average_close(&self, candles: &[Candle]) -> f64 {
        if candles.is_empty() {
            return 0.0;
        }
        candles.iter().map(|c| c.close).sum::<f64>() / candles.len() as f64
    }

    fn calculate_spring_strength(
        &self,
        spring_bar: &Candle,
        recovery_bar: &Candle,
        support: Price,
        avg_volume: Volume,
    ) -> f64 {
        let mut score = 0.0;

        // Recovery strength (how much above support)
        let recovery_pct = (recovery_bar.close - support) / support * 100.0;
        score += recovery_pct.min(20.0) * 2.0; // Max 40 points

        // Volume factor (lower is better for spring)
        let vol_ratio = spring_bar.volume / avg_volume;
        if vol_ratio < 0.5 {
            score += 30.0;
        } else if vol_ratio < 1.0 {
            score += 20.0;
        } else if vol_ratio < 1.5 {
            score += 10.0;
        }

        // Candle structure
        if recovery_bar.body_ratio() > 0.6 {
            score += 15.0; // Strong body
        }

        if spring_bar.lower_shadow() > spring_bar.body() {
            score += 15.0; // Rejection tail
        }

        score.min(100.0)
    }

    fn calculate_upthrust_strength(
        &self,
        upthrust_bar: &Candle,
        failure_bar: &Candle,
        resistance: Price,
        avg_volume: Volume,
    ) -> f64 {
        let mut score = 0.0;

        let failure_pct = (resistance - failure_bar.close) / resistance * 100.0;
        score += failure_pct.min(20.0) * 2.0;

        let vol_ratio = upthrust_bar.volume / avg_volume;
        if vol_ratio < 0.5 {
            score += 30.0;
        } else if vol_ratio < 1.0 {
            score += 20.0;
        } else if vol_ratio < 1.5 {
            score += 10.0;
        }

        if failure_bar.body_ratio() > 0.6 {
            score += 15.0;
        }

        if upthrust_bar.upper_shadow() > upthrust_bar.body() {
            score += 15.0;
        }

        score.min(100.0)
    }
}

impl Default for WyckoffAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candle(o: f64, h: f64, l: f64, c: f64, v: f64, time: i64) -> Candle {
        Candle::new(time, o, h, l, c, v)
    }

    #[test]
    fn test_wyckoff_phase() {
        let analyzer = WyckoffAnalyzer::new().with_lookback(10);

        // Create uptrend data
        let candles: Vec<Candle> = (0..30)
            .map(|i| {
                let base = 100.0 + i as f64 * 2.0;
                make_candle(base, base + 3.0, base - 1.0, base + 2.0, 1000.0, i * 60000)
            })
            .collect();

        let phase = analyzer.analyze_phase(&candles);
        assert_eq!(phase, WyckoffPhase::Markup);
    }
}
