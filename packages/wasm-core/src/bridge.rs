//! Bridge to JavaScript - calls JS functions from WASM

use wasm_bindgen::prelude::*;
use crate::types::{Candle, ConnectionStatus, ChartCandle};
use std::collections::VecDeque;

#[wasm_bindgen]
extern "C" {
    // Connection callbacks
    #[wasm_bindgen(js_namespace = window, js_name = wasmOnConnectionChange)]
    fn js_on_connection_change(status: &str);

    // Candle callbacks
    #[wasm_bindgen(js_namespace = window, js_name = wasmOnCandleUpdate)]
    fn js_on_candle_update(candle_json: &str);

    // Indicator callbacks
    #[wasm_bindgen(js_namespace = window, js_name = wasmOnIndicatorUpdate)]
    fn js_on_indicator_update(name: &str, data_json: &str);

    // Error callback
    #[wasm_bindgen(js_namespace = window, js_name = wasmOnError)]
    fn js_on_error(error: &str);

    // Chart API (direct calls to chart wrapper)
    #[wasm_bindgen(js_namespace = ["window", "loomChart"])]
    fn setCandles(series_id: &str, candles_json: &str);

    #[wasm_bindgen(js_namespace = ["window", "loomChart"])]
    fn updateCandle(series_id: &str, candle_json: &str);

    #[wasm_bindgen(js_namespace = ["window", "loomChart"])]
    fn setLineSeries(series_id: &str, points_json: &str);

    #[wasm_bindgen(js_namespace = ["window", "loomChart"])]
    fn updateLinePoint(series_id: &str, point_json: &str);

    #[wasm_bindgen(js_namespace = ["window", "loomChart"])]
    fn removeIndicator(series_id: &str);
}

/// Notify JS about connection status change
pub fn on_connection_change(status: &ConnectionStatus) {
    js_on_connection_change(&status.to_string());
}

/// Notify JS about candle update
pub fn on_candle_update(candle: &Candle) {
    if let Ok(json) = serde_json::to_string(candle) {
        js_on_candle_update(&json);
    }
}

/// Notify JS about indicator update
#[allow(dead_code)]
pub fn on_indicator_update(name: &str, data: &serde_json::Value) {
    if let Ok(json) = serde_json::to_string(data) {
        js_on_indicator_update(name, &json);
    }
}

/// Notify JS about error
pub fn on_error(error: &str) {
    js_on_error(error);
}

/// Set candle data on chart
pub fn set_candles(candles: &VecDeque<Candle>) {
    let chart_candles: Vec<ChartCandle> = candles.iter().map(ChartCandle::from).collect();
    if let Ok(json) = serde_json::to_string(&chart_candles) {
        setCandles("main", &json);
    }
}

/// Update single candle on chart
pub fn update_candle(candle: &Candle) {
    let chart_candle = ChartCandle::from(candle);
    if let Ok(json) = serde_json::to_string(&chart_candle) {
        updateCandle("main", &json);
    }
}

/// Set line series data (for indicators)
pub fn set_line_series(series_id: &str, points: &[(i64, f64)]) {
    let data: Vec<_> = points.iter().map(|(t, v)| {
        serde_json::json!({ "time": t, "value": v })
    }).collect();

    if let Ok(json) = serde_json::to_string(&data) {
        setLineSeries(series_id, &json);
    }
}

/// Update single point on line series
pub fn update_line_point(series_id: &str, time: i64, value: f64) {
    let point = serde_json::json!({ "time": time, "value": value });
    if let Ok(json) = serde_json::to_string(&point) {
        updateLinePoint(series_id, &json);
    }
}

/// Remove indicator series from chart
pub fn remove_indicator(series_id: &str) {
    removeIndicator(series_id);
}
