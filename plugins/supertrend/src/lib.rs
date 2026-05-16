use loom_plugin_sdk::prelude::*;

/// SuperTrend indicator (Olivier Seban).
///
/// Plots a trend line that flips between upper and lower ATR bands.
/// - Green / below price → uptrend
/// - Red / above price → downtrend
///
/// Inputs:
///   - `period` (int, 10): ATR period
///   - `multiplier` (float, 3.0): ATR multiplier
///
/// Outputs:
///   - `supertrend`: trend line value
///   - `direction`: 1.0 (up) or -1.0 (down) — useful for colouring
#[derive(Default)]
pub struct SuperTrend {
    // State for incremental calculation (not yet wired — full recalc used).
}

impl IndicatorPlugin for SuperTrend {
    fn id(&self) -> &str {
        "supertrend"
    }

    fn name(&self) -> &str {
        "SuperTrend"
    }

    fn description(&self) -> &str {
        "Trend-following overlay based on ATR bands (Olivier Seban)"
    }

    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig {
                id: "period".into(),
                title: "ATR Period".into(),
                input_type: InputType::Int { min: Some(1), max: Some(200) },
                default: InputValue::Int(10),
                tooltip: Some("Period for the Average True Range calculation".into()),
            },
            InputConfig {
                id: "multiplier".into(),
                title: "Multiplier".into(),
                input_type: InputType::Float { min: Some(0.1), max: Some(10.0), step: Some(0.1) },
                default: InputValue::Float(3.0),
                tooltip: Some("ATR band width multiplier".into()),
            },
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig {
                id: "supertrend".into(),
                title: "SuperTrend".into(),
                color: Color::rgb(0, 200, 100),
                line_width: 2,
                style: PlotStyle::Line,
            },
            PlotConfig {
                id: "direction".into(),
                title: "Direction".into(),
                color: Color::rgb(150, 150, 150),
                line_width: 1,
                style: PlotStyle::Line,
            },
        ]
    }

    fn calculate(&self, ctx: &CalculationContext) -> IndicatorResult {
        let period = ctx.input_int("period").unwrap_or(10) as usize;
        let multiplier = ctx.input_float("multiplier").unwrap_or(3.0);
        let candles = ctx.candles;

        let n = candles.len();
        if n < period + 1 {
            return IndicatorResult::new("SuperTrend", true)
                .add_plot("supertrend", vec![None; n])
                .add_plot("direction", vec![None; n]);
        }

        // --- True Range ---
        let tr: Vec<f64> = (0..n)
            .map(|i| {
                let c = &candles[i];
                if i == 0 {
                    c.h - c.l
                } else {
                    let prev_close = candles[i - 1].c;
                    (c.h - c.l)
                        .max((c.h - prev_close).abs())
                        .max((c.l - prev_close).abs())
                }
            })
            .collect();

        // --- Wilder's Smoothed ATR ---
        let mut atr = vec![0.0f64; n];
        // Seed with simple average of first `period` TRs.
        let seed: f64 = tr[..period].iter().sum::<f64>() / period as f64;
        atr[period - 1] = seed;
        for i in period..n {
            atr[i] = (atr[i - 1] * (period as f64 - 1.0) + tr[i]) / period as f64;
        }

        // --- SuperTrend bands ---
        let mut upper = vec![0.0f64; n];
        let mut lower = vec![0.0f64; n];
        let mut supertrend = vec![None::<f64>; n];
        let mut direction = vec![None::<f64>; n];

        // Direction: 1 = up, -1 = down
        let mut dir: i8 = 1;

        for i in (period - 1)..n {
            let hl2 = (candles[i].h + candles[i].l) / 2.0;
            let basic_upper = hl2 + multiplier * atr[i];
            let basic_lower = hl2 - multiplier * atr[i];

            // Prevent bands from moving against the trend.
            upper[i] = if i > 0 && basic_upper < upper[i - 1] {
                basic_upper
            } else if i > 0 && candles[i - 1].c <= upper[i - 1] {
                basic_upper.min(upper[i - 1])
            } else {
                basic_upper
            };

            lower[i] = if i > 0 && basic_lower > lower[i - 1] {
                basic_lower
            } else if i > 0 && candles[i - 1].c >= lower[i - 1] {
                basic_lower.max(lower[i - 1])
            } else {
                basic_lower
            };

            // Determine direction.
            if i > 0 {
                let prev_st = supertrend[i - 1].unwrap_or(upper[i]);
                if candles[i].c > prev_st {
                    dir = 1;
                } else if candles[i].c < prev_st {
                    dir = -1;
                }
            }

            supertrend[i] = Some(if dir == 1 { lower[i] } else { upper[i] });
            direction[i] = Some(dir as f64);
        }

        IndicatorResult::new("SuperTrend", true)
            .add_plot("supertrend", supertrend)
            .add_plot("direction", direction)
    }
}

export_indicator!(
    plugin: SuperTrend,
    api_version: "1",
    id: "supertrend",
    display_name: "SuperTrend",
    description: "Trend-following overlay based on ATR bands (Olivier Seban)",
    version: "0.1.0",
);
