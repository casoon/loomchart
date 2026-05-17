// Study / Script runtime — bar-by-bar execution model for the plugin API

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::core::Candle;
use crate::plugins::InputValue;

// ---------------------------------------------------------------------------
// Output primitives
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeType {
    ArrowUp,
    ArrowDown,
    Cross,
    Circle,
    Square,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StudyOutput {
    Plot {
        id: String,
        value: Option<f64>,
        color: Option<String>,
    },
    Histogram {
        id: String,
        value: Option<f64>,
        color: Option<String>,
    },
    Label {
        time: i64,
        price: f64,
        text: String,
        color: String,
    },
    Shape {
        time: i64,
        price: f64,
        shape: ShapeType,
        color: String,
    },
    BarColor {
        time: i64,
        color: String,
    },
    Alert {
        message: String,
        time: i64,
    },
}

// ---------------------------------------------------------------------------
// Capability flags
// ---------------------------------------------------------------------------

pub struct StudyCapabilities {
    pub multi_timeframe: bool,
    pub alerts: bool,
    pub drawings: bool,
    pub bar_coloring: bool,
}

impl Default for StudyCapabilities {
    fn default() -> Self {
        Self {
            multi_timeframe: false,
            alerts: false,
            drawings: false,
            bar_coloring: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Bar-by-bar execution context
// ---------------------------------------------------------------------------

pub struct StudyContext<'a> {
    pub candles: &'a [Candle],
    pub current_index: usize,
    pub inputs: HashMap<String, InputValue>,
}

impl<'a> StudyContext<'a> {
    pub fn close(&self) -> f64 {
        self.candles[self.current_index].c
    }

    pub fn open(&self) -> f64 {
        self.candles[self.current_index].o
    }

    pub fn high(&self) -> f64 {
        self.candles[self.current_index].h
    }

    pub fn low(&self) -> f64 {
        self.candles[self.current_index].l
    }

    pub fn volume(&self) -> f64 {
        self.candles[self.current_index].v
    }

    /// Returns the close price `offset` bars ago from `current_index`.
    /// `prev("close", 1)` is the previous bar's close.
    /// `prev("close", 0)` is the current bar's close.
    pub fn prev(&self, field: &str, offset: usize) -> Option<f64> {
        if offset > self.current_index {
            return None;
        }
        let idx = self.current_index - offset;
        let candle = &self.candles[idx];
        match field {
            "close" | "c" => Some(candle.c),
            "open" | "o" => Some(candle.o),
            "high" | "h" => Some(candle.h),
            "low" | "l" => Some(candle.l),
            "volume" | "v" => Some(candle.v),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Study trait
// ---------------------------------------------------------------------------

pub trait Study: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;

    /// Called once per bar in chronological order.
    fn on_bar(&self, ctx: &StudyContext, state: &mut serde_json::Value) -> Vec<StudyOutput>;

    /// Called to reset per-bar state (e.g. when the candle series is replaced).
    fn reset(&self, state: &mut serde_json::Value) {
        *state = serde_json::Value::Null;
    }
}

// ---------------------------------------------------------------------------
// Study runner
// ---------------------------------------------------------------------------

pub struct StudyRunner {
    study: Box<dyn Study>,
    state: serde_json::Value,
    /// One `Vec<StudyOutput>` per bar, indexed by bar index.
    outputs: Vec<Vec<StudyOutput>>,
}

impl StudyRunner {
    pub fn new(study: Box<dyn Study>) -> Self {
        Self {
            study,
            state: serde_json::Value::Null,
            outputs: Vec::new(),
        }
    }

    /// Run the study over all candles, producing one output vector per bar.
    pub fn run(
        &mut self,
        candles: &[Candle],
        inputs: HashMap<String, InputValue>,
    ) {
        self.outputs.clear();
        for index in 0..candles.len() {
            let ctx = StudyContext {
                candles,
                current_index: index,
                inputs: inputs.clone(),
            };
            let bar_outputs = self.study.on_bar(&ctx, &mut self.state);
            self.outputs.push(bar_outputs);
        }
    }

    pub fn outputs_for_bar(&self, index: usize) -> Option<&[StudyOutput]> {
        self.outputs.get(index).map(|v| v.as_slice())
    }

    pub fn reset(&mut self) {
        self.study.reset(&mut self.state);
        self.outputs.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_candles() -> Vec<Candle> {
        vec![
            Candle::new(1_000, 10.0, 12.0, 9.0, 11.0, 100.0),
            Candle::new(2_000, 11.0, 13.0, 10.0, 12.0, 200.0),
            Candle::new(3_000, 12.0, 14.0, 11.0, 13.0, 300.0),
        ]
    }

    // -- StudyContext helpers -------------------------------------------------

    #[test]
    fn study_context_close_returns_current_bar_close() {
        let candles = make_candles();
        let ctx = StudyContext {
            candles: &candles,
            current_index: 1,
            inputs: HashMap::new(),
        };
        assert_eq!(ctx.close(), 12.0);
    }

    #[test]
    fn study_context_prev_returns_value_n_bars_ago() {
        let candles = make_candles();
        let ctx = StudyContext {
            candles: &candles,
            current_index: 2,
            inputs: HashMap::new(),
        };
        // offset 0 → current bar close = 13.0
        assert_eq!(ctx.prev("close", 0), Some(13.0));
        // offset 1 → previous bar close = 12.0
        assert_eq!(ctx.prev("close", 1), Some(12.0));
        // offset 2 → two bars ago close = 11.0
        assert_eq!(ctx.prev("close", 2), Some(11.0));
        // offset 3 → before first bar → None
        assert_eq!(ctx.prev("close", 3), None);
    }

    #[test]
    fn study_context_prev_unknown_field_returns_none() {
        let candles = make_candles();
        let ctx = StudyContext {
            candles: &candles,
            current_index: 1,
            inputs: HashMap::new(),
        };
        assert_eq!(ctx.prev("unknown_field", 0), None);
    }

    // -- A concrete study for integration tests ------------------------------

    struct TestStudy;

    impl Study for TestStudy {
        fn id(&self) -> &str {
            "test_study"
        }

        fn name(&self) -> &str {
            "Test Study"
        }

        fn on_bar(&self, ctx: &StudyContext, _state: &mut serde_json::Value) -> Vec<StudyOutput> {
            vec![StudyOutput::Plot {
                id: "close_plot".to_string(),
                value: Some(ctx.close()),
                color: None,
            }]
        }
    }

    // -- StudyRunner ---------------------------------------------------------

    #[test]
    fn study_runner_calls_on_bar_for_each_candle() {
        let candles = make_candles();
        let mut runner = StudyRunner::new(Box::new(TestStudy));
        runner.run(&candles, HashMap::new());
        // One output vector per bar
        assert_eq!(runner.outputs.len(), candles.len());
    }

    #[test]
    fn study_runner_outputs_for_bar_returns_correct_output() {
        let candles = make_candles();
        let mut runner = StudyRunner::new(Box::new(TestStudy));
        runner.run(&candles, HashMap::new());

        // Bar 0 → close = 11.0
        let bar0 = runner.outputs_for_bar(0).unwrap();
        assert_eq!(bar0.len(), 1);
        if let StudyOutput::Plot { value, .. } = &bar0[0] {
            assert_eq!(*value, Some(11.0));
        } else {
            panic!("expected Plot output");
        }

        // Bar 2 → close = 13.0
        let bar2 = runner.outputs_for_bar(2).unwrap();
        if let StudyOutput::Plot { value, .. } = &bar2[0] {
            assert_eq!(*value, Some(13.0));
        } else {
            panic!("expected Plot output");
        }
    }

    #[test]
    fn study_runner_outputs_for_bar_out_of_range_returns_none() {
        let candles = make_candles();
        let mut runner = StudyRunner::new(Box::new(TestStudy));
        runner.run(&candles, HashMap::new());
        assert!(runner.outputs_for_bar(99).is_none());
    }

    #[test]
    fn study_runner_reset_clears_outputs() {
        let candles = make_candles();
        let mut runner = StudyRunner::new(Box::new(TestStudy));
        runner.run(&candles, HashMap::new());
        runner.reset();
        assert!(runner.outputs.is_empty());
        assert!(runner.outputs_for_bar(0).is_none());
    }
}
