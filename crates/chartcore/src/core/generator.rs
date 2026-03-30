// Candle Generator - Realistic market simulation
//
// Generate candles for different asset types with realistic characteristics

use super::types::{Candle, Timeframe};
use std::f64::consts::PI;

/// Market type with specific characteristics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketType {
    /// Stock market (9:30-16:00 EST, weekends closed, holidays)
    Stock,
    /// Forex (24/5, weekend gaps)
    Forex,
    /// Crypto (24/7, high volatility)
    Crypto,
    /// Futures (nearly 24/5, varying sessions)
    Futures,
    /// Commodities (specific trading hours)
    Commodities,
}

impl MarketType {
    /// Check if market is open at given timestamp
    pub fn is_market_open(&self, timestamp: i64) -> bool {
        use chrono::{DateTime, Datelike, Timelike, Utc};

        let dt = DateTime::<Utc>::from_timestamp(timestamp / 1000, 0).unwrap();
        let weekday = dt.weekday().num_days_from_monday();
        let hour = dt.hour();

        match self {
            MarketType::Stock => {
                // Monday-Friday, 9:30-16:00 EST (simplified, no holidays)
                weekday < 5 && hour >= 14 && hour < 21 // UTC approximation
            }
            MarketType::Forex => {
                // 24/5 - Sunday evening to Friday evening
                !(weekday == 5 && hour >= 21 || weekday == 6 || weekday == 0 && hour < 21)
            }
            MarketType::Crypto => {
                // Always open
                true
            }
            MarketType::Futures => {
                // Nearly 24/5 with brief gaps
                weekday < 5 || (weekday == 4 && hour < 21)
            }
            MarketType::Commodities => {
                // Similar to futures but more restricted hours
                weekday < 5 && (hour >= 8 && hour < 20)
            }
        }
    }

    /// Get typical volatility multiplier for this market
    pub fn volatility_multiplier(&self) -> f64 {
        match self {
            MarketType::Stock => 1.0,
            MarketType::Forex => 0.5,
            MarketType::Crypto => 3.0,
            MarketType::Futures => 1.5,
            MarketType::Commodities => 1.2,
        }
    }

    /// Get typical liquidity (affects wick size)
    pub fn liquidity(&self) -> f64 {
        match self {
            MarketType::Stock => 0.8,
            MarketType::Forex => 0.9,
            MarketType::Crypto => 0.6,
            MarketType::Futures => 0.85,
            MarketType::Commodities => 0.7,
        }
    }
}

/// Volatility regime
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolatilityRegime {
    /// Low volatility (0.5x)
    Low,
    /// Normal volatility (1.0x)
    Normal,
    /// High volatility (2.0x)
    High,
    /// Extreme volatility (5.0x) - flash crash, news events
    Extreme,
}

impl VolatilityRegime {
    pub fn multiplier(&self) -> f64 {
        match self {
            VolatilityRegime::Low => 0.5,
            VolatilityRegime::Normal => 1.0,
            VolatilityRegime::High => 2.0,
            VolatilityRegime::Extreme => 5.0,
        }
    }
}

/// Market trend
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Trend {
    /// Strong uptrend
    BullishStrong,
    /// Mild uptrend
    BullishMild,
    /// Sideways/ranging
    Sideways,
    /// Mild downtrend
    BearishMild,
    /// Strong downtrend
    BearishStrong,
}

impl Trend {
    /// Get drift per candle (as percentage)
    pub fn drift(&self) -> f64 {
        match self {
            Trend::BullishStrong => 0.002,  // +0.2% per candle
            Trend::BullishMild => 0.0005,   // +0.05%
            Trend::Sideways => 0.0,         // 0%
            Trend::BearishMild => -0.0005,  // -0.05%
            Trend::BearishStrong => -0.002, // -0.2%
        }
    }
}

/// Candle generator configuration
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Market type
    pub market_type: MarketType,
    /// Starting price
    pub initial_price: f64,
    /// Timeframe for candles
    pub timeframe: Timeframe,
    /// Base volatility (daily %)
    pub base_volatility: f64,
    /// Volatility regime
    pub volatility_regime: VolatilityRegime,
    /// Market trend
    pub trend: Trend,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Include gaps (weekends, holidays)
    pub include_gaps: bool,
    /// Mean reversion strength (0.0 = random walk, 1.0 = strong reversion)
    pub mean_reversion: f64,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            market_type: MarketType::Crypto,
            initial_price: 100.0,
            timeframe: Timeframe::M5,
            base_volatility: 2.0, // 2% daily
            volatility_regime: VolatilityRegime::Normal,
            trend: Trend::Sideways,
            seed: 42,
            include_gaps: true,
            mean_reversion: 0.3,
        }
    }
}

impl GeneratorConfig {
    pub fn new(market_type: MarketType) -> Self {
        Self {
            market_type,
            ..Default::default()
        }
    }

    pub fn stock() -> Self {
        Self {
            market_type: MarketType::Stock,
            base_volatility: 1.5,
            include_gaps: true,
            ..Default::default()
        }
    }

    pub fn forex() -> Self {
        Self {
            market_type: MarketType::Forex,
            base_volatility: 0.5,
            include_gaps: true,
            ..Default::default()
        }
    }

    pub fn crypto() -> Self {
        Self {
            market_type: MarketType::Crypto,
            base_volatility: 3.0,
            include_gaps: false,
            ..Default::default()
        }
    }

    pub fn with_volatility(mut self, volatility: f64) -> Self {
        self.base_volatility = volatility;
        self
    }

    pub fn with_trend(mut self, trend: Trend) -> Self {
        self.trend = trend;
        self
    }

    pub fn with_regime(mut self, regime: VolatilityRegime) -> Self {
        self.volatility_regime = regime;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}

/// Candle generator
pub struct CandleGenerator {
    config: GeneratorConfig,
    current_price: f64,
    current_time: i64,
    rng: Xoshiro256PlusPlus,
    /// Current partial candle (for streaming mode)
    current_candle: Option<Candle>,
    /// Time when current candle started
    candle_start_time: i64,
}

impl CandleGenerator {
    /// Create new generator
    pub fn new(config: GeneratorConfig) -> Self {
        let start_time = 1_600_000_000_000; // Sep 2020 as default start
        Self {
            current_price: config.initial_price,
            current_time: start_time,
            rng: Xoshiro256PlusPlus::seed_from_u64(config.seed),
            current_candle: None,
            candle_start_time: start_time,
            config,
        }
    }

    /// Generate N candles
    pub fn generate(&mut self, count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);

        for _ in 0..count {
            if let Some(candle) = self.next() {
                candles.push(candle);
            }
        }

        candles
    }

    /// Generate next candle
    pub fn next(&mut self) -> Option<Candle> {
        let tf_duration = self.config.timeframe.duration_ms();

        // Skip to next market open if needed
        if self.config.include_gaps {
            while !self.config.market_type.is_market_open(self.current_time) {
                self.current_time += tf_duration;
            }
        }

        let candle = self.generate_candle(self.current_time);
        self.current_time += tf_duration;

        Some(candle)
    }

    fn generate_candle(&mut self, time: i64) -> Candle {
        let open = self.current_price;

        // Calculate volatility for this candle
        let vol_multiplier = self.config.market_type.volatility_multiplier()
            * self.config.volatility_regime.multiplier();

        // Scale volatility to timeframe
        let tf_minutes = self.config.timeframe.duration_ms() as f64 / 60_000.0;
        let candle_vol =
            self.config.base_volatility * vol_multiplier * (tf_minutes / 1440.0).sqrt(); // Scale by sqrt(time)

        // Trend drift
        let drift = self.config.trend.drift() * (tf_minutes / 1440.0);

        // Generate price movement with GBM (Geometric Brownian Motion)
        let z = self.rng.normal(0.0, 1.0);
        let price_change = open * (drift + candle_vol * z / 100.0);

        // Mean reversion
        let mean_price = self.config.initial_price;
        let reversion = (mean_price - open) * self.config.mean_reversion * 0.01;

        let raw_close = open + price_change + reversion;

        // Generate high/low with realistic wicks
        let liquidity = self.config.market_type.liquidity();
        let wick_factor = (1.0 - liquidity) * 2.0;

        let range = (open - raw_close).abs() * (1.0 + wick_factor);
        let high_wick = self.rng.gen_range(0.0, range * 0.6);
        let low_wick = self.rng.gen_range(0.0, range * 0.6);

        let high = open.max(raw_close) + high_wick;
        let low = open.min(raw_close) - low_wick;

        // Ensure OHLC validity
        let close = raw_close.max(low).min(high);
        let open = open.max(low).min(high);

        // Generate volume (correlated with volatility)
        let base_volume = 1_000_000.0;
        let vol_impact = 1.0 + (high - low) / open;
        let volume = base_volume * vol_impact * self.rng.gen_range(0.7, 1.3);

        // Update current price for next candle
        self.current_price = close;

        Candle::new(time, open, high, low, close, volume)
    }

    /// Reset generator to initial state
    pub fn reset(&mut self) {
        let start_time = 1_600_000_000_000;
        self.current_price = self.config.initial_price;
        self.current_time = start_time;
        self.candle_start_time = start_time;
        self.current_candle = None;
        self.rng = Xoshiro256PlusPlus::seed_from_u64(self.config.seed);
    }

    /// Start a new streaming candle at current time
    pub fn start_streaming_candle(&mut self) -> Candle {
        let open = self.current_price;
        self.candle_start_time = self.current_time;

        let candle = Candle::new(self.candle_start_time, open, open, open, open, 0.0);

        self.current_candle = Some(candle.clone());
        candle
    }

    /// Update the current streaming candle with partial data
    /// Returns a partial candle showing current state
    pub fn update_streaming_candle(&mut self) -> Option<Candle> {
        let current_candle = self.current_candle.as_ref()?.clone();
        let elapsed = self.current_time - self.candle_start_time;
        let tf_duration = self.config.timeframe.duration_ms();

        if elapsed >= tf_duration {
            // Finalize candle
            return None;
        }

        // Calculate progress (0.0 to 1.0)
        let progress = elapsed as f64 / tf_duration as f64;

        // Simulate intra-candle price movement
        let vol_multiplier = self.config.market_type.volatility_multiplier()
            * self.config.volatility_regime.multiplier();

        let tf_minutes = self.config.timeframe.duration_ms() as f64 / 60_000.0;
        let candle_vol =
            self.config.base_volatility * vol_multiplier * (tf_minutes / 1440.0).sqrt();

        // Random walk from current price
        let z = self.rng.normal(0.0, 1.0);
        let price_change = current_candle.o * (candle_vol * z * progress / 100.0);
        let current = current_candle.c + price_change;

        // Update high/low
        let high = current_candle.h.max(current);
        let low = current_candle.l.min(current);

        // Volume accumulates over time
        let base_volume = 1_000_000.0;
        let volume = base_volume * progress * self.rng.gen_range(0.8, 1.2);

        let updated = Candle::new(
            current_candle.time,
            current_candle.o,
            high,
            low,
            current,
            volume,
        );

        self.current_candle = Some(updated.clone());
        self.current_price = current;

        Some(updated)
    }

    /// Finalize the current streaming candle and start a new one
    pub fn finalize_streaming_candle(&mut self) -> Option<Candle> {
        let final_candle = self.current_candle.take()?;

        // Move to next candle period
        self.current_time = self.candle_start_time + self.config.timeframe.duration_ms();

        Some(final_candle)
    }

    /// Advance time by specified milliseconds (for streaming simulation)
    pub fn advance_time(&mut self, ms: i64) {
        self.current_time += ms;
    }

    /// Generate a realistic scenario with patterns
    pub fn generate_scenario(&mut self, scenario: Scenario, count: usize) -> Vec<Candle> {
        match scenario {
            Scenario::Breakout => self.generate_breakout(count),
            Scenario::Consolidation => self.generate_consolidation(count),
            Scenario::NewsEvent => self.generate_news_event(count),
            Scenario::MarketCrash => self.generate_crash(count),
            Scenario::Recovery => self.generate_recovery(count),
            Scenario::DoubleTop => self.generate_double_top(count),
            Scenario::DoubleBottom => self.generate_double_bottom(count),
        }
    }

    fn generate_breakout(&mut self, count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);
        let consolidation_count = count * 2 / 3;
        let breakout_count = count - consolidation_count;

        // Consolidation phase (low volatility)
        let old_regime = self.config.volatility_regime;
        self.config.volatility_regime = VolatilityRegime::Low;
        for _ in 0..consolidation_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // Breakout phase (high volatility + trend)
        self.config.volatility_regime = VolatilityRegime::High;
        let old_trend = self.config.trend;
        self.config.trend = Trend::BullishStrong;
        for _ in 0..breakout_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        self.config.volatility_regime = old_regime;
        self.config.trend = old_trend;
        candles
    }

    fn generate_consolidation(&mut self, count: usize) -> Vec<Candle> {
        let old_regime = self.config.volatility_regime;
        let old_trend = self.config.trend;
        let old_reversion = self.config.mean_reversion;

        self.config.volatility_regime = VolatilityRegime::Low;
        self.config.trend = Trend::Sideways;
        self.config.mean_reversion = 0.8; // Strong mean reversion

        let candles = self.generate(count);

        self.config.volatility_regime = old_regime;
        self.config.trend = old_trend;
        self.config.mean_reversion = old_reversion;
        candles
    }

    fn generate_news_event(&mut self, count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);
        let normal_count = count * 4 / 5;
        let spike_count = count - normal_count;

        // Normal trading
        for _ in 0..normal_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // News spike (extreme volatility)
        let old_regime = self.config.volatility_regime;
        self.config.volatility_regime = VolatilityRegime::Extreme;
        for _ in 0..spike_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        self.config.volatility_regime = old_regime;
        candles
    }

    fn generate_crash(&mut self, count: usize) -> Vec<Candle> {
        let old_trend = self.config.trend;
        let old_regime = self.config.volatility_regime;

        self.config.trend = Trend::BearishStrong;
        self.config.volatility_regime = VolatilityRegime::Extreme;

        let candles = self.generate(count);

        self.config.trend = old_trend;
        self.config.volatility_regime = old_regime;
        candles
    }

    fn generate_recovery(&mut self, count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);

        // First phase: crash
        let crash_count = count / 4;
        candles.extend(self.generate_crash(crash_count));

        // Second phase: stabilization
        let stable_count = count / 4;
        let old_trend = self.config.trend;
        self.config.trend = Trend::Sideways;
        for _ in 0..stable_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // Third phase: recovery
        let recovery_count = count - crash_count - stable_count;
        self.config.trend = Trend::BullishMild;
        for _ in 0..recovery_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        self.config.trend = old_trend;
        candles
    }

    fn generate_double_top(&mut self, count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);
        let phase_count = count / 5;

        // Up to first peak
        let old_trend = self.config.trend;
        self.config.trend = Trend::BullishStrong;
        for _ in 0..phase_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // Down from first peak
        self.config.trend = Trend::BearishMild;
        for _ in 0..phase_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // Up to second peak
        self.config.trend = Trend::BullishStrong;
        for _ in 0..phase_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // Down from second peak (breakdown)
        self.config.trend = Trend::BearishStrong;
        for _ in 0..count - 3 * phase_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        self.config.trend = old_trend;
        candles
    }

    fn generate_double_bottom(&mut self, count: usize) -> Vec<Candle> {
        let mut candles = Vec::with_capacity(count);
        let phase_count = count / 5;

        let old_trend = self.config.trend;

        // Down to first bottom
        self.config.trend = Trend::BearishStrong;
        for _ in 0..phase_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // Up from first bottom
        self.config.trend = Trend::BullishMild;
        for _ in 0..phase_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // Down to second bottom
        self.config.trend = Trend::BearishStrong;
        for _ in 0..phase_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        // Breakout up
        self.config.trend = Trend::BullishStrong;
        for _ in 0..count - 3 * phase_count {
            if let Some(c) = self.next() {
                candles.push(c);
            }
        }

        self.config.trend = old_trend;
        candles
    }
}

/// Predefined market scenarios for testing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Scenario {
    /// Price consolidates then breaks out
    Breakout,
    /// Sideways consolidation with tight range
    Consolidation,
    /// Sudden news-driven volatility spike
    NewsEvent,
    /// Rapid market crash
    MarketCrash,
    /// Crash followed by recovery
    Recovery,
    /// Double top pattern
    DoubleTop,
    /// Double bottom pattern
    DoubleBottom,
}

// Simple PRNG (Xoshiro256++)
struct Xoshiro256PlusPlus {
    s: [u64; 4],
}

impl Xoshiro256PlusPlus {
    fn seed_from_u64(seed: u64) -> Self {
        let mut s = [0u64; 4];
        s[0] = seed;
        s[1] = seed.wrapping_mul(0x9e3779b97f4a7c15);
        s[2] = seed.wrapping_mul(0xbf58476d1ce4e5b9);
        s[3] = seed.wrapping_mul(0x94d049bb133111eb);
        Self { s }
    }

    fn next(&mut self) -> u64 {
        let result = self.s[0]
            .wrapping_add(self.s[3])
            .rotate_left(23)
            .wrapping_add(self.s[0]);

        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);

        result
    }

    fn gen_range(&mut self, min: f64, max: f64) -> f64 {
        let u = (self.next() >> 11) as f64 / (1u64 << 53) as f64;
        min + (max - min) * u
    }

    fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        // Box-Muller transform
        let u1 = self.gen_range(1e-10, 1.0);
        let u2 = self.gen_range(0.0, 1.0);
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        mean + std_dev * z
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generator_basic() {
        let config = GeneratorConfig::crypto();
        let mut gen = CandleGenerator::new(config);

        let candles = gen.generate(100);
        assert_eq!(candles.len(), 100);

        // Check OHLC validity
        for candle in &candles {
            assert!(candle.h >= candle.o);
            assert!(candle.h >= candle.c);
            assert!(candle.l <= candle.o);
            assert!(candle.l <= candle.c);
            assert!(candle.v > 0.0);
        }
    }

    #[test]
    fn test_trend() {
        let config = GeneratorConfig::crypto()
            .with_trend(Trend::BullishStrong)
            .with_seed(42);

        let mut gen = CandleGenerator::new(config);
        let candles = gen.generate(1000);

        // Price should trend up
        assert!(candles.last().unwrap().c > candles.first().unwrap().c);
    }

    #[test]
    fn test_market_types() {
        for market in [MarketType::Stock, MarketType::Forex, MarketType::Crypto] {
            let config = GeneratorConfig::new(market);
            let mut gen = CandleGenerator::new(config);
            let candles = gen.generate(50);

            assert_eq!(candles.len(), 50);
        }
    }

    #[test]
    fn test_reproducibility() {
        let config = GeneratorConfig::crypto().with_seed(123);

        let mut gen1 = CandleGenerator::new(config.clone());
        let candles1 = gen1.generate(10);

        let mut gen2 = CandleGenerator::new(config);
        let candles2 = gen2.generate(10);

        for (c1, c2) in candles1.iter().zip(candles2.iter()) {
            assert_eq!(c1.o, c2.o);
            assert_eq!(c1.h, c2.h);
            assert_eq!(c1.l, c2.l);
            assert_eq!(c1.c, c2.c);
        }
    }
}
