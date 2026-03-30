//! Application state management
//!
//! Implements a state machine for connection lifecycle:
//!
//! ```text
//! Disconnected → Connecting → Syncing → Connected
//!       ↑              ↓          ↓          ↓
//!       └──────────────┴──────────┴── Error ─┘
//!                      ↓          ↓          ↓
//!                      └──────────┴─ Reconnecting
//! ```

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

use crate::bridge;
use crate::indicators::IndicatorEngine;
use crate::types::{Candle, Config, ConnectionStatus, IndicatorConfig};
use crate::websocket::{make_topic, WsClient};

use chartcore::panels::{OverlayConfig, PanelConfig, PanelId, PanelManager, PanelType};

const MAX_CANDLES: usize = 2000;

/// Connection state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// WebSocket connecting
    Connecting,
    /// Connected, waiting for snapshot
    Syncing,
    /// Fully connected and receiving updates
    Connected,
    /// Reconnecting after disconnect
    Reconnecting,
    /// Error state
    Error,
}

impl From<ConnectionState> for ConnectionStatus {
    fn from(state: ConnectionState) -> Self {
        match state {
            ConnectionState::Disconnected => ConnectionStatus::Disconnected,
            ConnectionState::Connecting => ConnectionStatus::Connecting,
            ConnectionState::Syncing => ConnectionStatus::Syncing,
            ConnectionState::Connected => ConnectionStatus::Connected,
            ConnectionState::Reconnecting => ConnectionStatus::Reconnecting,
            ConnectionState::Error => ConnectionStatus::Error,
        }
    }
}

/// Main application state
pub struct AppState {
    config: Config,

    // Connection
    source: String,
    symbol: String,
    timeframe: String,
    state: ConnectionState,
    ws_client: Option<WsClient>,

    // Data
    candles: VecDeque<Candle>,
    last_ts: Option<String>, // ISO 8601 timestamp string

    // Indicators
    indicators: Vec<IndicatorConfig>,
    indicator_engine: IndicatorEngine,

    // Panel system
    panel_manager: PanelManager,
    main_chart_panel_id: Option<PanelId>,

    // Gap detection
    expected_interval_ms: i64,
    last_ts_ms: Option<i64>, // For gap detection (milliseconds)

    // Reconnection
    reconnect_attempt: u32,
    max_reconnect_attempts: u32,
    base_delay_ms: u32,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        // Initialize panel system with default main chart panel
        let mut panel_manager = PanelManager::new();
        let main_chart = PanelConfig::new_chart();
        let main_chart_id = main_chart.id;
        panel_manager.add_panel(main_chart, None);

        // Set default height (will be updated from frontend)
        panel_manager.set_total_height(600);

        Self {
            config,
            source: String::new(),
            symbol: String::new(),
            timeframe: String::new(),
            state: ConnectionState::Disconnected,
            ws_client: None,
            candles: VecDeque::with_capacity(MAX_CANDLES),
            last_ts: None,
            indicators: Vec::new(),
            indicator_engine: IndicatorEngine::new(),
            panel_manager,
            main_chart_panel_id: Some(main_chart_id),
            expected_interval_ms: 60000, // Default 1m
            last_ts_ms: None,
            // Reconnection defaults
            reconnect_attempt: 0,
            max_reconnect_attempts: 5,
            base_delay_ms: 1000, // 1s base, exponential: 1s, 2s, 4s, 8s, 16s
        }
    }

    /// Parse ISO 8601 timestamp to milliseconds since epoch
    fn parse_ts_ms(ts: &str) -> Option<i64> {
        // Simple ISO 8601 parser: "2024-01-15T10:30:00.000Z"
        if ts.len() < 19 {
            return None;
        }
        let year: i32 = ts[0..4].parse().ok()?;
        let month: u32 = ts[5..7].parse().ok()?;
        let day: u32 = ts[8..10].parse().ok()?;
        let hour: u32 = ts[11..13].parse().ok()?;
        let min: u32 = ts[14..16].parse().ok()?;
        let sec: u32 = ts[17..19].parse().ok()?;

        // Parse optional milliseconds
        let ms: u32 = if ts.len() >= 23 && &ts[19..20] == "." {
            ts[20..23].parse().unwrap_or(0)
        } else {
            0
        };

        let days = Self::days_from_epoch(year, month, day);
        let secs = (days as i64) * 86400 + (hour as i64) * 3600 + (min as i64) * 60 + (sec as i64);
        Some(secs * 1000 + ms as i64)
    }

    fn days_from_epoch(year: i32, month: u32, day: u32) -> i32 {
        let y = if month <= 2 { year - 1 } else { year };
        let m = if month <= 2 { month + 12 } else { month };
        let era = y / 400;
        let yoe = y - era * 400;
        let doy = (153 * (m as i32 - 3) + 2) / 5 + day as i32 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    fn set_state(&mut self, new_state: ConnectionState) {
        if self.state != new_state {
            log::info!("State transition: {:?} → {:?}", self.state, new_state);
            self.state = new_state;
            bridge::on_connection_change(&ConnectionStatus::from(new_state));
        }
    }

    fn timeframe_to_ms(tf: &str) -> i64 {
        match tf {
            "1s" => 1000,
            "1m" => 60_000,
            "5m" => 300_000,
            "15m" => 900_000,
            "30m" => 1_800_000,
            "1h" => 3_600_000,
            "4h" => 14_400_000,
            "1d" => 86_400_000,
            _ => 60_000,
        }
    }

    pub fn connect(
        &mut self,
        source: &str,
        symbol: &str,
        tf: &str,
        ws_url: &str,
    ) -> Result<(), JsValue> {
        // Clean up existing connection
        if self.ws_client.is_some() {
            self.disconnect()?;
        }

        self.source = source.to_string();
        self.symbol = symbol.to_string();
        self.timeframe = tf.to_string();
        self.expected_interval_ms = Self::timeframe_to_ms(tf);

        self.set_state(ConnectionState::Connecting);

        // Create WebSocket client (callbacks set up separately via connect_with_state)
        let mut ws = WsClient::new();
        let topic = make_topic(source, symbol, tf);
        ws.connect(ws_url, &topic)?;

        self.ws_client = Some(ws);
        self.set_state(ConnectionState::Syncing);

        log::info!("Connecting to {}:{}:{}", source, symbol, tf);
        Ok(())
    }

    /// Connect with Rc wrapper for proper callback wiring
    pub fn connect_with_callbacks(
        state: Rc<RefCell<Self>>,
        source: &str,
        symbol: &str,
        tf: &str,
        ws_url: &str,
    ) -> Result<(), JsValue> {
        // Clean up existing connection
        {
            let mut s = state.borrow_mut();
            if s.ws_client.is_some() {
                s.disconnect()?;
            }

            s.source = source.to_string();
            s.symbol = symbol.to_string();
            s.timeframe = tf.to_string();
            s.expected_interval_ms = Self::timeframe_to_ms(tf);
            s.set_state(ConnectionState::Connecting);
        }

        // Create WebSocket client with callbacks
        let mut ws = WsClient::new();

        // Wire up snapshot callback
        let state_snapshot = Rc::clone(&state);
        ws.on_snapshot(move |candles| {
            state_snapshot.borrow_mut().on_candle_snapshot(candles);
        });

        // Wire up candle update callback
        let state_candle = Rc::clone(&state);
        ws.on_candle(move |candle| {
            state_candle.borrow_mut().on_candle_update(candle);
        });

        // Wire up backfill callback (merges instead of replacing)
        let state_backfill = Rc::clone(&state);
        ws.on_backfill(move |candles| {
            state_backfill.borrow_mut().on_candle_backfill(candles);
        });

        // Wire up status change callback
        let state_status = Rc::clone(&state);
        ws.on_status(move |status| {
            let mut s = state_status.borrow_mut();
            match status {
                ConnectionStatus::Connected => s.on_connected(),
                ConnectionStatus::Disconnected => s.on_disconnected(),
                ConnectionStatus::Error => s.on_error("Connection error"),
                _ => {}
            }
        });

        // Connect to Phoenix channel
        let topic = make_topic(source, symbol, tf);
        ws.connect(ws_url, &topic)?;

        // Store WebSocket client
        {
            let mut s = state.borrow_mut();
            s.ws_client = Some(ws);
            s.set_state(ConnectionState::Syncing);
        }

        log::info!("Connected with callbacks to {}:{}:{}", source, symbol, tf);
        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), JsValue> {
        if let Some(ref mut ws) = self.ws_client {
            ws.disconnect();
        }
        self.ws_client = None;
        self.set_state(ConnectionState::Disconnected);
        Ok(())
    }

    pub fn set_symbol(&mut self, symbol: &str) -> Result<(), JsValue> {
        self.symbol = symbol.to_string();
        self.clear_data();
        self.resync()
    }

    pub fn set_timeframe(&mut self, tf: &str) -> Result<(), JsValue> {
        self.timeframe = tf.to_string();
        self.expected_interval_ms = Self::timeframe_to_ms(tf);
        self.clear_data();
        self.resync()
    }

    pub fn toggle_indicator(
        &mut self,
        name: &str,
        params_json: &str,
        enabled: bool,
    ) -> Result<(), JsValue> {
        let params: serde_json::Value =
            serde_json::from_str(params_json).unwrap_or(serde_json::Value::Null);

        // Find or create indicator config
        if let Some(ind) = self.indicators.iter_mut().find(|i| i.name == name) {
            ind.enabled = enabled;
            ind.params = params;
        } else {
            self.indicators.push(IndicatorConfig {
                id: name.to_string(), // Use name as ID for old system
                name: name.to_string(),
                params,
                enabled,
            });
        }

        // Update indicator engine
        if enabled {
            self.indicator_engine.enable(name, &self.candles);
        } else {
            self.indicator_engine.disable(name);
        }

        Ok(())
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        ConnectionStatus::from(self.state)
    }

    #[allow(dead_code)]
    pub fn connection_state(&self) -> ConnectionState {
        self.state
    }

    pub fn last_candle(&self) -> Option<Candle> {
        self.candles.back().cloned()
    }

    /// Handle incoming candle snapshot from WebSocket
    pub fn on_candle_snapshot(&mut self, candles: Vec<Candle>) {
        self.candles.clear();

        for c in candles {
            self.last_ts = Some(c.ts.clone());
            self.last_ts_ms = Self::parse_ts_ms(&c.ts);
            self.candles.push_back(c);
        }

        // Sync last_ts to WsClient for reconnect delta-sync
        if let (Some(ts_ms), Some(ref mut ws)) = (self.last_ts_ms, &mut self.ws_client) {
            ws.update_last_ts(ts_ms);
        }

        // Recalculate all indicators
        self.indicator_engine.recalculate(&self.candles);

        // Update chart
        bridge::set_candles(&self.candles);

        // Transition to connected state
        self.set_state(ConnectionState::Connected);

        log::info!("Snapshot received: {} candles", self.candles.len());
    }

    /// Handle incoming candle update from WebSocket
    pub fn on_candle_update(&mut self, candle: Candle) {
        let candle_ts_ms = Self::parse_ts_ms(&candle.ts);

        // Gap detection using millisecond timestamps
        if let (Some(last_ms), Some(curr_ms)) = (self.last_ts_ms, candle_ts_ms) {
            let gap = curr_ms - last_ms;
            if gap > self.expected_interval_ms * 2 {
                log::warn!(
                    "Gap detected: {} ms (expected {}). Requesting backfill.",
                    gap,
                    self.expected_interval_ms
                );
                // Request backfill if we have a WebSocket
                if let Some(ref mut ws) = self.ws_client {
                    let _ = ws.request_backfill(last_ms);
                }
            }
        }

        // Check if this updates the last candle or is a new one
        // ISO timestamps compare correctly lexicographically when in UTC
        if let Some(last) = self.candles.back_mut() {
            if last.ts == candle.ts && !last.is_final {
                // Update existing candle
                *last = candle.clone();
            } else if candle.ts > last.ts {
                // New candle
                self.push_candle(candle.clone());
            }
        } else {
            // First candle
            self.push_candle(candle.clone());
        }

        self.last_ts = Some(candle.ts.clone());
        self.last_ts_ms = candle_ts_ms;

        // Sync last_ts to WsClient for reconnect delta-sync
        if let (Some(ts_ms), Some(ref mut ws)) = (self.last_ts_ms, &mut self.ws_client) {
            ws.update_last_ts(ts_ms);
        }

        // Update indicators
        self.indicator_engine.on_candle(&candle);

        // Notify JS
        bridge::update_candle(&candle);
        bridge::on_candle_update(&candle);
    }

    /// Handle final candle (closed)
    #[allow(dead_code)]
    pub fn on_candle_final(&mut self, candle: Candle) {
        let mut final_candle = candle;
        final_candle.is_final = true;
        self.on_candle_update(final_candle);
    }

    /// Handle backfill response - merge candles instead of replacing
    pub fn on_candle_backfill(&mut self, candles: Vec<Candle>) {
        if candles.is_empty() {
            log::info!("Backfill received: 0 candles (no gap)");
            return;
        }

        log::info!("Backfill received: {} candles, merging...", candles.len());

        // Collect existing timestamps for deduplication
        let existing_ts: std::collections::HashSet<String> =
            self.candles.iter().map(|c| c.ts.clone()).collect();

        // Insert backfill candles that don't already exist
        let mut inserted = 0;
        for c in candles {
            if !existing_ts.contains(&c.ts) {
                // Find insertion point to maintain time order
                let insert_pos = self.candles.iter().position(|existing| existing.ts > c.ts);
                match insert_pos {
                    Some(pos) => {
                        // Insert at correct position
                        let mut new_deque =
                            std::collections::VecDeque::with_capacity(self.candles.len() + 1);
                        new_deque.extend(self.candles.drain(..pos));
                        new_deque.push_back(c);
                        new_deque.extend(self.candles.drain(..));
                        self.candles = new_deque;
                    }
                    None => {
                        // Append at end
                        self.candles.push_back(c);
                    }
                }
                inserted += 1;
            }
        }

        if inserted > 0 {
            // Trim if we exceeded max
            while self.candles.len() > MAX_CANDLES {
                self.candles.pop_front();
            }

            // Recalculate indicators with merged data
            self.indicator_engine.recalculate(&self.candles);

            // Update chart with complete data
            bridge::set_candles(&self.candles);

            log::info!("Backfill merged: {} new candles inserted", inserted);
        } else {
            log::info!("Backfill: all candles already present");
        }
    }

    pub fn push_candle(&mut self, candle: Candle) {
        if self.candles.len() >= MAX_CANDLES {
            self.candles.pop_front();
        }
        self.candles.push_back(candle);
    }

    fn clear_data(&mut self) {
        self.candles.clear();
        self.last_ts = None;
        self.last_ts_ms = None;
        self.indicator_engine.reset();
    }

    fn resync(&mut self) -> Result<(), JsValue> {
        if self.state == ConnectionState::Connected {
            // Reconnect to the new symbol/timeframe
            let source = self.source.clone();
            let symbol = self.symbol.clone();
            let tf = self.timeframe.clone();
            let ws_url = self.config.ws_url.clone();

            self.connect(&source, &symbol, &tf, &ws_url)?;
        }
        Ok(())
    }

    /// Handle connection established (called from WS callback)
    pub fn on_connected(&mut self) {
        // Reset reconnection counter on successful connection
        self.reconnect_attempt = 0;
        self.set_state(ConnectionState::Syncing);
        log::info!("Connected, waiting for snapshot...");
    }

    /// Handle connection error (called from WS callback)
    pub fn on_error(&mut self, error: &str) {
        log::error!("Connection error: {}", error);
        self.set_state(ConnectionState::Error);
        bridge::on_error(error);

        // Try to reconnect on error
        if self.should_reconnect() {
            self.set_state(ConnectionState::Reconnecting);
        }
    }

    /// Handle disconnection (called from WS callback)
    pub fn on_disconnected(&mut self) {
        if self.state == ConnectionState::Connected || self.state == ConnectionState::Syncing {
            // Unexpected disconnect - try to reconnect
            log::warn!("Unexpected disconnection, will attempt reconnect");
            if self.should_reconnect() {
                self.set_state(ConnectionState::Reconnecting);
            } else {
                log::error!("Max reconnection attempts reached");
                self.set_state(ConnectionState::Error);
                bridge::on_error("Max reconnection attempts exceeded");
            }
        } else {
            self.set_state(ConnectionState::Disconnected);
        }
    }

    /// Check if we should attempt reconnection
    fn should_reconnect(&self) -> bool {
        self.reconnect_attempt < self.max_reconnect_attempts
    }

    /// Calculate delay for next reconnection attempt (exponential backoff with jitter)
    pub fn next_reconnect_delay_ms(&mut self) -> u32 {
        let delay = self.base_delay_ms * (1 << self.reconnect_attempt.min(4));
        // Add jitter: ±25%
        let jitter = (delay / 4) as i32;
        let random_jitter = (js_sys::Math::random() * (jitter * 2) as f64) as i32 - jitter;
        let final_delay = (delay as i32 + random_jitter).max(100) as u32;

        self.reconnect_attempt += 1;
        log::info!(
            "Reconnect attempt {}/{} in {}ms",
            self.reconnect_attempt,
            self.max_reconnect_attempts,
            final_delay
        );
        final_delay
    }

    /// Get reconnection info for JS to schedule timer
    pub fn get_reconnect_info(&self) -> Option<(String, String, String, String)> {
        if self.state == ConnectionState::Reconnecting {
            Some((
                self.source.clone(),
                self.symbol.clone(),
                self.timeframe.clone(),
                self.config.ws_url.clone(),
            ))
        } else {
            None
        }
    }

    /// Reset reconnection state (call when user manually disconnects)
    pub fn reset_reconnect(&mut self) {
        self.reconnect_attempt = 0;
    }

    /// Send heartbeat to keep connection alive
    pub fn send_heartbeat(&mut self) {
        if let Some(ref mut ws) = self.ws_client {
            ws.send_heartbeat();
        }
    }

    /// Process heartbeat reply
    pub fn on_heartbeat_reply(&mut self, ref_id: &str) {
        if let Some(ref mut ws) = self.ws_client {
            ws.on_heartbeat_reply(ref_id);
        }
    }

    /// Check if connection is active (for heartbeat)
    pub fn has_active_connection(&self) -> bool {
        self.ws_client.is_some() && self.state == ConnectionState::Connected
    }

    /// Clear all candles (for test data loading)
    pub fn clear_candles(&mut self) {
        self.candles.clear();
        self.last_ts = None;
        self.last_ts_ms = None;
    }

    /// Send current candles to chart (for test data loading)
    pub fn send_candles_to_chart(&self) {
        bridge::set_candles(&self.candles);
    }

    // === Panel Management API ===

    /// Add an overlay indicator to the main chart panel (e.g., EMA, BB, MFI with scale mapping)
    pub fn add_chart_overlay(
        &mut self,
        indicator_id: &str,
        separate_scale: bool,
    ) -> Result<(), String> {
        let main_panel_id = self.main_chart_panel_id.ok_or("No main chart panel")?;

        // Enable indicator in engine
        self.indicator_engine.enable(indicator_id, &self.candles);

        // Determine scale range for separate-scale indicators
        let (scale_min, scale_max) = if separate_scale {
            match indicator_id {
                id if id.starts_with("rsi") => (Some(0.0), Some(100.0)),
                id if id.starts_with("mfi") => (Some(0.0), Some(100.0)),
                id if id.starts_with("stoch") => (Some(0.0), Some(100.0)),
                id if id.starts_with("cci") => (Some(-200.0), Some(200.0)),
                id if id.starts_with("williams") || id.starts_with("wpr") => {
                    (Some(-100.0), Some(0.0))
                }
                _ => (None, None),
            }
        } else {
            (None, None)
        };

        let overlay = OverlayConfig {
            id: indicator_id.to_string(),
            params: serde_json::json!({}),
            color: self.get_indicator_color(indicator_id),
            separate_scale,
            scale_min,
            scale_max,
        };

        // Add to panel config
        if let Some(panel) = self.panel_manager.get_panel_mut(main_panel_id) {
            panel.config.add_overlay(overlay);
        }

        Ok(())
    }

    /// Remove an overlay from the main chart panel
    pub fn remove_chart_overlay(&mut self, indicator_id: &str) -> Result<(), String> {
        let main_panel_id = self.main_chart_panel_id.ok_or("No main chart panel")?;

        // Disable indicator in engine
        self.indicator_engine.disable(indicator_id);

        // Remove from panel config
        if let Some(panel) = self.panel_manager.get_panel_mut(main_panel_id) {
            panel.config.remove_overlay(indicator_id);
        }

        Ok(())
    }

    /// Add a dedicated indicator panel (e.g., RSI, MACD in separate pane)
    pub fn add_indicator_panel(&mut self, indicator_id: &str, params: serde_json::Value) -> String {
        // Enable indicator in engine
        self.indicator_engine.enable(indicator_id, &self.candles);

        // Create panel
        let panel = PanelConfig::new_indicator(indicator_id, params);
        let panel_id = panel.id;
        self.panel_manager.add_panel(panel, None);

        panel_id.to_string()
    }

    /// Remove a panel by ID
    pub fn remove_panel(&mut self, panel_id_str: &str) -> Result<(), String> {
        let uuid = panel_id_str
            .parse::<uuid::Uuid>()
            .map_err(|_| format!("Invalid panel ID: {}", panel_id_str))?;
        let panel_id = unsafe { std::mem::transmute::<uuid::Uuid, PanelId>(uuid) };

        // Get panel config before removal to disable indicator
        if let Some(config) = self.panel_manager.remove_panel(panel_id) {
            if let PanelType::Indicator { indicator_id, .. } = config.panel_type {
                self.indicator_engine.disable(&indicator_id);
            }
            Ok(())
        } else {
            Err(format!("Panel not found: {}", panel_id_str))
        }
    }

    /// Set total container height (called from frontend on resize)
    pub fn set_panel_container_height(&mut self, height: u32) {
        self.panel_manager.set_total_height(height);
    }

    /// Set panel height (user dragged separator)
    pub fn resize_panel(&mut self, panel_id_str: &str, height: u32) -> Result<(), String> {
        let uuid = panel_id_str
            .parse::<uuid::Uuid>()
            .map_err(|_| format!("Invalid panel ID: {}", panel_id_str))?;
        let panel_id = unsafe { std::mem::transmute::<uuid::Uuid, PanelId>(uuid) };

        if self.panel_manager.set_panel_height(panel_id, height) {
            Ok(())
        } else {
            Err(format!("Failed to resize panel: {}", panel_id_str))
        }
    }

    /// Move panel to new position
    pub fn move_panel(&mut self, panel_id_str: &str, new_index: usize) -> Result<(), String> {
        let uuid = panel_id_str
            .parse::<uuid::Uuid>()
            .map_err(|_| format!("Invalid panel ID: {}", panel_id_str))?;
        let panel_id = unsafe { std::mem::transmute::<uuid::Uuid, PanelId>(uuid) };

        if self.panel_manager.move_panel(panel_id, new_index) {
            Ok(())
        } else {
            Err(format!("Failed to move panel: {}", panel_id_str))
        }
    }

    /// Get panel layout as JSON (for rendering)
    pub fn get_panel_layout(&self) -> String {
        self.panel_manager.to_json().to_string()
    }

    /// Restore panel layout from JSON (for workspace persistence)
    pub fn restore_panel_layout(&mut self, json: &str) -> Result<(), String> {
        self.panel_manager.from_json(json)
    }

    /// Get color for an indicator (simple mapping)
    fn get_indicator_color(&self, indicator_id: &str) -> String {
        match indicator_id {
            id if id.starts_with("ema9") => "#f59e0b".to_string(), // amber
            id if id.starts_with("ema21") => "#3b82f6".to_string(), // blue
            id if id.starts_with("ema50") => "#8b5cf6".to_string(), // purple
            id if id.starts_with("ema200") => "#ef4444".to_string(), // red
            id if id.starts_with("rsi") => "#10b981".to_string(),  // emerald
            id if id.starts_with("mfi") => "#f59e0b".to_string(),  // amber
            id if id.starts_with("macd") => "#06b6d4".to_string(), // cyan
            id if id.starts_with("bb") => "#a78bfa".to_string(),   // violet
            _ => "#6b7280".to_string(),                            // gray
        }
    }

    // ========================================================================
    // ChartCore Indicator Management (Phase 2 Task 4.3)
    // ========================================================================

    /// Add indicator using chartcore's plugin system
    pub fn add_chartcore_indicator(
        &mut self,
        indicator_type: &str,
        params: serde_json::Value,
    ) -> Result<String, String> {
        // Store indicator config
        let id = format!("{}_{}", indicator_type, self.indicators.len());
        let config = IndicatorConfig {
            id: id.clone(),
            name: indicator_type.to_string(),
            enabled: true,
            params,
        };
        self.indicators.push(config);

        log::info!("Added chartcore indicator: {} (id: {})", indicator_type, id);
        Ok(id)
    }

    /// Update indicator parameters
    pub fn update_chartcore_indicator_params(
        &mut self,
        indicator_id: &str,
        params: serde_json::Value,
    ) -> Result<(), String> {
        // Find indicator config
        let config = self
            .indicators
            .iter_mut()
            .find(|c| c.id == indicator_id)
            .ok_or_else(|| format!("Indicator not found: {}", indicator_id))?;

        // Update params
        config.params = params;

        // TODO: Update the actual indicator instance
        // This requires maintaining a map of indicator instances

        log::info!("Updated indicator params: {}", indicator_id);
        Ok(())
    }

    /// Remove chartcore indicator
    pub fn remove_chartcore_indicator(&mut self, indicator_id: &str) -> Result<(), String> {
        // Remove from config list
        let index = self
            .indicators
            .iter()
            .position(|c| c.id == indicator_id)
            .ok_or_else(|| format!("Indicator not found: {}", indicator_id))?;

        self.indicators.remove(index);

        // TODO: Remove from chart renderer

        log::info!("Removed chartcore indicator: {}", indicator_id);
        Ok(())
    }

    /// Get active chartcore indicators as JSON
    pub fn get_active_chartcore_indicators(&self) -> String {
        serde_json::to_string(&self.indicators).unwrap_or_else(|_| "[]".to_string())
    }

    /// Reorder panels by moving from_index to to_index (Task 5.2)
    pub fn reorder_panels(&mut self, from_index: usize, to_index: usize) -> Result<(), String> {
        // Delegate to panel manager
        if self.panel_manager.reorder_panels(from_index, to_index) {
            Ok(())
        } else {
            Err(format!(
                "Failed to reorder panels: {} -> {}",
                from_index, to_index
            ))
        }
    }

    /// Helper: Parse panel ID string to PanelId
    fn parse_panel_id(&self, panel_id_str: &str) -> Result<PanelId, String> {
        let uuid = panel_id_str
            .parse::<uuid::Uuid>()
            .map_err(|_| format!("Invalid panel ID: {}", panel_id_str))?;
        Ok(unsafe { std::mem::transmute::<uuid::Uuid, PanelId>(uuid) })
    }

    /// Collapse/minimize a panel (Task 5.3)
    pub fn collapse_panel(&mut self, panel_id: &str) -> Result<(), String> {
        let id = self.parse_panel_id(panel_id)?;
        if self.panel_manager.collapse_panel(id) {
            Ok(())
        } else {
            Err(format!("Failed to collapse panel: {}", panel_id))
        }
    }

    /// Expand/restore a panel (Task 5.3)
    pub fn expand_panel(&mut self, panel_id: &str) -> Result<(), String> {
        let id = self.parse_panel_id(panel_id)?;
        if self.panel_manager.expand_panel(id) {
            Ok(())
        } else {
            Err(format!("Failed to expand panel: {}", panel_id))
        }
    }

    /// Maximize a panel (collapse all others) (Task 5.3)
    pub fn maximize_panel(&mut self, panel_id: &str) -> Result<(), String> {
        let id = self.parse_panel_id(panel_id)?;
        if self.panel_manager.maximize_panel(id) {
            Ok(())
        } else {
            Err(format!("Failed to maximize panel: {}", panel_id))
        }
    }

    /// Restore all panels (expand all collapsed) (Task 5.3)
    pub fn restore_all_panels(&mut self) -> Result<(), String> {
        if self.panel_manager.restore_all_panels() {
            Ok(())
        } else {
            Err("No panels were collapsed".to_string())
        }
    }

    /// Check if a panel is collapsed
    pub fn is_panel_collapsed(&self, panel_id: &str) -> Result<bool, String> {
        let id = self.parse_panel_id(panel_id)?;
        self.panel_manager
            .is_panel_collapsed(id)
            .ok_or_else(|| format!("Panel not found: {}", panel_id))
    }

    /// Check if a panel is maximized
    pub fn is_panel_maximized(&self, panel_id: &str) -> Result<bool, String> {
        let id = self.parse_panel_id(panel_id)?;
        self.panel_manager
            .is_panel_maximized(id)
            .ok_or_else(|| format!("Panel not found: {}", panel_id))
    }
}
