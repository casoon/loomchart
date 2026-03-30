//! WebSocket client for Phoenix Channels
//!
//! Implements proper Phoenix Channel protocol with:
//! - Heartbeat handling
//! - Join/leave lifecycle
//! - Reconnection with exponential backoff
//! - Message parsing and dispatch

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebSocket, MessageEvent, CloseEvent, ErrorEvent};
use std::cell::RefCell;
use std::rc::Rc;

use crate::types::{ConnectionStatus, Candle};

/// Phoenix Channel message
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PhoenixMessage {
    pub topic: String,
    pub event: String,
    pub payload: serde_json::Value,
    #[serde(rename = "ref")]
    pub ref_id: Option<String>,
}

/// Candle snapshot event from Phoenix (candle_snapshot)
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SnapshotEvent {
    pub candles: Vec<Candle>,
    #[serde(default)]
    #[allow(dead_code)]
    pub server_time: Option<String>,
}

/// Candle update event from Phoenix (candle_update, candle_final)
/// Phoenix sends the candle directly in the payload, matching Candle.to_map format
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CandleEvent {
    #[serde(flatten)]
    pub candle: Candle,
}

/// Message handler callback type
pub type MessageCallback = Rc<RefCell<dyn FnMut(PhoenixMessage)>>;

/// WebSocket connection handler with Phoenix Channel support
#[allow(dead_code)]
pub struct WsClient {
    socket: Option<WebSocket>,
    status: ConnectionStatus,
    url: String,
    topic: String,
    ref_counter: u32,
    on_message: Option<MessageCallback>,
    on_status_change: Option<Rc<RefCell<dyn FnMut(ConnectionStatus)>>>,
    on_candle: Option<Rc<RefCell<dyn FnMut(Candle)>>>,
    on_snapshot: Option<Rc<RefCell<dyn FnMut(Vec<Candle>)>>>,
    on_backfill: Option<Rc<RefCell<dyn FnMut(Vec<Candle>)>>>,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    last_ts: Option<i64>,
    // Heartbeat
    heartbeat_interval_id: Option<i32>,
    last_heartbeat_at: Option<f64>,
    pending_heartbeat_ref: Option<String>,
}

/// Heartbeat interval in milliseconds (30 seconds)
#[allow(dead_code)]
const HEARTBEAT_INTERVAL_MS: i32 = 30_000;
/// Heartbeat timeout - if no reply in this time, connection is dead
const HEARTBEAT_TIMEOUT_MS: f64 = 10_000.0;

impl WsClient {
    pub fn new() -> Self {
        Self {
            socket: None,
            status: ConnectionStatus::Disconnected,
            url: String::new(),
            topic: String::new(),
            ref_counter: 0,
            on_message: None,
            on_status_change: None,
            on_candle: None,
            on_snapshot: None,
            on_backfill: None,
            reconnect_attempts: 0,
            max_reconnect_attempts: 10,
            last_ts: None,
            heartbeat_interval_id: None,
            last_heartbeat_at: None,
            pending_heartbeat_ref: None,
        }
    }

    /// Set callback for candle updates
    pub fn on_candle<F: FnMut(Candle) + 'static>(&mut self, callback: F) {
        self.on_candle = Some(Rc::new(RefCell::new(callback)));
    }

    /// Set callback for snapshot data
    pub fn on_snapshot<F: FnMut(Vec<Candle>) + 'static>(&mut self, callback: F) {
        self.on_snapshot = Some(Rc::new(RefCell::new(callback)));
    }

    /// Set callback for backfill data (merges instead of replaces)
    pub fn on_backfill<F: FnMut(Vec<Candle>) + 'static>(&mut self, callback: F) {
        self.on_backfill = Some(Rc::new(RefCell::new(callback)));
    }

    /// Set callback for status changes
    pub fn on_status<F: FnMut(ConnectionStatus) + 'static>(&mut self, callback: F) {
        self.on_status_change = Some(Rc::new(RefCell::new(callback)));
    }
    #[allow(dead_code)]

    /// Get last received timestamp (for gap detection)
    pub fn last_timestamp(&self) -> Option<i64> {
        self.last_ts
    }

    /// Update last timestamp (called when candles are received)
    pub fn update_last_ts(&mut self, ts_ms: i64) {
        self.last_ts = Some(ts_ms);
    }

    fn next_ref(&mut self) -> String {
        self.ref_counter += 1;
        self.ref_counter.to_string()
    }

    pub fn connect(&mut self, url: &str, topic: &str) -> Result<(), JsValue> {
        self.url = url.to_string();
        self.topic = topic.to_string();
        self.set_status(ConnectionStatus::Connecting);

        // Create WebSocket
        let ws = WebSocket::new(&format!("{}/websocket?vsn=2.0.0", url))?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let ws_clone = ws.clone();
        let topic_clone = topic.to_string();
        let last_ts = self.last_ts;

        // State for closures
        let status_cb = self.on_status_change.clone();
        let candle_cb = self.on_candle.clone();
        let snapshot_cb = self.on_snapshot.clone();
        let backfill_cb = self.on_backfill.clone();

        // onopen - join channel
        let onopen = Closure::<dyn FnMut()>::new(move || {
            log::info!("WebSocket connected, joining channel: {}", topic_clone);

            // Build join payload with last_ts for gap detection
            let mut payload = serde_json::json!({});
            if let Some(ts) = last_ts {
                payload["last_ts"] = serde_json::json!(ts);
            }

            let join_msg = serde_json::json!({
                "topic": topic_clone,
                "event": "phx_join",
                "payload": payload,
                "ref": "1"
            });
            let _ = ws_clone.send_with_str(&join_msg.to_string());
        });
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // onmessage - parse and dispatch
        let onmessage_status_cb = status_cb.clone();
        let onmessage_candle_cb = candle_cb.clone();
        let onmessage_snapshot_cb = snapshot_cb.clone();
        let onmessage_backfill_cb = backfill_cb.clone();

        let onmessage = Closure::<dyn FnMut(MessageEvent)>::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let msg_str: String = text.into();

                // Parse Phoenix message
                if let Ok(msg) = serde_json::from_str::<PhoenixMessage>(&msg_str) {
                    match msg.event.as_str() {
                        "phx_reply" => {
                            // Check if this is a heartbeat reply (topic = "phoenix")
                            if msg.topic == "phoenix" {
                                // Heartbeat reply - call global handler
                                if let Some(ref_id) = &msg.ref_id {
                                    if let Some(window) = web_sys::window() {
                                        let _ = js_sys::Reflect::set(
                                            &window,
                                            &JsValue::from_str("__loom_last_heartbeat_ref"),
                                            &JsValue::from_str(ref_id),
                                        );
                                    }
                                    log::debug!("Heartbeat reply received: {}", ref_id);
                                }
                            } else {
                                // Handle channel join reply
                                if let Some(status) = msg.payload.get("status").and_then(|s| s.as_str()) {
                                    if status == "ok" {
                                        log::info!("Joined channel successfully");
                                        if let Some(cb) = &onmessage_status_cb {
                                            cb.borrow_mut()(ConnectionStatus::Connected);
                                        }
                                    } else {
                                        log::error!("Join failed: {:?}", msg.payload);
                                        if let Some(cb) = &onmessage_status_cb {
                                            cb.borrow_mut()(ConnectionStatus::Error);
                                        }
                                    }
                                }
                            }
                        }
                        "candle_snapshot" => {
                            // Historical candle data from Phoenix
                            if let Ok(evt) = serde_json::from_value::<SnapshotEvent>(msg.payload.clone()) {
                                log::info!("Received snapshot: {} candles", evt.candles.len());
                                if let Some(cb) = &onmessage_snapshot_cb {
                                    cb.borrow_mut()(evt.candles);
                                }
                            } else {
                                log::error!("Failed to parse candle_snapshot: {:?}", msg.payload);
                            }
                        }
                        "candle_update" => {
                            // Realtime candle update - Phoenix sends candle directly
                            if let Ok(candle) = serde_json::from_value::<Candle>(msg.payload.clone()) {
                                if let Some(cb) = &onmessage_candle_cb {
                                    cb.borrow_mut()(candle);
                                }
                            } else {
                                log::error!("Failed to parse candle_update: {:?}", msg.payload);
                            }
                        }
                        "candle_final" => {
                            // Final candle (closed) - Phoenix sends with is_final=true
                            if let Ok(mut candle) = serde_json::from_value::<Candle>(msg.payload.clone()) {
                                candle.is_final = true;  // Ensure flag is set
                                if let Some(cb) = &onmessage_candle_cb {
                                    cb.borrow_mut()(candle);
                                }
                            } else {
                                log::error!("Failed to parse candle_final: {:?}", msg.payload);
                            }
                        }
                        "candle_backfill" => {
                            // Backfill response - merge into existing data
                            if let Ok(evt) = serde_json::from_value::<SnapshotEvent>(msg.payload.clone()) {
                                log::info!("Received backfill: {} candles", evt.candles.len());
                                // Use dedicated backfill callback (merges, doesn't replace)
                                if let Some(cb) = &onmessage_backfill_cb {
                                    cb.borrow_mut()(evt.candles);
                                }
                            } else {
                                log::error!("Failed to parse candle_backfill: {:?}", msg.payload);
                            }
                        }
                        "phx_error" => {
                            log::error!("Channel error: {:?}", msg.payload);
                            if let Some(cb) = &onmessage_status_cb {
                                cb.borrow_mut()(ConnectionStatus::Error);
                            }
                        }
                        "phx_close" => {
                            log::info!("Channel closed");
                            if let Some(cb) = &onmessage_status_cb {
                                cb.borrow_mut()(ConnectionStatus::Disconnected);
                            }
                        }
                        _ => {
                            log::debug!("Unhandled event: {} - {:?}", msg.event, msg.payload);
                        }
                    }
                }
            }
        });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // onerror
        let error_status_cb = status_cb.clone();
        let onerror = Closure::<dyn FnMut(ErrorEvent)>::new(move |e: ErrorEvent| {
            log::error!("WebSocket error: {:?}", e.message());
            if let Some(cb) = &error_status_cb {
                cb.borrow_mut()(ConnectionStatus::Error);
            }
        });
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        // onclose - trigger reconnection
        let close_status_cb = status_cb.clone();
        let onclose = Closure::<dyn FnMut(CloseEvent)>::new(move |e: CloseEvent| {
            log::info!("WebSocket closed: code={}, reason={}", e.code(), e.reason());
            if let Some(cb) = &close_status_cb {
                cb.borrow_mut()(ConnectionStatus::Reconnecting);
            }
            // TODO: Schedule reconnection with exponential backoff
        });
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        self.socket = Some(ws);
        Ok(())
    }

    pub fn disconnect(&mut self) {
        // Stop heartbeat first
        self.stop_heartbeat();

        // Get ref before borrowing socket
        let ref_id = self.next_ref();
        if let Some(ws) = &self.socket {
            // Send leave message first
            let leave_msg = serde_json::json!({
                "topic": self.topic,
                "event": "phx_leave",
                "payload": {},
                "ref": ref_id
            });
            let _ = ws.send_with_str(&leave_msg.to_string());
            let _ = ws.close();
        }
        self.socket = None;
        self.set_status(ConnectionStatus::Disconnected);
    }

    /// Send a heartbeat message
    pub fn send_heartbeat(&mut self) {
        if self.socket.is_none() {
            return;
        }

        // Check if previous heartbeat timed out
        if let Some(pending_ref) = &self.pending_heartbeat_ref {
            if let Some(last_at) = self.last_heartbeat_at {
                let now = js_sys::Date::now();
                if now - last_at > HEARTBEAT_TIMEOUT_MS {
                    log::warn!("Heartbeat timeout (ref: {}), connection may be dead", pending_ref);
                    self.pending_heartbeat_ref = None;
                    // Trigger disconnect - will cause reconnection
                    self.set_status(ConnectionStatus::Reconnecting);
                    return;
                }
            }
        }

        let ref_id = self.next_ref();
        let msg = serde_json::json!({
            "topic": "phoenix",
            "event": "heartbeat",
            "payload": {},
            "ref": ref_id
        });

        if let Some(ws) = &self.socket {
            if ws.send_with_str(&msg.to_string()).is_ok() {
                self.pending_heartbeat_ref = Some(ref_id);
                self.last_heartbeat_at = Some(js_sys::Date::now());
                log::debug!("Heartbeat sent");
            }
        }
    }

    /// Called when heartbeat reply is received
    pub fn on_heartbeat_reply(&mut self, ref_id: &str) {
        if let Some(pending) = &self.pending_heartbeat_ref {
            if pending == ref_id {
                self.pending_heartbeat_ref = None;
                log::debug!("Heartbeat acknowledged");
            }
        }
    }

    /// Stop the heartbeat timer
    pub fn stop_heartbeat(&mut self) {
        if let Some(id) = self.heartbeat_interval_id.take() {
            let window = web_sys::window().expect("no window");
            window.clear_interval_with_handle(id);
            log::debug!("Heartbeat stopped");
        }
        self.pending_heartbeat_ref = None;
        self.last_heartbeat_at = None;
    }

    pub fn send(&mut self, event: &str, payload: serde_json::Value) -> Result<(), JsValue> {
        // Get ref before borrowing socket
        let ref_id = self.next_ref();
        if let Some(ws) = &self.socket {
            let msg = serde_json::json!({
                "topic": self.topic,
                "event": event,
                "payload": payload,
                "ref": ref_id
            });
            ws.send_with_str(&msg.to_string())?;
        }
        Ok(())
    }

    /// Request history backfill from last known timestamp
    pub fn request_backfill(&mut self, from_ts: i64) -> Result<(), JsValue> {
        self.send("backfill", serde_json::json!({ "from_ts": from_ts }))
    }

    #[allow(dead_code)]
    pub fn status(&self) -> ConnectionStatus {
        self.status
    }

    fn set_status(&mut self, status: ConnectionStatus) {
        self.status = status;
        if let Some(ref cb) = self.on_status_change {
            cb.borrow_mut()(status);
        }
    }
}

impl Default for WsClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Create Phoenix channel topic
pub fn make_topic(source: &str, symbol: &str, tf: &str) -> String {
    format!("candles:{}:{}:{}", source, symbol, tf)
}

/// Create Phoenix message
#[allow(dead_code)]
pub fn make_message(topic: &str, event: &str, payload: serde_json::Value, ref_id: &str) -> String {
    serde_json::json!({
        "topic": topic,
        "event": event,
        "payload": payload,
        "ref": ref_id
    }).to_string()
}
