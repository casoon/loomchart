//! WASM Core for Loom Trading UI
//!
//! This module provides the core functionality for:
//! - WebSocket connection to Phoenix channels
//! - State management for candles and indicators
//! - Bridge to JavaScript chart library
//!
//! # Exports
//! - `init(config_json)` - Initialize with configuration
//! - `connect(source, symbol, tf, ws_url)` - Connect to data stream
//! - `disconnect()` - Disconnect from stream
//! - `set_symbol(symbol)` - Change symbol (triggers resync)
//! - `set_timeframe(tf)` - Change timeframe (triggers resync)
//! - `toggle_indicator(name, params_json, enabled)` - Toggle indicator

mod bridge;
mod indicators;
mod state;
mod types;
mod websocket;

// Re-export chartcore plugin system and WasmChart
pub use chartcore::wasm::{
    WasmChart, WasmLempelZivComplexity, WasmPermutationEntropy, WasmShannonEntropy,
};
pub use chartcore::{plugins, ta, Candle as ChartCoreCandle};

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use state::AppState;
use types::Config;

// Global app state
thread_local! {
    static APP_STATE: RefCell<Option<Rc<RefCell<AppState>>>> = RefCell::new(None);
}

/// Initialize the WASM module with configuration
#[wasm_bindgen]
pub fn init(config_json: &str) -> Result<(), JsValue> {
    // Set up panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    // Initialize logging
    console_log::init_with_level(log::Level::Debug)
        .map_err(|e| JsValue::from_str(&format!("Failed to init logging: {}", e)))?;

    log::info!("Initializing Loom WASM core...");

    // Parse config
    let config: Config = serde_json::from_str(config_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid config: {}", e)))?;

    // Create app state
    let state = Rc::new(RefCell::new(AppState::new(config)));

    APP_STATE.with(|s| {
        *s.borrow_mut() = Some(state);
    });

    log::info!("WASM core initialized successfully");
    Ok(())
}

/// Connect to data stream with properly wired callbacks
#[wasm_bindgen]
pub fn connect(source: &str, symbol: &str, tf: &str, ws_url: &str) -> Result<(), JsValue> {
    log::info!("Connecting to {}:{}:{}", source, symbol, tf);

    APP_STATE.with(|s| {
        let state = s.borrow();
        match state.as_ref() {
            Some(app) => {
                AppState::connect_with_callbacks(Rc::clone(app), source, symbol, tf, ws_url)
            }
            None => Err(JsValue::from_str(
                "WASM not initialized. Call init() first.",
            )),
        }
    })
}

/// Disconnect from data stream (manual disconnect resets reconnection)
#[wasm_bindgen]
pub fn disconnect() -> Result<(), JsValue> {
    log::info!("Disconnecting...");

    with_state(|state| {
        let mut s = state.borrow_mut();
        s.reset_reconnect(); // Don't auto-reconnect on manual disconnect
        s.disconnect()
    })
}

/// Change symbol (triggers resync)
#[wasm_bindgen]
pub fn set_symbol(symbol: &str) -> Result<(), JsValue> {
    log::info!("Setting symbol: {}", symbol);

    with_state(|state| state.borrow_mut().set_symbol(symbol))
}

/// Change timeframe (triggers resync)
#[wasm_bindgen]
pub fn set_timeframe(tf: &str) -> Result<(), JsValue> {
    log::info!("Setting timeframe: {}", tf);

    with_state(|state| state.borrow_mut().set_timeframe(tf))
}

/// Toggle indicator on/off
#[wasm_bindgen]
pub fn toggle_indicator(name: &str, params_json: &str, enabled: bool) -> Result<(), JsValue> {
    log::info!("Toggle indicator {}: {}", name, enabled);

    with_state(|state| {
        state
            .borrow_mut()
            .toggle_indicator(name, params_json, enabled)
    })
}

/// Get current connection status
#[wasm_bindgen]
pub fn get_connection_status() -> Result<String, JsValue> {
    with_state(|state| Ok(state.borrow().connection_status().to_string()))
}

/// Get last candle as JSON
#[wasm_bindgen]
pub fn get_last_candle() -> Result<Option<String>, JsValue> {
    with_state(|state| {
        let candle = state.borrow().last_candle();
        match candle {
            Some(c) => Ok(Some(serde_json::to_string(&c).unwrap())),
            None => Ok(None),
        }
    })
}

/// Get reconnection delay (returns 0 if not in reconnecting state)
/// JS should call this, wait the returned ms, then call do_reconnect()
#[wasm_bindgen]
pub fn get_reconnect_delay() -> Result<u32, JsValue> {
    with_state(|state| {
        let mut s = state.borrow_mut();
        if s.get_reconnect_info().is_some() {
            Ok(s.next_reconnect_delay_ms())
        } else {
            Ok(0)
        }
    })
}

/// Perform reconnection attempt
#[wasm_bindgen]
pub fn do_reconnect() -> Result<(), JsValue> {
    APP_STATE.with(|s| {
        let state_ref = s.borrow();
        if let Some(app) = state_ref.as_ref() {
            let info = app.borrow().get_reconnect_info();
            if let Some((source, symbol, tf, ws_url)) = info {
                log::info!("Attempting reconnection to {}:{}:{}", source, symbol, tf);
                AppState::connect_with_callbacks(Rc::clone(app), &source, &symbol, &tf, &ws_url)
            } else {
                Ok(()) // Not in reconnecting state
            }
        } else {
            Err(JsValue::from_str("WASM not initialized"))
        }
    })
}

/// Check if currently in reconnecting state
#[wasm_bindgen]
pub fn is_reconnecting() -> Result<bool, JsValue> {
    with_state(|state| Ok(state.borrow().get_reconnect_info().is_some()))
}

/// Send heartbeat - should be called periodically by JS (every 30s)
/// Returns true if heartbeat was sent, false if connection is not active
#[wasm_bindgen]
pub fn send_heartbeat() -> Result<bool, JsValue> {
    with_state(|state| {
        let mut s = state.borrow_mut();
        if s.has_active_connection() {
            s.send_heartbeat();
            Ok(true)
        } else {
            Ok(false)
        }
    })
}

/// Check and clear last heartbeat reply from window object
/// Returns the ref ID if a heartbeat reply was received
#[wasm_bindgen]
pub fn check_heartbeat_reply() -> Result<Option<String>, JsValue> {
    if let Some(window) = web_sys::window() {
        let key = JsValue::from_str("__loom_last_heartbeat_ref");
        if let Ok(val) = js_sys::Reflect::get(&window, &key) {
            if !val.is_undefined() && !val.is_null() {
                // Clear it
                let _ = js_sys::Reflect::set(&window, &key, &JsValue::NULL);
                if let Some(ref_str) = val.as_string() {
                    // Update WsClient
                    with_state(|state| {
                        state.borrow_mut().on_heartbeat_reply(&ref_str);
                        Ok(())
                    })?;
                    return Ok(Some(ref_str));
                }
            }
        }
    }
    Ok(None)
}

/// Load test data using chartcore generator
///
/// # Parameters
/// - market_type: "crypto", "stock", "forex", "futures", "commodities"
/// - trend: "bullish_strong", "bullish_mild", "sideways", "bearish_mild", "bearish_strong"
/// - volatility: "low", "normal", "high", "extreme"
/// - count: number of candles to generate
#[wasm_bindgen]
pub fn load_test_data(
    market_type: &str,
    trend: &str,
    volatility: &str,
    count: usize,
) -> Result<(), JsValue> {
    use chartcore::{CandleGenerator, GeneratorConfig, MarketType, Trend, VolatilityRegime};

    log::info!(
        "Generating {} {} candles with {} trend and {} volatility",
        count,
        market_type,
        trend,
        volatility
    );

    // Parse market type
    let market = match market_type {
        "crypto" => MarketType::Crypto,
        "stock" => MarketType::Stock,
        "forex" => MarketType::Forex,
        "futures" => MarketType::Futures,
        "commodities" => MarketType::Commodities,
        _ => return Err(JsValue::from_str("Invalid market type")),
    };

    // Parse trend
    let trend_type = match trend {
        "bullish_strong" => Trend::BullishStrong,
        "bullish_mild" => Trend::BullishMild,
        "sideways" => Trend::Sideways,
        "bearish_mild" => Trend::BearishMild,
        "bearish_strong" => Trend::BearishStrong,
        _ => return Err(JsValue::from_str("Invalid trend")),
    };

    // Parse volatility
    let vol_regime = match volatility {
        "low" => VolatilityRegime::Low,
        "normal" => VolatilityRegime::Normal,
        "high" => VolatilityRegime::High,
        "extreme" => VolatilityRegime::Extreme,
        _ => return Err(JsValue::from_str("Invalid volatility")),
    };

    // Create generator config
    let config = GeneratorConfig::new(market)
        .with_trend(trend_type)
        .with_regime(vol_regime)
        .with_seed(42);

    // Generate candles
    let mut generator = CandleGenerator::new(config);
    let chartcore_candles = generator.generate(count);

    // Convert chartcore::Candle to wasm-core Candle format
    with_state(|state| {
        let mut s = state.borrow_mut();

        // Clear existing candles
        s.clear_candles();

        // Convert and load candles
        for cc_candle in chartcore_candles {
            let candle = types::Candle {
                source: "generator".to_string(),
                symbol: format!("TEST_{}", market_type.to_uppercase()),
                tf: "5m".to_string(),
                ts: format_timestamp(cc_candle.time),
                o: cc_candle.o,
                h: cc_candle.h,
                l: cc_candle.l,
                c: cc_candle.c,
                v: cc_candle.v,
                is_final: true,
            };
            s.push_candle(candle);
        }

        // Send to chart
        s.send_candles_to_chart();

        log::info!("Loaded {} generated candles", count);
        Ok(())
    })
}

// Helper to format timestamp
fn format_timestamp(ms: i64) -> String {
    // Convert milliseconds to ISO 8601 format
    let secs = ms / 1000;
    let nanos = ((ms % 1000) * 1_000_000) as u32;

    if let Some(dt) = chrono::DateTime::from_timestamp(secs, nanos) {
        dt.to_rfc3339()
    } else {
        "2020-01-01T00:00:00Z".to_string()
    }
}

// === Panel Management API ===

/// Add an overlay indicator to the main chart (e.g., EMA, BB)
/// Set separate_scale=true for indicators like MFI (0-100) to map to price range
#[wasm_bindgen]
pub fn add_chart_overlay(indicator_id: &str, separate_scale: bool) -> Result<(), JsValue> {
    log::info!(
        "Adding chart overlay: {} (separate_scale: {})",
        indicator_id,
        separate_scale
    );

    with_state(|state| {
        state
            .borrow_mut()
            .add_chart_overlay(indicator_id, separate_scale)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Remove an overlay from the main chart
#[wasm_bindgen]
pub fn remove_chart_overlay(indicator_id: &str) -> Result<(), JsValue> {
    log::info!("Removing chart overlay: {}", indicator_id);

    with_state(|state| {
        state
            .borrow_mut()
            .remove_chart_overlay(indicator_id)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Add a dedicated indicator panel (e.g., RSI, MACD)
/// Returns panel ID as string
#[wasm_bindgen]
pub fn add_indicator_panel(indicator_id: &str, params_json: &str) -> Result<String, JsValue> {
    log::info!("Adding indicator panel: {}", indicator_id);

    let params: serde_json::Value = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid params JSON: {}", e)))?;

    with_state(|state| Ok(state.borrow_mut().add_indicator_panel(indicator_id, params)))
}

/// Remove a panel by ID
#[wasm_bindgen]
pub fn remove_panel(panel_id: &str) -> Result<(), JsValue> {
    log::info!("Removing panel: {}", panel_id);

    with_state(|state| {
        state
            .borrow_mut()
            .remove_panel(panel_id)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Set total container height (call from window resize)
#[wasm_bindgen]
pub fn set_panel_container_height(height: u32) -> Result<(), JsValue> {
    with_state(|state| {
        state.borrow_mut().set_panel_container_height(height);
        Ok(())
    })
}

/// Resize a panel (user dragged separator)
#[wasm_bindgen]
pub fn resize_panel(panel_id: &str, height: u32) -> Result<(), JsValue> {
    with_state(|state| {
        state
            .borrow_mut()
            .resize_panel(panel_id, height)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Move panel to new position
#[wasm_bindgen]
pub fn move_panel(panel_id: &str, new_index: u32) -> Result<(), JsValue> {
    with_state(|state| {
        state
            .borrow_mut()
            .move_panel(panel_id, new_index as usize)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Reorder panels by swapping two indices (Task 5.2)
#[wasm_bindgen]
pub fn reorder_panels(from_index: u32, to_index: u32) -> Result<(), JsValue> {
    log::info!("Reordering panels: {} -> {}", from_index, to_index);

    with_state(|state| {
        state
            .borrow_mut()
            .reorder_panels(from_index as usize, to_index as usize)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Collapse/minimize a panel (Task 5.3)
#[wasm_bindgen]
pub fn collapse_panel(panel_id: &str) -> Result<(), JsValue> {
    log::info!("Collapsing panel: {}", panel_id);

    with_state(|state| {
        state
            .borrow_mut()
            .collapse_panel(panel_id)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Expand/restore a panel (Task 5.3)
#[wasm_bindgen]
pub fn expand_panel(panel_id: &str) -> Result<(), JsValue> {
    log::info!("Expanding panel: {}", panel_id);

    with_state(|state| {
        state
            .borrow_mut()
            .expand_panel(panel_id)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Maximize a panel (collapse all others) (Task 5.3)
#[wasm_bindgen]
pub fn maximize_panel(panel_id: &str) -> Result<(), JsValue> {
    log::info!("Maximizing panel: {}", panel_id);

    with_state(|state| {
        state
            .borrow_mut()
            .maximize_panel(panel_id)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Restore all panels (expand all collapsed) (Task 5.3)
#[wasm_bindgen]
pub fn restore_all_panels() -> Result<(), JsValue> {
    log::info!("Restoring all panels");

    with_state(|state| {
        state
            .borrow_mut()
            .restore_all_panels()
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Check if a panel is collapsed (Task 5.3)
#[wasm_bindgen]
pub fn is_panel_collapsed(panel_id: &str) -> Result<bool, JsValue> {
    with_state(|state| {
        state
            .borrow()
            .is_panel_collapsed(panel_id)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Check if a panel is maximized (Task 5.3)
#[wasm_bindgen]
pub fn is_panel_maximized(panel_id: &str) -> Result<bool, JsValue> {
    with_state(|state| {
        state
            .borrow()
            .is_panel_maximized(panel_id)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Get panel layout as JSON (for rendering)
#[wasm_bindgen]
pub fn get_panel_layout() -> Result<String, JsValue> {
    with_state(|state| Ok(state.borrow().get_panel_layout()))
}

/// Restore panel layout from JSON (workspace persistence)
#[wasm_bindgen]
pub fn restore_panel_layout(json: &str) -> Result<(), JsValue> {
    with_state(|state| {
        state
            .borrow_mut()
            .restore_panel_layout(json)
            .map_err(|e| JsValue::from_str(&e))
    })
}

// ============================================================================
// ChartCore Indicator API (Phase 2 Task 4.3)
// ============================================================================

/// Add indicator to chart using chartcore's new indicator system
///
/// # Parameters
/// - `indicator_type`: Indicator ID (e.g., "rsi", "sma", "macd")
/// - `params_json`: JSON string with indicator parameters (e.g., `{"period": 14}`)
///
/// # Returns
/// - Indicator ID for later reference
///
/// # Example
/// ```javascript
/// const id = await wasm.add_chartcore_indicator("rsi", '{"period": 14}');
/// ```
#[wasm_bindgen]
pub fn add_chartcore_indicator(indicator_type: &str, params_json: &str) -> Result<String, JsValue> {
    log::info!("Adding chartcore indicator: {}", indicator_type);

    let params: serde_json::Value = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid params JSON: {}", e)))?;

    with_state(|state| {
        state
            .borrow_mut()
            .add_chartcore_indicator(indicator_type, params)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Update indicator parameters
///
/// # Parameters
/// - `indicator_id`: ID returned from add_chartcore_indicator
/// - `params_json`: New parameters as JSON
#[wasm_bindgen]
pub fn update_chartcore_indicator_params(
    indicator_id: &str,
    params_json: &str,
) -> Result<(), JsValue> {
    log::info!("Updating indicator params: {}", indicator_id);

    let params: serde_json::Value = serde_json::from_str(params_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid params JSON: {}", e)))?;

    with_state(|state| {
        state
            .borrow_mut()
            .update_chartcore_indicator_params(indicator_id, params)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Remove indicator from chart
#[wasm_bindgen]
pub fn remove_chartcore_indicator(indicator_id: &str) -> Result<(), JsValue> {
    log::info!("Removing chartcore indicator: {}", indicator_id);

    with_state(|state| {
        state
            .borrow_mut()
            .remove_chartcore_indicator(indicator_id)
            .map_err(|e| JsValue::from_str(&e))
    })
}

/// Get list of active indicators with their configurations
///
/// Returns JSON array of indicator objects
#[wasm_bindgen]
pub fn get_active_chartcore_indicators() -> Result<String, JsValue> {
    with_state(|state| Ok(state.borrow().get_active_chartcore_indicators()))
}

/// Get list of all available indicator types
///
/// Returns JSON array with indicator metadata (id, name, category, params)
#[wasm_bindgen]
pub fn get_available_indicators() -> Result<String, JsValue> {
    // Return metadata for all 12 migrated indicators
    let indicators = serde_json::json!([
        {
            "id": "rsi",
            "name": "RSI",
            "category": "Momentum",
            "description": "Relative Strength Index",
            "supportsOverlay": true,
            "requiresPanel": false,
            "defaultParams": {"period": 14},
            "defaultScale": {"min": 0.0, "max": 100.0}
        },
        {
            "id": "sma",
            "name": "SMA",
            "category": "Trend",
            "description": "Simple Moving Average",
            "supportsOverlay": true,
            "requiresPanel": false,
            "defaultParams": {"period": 20}
        },
        {
            "id": "ema",
            "name": "EMA",
            "category": "Trend",
            "description": "Exponential Moving Average",
            "supportsOverlay": true,
            "requiresPanel": false,
            "defaultParams": {"period": 20}
        },
        {
            "id": "macd",
            "name": "MACD",
            "category": "Momentum",
            "description": "Moving Average Convergence Divergence",
            "supportsOverlay": false,
            "requiresPanel": true,
            "defaultParams": {"fast": 12, "slow": 26, "signal": 9}
        },
        {
            "id": "bb",
            "name": "Bollinger Bands",
            "category": "Volatility",
            "description": "Bollinger Bands",
            "supportsOverlay": true,
            "requiresPanel": false,
            "defaultParams": {"period": 20, "std_dev": 2.0}
        },
        {
            "id": "stochastic",
            "name": "Stochastic",
            "category": "Momentum",
            "description": "Stochastic Oscillator",
            "supportsOverlay": false,
            "requiresPanel": true,
            "defaultParams": {"k_period": 14, "k_smooth": 3, "d_period": 3},
            "defaultScale": {"min": 0.0, "max": 100.0}
        },
        {
            "id": "atr",
            "name": "ATR",
            "category": "Volatility",
            "description": "Average True Range",
            "supportsOverlay": false,
            "requiresPanel": true,
            "defaultParams": {"period": 14}
        },
        {
            "id": "williams_r",
            "name": "Williams %R",
            "category": "Momentum",
            "description": "Williams Percent Range",
            "supportsOverlay": false,
            "requiresPanel": true,
            "defaultParams": {"period": 14},
            "defaultScale": {"min": -100.0, "max": 0.0}
        },
        {
            "id": "adx",
            "name": "ADX",
            "category": "Trend",
            "description": "Average Directional Index",
            "supportsOverlay": false,
            "requiresPanel": true,
            "defaultParams": {"period": 14}
        },
        {
            "id": "hma",
            "name": "HMA",
            "category": "Trend",
            "description": "Hull Moving Average",
            "supportsOverlay": true,
            "requiresPanel": false,
            "defaultParams": {"period": 20}
        },
        {
            "id": "wma",
            "name": "WMA",
            "category": "Trend",
            "description": "Weighted Moving Average",
            "supportsOverlay": true,
            "requiresPanel": false,
            "defaultParams": {"period": 20}
        },
        {
            "id": "vwma",
            "name": "VWMA",
            "category": "Volume",
            "description": "Volume Weighted Moving Average",
            "supportsOverlay": true,
            "requiresPanel": false,
            "defaultParams": {"period": 20}
        }
    ]);

    Ok(indicators.to_string())
}

// ============================================================================
// Indicator Metadata API (re-exported from chartcore with wasm_bindgen)
// ============================================================================

/// Get all available indicators with metadata
/// Returns JSON array of indicator metadata
#[wasm_bindgen]
pub fn get_all_indicators() -> String {
    chartcore::wasm::get_all_indicators()
}

/// Get metadata for a specific indicator by ID
/// Returns JSON object with indicator metadata, or null if not found
#[wasm_bindgen]
pub fn get_indicator_metadata(indicator_id: &str) -> JsValue {
    chartcore::wasm::get_indicator_metadata(indicator_id)
}

// Helper to access global state
fn with_state<F, R>(f: F) -> Result<R, JsValue>
where
    F: FnOnce(&Rc<RefCell<AppState>>) -> Result<R, JsValue>,
{
    APP_STATE.with(|s| {
        let state = s.borrow();
        match state.as_ref() {
            Some(app) => f(app),
            None => Err(JsValue::from_str(
                "WASM not initialized. Call init() first.",
            )),
        }
    })
}
