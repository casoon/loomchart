//! WASM Entry Point - JavaScript API for the chart engine

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use web_sys::HtmlCanvasElement;

#[cfg(feature = "wasm")]
use crate::core::{
    Candle, ChartState, EventHandler, FootprintCandle, KeyboardEvent, MouseButton, MouseEvent,
    Timeframe, TouchEvent,
};

#[cfg(feature = "wasm")]
use crate::rendering::{Canvas2DRenderer, Renderer};

/// Main WASM Chart instance that can be controlled from JavaScript
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmChart {
    state: ChartState,
    event_handler: EventHandler,
    renderer: Option<Canvas2DRenderer>,
    undo_stack: Vec<String>,
    redo_stack: Vec<String>,
    compare_symbols: Vec<CompareSymbol>,
    footprint_candles: Vec<FootprintCandle>,
    footprint_enabled: bool,
    drawing_drag_anchor: Option<(i64, f64)>,
    indicator_panes: Vec<IndicatorPane>,
}

#[cfg(feature = "wasm")]
struct CompareSymbol {
    symbol: String,
    candles: Vec<Candle>,
    color: crate::primitives::Color,
}

#[cfg(feature = "wasm")]
struct IndicatorPane {
    pane_id: String,
    indicator_id: String,
    params_json: String,
    height_fraction: f64,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmChart {
    /// Create a new chart instance
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, timeframe: &str) -> Result<WasmChart, JsValue> {
        // Set panic hook for better error messages
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();

        let tf =
            Timeframe::from_str(timeframe).ok_or_else(|| JsValue::from_str("Invalid timeframe"))?;

        Ok(WasmChart {
            state: ChartState::new(width, height, tf),
            event_handler: EventHandler::new(),
            renderer: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            compare_symbols: Vec::new(),
            footprint_candles: Vec::new(),
            footprint_enabled: false,
            drawing_drag_anchor: None,
            indicator_panes: Vec::new(),
        })
    }

    /// Attach a canvas element for rendering
    #[wasm_bindgen(js_name = attachCanvas)]
    pub fn attach_canvas(&mut self, canvas: HtmlCanvasElement) -> Result<(), JsValue> {
        let mut renderer = Canvas2DRenderer::new(canvas)?;

        // Get pixel ratio from renderer
        let pixel_ratio = renderer.pixel_ratio();

        // Resize canvas to match chart dimensions
        let width = self.state.viewport.dimensions.width;
        let height = self.state.viewport.dimensions.height;
        renderer.resize(width, height)?;

        // Update viewport with pixel ratio
        self.state
            .viewport
            .set_dimensions(width, height, pixel_ratio);

        self.renderer = Some(renderer);
        Ok(())
    }

    /// Set candle data from JavaScript array
    #[wasm_bindgen(js_name = setCandles)]
    pub fn set_candles(&mut self, candles_json: &str) -> Result<(), JsValue> {
        let candles: Vec<Candle> = serde_json::from_str(candles_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse candles: {}", e)))?;

        web_sys::console::log_1(
            &format!(
                "[ChartCore] setCandles called with {} candles",
                candles.len()
            )
            .into(),
        );

        self.state.set_candles(candles);

        web_sys::console::log_1(
            &format!(
                "[ChartCore] After set_candles: dirty={}, viewport: {}-{}, price: {:.2}-{:.2}",
                self.state.is_dirty(),
                self.state.viewport.time.start,
                self.state.viewport.time.end,
                self.state.viewport.price.min,
                self.state.viewport.price.max
            )
            .into(),
        );

        Ok(())
    }

    /// Replace all candles (delegates to CandleBuffer::snapshot).
    ///
    /// Backward-compatible alias for `setCandles`; both methods accept the
    /// same JSON format.
    #[wasm_bindgen(js_name = setCandlesBatch)]
    pub fn set_candles_batch(&mut self, candles_json: &str) -> Result<(), JsValue> {
        use crate::core::CandleBuffer;

        let incoming: Vec<Candle> = serde_json::from_str(candles_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse candles: {}", e)))?;

        let mut buf = CandleBuffer::new();
        buf.snapshot(incoming);
        self.state.set_candles(buf.candles().to_vec());
        Ok(())
    }

    /// Merge new candles into the existing dataset (delegates to CandleBuffer::append).
    ///
    /// Existing candles are kept; incoming candles are sorted and deduped.
    /// Duplicate timestamps are overwritten by the incoming value.
    #[wasm_bindgen(js_name = appendCandles)]
    pub fn append_candles(&mut self, candles_json: &str) -> Result<(), JsValue> {
        use crate::core::CandleBuffer;

        let incoming: Vec<Candle> = serde_json::from_str(candles_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse candles: {}", e)))?;

        let mut buf = CandleBuffer::new();
        buf.snapshot(self.state.candles.clone());
        buf.append(&incoming);
        self.state.set_candles(buf.candles().to_vec());
        Ok(())
    }

    /// Upsert a single candle by timestamp (delegates to CandleBuffer::update_running).
    ///
    /// Accepts a JSON object representing one candle.  If a candle with the
    /// same `time` already exists it is replaced in-place; otherwise it is
    /// inserted at the correct sorted position.
    #[wasm_bindgen(js_name = updateRunningCandle)]
    pub fn update_running_candle(&mut self, candle_json: &str) -> Result<(), JsValue> {
        use crate::core::CandleBuffer;

        let candle: Candle = serde_json::from_str(candle_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse candle: {}", e)))?;

        let mut buf = CandleBuffer::new();
        buf.snapshot(self.state.candles.clone());
        buf.update_running(candle);
        self.state.set_candles(buf.candles().to_vec());
        Ok(())
    }

    /// Add a single candle
    #[wasm_bindgen(js_name = addCandle)]
    pub fn add_candle(&mut self, time: i64, o: f64, h: f64, l: f64, c: f64, v: f64) {
        let candle = Candle::new(time, o, h, l, c, v);
        self.state.add_candle(candle);
    }

    /// Get all candles as JSON (for indicator calculations)
    #[wasm_bindgen(js_name = getCandles)]
    pub fn get_candles(&self) -> String {
        serde_json::to_string(&self.state.candles).unwrap_or_else(|_| "[]".to_string())
    }

    /// Add or replace a comparison symbol rendered as normalized percent performance.
    #[wasm_bindgen(js_name = addCompareSymbol)]
    pub fn add_compare_symbol(
        &mut self,
        symbol: &str,
        candles_json: &str,
        color: &str,
    ) -> Result<(), JsValue> {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(JsValue::from_str("Compare symbol must not be empty"));
        }

        let candles: Vec<Candle> = serde_json::from_str(candles_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse compare candles: {}", e)))?;
        if candles.is_empty() {
            return Err(JsValue::from_str("Compare candles must not be empty"));
        }

        let color = crate::primitives::Color::from_hex(color)
            .map_err(|e| JsValue::from_str(&format!("Invalid compare color: {}", e)))?;

        if let Some(existing) = self
            .compare_symbols
            .iter_mut()
            .find(|entry| entry.symbol == symbol)
        {
            existing.candles = candles;
            existing.color = color;
        } else {
            if self.compare_symbols.len() >= 3 {
                return Err(JsValue::from_str(
                    "A maximum of 3 compare symbols is supported",
                ));
            }
            self.compare_symbols.push(CompareSymbol {
                symbol: symbol.to_string(),
                candles,
                color,
            });
        }

        self.state.mark_dirty();
        Ok(())
    }

    /// Remove a comparison symbol.
    #[wasm_bindgen(js_name = removeCompareSymbol)]
    pub fn remove_compare_symbol(&mut self, symbol: &str) {
        self.compare_symbols
            .retain(|entry| entry.symbol != symbol.trim());
        self.state.mark_dirty();
    }

    /// Return active comparison symbols as JSON.
    #[wasm_bindgen(js_name = getCompareSymbols)]
    pub fn get_compare_symbols(&self) -> String {
        let symbols: Vec<&str> = self
            .compare_symbols
            .iter()
            .map(|entry| entry.symbol.as_str())
            .collect();
        serde_json::to_string(&symbols).unwrap_or_else(|_| "[]".to_string())
    }

    /// Replace footprint candle data.
    #[wasm_bindgen(js_name = setFootprintData)]
    pub fn set_footprint_data(&mut self, candles_json: &str) -> Result<(), JsValue> {
        let candles: Vec<FootprintCandle> = serde_json::from_str(candles_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse footprint data: {}", e)))?;
        self.footprint_candles = candles;
        self.state.mark_dirty();
        Ok(())
    }

    /// Append or replace one footprint candle by timestamp.
    #[wasm_bindgen(js_name = addFootprintCandle)]
    pub fn add_footprint_candle(&mut self, candle_json: &str) -> Result<(), JsValue> {
        let candle: FootprintCandle = serde_json::from_str(candle_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse footprint candle: {}", e)))?;
        if let Some(existing) = self
            .footprint_candles
            .iter_mut()
            .find(|entry| entry.time == candle.time)
        {
            *existing = candle;
        } else {
            self.footprint_candles.push(candle);
            self.footprint_candles.sort_by_key(|entry| entry.time);
        }
        self.state.mark_dirty();
        Ok(())
    }

    /// Enable or disable footprint rendering.
    #[wasm_bindgen(js_name = setFootprintEnabled)]
    pub fn set_footprint_enabled(&mut self, enabled: bool) {
        self.footprint_enabled = enabled;
        if enabled {
            self.state.options.candle_style = crate::primitives::CandleStyle::Footprint;
        } else if self.state.options.candle_style == crate::primitives::CandleStyle::Footprint {
            self.state.options.candle_style = crate::primitives::CandleStyle::Candlestick;
        }
        self.state.mark_dirty();
    }

    fn render_compare_symbols(
        compare_symbols: &[CompareSymbol],
        state: &ChartState,
        renderer: &mut Canvas2DRenderer,
    ) {
        if compare_symbols.is_empty() {
            return;
        }

        let vp = &state.viewport;
        let chart_height = vp.dimensions.height as f64;
        let chart_width = vp.dimensions.width as f64;
        let time_start = vp.time.start;
        let time_end = vp.time.end;

        let mut series = Vec::new();
        let mut percent_min = 0.0_f64;
        let mut percent_max = 0.0_f64;

        for entry in compare_symbols {
            let visible: Vec<&Candle> = entry
                .candles
                .iter()
                .filter(|candle| candle.time >= time_start && candle.time <= time_end)
                .collect();
            if visible.len() < 2 {
                continue;
            }

            let base_close = visible
                .iter()
                .find(|candle| candle.c.is_finite() && candle.c > 0.0)
                .map(|candle| candle.c);
            let Some(base_close) = base_close else {
                continue;
            };

            let values: Vec<(i64, f64)> = visible
                .iter()
                .filter_map(|candle| {
                    if candle.c.is_finite() {
                        Some((candle.time, ((candle.c / base_close) - 1.0) * 100.0))
                    } else {
                        None
                    }
                })
                .collect();

            if values.len() < 2 {
                continue;
            }

            for (_, percent) in &values {
                percent_min = percent_min.min(*percent);
                percent_max = percent_max.max(*percent);
            }

            series.push((entry.symbol.as_str(), entry.color, values));
        }

        if series.is_empty() {
            return;
        }

        let padding = ((percent_max - percent_min).abs() * 0.12).max(1.0);
        percent_min -= padding;
        percent_max += padding;
        let percent_range = (percent_max - percent_min).max(1.0);
        let percent_to_y =
            |percent: f64| chart_height - ((percent - percent_min) / percent_range) * chart_height;

        let axis_color = state.options.text_color.with_alpha(0.65);
        let grid_color = state.options.grid_color.with_alpha(0.45);

        let axis_x = chart_width - 1.0;
        renderer.draw_line(axis_x, 0.0, axis_x, chart_height, axis_color, 1.0);

        for i in 0..=4 {
            let percent = percent_min + (percent_range * i as f64 / 4.0);
            let y = percent_to_y(percent);
            renderer.draw_horizontal_line(y, chart_width - 42.0, chart_width, &grid_color, 1.0);
            renderer.draw_text(
                &format!("{:+.1}%", percent),
                chart_width - 4.0,
                y,
                axis_color,
                10.0,
                crate::rendering::TextAlign::Right,
                crate::rendering::TextBaseline::Middle,
            );
        }

        for (idx, (symbol, color, values)) in series.iter().enumerate() {
            let points: Vec<(f64, f64)> = values
                .iter()
                .map(|(time, percent)| (vp.time_to_x(*time), percent_to_y(*percent)))
                .collect();
            renderer.draw_polyline(&points, *color, 1.8);

            if let Some((_, latest_percent)) = values.last() {
                let legend_y = 8.0 + idx as f64 * 15.0;
                renderer.draw_text(
                    &format!("{} {:+.2}%", symbol, latest_percent),
                    8.0,
                    legend_y,
                    *color,
                    11.0,
                    crate::rendering::TextAlign::Left,
                    crate::rendering::TextBaseline::Top,
                );
            }
        }
    }

    fn render_footprint_candles(
        footprint_candles: &[FootprintCandle],
        state: &ChartState,
        renderer: &mut Canvas2DRenderer,
        candle_width: f64,
    ) {
        if footprint_candles.is_empty() || candle_width < 20.0 {
            return;
        }

        let vp = &state.viewport;
        let visible: Vec<&FootprintCandle> = footprint_candles
            .iter()
            .filter(|candle| candle.time >= vp.time.start && candle.time <= vp.time.end)
            .collect();
        if visible.is_empty() {
            return;
        }

        let price_per_pixel =
            ((vp.price.max - vp.price.min) / vp.dimensions.height as f64).abs().max(0.0000001);
        let bid_color = crate::primitives::Color::rgba(248, 81, 73, 0.72);
        let ask_color = crate::primitives::Color::rgba(63, 185, 80, 0.72);
        let poc_color = crate::primitives::Color::rgba(227, 179, 65, 0.32);
        let text_color = state.options.text_color.with_alpha(0.78);
        let slot_w = candle_width * 0.9;
        let half_slot = slot_w / 2.0;
        let show_delta_text = candle_width >= 42.0;

        for candle in visible {
            if candle.levels.is_empty() {
                continue;
            }

            let cx = vp.time_to_x(candle.time);
            let max_vol = candle.max_level_volume().max(1.0);
            let poc_price = candle.poc_price();
            let row_height = if candle.levels.len() >= 2 {
                let step = (candle.levels[1].price - candle.levels[0].price).abs();
                (step / price_per_pixel).clamp(3.0, 18.0)
            } else {
                8.0
            };

            for level in &candle.levels {
                let y = vp.price_to_y(level.price);
                let y_top = y - row_height / 2.0;

                if poc_price == Some(level.price) {
                    renderer.fill_rect(cx - half_slot, y_top, slot_w, row_height, poc_color);
                }

                if level.bid_volume > 0.0 {
                    let width = (level.bid_volume / max_vol) * half_slot;
                    renderer.fill_rect(
                        cx - width,
                        y_top + 1.0,
                        width,
                        (row_height - 2.0).max(1.0),
                        bid_color,
                    );
                }

                if level.ask_volume > 0.0 {
                    let width = (level.ask_volume / max_vol) * half_slot;
                    renderer.fill_rect(
                        cx,
                        y_top + 1.0,
                        width,
                        (row_height - 2.0).max(1.0),
                        ask_color,
                    );
                }

                if show_delta_text {
                    let delta = crate::core::FootprintCandle::level_delta(level);
                    renderer.draw_text(
                        &format!("{:+.0}", delta),
                        cx,
                        y,
                        text_color,
                        8.0,
                        crate::rendering::TextAlign::Center,
                        crate::rendering::TextBaseline::Middle,
                    );
                }
            }
        }
    }

    fn render_selected_drawing_highlights(state: &ChartState, renderer: &mut Canvas2DRenderer) {
        if state.selected_drawings.is_empty() {
            return;
        }

        let highlight = crate::primitives::Color::rgba(88, 166, 255, 0.95);
        let fill = crate::primitives::Color::rgba(88, 166, 255, 0.22);

        for tool in state.tool_manager.tools() {
            if !state
                .selected_drawings
                .iter()
                .any(|id| id == tool.id())
            {
                continue;
            }

            let mut min_x = f64::INFINITY;
            let mut max_x = f64::NEG_INFINITY;
            let mut min_y = f64::INFINITY;
            let mut max_y = f64::NEG_INFINITY;

            for node in tool.nodes() {
                let x = state.viewport.time_to_x(node.time);
                let y = state.viewport.price_to_y(node.price);
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                renderer.fill_rect(x - 3.0, y - 3.0, 6.0, 6.0, fill);
                renderer.stroke_rect(x - 3.0, y - 3.0, 6.0, 6.0, highlight, 1.0);
            }

            if min_x.is_finite() {
                let pad = 5.0;
                renderer.stroke_rect(
                    min_x - pad,
                    min_y - pad,
                    (max_x - min_x).max(1.0) + pad * 2.0,
                    (max_y - min_y).max(1.0) + pad * 2.0,
                    highlight,
                    1.0,
                );
            }
        }
    }

    fn render_indicator_panes(
        indicator_panes: &[IndicatorPane],
        state: &ChartState,
        renderer: &mut Canvas2DRenderer,
    ) {
        if indicator_panes.is_empty() || state.candles.len() < 3 {
            return;
        }

        let vp = &state.viewport;
        let width = vp.dimensions.width as f64;
        let height = vp.dimensions.height as f64;
        let indicator_total: f64 = indicator_panes.iter().map(|pane| pane.height_fraction).sum();
        let mut top = height * (1.0 - indicator_total).max(0.38);
        let bg = state.options.background_color.with_alpha(0.96);
        let border = state.options.grid_color.with_alpha(0.8);
        let text = state.options.text_color.with_alpha(0.82);

        for pane in indicator_panes {
            let pane_h = (height * pane.height_fraction).max(56.0);
            renderer.fill_rect(0.0, top, width, pane_h, bg);
            renderer.draw_line(0.0, top, width, top, border, 1.0);

            let series = Self::oscillator_series(&pane.indicator_id, &state.candles);
            let visible: Vec<(i64, f64)> = series
                .into_iter()
                .filter(|(time, value)| {
                    *time >= vp.time.start && *time <= vp.time.end && value.is_finite()
                })
                .collect();

            if visible.len() >= 2 {
                let mut min_v = visible.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
                let mut max_v = visible
                    .iter()
                    .map(|(_, v)| *v)
                    .fold(f64::NEG_INFINITY, f64::max);
                if matches!(pane.indicator_id.as_str(), "rsi" | "stochastic") {
                    min_v = 0.0;
                    max_v = 100.0;
                } else if pane.indicator_id == "williams_r" {
                    min_v = -100.0;
                    max_v = 0.0;
                }
                let pad = ((max_v - min_v).abs() * 0.1).max(1.0);
                min_v -= pad;
                max_v += pad;
                let span = (max_v - min_v).max(1.0);
                let value_to_y = |value: f64| top + pane_h - ((value - min_v) / span) * pane_h;
                let points: Vec<(f64, f64)> = visible
                    .iter()
                    .map(|(time, value)| (vp.time_to_x(*time), value_to_y(*value)))
                    .collect();
                renderer.draw_polyline(&points, crate::primitives::Color::rgba(88, 166, 255, 0.95), 1.6);

                for i in 0..=2 {
                    let value = min_v + span * i as f64 / 2.0;
                    let y = value_to_y(value);
                    renderer.draw_line(0.0, y, width, y, state.options.grid_color, 1.0);
                    renderer.draw_text(
                        &format!("{:.1}", value),
                        width - 4.0,
                        y,
                        text,
                        9.0,
                        crate::rendering::TextAlign::Right,
                        crate::rendering::TextBaseline::Middle,
                    );
                }
            }

            renderer.draw_text(
                &pane.indicator_id.to_uppercase(),
                8.0,
                top + 6.0,
                text,
                10.0,
                crate::rendering::TextAlign::Left,
                crate::rendering::TextBaseline::Top,
            );
            top += pane_h;
        }
    }

    fn oscillator_series(indicator_id: &str, candles: &[Candle]) -> Vec<(i64, f64)> {
        let closes: Vec<f64> = candles.iter().map(|c| c.c).collect();
        let highs: Vec<f64> = candles.iter().map(|c| c.h).collect();
        let lows: Vec<f64> = candles.iter().map(|c| c.l).collect();
        let values = match indicator_id {
            "macd" => {
                let (macd, _signal, histogram) = crate::ta::momentum::macd(&closes, 12, 26, 9);
                histogram
                    .into_iter()
                    .zip(macd)
                    .map(|(hist, macd)| hist.or(macd))
                    .collect()
            }
            "stochastic" => {
                let (k, _d) = crate::ta::momentum::stochastic(&highs, &lows, &closes, 14, 3, 3);
                k
            }
            "cci" => crate::ta::momentum::cci(&highs, &lows, &closes, 20),
            "williams_r" => crate::ta::momentum::williams_r(&highs, &lows, &closes, 14),
            _ => crate::ta::momentum::rsi(&closes, 14),
        };
        candles
            .iter()
            .zip(values)
            .filter_map(|(candle, value)| value.map(|v| (candle.time, v)))
            .collect()
    }

    /// Resize the chart
    #[wasm_bindgen(js_name = resize)]
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        if let Some(renderer) = &mut self.renderer {
            renderer.resize(width, height)?;

            // Get pixel ratio from renderer and update viewport
            let pixel_ratio = renderer.pixel_ratio();
            self.state
                .viewport
                .set_dimensions(width, height, pixel_ratio);
        } else {
            // No renderer, just update state dimensions
            self.state.resize(width, height);
        }

        Ok(())
    }

    /// Handle mouse down event
    #[wasm_bindgen(js_name = onMouseDown)]
    pub fn on_mouse_down(&mut self, x: f64, y: f64, button: u8) {
        let mouse_button = match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            _ => MouseButton::Left,
        };

        let event = MouseEvent::Down {
            x,
            y,
            button: mouse_button,
        };

        self.event_handler
            .handle_mouse_event(event, &mut self.state);
    }

    /// Handle mouse up event
    #[wasm_bindgen(js_name = onMouseUp)]
    pub fn on_mouse_up(&mut self, x: f64, y: f64, button: u8) {
        let mouse_button = match button {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            _ => MouseButton::Left,
        };

        let event = MouseEvent::Up {
            x,
            y,
            button: mouse_button,
        };

        self.event_handler
            .handle_mouse_event(event, &mut self.state);
    }

    /// Handle mouse move event
    #[wasm_bindgen(js_name = onMouseMove)]
    pub fn on_mouse_move(&mut self, x: f64, y: f64) {
        let event = MouseEvent::Move { x, y };
        self.event_handler
            .handle_mouse_event(event, &mut self.state);
    }

    /// Handle mouse wheel event
    #[wasm_bindgen(js_name = onMouseWheel)]
    pub fn on_mouse_wheel(&mut self, x: f64, y: f64, delta_y: f64) {
        let event = MouseEvent::Wheel { x, y, delta_y };
        self.event_handler
            .handle_mouse_event(event, &mut self.state);
    }

    /// Handle mouse leave event
    #[wasm_bindgen(js_name = onMouseLeave)]
    pub fn on_mouse_leave(&mut self) {
        let event = MouseEvent::Leave;
        self.event_handler
            .handle_mouse_event(event, &mut self.state);
    }

    /// Handle double click event
    #[wasm_bindgen(js_name = onDoubleClick)]
    pub fn on_double_click(&mut self, x: f64, y: f64) {
        let event = MouseEvent::DoubleClick { x, y };
        self.event_handler
            .handle_mouse_event(event, &mut self.state);
    }

    /// Handle touch start
    #[wasm_bindgen(js_name = onTouchStart)]
    pub fn on_touch_start(&mut self, x: f64, y: f64) {
        let event = TouchEvent::Start { x, y };
        self.event_handler
            .handle_touch_event(event, &mut self.state);
    }

    /// Handle touch move
    #[wasm_bindgen(js_name = onTouchMove)]
    pub fn on_touch_move(&mut self, x: f64, y: f64) {
        let event = TouchEvent::Move { x, y };
        self.event_handler
            .handle_touch_event(event, &mut self.state);
    }

    /// Handle touch end
    #[wasm_bindgen(js_name = onTouchEnd)]
    pub fn on_touch_end(&mut self, x: f64, y: f64) {
        let event = TouchEvent::End { x, y };
        self.event_handler
            .handle_touch_event(event, &mut self.state);
    }

    /// Handle keyboard event
    #[wasm_bindgen(js_name = onKeyDown)]
    pub fn on_key_down(&mut self, key: String) {
        let event = KeyboardEvent::KeyDown { key };
        self.event_handler
            .handle_keyboard_event(event, &mut self.state);
    }

    /// Fit viewport to data
    #[wasm_bindgen(js_name = fitToData)]
    pub fn fit_to_data(&mut self) {
        self.state.fit_to_data();
    }

    /// Set candle rendering style
    #[wasm_bindgen(js_name = setCandleStyle)]
    pub fn set_candle_style(&mut self, style: &str) -> Result<(), JsValue> {
        let candle_style = match style {
            "candlestick" => crate::primitives::CandleStyle::Candlestick,
            "ohlc" => crate::primitives::CandleStyle::OHLC,
            "hollow" => crate::primitives::CandleStyle::Hollow,
            "line" => crate::primitives::CandleStyle::Line,
            "area" => crate::primitives::CandleStyle::Area,
            "footprint" => crate::primitives::CandleStyle::Footprint,
            s if s.starts_with("renko") => {
                // Accept "renko" (default brick) or "renko:5.0"
                let brick_size = if let Some(rest) = s.strip_prefix("renko:") {
                    rest.parse::<f64>().unwrap_or(10.0)
                } else {
                    10.0
                };
                crate::primitives::CandleStyle::Renko { brick_size }
            }
            _ => {
                return Err(JsValue::from_str(
                    "Invalid candle style. Use: candlestick, ohlc, hollow, line, area, footprint, or renko[:brick_size]",
                ))
            }
        };

        self.state.options.candle_style = candle_style;
        self.footprint_enabled = candle_style == crate::primitives::CandleStyle::Footprint;
        self.state.mark_dirty();
        Ok(())
    }

    /// Toggle logarithmic price scale
    #[wasm_bindgen(js_name = setLogScale)]
    pub fn set_log_scale(&mut self, enabled: bool) {
        self.state.viewport.log_scale = enabled;
        self.state.mark_dirty();
    }

    /// Query current log scale mode
    #[wasm_bindgen(js_name = isLogScale)]
    pub fn is_log_scale(&self) -> bool {
        self.state.viewport.log_scale
    }

    /// Lock or unlock the price axis. When locked, fit_to_data() and reloading
    /// candles will leave the price range unchanged.
    #[wasm_bindgen(js_name = setPriceLocked)]
    pub fn set_price_locked(&mut self, locked: bool) {
        self.state.viewport.price_locked = locked;
        self.state.mark_dirty();
    }

    /// Query current price axis lock state
    #[wasm_bindgen(js_name = isPriceLocked)]
    pub fn is_price_locked(&self) -> bool {
        self.state.viewport.price_locked
    }

    /// Switch between dark and light theme
    #[wasm_bindgen(js_name = setTheme)]
    pub fn set_theme(&mut self, dark: bool) {
        use crate::primitives::Color;
        let opts = &mut self.state.options;
        if dark {
            opts.background_color = Color::rgba(10, 14, 18, 1.0);
            opts.grid_color = Color::rgba(45, 54, 64, 0.3);
            opts.text_color = Color::rgba(231, 233, 234, 1.0);
            opts.crosshair_color = Color::rgba(139, 152, 165, 0.5);
        } else {
            opts.background_color = Color::rgba(255, 255, 255, 1.0);
            opts.grid_color = Color::rgba(180, 180, 180, 0.3);
            opts.text_color = Color::rgba(20, 20, 20, 1.0);
            opts.crosshair_color = Color::rgba(80, 80, 80, 0.5);
        }
        self.state.mark_dirty();
    }

    /// Get crosshair position as JSON
    #[wasm_bindgen(js_name = getCrosshairInfo)]
    pub fn get_crosshair_info(&self) -> JsValue {
        let crosshair = &self.state.crosshair;

        if !crosshair.visible {
            return JsValue::NULL;
        }

        let ohlcv = self.state.get_ohlcv_at_crosshair();

        let info = serde_json::json!({
            "time": crosshair.time,
            "price": crosshair.price,
            "x": crosshair.x,
            "y": crosshair.y,
            "ohlcv": ohlcv.map(|(o, h, l, c, v)| {
                serde_json::json!({
                    "open": o,
                    "high": h,
                    "low": l,
                    "close": c,
                    "volume": v,
                })
            })
        });

        JsValue::from_str(&info.to_string())
    }

    /// Get viewport info as JSON
    #[wasm_bindgen(js_name = getViewportInfo)]
    pub fn get_viewport_info(&self) -> JsValue {
        use crate::core::ViewportScaleMode;
        let vp = &self.state.viewport;

        let scale_mode_str = match vp.scale_mode {
            ViewportScaleMode::Price => "price",
            ViewportScaleMode::Log => "log",
            ViewportScaleMode::Percent => "percent",
            ViewportScaleMode::Indexed => "indexed",
        };

        let info = serde_json::json!({
            "time": {
                "start": vp.time.start,
                "end": vp.time.end,
            },
            "price": {
                "min": vp.price.min,
                "max": vp.price.max,
            },
            "dimensions": {
                "width": vp.dimensions.width,
                "height": vp.dimensions.height,
                "pixelRatio": vp.dimensions.pixel_ratio,
            },
            "visibleBars": vp.visible_bars(),
            "barWidth": vp.bar_width(),
            "timezoneOffsetMinutes": vp.timezone_offset_minutes,
            "scaleMode": scale_mode_str,
            "scaleBasePrice": vp.scale_base_price,
        });

        JsValue::from_str(&info.to_string())
    }

    /// Get candle at position (with hit-testing)
    #[wasm_bindgen(js_name = getCandleAtPosition)]
    pub fn get_candle_at_position(&self, x: f64, y: f64) -> JsValue {
        match self.state.candle_at_position(x, y) {
            Some(candle) => {
                let info = serde_json::json!({
                    "time": candle.time,
                    "open": candle.o,
                    "high": candle.h,
                    "low": candle.l,
                    "close": candle.c,
                    "volume": candle.v,
                    "ohlc": candle.format_ohlc(),
                });
                JsValue::from_str(&info.to_string())
            }
            None => JsValue::NULL,
        }
    }

    /// Get OHLC formatted string at crosshair
    #[wasm_bindgen(js_name = getOHLCFormatted)]
    pub fn get_ohlc_formatted(&self) -> JsValue {
        match self.state.get_ohlc_formatted() {
            Some(formatted) => JsValue::from_str(&formatted),
            None => JsValue::NULL,
        }
    }

    /// Export chart state to JSON
    #[wasm_bindgen(js_name = exportState)]
    pub fn export_state(&self) -> Result<String, JsValue> {
        self.state
            .export()
            .map_err(|e| JsValue::from_str(&format!("Export error: {}", e)))
    }

    /// Import chart state from JSON
    #[wasm_bindgen(js_name = importState)]
    pub fn import_state(&mut self, json: &str) -> Result<(), JsValue> {
        self.state
            .import(json)
            .map_err(|e| JsValue::from_str(&format!("Import error: {}", e)))?;
        self.state.mark_dirty();
        Ok(())
    }

    /// Render the chart
    #[wasm_bindgen(js_name = render)]
    pub fn render(&mut self) -> Result<(), JsValue> {
        if !self.state.is_dirty() {
            return Ok(());
        }

        web_sys::console::log_1(
            &format!(
                "[ChartCore] Rendering {} candles (viewport: {}-{}, price: {:.2}-{:.2})",
                self.state.candles.len(),
                self.state.viewport.time.start,
                self.state.viewport.time.end,
                self.state.viewport.price.min,
                self.state.viewport.price.max
            )
            .into(),
        );

        let renderer = self
            .renderer
            .as_mut()
            .ok_or_else(|| JsValue::from_str("No renderer attached"))?;

        renderer.begin_frame();

        // Clear background
        let bg_color = self.state.options.background_color;
        renderer.clear(bg_color);

        // Draw grid
        if self.state.options.show_grid {
            let grid_color = self.state.options.grid_color;
            let vp = &self.state.viewport;

            // Draw horizontal grid lines — log-spaced when log scale is active
            let num_lines = 10;
            let grid_prices: Vec<f64> = if vp.log_scale {
                vp.log_grid_prices(num_lines)
            } else {
                let price_step = (vp.price.max - vp.price.min) / num_lines as f64;
                (0..=num_lines)
                    .map(|i| vp.price.min + i as f64 * price_step)
                    .collect()
            };

            for price in &grid_prices {
                let y = vp.price_to_y(*price);
                renderer.draw_line(0.0, y, vp.dimensions.width as f64, y, grid_color, 1.0);
            }

            // Draw vertical grid lines (time levels)
            let bar_width = vp.bar_width();
            let step = (vp.dimensions.width as f64 / 10.0).max(bar_width * 5.0);
            let mut x = 0.0;
            while x < vp.dimensions.width as f64 {
                renderer.draw_line(x, 0.0, x, vp.dimensions.height as f64, grid_color, 1.0);
                x += step;
            }
        }

        // Draw candles with TradingView-style optimal width calculation
        {
            use crate::utils::bar_width::{optimal_candlestick_width, symmetric_bar_width};

            let vp = &self.state.viewport;
            let visible_candles = self.state.visible_candles();
            let bar_spacing = vp.bar_width();
            let pixel_ratio = vp.dimensions.pixel_ratio;
            let bullish_color = self.state.options.bullish_color;
            let bearish_color = self.state.options.bearish_color;
            let unchanged_color = self.state.options.unchanged_color;

            // Calculate optimal candlestick width using TradingView algorithm
            let optimal_width = if vp.bar_width_ratio > 0.0 {
                bar_spacing * pixel_ratio * vp.bar_width_ratio
            } else {
                optimal_candlestick_width(bar_spacing, pixel_ratio)
            };
            let (bar_width, _line_width) = symmetric_bar_width(optimal_width, pixel_ratio, false);

            web_sys::console::log_1(
                &format!(
                    "[ChartCore] Drawing {} visible candles (spacing: {:.2}, optimal_width: {:.2}, bar_width: {:.2}, dpr: {:.2})",
                    visible_candles.len(),
                    bar_spacing,
                    optimal_width,
                    bar_width,
                    pixel_ratio
                )
                .into(),
            );

            let candle_style = self.state.options.candle_style;

            // For Renko: transform candles first, then render as candlestick bricks
            let renko_bricks: Vec<Candle>;
            let owned_visible: Vec<Candle>;
            let render_candles: &[Candle] = if let crate::primitives::CandleStyle::Renko { brick_size } = candle_style {
                renko_bricks = crate::core::renko::compute_renko(&self.state.candles, brick_size);
                &renko_bricks
            } else {
                owned_visible = visible_candles.iter().map(|c| (*c).clone()).collect();
                &owned_visible
            };

            match candle_style {
                crate::primitives::CandleStyle::Line | crate::primitives::CandleStyle::Area => {
                    // Render as polyline / filled area through close prices
                    let points: Vec<(f64, f64)> = render_candles
                        .iter()
                        .map(|c| (vp.time_to_x(c.time), vp.price_to_y(c.c)))
                        .collect();

                    let line_color = bullish_color;
                    if candle_style == crate::primitives::CandleStyle::Area {
                        // Build a semi-transparent fill from the line color
                        let fill = crate::primitives::Color {
                            r: line_color.r,
                            g: line_color.g,
                            b: line_color.b,
                            a: 0.15,
                        };
                        let baseline_y = vp.dimensions.height as f64;
                        renderer.draw_area(&points, baseline_y, fill, line_color, 2.0);
                    } else {
                        renderer.draw_polyline(&points, line_color, 2.0);
                    }
                }
                crate::primitives::CandleStyle::Renko { brick_size: _ } => {
                    // Renko bricks rendered as filled rectangles
                    for candle in render_candles {
                        if candle.time < vp.time.start || candle.time > vp.time.end {
                            continue;
                        }
                        let x = vp.time_to_x(candle.time);
                        let top_y = vp.price_to_y(candle.h);
                        let bot_y = vp.price_to_y(candle.l);
                        let height = (bot_y - top_y).abs().max(1.0);
                        let w = bar_width / pixel_ratio;
                        let color = if candle.c >= candle.o { bullish_color } else { bearish_color };
                        renderer.fill_rect(x - w / 2.0, top_y, w, height, color);
                        renderer.stroke_rect(x - w / 2.0, top_y, w, height, unchanged_color, 0.5);
                    }
                }
                crate::primitives::CandleStyle::Footprint => {
                    Self::render_footprint_candles(
                        &self.footprint_candles,
                        &self.state,
                        renderer,
                        bar_width / pixel_ratio,
                    );
                }
                _ => {
                    for candle in render_candles {
                        let x = vp.time_to_x(candle.time);
                        let open_y = vp.price_to_y(candle.o);
                        let high_y = vp.price_to_y(candle.h);
                        let low_y = vp.price_to_y(candle.l);
                        let close_y = vp.price_to_y(candle.c);
                        let width = bar_width / pixel_ratio; // Convert back to CSS pixels

                        match candle_style {
                            crate::primitives::CandleStyle::Candlestick => {
                                renderer.draw_candle(
                                    x, open_y, high_y, low_y, close_y, width,
                                    bullish_color, bearish_color, unchanged_color,
                                );
                            }
                            crate::primitives::CandleStyle::OHLC => {
                                renderer.draw_ohlc(
                                    x, open_y, high_y, low_y, close_y, width,
                                    bullish_color, bearish_color, unchanged_color,
                                );
                            }
                            crate::primitives::CandleStyle::Hollow => {
                                renderer.draw_hollow_candle(
                                    x, open_y, high_y, low_y, close_y, width,
                                    bullish_color, bearish_color, unchanged_color,
                                );
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
        }

        // Draw session markers (rendered behind candles — drawn before tools)
        if self.state.options.show_sessions && !self.state.options.sessions.is_empty() {
            let vp = &self.state.viewport;
            let tf_secs = self.state.timeframe.duration_ms() as i64 / 1000;
            // Only show session markers for timeframes <= 1h (3600s)
            if tf_secs <= 3600 {
                let chart_width = vp.dimensions.width as f64;
                let chart_height = vp.dimensions.height as f64;
                let time_start = vp.time.start;
                let time_end = vp.time.end;

                // Iterate over each day in the visible range (±1 day buffer)
                let day_secs: i64 = 86400;
                let first_day = ((time_start - day_secs) / day_secs) * day_secs;
                let last_day = ((time_end + day_secs) / day_secs) * day_secs;

                for session in &self.state.options.sessions.clone() {
                    let line_color = crate::primitives::Color::rgba(
                        session.color.0,
                        session.color.1,
                        session.color.2,
                        session.color.3 as f32 / 255.0,
                    );

                    let mut day = first_day;
                    while day <= last_day {
                        if session.show_open {
                            let open_ts = day
                                + session.open_utc.0 as i64 * 3600
                                + session.open_utc.1 as i64 * 60;
                            if open_ts >= time_start && open_ts <= time_end {
                                let x = vp.time_to_x(open_ts);
                                if x >= 0.0 && x <= chart_width {
                                    renderer.draw_line(x, 0.0, x, chart_height, line_color, 1.0);
                                }
                            }
                        }
                        if session.show_close {
                            let close_ts = day
                                + session.close_utc.0 as i64 * 3600
                                + session.close_utc.1 as i64 * 60;
                            if close_ts >= time_start && close_ts <= time_end {
                                let x = vp.time_to_x(close_ts);
                                if x >= 0.0 && x <= chart_width {
                                    let dashed_color = crate::primitives::Color::rgba(
                                        session.color.0,
                                        session.color.1,
                                        session.color.2,
                                        (session.color.3 as f32 / 255.0) * 0.5,
                                    );
                                    renderer.draw_line(x, 0.0, x, chart_height, dashed_color, 1.0);
                                }
                            }
                        }
                        day += day_secs;
                    }
                }
            }
        }

        // Draw comparison symbols as percent-performance overlays with their
        // own right-side scale.
        Self::render_compare_symbols(&self.compare_symbols, &self.state, renderer);
        Self::render_indicator_panes(&self.indicator_panes, &self.state, renderer);

        // Draw drawing tools
        {
            let vp = &self.state.viewport;
            let tools = self.state.tool_manager.tools();

            for tool in tools {
                tool.render(renderer, vp);
            }
        }

        Self::render_selected_drawing_highlights(&self.state, renderer);

        // Draw crosshair with pixel-perfect rendering
        if self.state.options.show_crosshair && self.state.crosshair.visible {
            let crosshair = &self.state.crosshair;
            let color = self.state.options.crosshair_color;
            let vp = &self.state.viewport;

            // Set line cap to butt for crisp crosshair lines (TradingView style)
            renderer.ctx.set_line_cap("butt");

            // Vertical line - use optimized method
            renderer.draw_vertical_line(crosshair.x, 0.0, vp.dimensions.height as f64, &color, 1.0);

            // Horizontal line - use optimized method
            renderer.draw_horizontal_line(
                crosshair.y,
                0.0,
                vp.dimensions.width as f64,
                &color,
                1.0,
            );
        }

        // Note: Axes are now rendered separately in JavaScript/TypeScript
        // using getViewportInfo() to get the current time/price ranges

        renderer.end_frame();

        self.state.clear_dirty();

        Ok(())
    }

    /// Check if chart needs redraw
    #[wasm_bindgen(js_name = isDirty)]
    pub fn is_dirty(&self) -> bool {
        self.state.is_dirty()
    }

    // ========== Undo/Redo API ==========

    fn _snapshot_tools(&self) -> String {
        self.state.tool_manager.to_json().unwrap_or_default()
    }

    fn _push_undo(&mut self) {
        let snap = self._snapshot_tools();
        self.undo_stack.push(snap);
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Undo the last drawing action. Returns true if there was something to undo.
    #[wasm_bindgen(js_name = undo)]
    pub fn undo(&mut self) -> bool {
        let Some(snap) = self.undo_stack.pop() else {
            return false;
        };
        let current = self._snapshot_tools();
        self.redo_stack.push(current);
        if self.redo_stack.len() > 50 {
            self.redo_stack.remove(0);
        }
        if let Ok(manager) = crate::tools::ToolManager::from_json(&snap) {
            self.state.tool_manager = manager;
            self.state.mark_dirty();
        }
        true
    }

    /// Redo the last undone drawing action. Returns true if there was something to redo.
    #[wasm_bindgen(js_name = redo)]
    pub fn redo(&mut self) -> bool {
        let Some(snap) = self.redo_stack.pop() else {
            return false;
        };
        let current = self._snapshot_tools();
        self.undo_stack.push(current);
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        if let Ok(manager) = crate::tools::ToolManager::from_json(&snap) {
            self.state.tool_manager = manager;
            self.state.mark_dirty();
        }
        true
    }

    // ========== Drawing Tools API ==========

    /// Create a new trend line tool
    #[wasm_bindgen(js_name = createTrendLine)]
    pub fn create_trend_line(
        &mut self,
        id: &str,
        start_time: i64,
        start_price: f64,
        end_time: i64,
        end_price: f64,
    ) -> Result<(), JsValue> {
        use crate::tools::{ToolNode, TrendLine};

        self._push_undo();

        let start_node = ToolNode {
            time: start_time,
            price: start_price,
        };

        let end_node = ToolNode {
            time: end_time,
            price: end_price,
        };

        let tool = TrendLine::with_nodes(id.to_string(), start_node, end_node);
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();

        Ok(())
    }

    /// Create a new horizontal line tool
    #[wasm_bindgen(js_name = createHorizontalLine)]
    pub fn create_horizontal_line(&mut self, id: &str, price: f64) -> Result<(), JsValue> {
        use crate::tools::HorizontalLine;

        self._push_undo();
        let tool = HorizontalLine::with_price(id.to_string(), 0, price);
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();

        Ok(())
    }

    /// Create a new vertical line tool
    #[wasm_bindgen(js_name = createVerticalLine)]
    pub fn create_vertical_line(&mut self, id: &str, time: i64) -> Result<(), JsValue> {
        use crate::tools::VerticalLine;

        self._push_undo();
        let tool = VerticalLine::with_time(id.to_string(), time, 0.0);
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();

        Ok(())
    }

    /// Remove a tool by ID
    #[wasm_bindgen(js_name = removeTool)]
    pub fn remove_tool(&mut self, id: &str) -> Result<(), JsValue> {
        self._push_undo();
        self.state.tool_manager.remove_tool(id);
        self.state.mark_dirty();
        Ok(())
    }

    /// Clear all tools
    #[wasm_bindgen(js_name = clearTools)]
    pub fn clear_tools(&mut self) -> Result<(), JsValue> {
        self._push_undo();
        self.state.tool_manager.clear();
        self.state.selected_drawings.clear();
        self.state.mark_dirty();
        Ok(())
    }

    /// Create a rectangle drawing tool
    #[wasm_bindgen(js_name = createRectangle)]
    pub fn create_rectangle(
        &mut self,
        id: &str,
        t1: i64,
        p1: f64,
        t2: i64,
        p2: f64,
    ) -> Result<(), JsValue> {
        use crate::tools::{Rectangle, ToolNode};
        self._push_undo();
        let tool = Rectangle::with_corners(
            id.to_string(),
            ToolNode { time: t1, price: p1 },
            ToolNode { time: t2, price: p2 },
        );
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();
        Ok(())
    }

    /// Create a Fibonacci retracement drawing tool
    #[wasm_bindgen(js_name = createFibonacci)]
    pub fn create_fibonacci(
        &mut self,
        id: &str,
        t1: i64,
        p1: f64,
        t2: i64,
        p2: f64,
    ) -> Result<(), JsValue> {
        use crate::tools::{FibonacciRetracement, ToolNode};
        self._push_undo();
        let tool = FibonacciRetracement::with_points(
            id.to_string(),
            ToolNode { time: t1, price: p1 },
            ToolNode { time: t2, price: p2 },
        );
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();
        Ok(())
    }

    /// Create a text label drawing tool
    #[wasm_bindgen(js_name = createTextLabel)]
    pub fn create_text_label(
        &mut self,
        id: &str,
        time: i64,
        price: f64,
        text: &str,
    ) -> Result<(), JsValue> {
        use crate::tools::{TextLabel, ToolNode};
        self._push_undo();
        let tool = TextLabel::new(
            id.to_string(),
            ToolNode { time, price },
            text,
        );
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();
        Ok(())
    }

    // ========== Bar Spacing API ==========

    /// Set additional bar spacing in CSS pixels (positive = wider bars, negative = narrower)
    #[wasm_bindgen(js_name = setBarSpacing)]
    pub fn set_bar_spacing(&mut self, extra_px: f64) {
        self.state.viewport.bar_spacing_extra = extra_px.clamp(-40.0, 200.0);
        self.state.mark_dirty();
    }

    /// Set bar width ratio (0.0 = auto, 0.1–0.95 = explicit fraction of slot)
    #[wasm_bindgen(js_name = setBarWidthRatio)]
    pub fn set_bar_width_ratio(&mut self, ratio: f64) {
        self.state.viewport.bar_width_ratio = ratio.clamp(0.0, 0.95);
        self.state.mark_dirty();
    }

    /// Get current bar spacing extra value
    #[wasm_bindgen(js_name = getBarSpacing)]
    pub fn get_bar_spacing(&self) -> f64 {
        self.state.viewport.bar_spacing_extra
    }

    /// Get all tools as JSON
    #[wasm_bindgen(js_name = getTools)]
    pub fn get_tools(&self) -> String {
        // Return array of tool JSON strings
        let tools: Vec<String> = self
            .state
            .tool_manager
            .tools()
            .iter()
            .filter_map(|tool| tool.to_json().ok())
            .collect();

        format!("[{}]", tools.join(","))
    }

    /// Select drawing at canvas position. If additive is true, toggles membership.
    #[wasm_bindgen(js_name = selectDrawingAt)]
    pub fn select_drawing_at(&mut self, x: f64, y: f64, additive: bool) -> bool {
        let hit_id = self
            .state
            .tool_manager
            .hit_test(x, y, &self.state.viewport)
            .map(|id| id.to_string());

        let Some(id) = hit_id else {
            if !additive {
                self.state.selected_drawings.clear();
                self.state.mark_dirty();
            }
            return false;
        };

        if additive {
            if let Some(pos) = self
                .state
                .selected_drawings
                .iter()
                .position(|selected| selected == &id)
            {
                self.state.selected_drawings.remove(pos);
            } else {
                self.state.selected_drawings.push(id);
            }
        } else {
            self.state.selected_drawings = vec![id];
        }
        self.state.mark_dirty();
        true
    }

    /// Select all drawings whose nodes are fully inside a screen-space rectangle.
    #[wasm_bindgen(js_name = selectDrawingsInRect)]
    pub fn select_drawings_in_rect(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        additive: bool,
    ) -> String {
        let left = x1.min(x2);
        let right = x1.max(x2);
        let top = y1.min(y2);
        let bottom = y1.max(y2);

        let mut selected = if additive {
            self.state.selected_drawings.clone()
        } else {
            Vec::new()
        };

        for tool in self.state.tool_manager.tools() {
            if !tool.is_complete() || tool.nodes().is_empty() {
                continue;
            }
            let inside = tool.nodes().iter().all(|node| {
                let x = self.state.viewport.time_to_x(node.time);
                let y = self.state.viewport.price_to_y(node.price);
                x >= left && x <= right && y >= top && y <= bottom
            });
            if inside && !selected.iter().any(|id| id == tool.id()) {
                selected.push(tool.id().to_string());
            }
        }

        self.state.selected_drawings = selected;
        self.state.mark_dirty();
        self.get_selected_drawings()
    }

    /// Start bulk-dragging selected drawings from a canvas position.
    #[wasm_bindgen(js_name = startSelectedDrawingsDrag)]
    pub fn start_selected_drawings_drag(&mut self, x: f64, y: f64) -> bool {
        if self.state.selected_drawings.is_empty() {
            return false;
        }
        self._push_undo();
        self.drawing_drag_anchor = Some((
            self.state.viewport.x_to_time(x),
            self.state.viewport.y_to_price(y),
        ));
        true
    }

    /// Move all selected drawings to follow the current canvas position.
    #[wasm_bindgen(js_name = dragSelectedDrawingsTo)]
    pub fn drag_selected_drawings_to(&mut self, x: f64, y: f64) {
        let Some((last_time, last_price)) = self.drawing_drag_anchor else {
            return;
        };
        let next_time = self.state.viewport.x_to_time(x);
        let next_price = self.state.viewport.y_to_price(y);
        let dt = next_time - last_time;
        let dp = next_price - last_price;
        self.state
            .tool_manager
            .move_many(&self.state.selected_drawings, dt, dp);
        self.drawing_drag_anchor = Some((next_time, next_price));
        self.state.mark_dirty();
    }

    /// End a bulk drawing drag.
    #[wasm_bindgen(js_name = endSelectedDrawingsDrag)]
    pub fn end_selected_drawings_drag(&mut self) {
        self.drawing_drag_anchor = None;
    }

    /// Delete all selected drawings as one undoable operation.
    #[wasm_bindgen(js_name = deleteSelectedDrawings)]
    pub fn delete_selected_drawings(&mut self) -> usize {
        if self.state.selected_drawings.is_empty() {
            return 0;
        }
        self._push_undo();
        let deleted = self
            .state
            .tool_manager
            .remove_many(&self.state.selected_drawings);
        self.state.selected_drawings.clear();
        self.state.mark_dirty();
        deleted
    }

    /// Return selected drawing IDs as JSON.
    #[wasm_bindgen(js_name = getSelectedDrawings)]
    pub fn get_selected_drawings(&self) -> String {
        serde_json::to_string(&self.state.selected_drawings).unwrap_or_else(|_| "[]".to_string())
    }

    /// Create or replace an indicator pane. Returns the pane ID.
    #[wasm_bindgen(js_name = addIndicatorPane)]
    pub fn add_indicator_pane(&mut self, indicator_id: &str, params_json: &str) -> String {
        let pane_id = format!("pane-{}", indicator_id.trim());
        if let Some(pane) = self
            .indicator_panes
            .iter_mut()
            .find(|pane| pane.indicator_id == indicator_id)
        {
            pane.params_json = params_json.to_string();
            self.state.mark_dirty();
            return pane.pane_id.clone();
        }

        self.indicator_panes.push(IndicatorPane {
            pane_id: pane_id.clone(),
            indicator_id: indicator_id.to_string(),
            params_json: params_json.to_string(),
            height_fraction: 0.28,
        });
        self.normalize_indicator_panes();
        self.state.mark_dirty();
        pane_id
    }

    /// Remove an indicator pane by pane ID.
    #[wasm_bindgen(js_name = removePane)]
    pub fn remove_pane(&mut self, pane_id: &str) -> bool {
        let before = self.indicator_panes.len();
        self.indicator_panes
            .retain(|pane| pane.pane_id != pane_id && pane.indicator_id != pane_id);
        let changed = before != self.indicator_panes.len();
        if changed {
            self.normalize_indicator_panes();
            self.state.mark_dirty();
        }
        changed
    }

    /// Set one pane height fraction, then normalize all panes.
    #[wasm_bindgen(js_name = setPaneHeightFraction)]
    pub fn set_pane_height_fraction(&mut self, pane_id: &str, fraction: f64) {
        if let Some(pane) = self
            .indicator_panes
            .iter_mut()
            .find(|pane| pane.pane_id == pane_id || pane.indicator_id == pane_id)
        {
            pane.height_fraction = fraction.clamp(0.14, 0.5);
            self.normalize_indicator_panes();
            self.state.mark_dirty();
        }
    }

    /// Return pane layout as JSON with main + indicator fractions.
    #[wasm_bindgen(js_name = getPaneLayout)]
    pub fn get_pane_layout(&self) -> String {
        serde_json::to_string(&self.pane_layout_json()).unwrap_or_else(|_| "[]".to_string())
    }

    fn normalize_indicator_panes(&mut self) {
        if self.indicator_panes.is_empty() {
            return;
        }
        let max_indicator_total = 0.62_f64;
        let total: f64 = self
            .indicator_panes
            .iter()
            .map(|pane| pane.height_fraction)
            .sum();
        if total <= max_indicator_total {
            return;
        }
        for pane in &mut self.indicator_panes {
            pane.height_fraction = pane.height_fraction / total * max_indicator_total;
        }
    }

    fn pane_layout_json(&self) -> Vec<serde_json::Value> {
        let indicator_total: f64 = self
            .indicator_panes
            .iter()
            .map(|pane| pane.height_fraction)
            .sum();
        let mut panes = vec![serde_json::json!({
            "id": "main",
            "indicatorId": "price",
            "heightFraction": (1.0 - indicator_total).max(0.38),
        })];
        panes.extend(self.indicator_panes.iter().map(|pane| {
            serde_json::json!({
                "id": pane.pane_id,
                "indicatorId": pane.indicator_id,
                "heightFraction": pane.height_fraction,
            })
        }));
        panes
    }

    // ========== Price Scale Interaction API ==========

    /// Start price scaling - user pressed mouse on price axis
    #[wasm_bindgen(js_name = startPriceScale)]
    pub fn start_price_scale(&mut self, y: f64) -> Result<(), JsValue> {
        use crate::core::InteractionState;

        // Capture inverted Y and snapshot price range
        let start_y = self.state.viewport.start_price_scale(y);
        let initial_price_range = self.state.viewport.price;

        self.state.interaction = InteractionState::ScalingPrice {
            start_y,
            initial_price_range,
        };

        Ok(())
    }

    /// Apply price scaling - user is dragging on price axis
    #[wasm_bindgen(js_name = scalePriceTo)]
    pub fn scale_price_to(&mut self, y: f64) -> Result<(), JsValue> {
        use crate::core::InteractionState;

        // Only apply if we're in scaling mode
        if let InteractionState::ScalingPrice {
            start_y,
            ref initial_price_range,
        } = self.state.interaction
        {
            self.state
                .viewport
                .apply_price_scale(start_y, y, initial_price_range);
            self.state.mark_dirty();
        }

        Ok(())
    }

    /// End price scaling - user released mouse
    #[wasm_bindgen(js_name = endPriceScale)]
    pub fn end_price_scale(&mut self) -> Result<(), JsValue> {
        use crate::core::InteractionState;

        self.state.interaction = InteractionState::Idle;
        Ok(())
    }

    /// Reset price scale to auto-fit data (double-click)
    #[wasm_bindgen(js_name = resetPriceScale)]
    pub fn reset_price_scale(&mut self) -> Result<(), JsValue> {
        // Re-fit to current candle data
        if !self.state.candles.is_empty() {
            let visible_candles = self.state.visible_candles();
            if !visible_candles.is_empty() {
                let mut min_price = f64::MAX;
                let mut max_price = f64::MIN;

                for candle in visible_candles {
                    min_price = min_price.min(candle.l);
                    max_price = max_price.max(candle.h);
                }

                // Add 5% padding
                let range = max_price - min_price;
                let padding = range * 0.05;

                self.state.viewport.price.min = min_price - padding;
                self.state.viewport.price.max = max_price + padding;
                self.state.mark_dirty();
            }
        }

        Ok(())
    }

    /// Start time scaling - user clicked on time axis
    #[wasm_bindgen(js_name = startTimeScale)]
    pub fn start_time_scale(&mut self, x: f64) -> Result<(), JsValue> {
        use crate::core::InteractionState;

        // Capture X and snapshot time range
        let start_x = self.state.viewport.start_time_scale(x);
        let initial_time_range = self.state.viewport.time;

        self.state.interaction = InteractionState::ScalingTime {
            start_x,
            initial_time_range,
        };

        Ok(())
    }

    /// Apply time scaling - user is dragging on time axis
    #[wasm_bindgen(js_name = scaleTimeTo)]
    pub fn scale_time_to(&mut self, x: f64) -> Result<(), JsValue> {
        use crate::core::InteractionState;

        // Only apply if we're in scaling mode
        if let InteractionState::ScalingTime {
            start_x,
            ref initial_time_range,
        } = self.state.interaction
        {
            self.state
                .viewport
                .apply_time_scale(start_x, x, initial_time_range);
            self.state.mark_dirty();
        }

        Ok(())
    }

    /// End time scaling - user released mouse
    #[wasm_bindgen(js_name = endTimeScale)]
    pub fn end_time_scale(&mut self) -> Result<(), JsValue> {
        use crate::core::InteractionState;

        self.state.interaction = InteractionState::Idle;
        Ok(())
    }

    /// Reset time scale to fit all data (double-click)
    #[wasm_bindgen(js_name = resetTimeScale)]
    pub fn reset_time_scale(&mut self) -> Result<(), JsValue> {
        // Re-fit to all candle data
        if !self.state.candles.is_empty() {
            let first_time = self.state.candles[0].time;
            let last_time = self.state.candles[self.state.candles.len() - 1].time;

            // Add 5% padding
            let range = (last_time - first_time) as f64;
            let padding = (range * 0.05) as i64;

            self.state.viewport.time.start = first_time - padding;
            self.state.viewport.time.end = last_time + padding;
            self.state.mark_dirty();
        }

        Ok(())
    }

    // ========== Ellipse Drawing Tool ==========

    /// Create an ellipse drawing tool (bounding box defined by two corner points)
    #[wasm_bindgen(js_name = createEllipse)]
    pub fn create_ellipse(
        &mut self,
        id: &str,
        t1: i64,
        p1: f64,
        t2: i64,
        p2: f64,
    ) -> Result<(), JsValue> {
        use crate::tools::{Ellipse, ToolNode};
        self._push_undo();
        let tool = Ellipse::with_corners(
            id.to_string(),
            ToolNode { time: t1, price: p1 },
            ToolNode { time: t2, price: p2 },
        );
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();
        Ok(())
    }

    // ========== Magnet / Snap Mode ==========

    /// Set the magnet/snap mode for drawing tool placement.
    /// `mode` must be one of: "off", "weak", "strong"
    #[wasm_bindgen(js_name = setMagnetMode)]
    pub fn set_magnet_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        use crate::core::MagnetMode;
        self.state.magnet_mode = match mode {
            "off" => MagnetMode::Off,
            "weak" => MagnetMode::Weak,
            "strong" => MagnetMode::Strong,
            _ => return Err(JsValue::from_str("Invalid magnet mode. Use: off, weak, strong")),
        };
        Ok(())
    }

    /// Get current magnet mode as string
    #[wasm_bindgen(js_name = getMagnetMode)]
    pub fn get_magnet_mode(&self) -> String {
        use crate::core::MagnetMode;
        match self.state.magnet_mode {
            MagnetMode::Off => "off".to_string(),
            MagnetMode::Weak => "weak".to_string(),
            MagnetMode::Strong => "strong".to_string(),
        }
    }

    /// Snap a (time, price) coordinate to the nearest OHLC point when magnet is active.
    /// Returns JSON: `{ time: i64, price: f64, snapped: bool }`
    #[wasm_bindgen(js_name = snapToCandle)]
    pub fn snap_to_candle_wasm(&self, time: i64, price: f64) -> JsValue {
        use crate::core::MagnetMode;
        let threshold_px = 20.0;
        let (snapped_time, snapped_price) = match self.state.magnet_mode {
            MagnetMode::Off => (time, price),
            MagnetMode::Weak | MagnetMode::Strong => {
                self.state.tool_manager.snap_to_candle(
                    time, price, &self.state.candles, threshold_px, &self.state.viewport,
                )
            }
        };
        let snapped = snapped_time != time || snapped_price != price;
        let info = serde_json::json!({
            "time": snapped_time,
            "price": snapped_price,
            "snapped": snapped,
        });
        JsValue::from_str(&info.to_string())
    }

    // ========== Session Markers ==========

    /// Set trading session configurations from JSON array.
    /// Each session: `{ name, open_utc: [h,m], close_utc: [h,m], color: [r,g,b,a], show_open, show_close }`
    /// Pass an empty array `[]` to clear sessions.
    /// Pass `"default"` as the string to load NYSE, London, Tokyo, Sydney presets.
    #[wasm_bindgen(js_name = setSessions)]
    pub fn set_sessions(&mut self, sessions_json: &str) -> Result<(), JsValue> {
        use crate::core::SessionConfig;
        if sessions_json == "default" {
            self.state.options.sessions = vec![
                SessionConfig::nyse(),
                SessionConfig::london(),
                SessionConfig::tokyo(),
                SessionConfig::sydney(),
            ];
        } else {
            let sessions: Vec<SessionConfig> = serde_json::from_str(sessions_json)
                .map_err(|e| JsValue::from_str(&format!("Invalid sessions JSON: {}", e)))?;
            self.state.options.sessions = sessions;
        }
        self.state.mark_dirty();
        Ok(())
    }

    /// Show or hide session marker lines
    #[wasm_bindgen(js_name = setShowSessions)]
    pub fn set_show_sessions(&mut self, show: bool) {
        self.state.options.show_sessions = show;
        self.state.mark_dirty();
    }

    // ========== Timezone ==========

    /// Set timezone offset in minutes from UTC.
    /// Examples: 60 = UTC+1, -300 = UTC-5, 540 = UTC+9, 0 = UTC
    #[wasm_bindgen(js_name = setTimezone)]
    pub fn set_timezone(&mut self, offset_minutes: i32) {
        self.state.viewport.timezone_offset_minutes = offset_minutes;
        self.state.mark_dirty();
    }

    /// Get current timezone offset in minutes
    #[wasm_bindgen(js_name = getTimezoneOffset)]
    pub fn get_timezone_offset(&self) -> i32 {
        self.state.viewport.timezone_offset_minutes
    }

    // ========== Price Scale Mode ==========

    /// Set price scale display mode.
    /// `mode` must be one of: "price", "log", "percent", "indexed"
    #[wasm_bindgen(js_name = setScaleMode)]
    pub fn set_scale_mode(&mut self, mode: &str) -> Result<(), JsValue> {
        use crate::core::ViewportScaleMode;
        self.state.viewport.scale_mode = match mode {
            "price" => {
                self.state.viewport.log_scale = false;
                ViewportScaleMode::Price
            }
            "log" => {
                self.state.viewport.log_scale = true;
                ViewportScaleMode::Log
            }
            "percent" => {
                self.state.viewport.log_scale = false;
                // Set base price from first visible candle
                if let Some(first) = self.state.visible_candles().first() {
                    self.state.viewport.scale_base_price = first.c;
                }
                ViewportScaleMode::Percent
            }
            "indexed" => {
                self.state.viewport.log_scale = false;
                if let Some(first) = self.state.visible_candles().first() {
                    self.state.viewport.scale_base_price = first.c;
                }
                ViewportScaleMode::Indexed
            }
            _ => return Err(JsValue::from_str("Invalid scale mode. Use: price, log, percent, indexed")),
        };
        self.state.mark_dirty();
        Ok(())
    }

    /// Get current scale mode as string
    #[wasm_bindgen(js_name = getScaleMode)]
    pub fn get_scale_mode(&self) -> String {
        use crate::core::ViewportScaleMode;
        match self.state.viewport.scale_mode {
            ViewportScaleMode::Price => "price".to_string(),
            ViewportScaleMode::Log => "log".to_string(),
            ViewportScaleMode::Percent => "percent".to_string(),
            ViewportScaleMode::Indexed => "indexed".to_string(),
        }
    }

    // ========== Renko ==========

    /// Set candle style, including Renko with brick size.
    /// For renko: pass "renko" and provide brick_size > 0.
    #[wasm_bindgen(js_name = setRenkoBrickSize)]
    pub fn set_renko_brick_size(&mut self, brick_size: f64) -> Result<(), JsValue> {
        if brick_size <= 0.0 {
            return Err(JsValue::from_str("brick_size must be > 0"));
        }
        self.state.options.candle_style = crate::primitives::CandleStyle::Renko { brick_size };
        self.state.mark_dirty();
        Ok(())
    }
}

// ===== Scientific Indicators WASM Bindings =====

#[cfg(feature = "wasm")]
use crate::indicators::{
    all_indicators, get_indicator, LempelZivComplexity, PermutationEntropy, ShannonEntropy,
};

/// Get all available indicator metadata as JSON
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = getAllIndicators)]
pub fn get_all_indicators() -> String {
    let indicators = all_indicators();
    serde_json::to_string(&indicators).unwrap_or_else(|_| "[]".to_string())
}

/// Get specific indicator metadata by ID as JSON
#[cfg(feature = "wasm")]
#[wasm_bindgen(js_name = getIndicatorMetadata)]
pub fn get_indicator_metadata(id: &str) -> JsValue {
    match get_indicator(id) {
        Some(metadata) => {
            let json = serde_json::to_string(&metadata).unwrap_or_else(|_| "null".to_string());
            JsValue::from_str(&json)
        }
        None => JsValue::NULL,
    }
}

/// Shannon Entropy indicator (WASM wrapper)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmShannonEntropy {
    indicator: ShannonEntropy,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmShannonEntropy {
    /// Create a new Shannon Entropy indicator
    ///
    /// # Arguments
    /// * `period` - Window size (recommended: 14-50)
    /// * `bins` - Number of histogram bins (recommended: 10-20)
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, bins: usize) -> WasmShannonEntropy {
        WasmShannonEntropy {
            indicator: ShannonEntropy::new(period, bins),
        }
    }

    /// Calculate entropy for next value
    ///
    /// Returns normalized entropy [0, 1] or null if insufficient data
    /// - High (> 0.8): Random market
    /// - Medium (0.4-0.8): Normal market
    /// - Low (< 0.4): Structured market
    #[wasm_bindgen(js_name = next)]
    pub fn next(&mut self, value: f64) -> JsValue {
        match self.indicator.next(value) {
            Some(entropy) => JsValue::from_f64(entropy),
            None => JsValue::NULL,
        }
    }

    /// Calculate Shannon Entropy for array of values
    ///
    /// Returns JSON array of entropy values
    #[wasm_bindgen(js_name = calculate)]
    pub fn calculate(values: &[f64], period: usize, bins: usize) -> String {
        let result =
            crate::indicators::builtin::shannon_entropy::shannon_entropy(values, period, bins);
        serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
    }

    /// Reset the indicator state
    #[wasm_bindgen(js_name = reset)]
    pub fn reset(&mut self) {
        self.indicator.reset();
    }

    /// Get current buffer length
    #[wasm_bindgen(js_name = len)]
    pub fn len(&self) -> usize {
        self.indicator.len()
    }
}

/// Lempel-Ziv Complexity indicator (WASM wrapper)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmLempelZivComplexity {
    indicator: LempelZivComplexity,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmLempelZivComplexity {
    /// Create a new Lempel-Ziv Complexity indicator
    ///
    /// # Arguments
    /// * `period` - Window size (recommended: 50-200)
    /// * `threshold` - Binary conversion threshold (0.0 = auto/median)
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, threshold: f64) -> WasmLempelZivComplexity {
        WasmLempelZivComplexity {
            indicator: LempelZivComplexity::new(period, threshold),
        }
    }

    /// Calculate complexity for next value
    ///
    /// Returns normalized complexity [0, 1] or null if insufficient data
    /// - High (> 0.7): Random, chaotic
    /// - Medium (0.4-0.7): Normal
    /// - Low (< 0.4): Structured, repeating patterns
    #[wasm_bindgen(js_name = next)]
    pub fn next(&mut self, value: f64) -> JsValue {
        match self.indicator.next(value) {
            Some(complexity) => JsValue::from_f64(complexity),
            None => JsValue::NULL,
        }
    }

    /// Calculate Lempel-Ziv Complexity for array of values
    ///
    /// Returns JSON array of complexity values
    #[wasm_bindgen(js_name = calculate)]
    pub fn calculate(values: &[f64], period: usize, threshold: f64) -> String {
        let result = crate::indicators::builtin::lempel_ziv::lempel_ziv_complexity(
            values, period, threshold,
        );
        serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
    }

    /// Reset the indicator state
    #[wasm_bindgen(js_name = reset)]
    pub fn reset(&mut self) {
        self.indicator.reset();
    }

    /// Get current buffer length
    #[wasm_bindgen(js_name = len)]
    pub fn len(&self) -> usize {
        self.indicator.len()
    }
}

/// Permutation Entropy indicator (WASM wrapper)
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmPermutationEntropy {
    indicator: PermutationEntropy,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmPermutationEntropy {
    /// Create a new Permutation Entropy indicator
    ///
    /// # Arguments
    /// * `period` - Window size (recommended: 50-200)
    /// * `embedding_dimension` - Pattern length (recommended: 3-5)
    /// * `delay` - Time delay (recommended: 1)
    #[wasm_bindgen(constructor)]
    pub fn new(period: usize, embedding_dimension: usize, delay: usize) -> WasmPermutationEntropy {
        WasmPermutationEntropy {
            indicator: PermutationEntropy::new(period, embedding_dimension, delay),
        }
    }

    /// Calculate permutation entropy for next value
    ///
    /// Returns normalized entropy [0, 1] or null if insufficient data
    /// - High (> 0.8): Random, unpredictable
    /// - Medium (0.4-0.8): Normal
    /// - Low (< 0.4): Strong ordinal patterns
    #[wasm_bindgen(js_name = next)]
    pub fn next(&mut self, value: f64) -> JsValue {
        match self.indicator.next(value) {
            Some(entropy) => JsValue::from_f64(entropy),
            None => JsValue::NULL,
        }
    }

    /// Calculate Permutation Entropy for array of values
    ///
    /// Returns JSON array of entropy values
    #[wasm_bindgen(js_name = calculate)]
    pub fn calculate(
        values: &[f64],
        period: usize,
        embedding_dimension: usize,
        delay: usize,
    ) -> String {
        let result = crate::indicators::builtin::permutation_entropy::permutation_entropy(
            values,
            period,
            embedding_dimension,
            delay,
        );
        serde_json::to_string(&result).unwrap_or_else(|_| "[]".to_string())
    }

    /// Reset the indicator state
    #[wasm_bindgen(js_name = reset)]
    pub fn reset(&mut self) {
        self.indicator.reset();
    }

    /// Get current buffer length
    #[wasm_bindgen(js_name = len)]
    pub fn len(&self) -> usize {
        self.indicator.len()
    }
}
