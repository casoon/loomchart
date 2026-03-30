// Built-in Indicator Plugins (Rust implementations)
// Total: 79+ indicators across 6 categories

// === Moving Averages (13) ===
pub mod alma;
pub mod dema;
pub mod ema;
pub mod hma;
pub mod jma;
pub mod lsma;
pub mod mcginley;
pub mod rma;
pub mod sma;
pub mod smma;
pub mod tema;
pub mod vwma;
pub mod wma;

// === Momentum/Oscillators (19) ===
pub mod awesome;
pub mod cci;
pub mod chande_mo;
pub mod coppock;
pub mod dpo;
pub mod macd;
pub mod mfi;
pub mod momentum;
pub mod price_osc;
pub mod roc;
pub mod rsi;
pub mod rvi;
pub mod smi_ergodic;
pub mod stoch;
pub mod stoch_rsi;
pub mod trix;
pub mod tsi;
pub mod ultimate;
pub mod williams_r;

// === Trend (10) ===
pub mod adx;
pub mod aroon;
pub mod dmi;
pub mod ichimoku;
pub mod ma_cross;
pub mod ma_ribbon;
pub mod parabolic_sar;
pub mod supertrend;
pub mod trend_strength;
pub mod zigzag;

// === Volatility (13) ===
pub mod adr;
pub mod atr;
pub mod bollinger;
pub mod choppiness;
pub mod donchian;
pub mod envelope;
pub mod hist_vol;
pub mod keltner;
pub mod mass_index;
pub mod stdev;

// === Volume (12) ===
pub mod chaikin;
pub mod cvd;
pub mod elder_force;
pub mod eom;
pub mod klinger;
pub mod obv;
pub mod pvt;
pub mod volume_osc;

// === Other (9) ===
pub mod alligator;
pub mod bop;
pub mod bull_bear_power;
pub mod chande_kroll;
pub mod fisher;
pub mod median;
pub mod rci;
pub mod vortex;
pub mod woodies_cci;

// Re-exports - Moving Averages
pub use alma::ALMAIndicator;
pub use dema::DEMAIndicator;
pub use ema::EMAIndicator;
pub use hma::HMAIndicator;
pub use jma::JMAIndicator;
pub use lsma::LSMAIndicator;
pub use mcginley::McGinleyIndicator;
pub use rma::RMAIndicator;
pub use sma::SMAIndicator;
pub use smma::SMMAIndicator;
pub use tema::TEMAIndicator;
pub use vwma::VWMAIndicator;
pub use wma::WMAIndicator;

// Re-exports - Momentum/Oscillators
pub use awesome::AwesomeOscillatorIndicator;
pub use cci::CCIIndicator;
pub use chande_mo::ChandeMOIndicator;
pub use coppock::CoppockIndicator;
pub use dpo::DPOIndicator;
pub use macd::MACDIndicator;
pub use mfi::MFIIndicator;
pub use momentum::MomentumIndicator;
pub use price_osc::PriceOscillatorIndicator;
pub use roc::ROCIndicator;
pub use rsi::RSIIndicator;
pub use rvi::RVIIndicator;
pub use smi_ergodic::SMIErgodicIndicator;
pub use stoch::StochasticIndicator;
pub use stoch_rsi::StochRSIIndicator;
pub use trix::TRIXIndicator;
pub use tsi::TSIIndicator;
pub use ultimate::UltimateOscillatorIndicator;
pub use williams_r::WilliamsRIndicator;

// Re-exports - Trend
pub use adx::ADXIndicator;
pub use aroon::AroonIndicator;
pub use dmi::DMIIndicator;
pub use ichimoku::IchimokuIndicator;
pub use ma_cross::MACrossIndicator;
pub use ma_ribbon::MARibbonIndicator;
pub use parabolic_sar::ParabolicSARIndicator;
pub use supertrend::SupertrendIndicator;
pub use trend_strength::TrendStrengthIndicator;
pub use zigzag::ZigZagIndicator;

// Re-exports - Volatility
pub use adr::ADRIndicator;
pub use atr::ATRIndicator;
pub use bollinger::{
    BBBandwidthIndicator, BBPercentBIndicator, BBTrendIndicator, BollingerBandsIndicator,
};
pub use choppiness::ChoppinessIndicator;
pub use donchian::DonchianIndicator;
pub use envelope::EnvelopeIndicator;
pub use hist_vol::HistoricalVolatilityIndicator;
pub use keltner::KeltnerIndicator;
pub use mass_index::MassIndex;
pub use stdev::StdDevIndicator;

// Re-exports - Volume
pub use chaikin::{ChaikinMFIndicator, ChaikinOscillatorIndicator};
pub use cvd::{CVDIndicator, VolumeDeltaIndicator};
pub use elder_force::ElderForceIndicator;
pub use eom::EaseOfMovementIndicator;
pub use klinger::KlingerIndicator;
pub use obv::OBVIndicator;
pub use pvt::PVTIndicator;
pub use volume_osc::{NetVolumeIndicator, VolumeOscillatorIndicator};

// Re-exports - Other
pub use alligator::AlligatorIndicator;
pub use bop::BOPIndicator;
pub use bull_bear_power::BullBearPowerIndicator;
pub use chande_kroll::ChandeKrollIndicator;
pub use fisher::FisherIndicator;
pub use median::MedianIndicator;
pub use rci::RCIIndicator;
pub use vortex::VortexIndicator;
pub use woodies_cci::WoodiesCCIIndicator;

use crate::plugins::IndicatorPlugin;
use std::sync::Arc;

/// Get all built-in plugins (76+ indicators)
pub fn get_builtin_plugins() -> Vec<Arc<dyn IndicatorPlugin>> {
    vec![
        // Moving Averages (13)
        Arc::new(SMAIndicator::default()),
        Arc::new(EMAIndicator::default()),
        Arc::new(WMAIndicator::default()),
        Arc::new(DEMAIndicator::default()),
        Arc::new(TEMAIndicator::default()),
        Arc::new(HMAIndicator::default()),
        Arc::new(VWMAIndicator::default()),
        Arc::new(SMMAIndicator::default()),
        Arc::new(RMAIndicator::default()),
        Arc::new(LSMAIndicator::default()),
        Arc::new(ALMAIndicator::default()),
        Arc::new(McGinleyIndicator::default()),
        Arc::new(JMAIndicator::default()),
        // Momentum/Oscillators (19)
        Arc::new(RSIIndicator::default()),
        Arc::new(MACDIndicator::default()),
        Arc::new(StochasticIndicator::default()),
        Arc::new(StochRSIIndicator::default()),
        Arc::new(CCIIndicator::default()),
        Arc::new(MFIIndicator::default()),
        Arc::new(MomentumIndicator::default()),
        Arc::new(ROCIndicator::default()),
        Arc::new(AwesomeOscillatorIndicator::default()),
        Arc::new(TSIIndicator::default()),
        Arc::new(TRIXIndicator::default()),
        Arc::new(RVIIndicator::default()),
        Arc::new(WilliamsRIndicator::default()),
        Arc::new(ChandeMOIndicator::default()),
        Arc::new(CoppockIndicator::default()),
        Arc::new(DPOIndicator::default()),
        Arc::new(PriceOscillatorIndicator::default()),
        Arc::new(SMIErgodicIndicator::default()),
        Arc::new(UltimateOscillatorIndicator::default()),
        // Trend (10)
        Arc::new(ADXIndicator::default()),
        Arc::new(DMIIndicator::default()),
        Arc::new(AroonIndicator::default()),
        Arc::new(SupertrendIndicator::default()),
        Arc::new(IchimokuIndicator::default()),
        Arc::new(ParabolicSARIndicator::default()),
        Arc::new(MACrossIndicator::default()),
        Arc::new(MARibbonIndicator::default()),
        Arc::new(TrendStrengthIndicator::default()),
        Arc::new(ZigZagIndicator::default()),
        // Volatility (13)
        Arc::new(ATRIndicator::default()),
        Arc::new(BollingerBandsIndicator::default()),
        Arc::new(BBBandwidthIndicator::default()),
        Arc::new(BBPercentBIndicator::default()),
        Arc::new(BBTrendIndicator::default()),
        Arc::new(KeltnerIndicator::default()),
        Arc::new(DonchianIndicator::default()),
        Arc::new(EnvelopeIndicator::default()),
        Arc::new(ADRIndicator::default()),
        Arc::new(HistoricalVolatilityIndicator::default()),
        Arc::new(StdDevIndicator::default()),
        Arc::new(ChoppinessIndicator::default()),
        Arc::new(MassIndex),
        // Volume (12)
        Arc::new(OBVIndicator::default()),
        Arc::new(ChaikinMFIndicator::default()),
        Arc::new(ChaikinOscillatorIndicator::default()),
        Arc::new(CVDIndicator::default()),
        Arc::new(VolumeDeltaIndicator::default()),
        Arc::new(KlingerIndicator::default()),
        Arc::new(VolumeOscillatorIndicator::default()),
        Arc::new(NetVolumeIndicator::default()),
        Arc::new(PVTIndicator::default()),
        Arc::new(ElderForceIndicator::default()),
        Arc::new(EaseOfMovementIndicator::default()),
        // Other (9)
        Arc::new(BOPIndicator::default()),
        Arc::new(BullBearPowerIndicator::default()),
        Arc::new(MedianIndicator::default()),
        Arc::new(ChandeKrollIndicator::default()),
        Arc::new(VortexIndicator::default()),
        Arc::new(FisherIndicator::default()),
        Arc::new(AlligatorIndicator::default()),
        Arc::new(RCIIndicator::default()),
        Arc::new(WoodiesCCIIndicator::default()),
    ]
}
