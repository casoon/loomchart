use crate::core::{ChartState, Viewport};
use crate::Candle;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewportExport {
    pub time_start: i64,
    pub time_end: i64,
    pub price_min: f64,
    pub price_max: f64,
}

impl Viewport {
    pub fn export(&self) -> ViewportExport {
        ViewportExport {
            time_start: self.time.start,
            time_end: self.time.end,
            price_min: self.price.min,
            price_max: self.price.max,
        }
    }

    pub fn import(&mut self, export: ViewportExport) {
        self.time.start = export.time_start;
        self.time.end = export.time_end;
        self.price.min = export.price_min;
        self.price.max = export.price_max;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChartStateExport {
    pub version: String,
    pub timestamp: i64,
    pub timeframe: String,
    pub candles: Vec<Candle>,
    pub viewport: ViewportExport,
}

impl ChartState {
    pub fn export(&self) -> Result<String, String> {
        let export = ChartStateExport {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            timeframe: format!("{:?}", self.timeframe),
            candles: self.candles.clone(),
            viewport: self.viewport.export(),
        };

        serde_json::to_string(&export).map_err(|e| format!("Serialization error: {}", e))
    }

    pub fn import(&mut self, json: &str) -> Result<(), String> {
        let export: ChartStateExport =
            serde_json::from_str(json).map_err(|e| format!("Deserialization error: {}", e))?;

        // Version check (warn only, don't fail)
        if export.version != env!("CARGO_PKG_VERSION") {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::warn_1(
                &format!(
                    "State version mismatch: {} != {}",
                    export.version,
                    env!("CARGO_PKG_VERSION")
                )
                .into(),
            );
        }

        // Restore candles
        self.set_candles(export.candles);

        // Restore viewport
        self.viewport.import(export.viewport);

        Ok(())
    }
}
