//! Indicator Registry - Auto-generated from plugin implementations

use crate::indicators::builtin::*;
use crate::indicators::metadata::{
    IndicatorCategory, IndicatorMetadata, ParameterDefinition, ParameterType,
};
use crate::plugins::IndicatorPlugin;

/// Collect all indicator metadata from plugin implementations
pub fn collect_all_indicators() -> Vec<IndicatorMetadata> {
    let mut indicators = vec![];

    // Helper to convert plugin to metadata
    fn plugin_to_metadata<T: IndicatorPlugin + Default>(plugin: T) -> IndicatorMetadata {
        let inputs = plugin.inputs();

        let parameters: Vec<ParameterDefinition> = inputs
            .iter()
            .map(|input| {
                use crate::plugins::InputValue;

                let (param_type, default_val) = match &input.input_type {
                    crate::plugins::InputType::Int { min, max } => {
                        let def = if let InputValue::Int(v) = &input.default {
                            *v
                        } else {
                            14
                        };
                        (
                            ParameterType::Integer {
                                min: min.unwrap_or(0),
                                max: max.unwrap_or(1000),
                                default: def,
                                step: 1,
                            },
                            def,
                        )
                    }
                    crate::plugins::InputType::Float { min, max, step } => {
                        let def = if let InputValue::Float(v) = &input.default {
                            *v
                        } else {
                            14.0
                        };
                        (
                            ParameterType::Float {
                                min: min.unwrap_or(0.0),
                                max: max.unwrap_or(1000.0),
                                default: def,
                                step: step.unwrap_or(0.1),
                            },
                            def as i32,
                        )
                    }
                    crate::plugins::InputType::Bool => {
                        let def = if let InputValue::Bool(v) = &input.default {
                            *v
                        } else {
                            false
                        };
                        (
                            ParameterType::Boolean { default: def },
                            if def { 1 } else { 0 },
                        )
                    }
                    crate::plugins::InputType::Source => (
                        ParameterType::Choice {
                            options: vec![
                                "close".to_string(),
                                "open".to_string(),
                                "high".to_string(),
                                "low".to_string(),
                            ],
                            default: 0,
                        },
                        0,
                    ),
                    crate::plugins::InputType::Color => (
                        ParameterType::Color {
                            default: "#000000".to_string(),
                        },
                        0,
                    ),
                    crate::plugins::InputType::String { .. } => {
                        let def = if let InputValue::String(v) = &input.default {
                            v.clone()
                        } else {
                            String::new()
                        };
                        (ParameterType::String { default: def }, 0)
                    }
                };

                ParameterDefinition {
                    id: input.id.clone(),
                    name: input.title.clone(),
                    description: input.tooltip.clone().unwrap_or_else(|| input.title.clone()),
                    param_type,
                    tooltip: input.tooltip.clone(),
                }
            })
            .collect();

        let category = match plugin.category() {
            crate::plugins::IndicatorCategory::Momentum => IndicatorCategory::Momentum,
            crate::plugins::IndicatorCategory::Trend => IndicatorCategory::Trend,
            crate::plugins::IndicatorCategory::Volatility => IndicatorCategory::Volatility,
            crate::plugins::IndicatorCategory::Volume => IndicatorCategory::Volume,
            _ => IndicatorCategory::Custom,
        };

        IndicatorMetadata {
            id: plugin.id().to_string(),
            name: plugin.name().to_string(),
            short_description: plugin.description().to_string(),
            long_description: plugin.description().to_string(),
            category,
            parameters,
            interpretation: None,
            use_cases: vec![],
            output_range: (f64::NEG_INFINITY, f64::INFINITY),
            normalized: false,
            recommended_timeframes: vec![
                "5m".to_string(),
                "15m".to_string(),
                "1h".to_string(),
                "4h".to_string(),
                "1d".to_string(),
            ],
            complexity: "O(n)".to_string(),
            related: vec![],
        }
    }

    // Auto-generated indicator list
    indicators.push(plugin_to_metadata(ADRIndicator::default()));
    indicators.push(plugin_to_metadata(ADRPercentIndicator::default()));
    indicators.push(plugin_to_metadata(ADXIndicator::default()));
    indicators.push(plugin_to_metadata(ALMAIndicator::default()));
    indicators.push(plugin_to_metadata(ATRIndicator::default()));
    indicators.push(plugin_to_metadata(AlligatorIndicator::default()));
    indicators.push(plugin_to_metadata(AroonIndicator::default()));
    indicators.push(plugin_to_metadata(AwesomeOscillatorIndicator::default()));
    indicators.push(plugin_to_metadata(BBBandwidthIndicator::default()));
    indicators.push(plugin_to_metadata(BBPercentBIndicator::default()));
    indicators.push(plugin_to_metadata(BBTrendIndicator::default()));
    indicators.push(plugin_to_metadata(BOPIndicator::default()));
    indicators.push(plugin_to_metadata(BollingerBandsIndicator::default()));
    indicators.push(plugin_to_metadata(BullBearPowerIndicator::default()));
    indicators.push(plugin_to_metadata(CCIIndicator::default()));
    indicators.push(plugin_to_metadata(CVDIndicator::default()));
    indicators.push(plugin_to_metadata(ChaikinMFIndicator::default()));
    indicators.push(plugin_to_metadata(ChaikinOscillatorIndicator::default()));
    indicators.push(plugin_to_metadata(ChandeKrollIndicator::default()));
    indicators.push(plugin_to_metadata(ChandeMOIndicator::default()));
    indicators.push(plugin_to_metadata(ChoppinessIndicator::default()));
    indicators.push(plugin_to_metadata(CoppockIndicator::default()));
    indicators.push(plugin_to_metadata(DEMAIndicator::default()));
    indicators.push(plugin_to_metadata(DMIIndicator::default()));
    indicators.push(plugin_to_metadata(DPOIndicator::default()));
    indicators.push(plugin_to_metadata(DonchianIndicator::default()));
    indicators.push(plugin_to_metadata(EMAIndicator::default()));
    indicators.push(plugin_to_metadata(EaseOfMovementIndicator::default()));
    indicators.push(plugin_to_metadata(ElderForceIndicator::default()));
    indicators.push(plugin_to_metadata(EnvelopeIndicator::default()));
    indicators.push(plugin_to_metadata(FisherIndicator::default()));
    indicators.push(plugin_to_metadata(HMAIndicator::default()));
    indicators.push(plugin_to_metadata(HistoricalVolatilityIndicator::default()));
    indicators.push(plugin_to_metadata(IchimokuIndicator::default()));
    indicators.push(plugin_to_metadata(JMAIndicator::default()));
    indicators.push(plugin_to_metadata(KeltnerIndicator::default()));
    indicators.push(plugin_to_metadata(KlingerIndicator::default()));
    indicators.push(plugin_to_metadata(LSMAIndicator::default()));
    indicators.push(plugin_to_metadata(MACDIndicator::default()));
    indicators.push(plugin_to_metadata(MACrossIndicator::default()));
    indicators.push(plugin_to_metadata(MARibbonIndicator::default()));
    indicators.push(plugin_to_metadata(MFIIndicator::default()));
    indicators.push(plugin_to_metadata(McGinleyIndicator::default()));
    indicators.push(plugin_to_metadata(MedianIndicator::default()));
    indicators.push(plugin_to_metadata(MomentumIndicator::default()));
    indicators.push(plugin_to_metadata(NetVolumeIndicator::default()));
    indicators.push(plugin_to_metadata(OBVIndicator::default()));
    indicators.push(plugin_to_metadata(PVTIndicator::default()));
    indicators.push(plugin_to_metadata(ParabolicSARIndicator::default()));
    indicators.push(plugin_to_metadata(PriceOscillatorIndicator::default()));
    indicators.push(plugin_to_metadata(RCIIndicator::default()));
    indicators.push(plugin_to_metadata(RMAIndicator::default()));
    indicators.push(plugin_to_metadata(ROCIndicator::default()));
    indicators.push(plugin_to_metadata(RSIIndicator::default()));
    indicators.push(plugin_to_metadata(RVIIndicator::default()));
    indicators.push(plugin_to_metadata(SMAIndicator::default()));
    indicators.push(plugin_to_metadata(SMIErgodicIndicator::default()));
    indicators.push(plugin_to_metadata(SMIErgodicOscillatorIndicator::default()));
    indicators.push(plugin_to_metadata(SMMAIndicator::default()));
    indicators.push(plugin_to_metadata(StdDevIndicator::default()));
    indicators.push(plugin_to_metadata(StochRSIIndicator::default()));
    indicators.push(plugin_to_metadata(StochasticIndicator::default()));
    indicators.push(plugin_to_metadata(SupertrendIndicator::default()));
    indicators.push(plugin_to_metadata(TEMAIndicator::default()));
    indicators.push(plugin_to_metadata(TRIXIndicator::default()));
    indicators.push(plugin_to_metadata(TSIIndicator::default()));
    indicators.push(plugin_to_metadata(TrendStrengthIndicator::default()));
    indicators.push(plugin_to_metadata(UltimateOscillatorIndicator::default()));
    indicators.push(plugin_to_metadata(VWMAIndicator::default()));
    indicators.push(plugin_to_metadata(VolumeDeltaIndicator::default()));
    indicators.push(plugin_to_metadata(VolumeOscillatorIndicator::default()));
    indicators.push(plugin_to_metadata(VortexIndicator::default()));
    indicators.push(plugin_to_metadata(WMAIndicator::default()));
    indicators.push(plugin_to_metadata(WilliamsRIndicator::default()));
    indicators.push(plugin_to_metadata(WoodiesCCIIndicator::default()));
    indicators.push(plugin_to_metadata(ZigZagIndicator::default()));

    indicators
}
