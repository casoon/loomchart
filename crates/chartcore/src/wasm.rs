//! WASM Entry Point - JavaScript API for the chart engine

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use web_sys::HtmlCanvasElement;

#[cfg(feature = "wasm")]
use crate::core::{
    Candle, ChartState, EventHandler, KeyboardEvent, MouseButton, MouseEvent, Timeframe, TouchEvent,
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
            _ => {
                return Err(JsValue::from_str(
                    "Invalid candle style. Use: candlestick, ohlc, or hollow",
                ))
            }
        };

        self.state.options.candle_style = candle_style;
        self.state.mark_dirty();
        Ok(())
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
        let vp = &self.state.viewport;

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

            // Draw horizontal grid lines (price levels)
            let price_range = vp.price.max - vp.price.min;
            let num_lines = 10;
            let price_step = price_range / num_lines as f64;

            for i in 0..=num_lines {
                let price = vp.price.min + (i as f64 * price_step);
                let y = vp.price_to_y(price);
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
            let optimal_width = optimal_candlestick_width(bar_spacing, pixel_ratio);
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

            for candle in visible_candles {
                let x = vp.time_to_x(candle.time);
                let open_y = vp.price_to_y(candle.o);
                let high_y = vp.price_to_y(candle.h);
                let low_y = vp.price_to_y(candle.l);
                let close_y = vp.price_to_y(candle.c);
                let width = bar_width / pixel_ratio; // Convert back to CSS pixels

                match candle_style {
                    crate::primitives::CandleStyle::Candlestick => {
                        renderer.draw_candle(
                            x,
                            open_y,
                            high_y,
                            low_y,
                            close_y,
                            width,
                            bullish_color,
                            bearish_color,
                            unchanged_color,
                        );
                    }
                    crate::primitives::CandleStyle::OHLC => {
                        renderer.draw_ohlc(
                            x,
                            open_y,
                            high_y,
                            low_y,
                            close_y,
                            width,
                            bullish_color,
                            bearish_color,
                            unchanged_color,
                        );
                    }
                    crate::primitives::CandleStyle::Hollow => {
                        renderer.draw_hollow_candle(
                            x,
                            open_y,
                            high_y,
                            low_y,
                            close_y,
                            width,
                            bullish_color,
                            bearish_color,
                            unchanged_color,
                        );
                    }
                }
            }
        }

        // Draw drawing tools
        {
            let vp = &self.state.viewport;
            let tools = self.state.tool_manager.tools();

            for tool in tools {
                tool.render(renderer, vp);
            }
        }

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

        let tool = HorizontalLine::with_price(id.to_string(), 0, price);
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();

        Ok(())
    }

    /// Create a new vertical line tool
    #[wasm_bindgen(js_name = createVerticalLine)]
    pub fn create_vertical_line(&mut self, id: &str, time: i64) -> Result<(), JsValue> {
        use crate::tools::VerticalLine;

        let tool = VerticalLine::with_time(id.to_string(), time, 0.0);
        self.state.tool_manager.add_tool(Box::new(tool));
        self.state.mark_dirty();

        Ok(())
    }

    /// Remove a tool by ID
    #[wasm_bindgen(js_name = removeTool)]
    pub fn remove_tool(&mut self, id: &str) -> Result<(), JsValue> {
        self.state.tool_manager.remove_tool(id);
        self.state.mark_dirty();
        Ok(())
    }

    /// Clear all tools
    #[wasm_bindgen(js_name = clearTools)]
    pub fn clear_tools(&mut self) -> Result<(), JsValue> {
        self.state.tool_manager.clear();
        self.state.mark_dirty();
        Ok(())
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
