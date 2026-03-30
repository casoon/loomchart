// Williams Alligator Indicator Plugin

use crate::plugins::{
    CalculationContext, IndicatorCategory, IndicatorPlugin, IndicatorResult, InputConfig,
    PlotConfig,
};
use crate::ta;

#[derive(Default)]
pub struct AlligatorIndicator;

impl IndicatorPlugin for AlligatorIndicator {
    fn id(&self) -> &str {
        "alligator"
    }

    fn name(&self) -> &str {
        "Williams Alligator"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Other
    }

    fn description(&self) -> &str {
        "Bill Williams' Alligator: Jaw, Teeth, and Lips smoothed moving averages showing trend states"
    }

    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("jaw_length", "Jaw Length", 13)
                .min(1)
                .max(100),
            InputConfig::int("jaw_offset", "Jaw Offset", 8)
                .min(0)
                .max(50),
            InputConfig::int("teeth_length", "Teeth Length", 8)
                .min(1)
                .max(100),
            InputConfig::int("teeth_offset", "Teeth Offset", 5)
                .min(0)
                .max(50),
            InputConfig::int("lips_length", "Lips Length", 5)
                .min(1)
                .max(100),
            InputConfig::int("lips_offset", "Lips Offset", 3)
                .min(0)
                .max(50),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("jaw", "Jaw", "#2196F3").line_width(2),
            PlotConfig::new("teeth", "Teeth", "#F44336").line_width(2),
            PlotConfig::new("lips", "Lips", "#4CAF50").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let jaw_length = context.input_int("jaw_length").unwrap_or(13) as usize;
        let jaw_offset = context.input_int("jaw_offset").unwrap_or(8) as usize;
        let teeth_length = context.input_int("teeth_length").unwrap_or(8) as usize;
        let teeth_offset = context.input_int("teeth_offset").unwrap_or(5) as usize;
        let lips_length = context.input_int("lips_length").unwrap_or(5) as usize;
        let lips_offset = context.input_int("lips_offset").unwrap_or(3) as usize;

        let highs = context.high();
        let lows = context.low();

        let (jaw, teeth, lips) = ta::williams_alligator(
            &highs,
            &lows,
            jaw_length,
            jaw_offset,
            teeth_length,
            teeth_offset,
            lips_length,
            lips_offset,
        );

        IndicatorResult::new("Williams Alligator", "Alligator", true)
            .add_plot("jaw", jaw)
            .add_plot("teeth", teeth)
            .add_plot("lips", lips)
    }
}
