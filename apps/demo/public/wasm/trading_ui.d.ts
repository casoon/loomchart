/* tslint:disable */
/* eslint-disable */

export class WasmChart {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Add a single candle
   */
  addCandle(time: bigint, o: number, h: number, l: number, c: number, v: number): void;
  /**
   * Clear all tools
   */
  clearTools(): void;
  /**
   * Fit viewport to data
   */
  fitToData(): void;
  /**
   * Get all candles as JSON (for indicator calculations)
   */
  getCandles(): string;
  /**
   * Handle keyboard event
   */
  onKeyDown(key: string): void;
  /**
   * Handle mouse up event
   */
  onMouseUp(x: number, y: number, button: number): void;
  /**
   * Remove a tool by ID
   */
  removeTool(id: string): void;
  /**
   * Set candle data from JavaScript array
   */
  setCandles(candles_json: string): void;
  /**
   * Export chart state to JSON
   */
  exportState(): string;
  /**
   * Import chart state from JSON
   */
  importState(json: string): void;
  /**
   * Handle touch end
   */
  onTouchEnd(x: number, y: number): void;
  /**
   * Attach a canvas element for rendering
   */
  attachCanvas(canvas: HTMLCanvasElement): void;
  /**
   * Handle mouse down event
   */
  onMouseDown(x: number, y: number, button: number): void;
  /**
   * Handle mouse move event
   */
  onMouseMove(x: number, y: number): void;
  /**
   * Handle touch move
   */
  onTouchMove(x: number, y: number): void;
  /**
   * Apply time scaling - user is dragging on time axis
   */
  scaleTimeTo(x: number): void;
  /**
   * End time scaling - user released mouse
   */
  endTimeScale(): void;
  /**
   * Handle mouse leave event
   */
  onMouseLeave(): void;
  /**
   * Handle mouse wheel event
   */
  onMouseWheel(x: number, y: number, delta_y: number): void;
  /**
   * Handle touch start
   */
  onTouchStart(x: number, y: number): void;
  /**
   * Apply price scaling - user is dragging on price axis
   */
  scalePriceTo(y: number): void;
  /**
   * End price scaling - user released mouse
   */
  endPriceScale(): void;
  /**
   * Handle double click event
   */
  onDoubleClick(x: number, y: number): void;
  /**
   * Reset time scale to fit all data (double-click)
   */
  resetTimeScale(): void;
  /**
   * Set candle rendering style
   */
  setCandleStyle(style: string): void;
  /**
   * Start time scaling - user clicked on time axis
   */
  startTimeScale(x: number): void;
  /**
   * Create a new trend line tool
   */
  createTrendLine(id: string, start_time: bigint, start_price: number, end_time: bigint, end_price: number): void;
  /**
   * Get viewport info as JSON
   */
  getViewportInfo(): any;
  /**
   * Reset price scale to auto-fit data (double-click)
   */
  resetPriceScale(): void;
  /**
   * Start price scaling - user pressed mouse on price axis
   */
  startPriceScale(y: number): void;
  /**
   * Get crosshair position as JSON
   */
  getCrosshairInfo(): any;
  /**
   * Get OHLC formatted string at crosshair
   */
  getOHLCFormatted(): any;
  /**
   * Create a new vertical line tool
   */
  createVerticalLine(id: string, time: bigint): void;
  /**
   * Create a new horizontal line tool
   */
  createHorizontalLine(id: string, price: number): void;
  /**
   * Get candle at position (with hit-testing)
   */
  getCandleAtPosition(x: number, y: number): any;
  /**
   * Create a new chart instance
   */
  constructor(width: number, height: number, timeframe: string);
  /**
   * Render the chart
   */
  render(): void;
  /**
   * Resize the chart
   */
  resize(width: number, height: number): void;
  /**
   * Check if chart needs redraw
   */
  isDirty(): boolean;
  /**
   * Get all tools as JSON
   */
  getTools(): string;
}

export class WasmLempelZivComplexity {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get current buffer length
   */
  len(): number;
  /**
   * Create a new Lempel-Ziv Complexity indicator
   *
   * # Arguments
   * * `period` - Window size (recommended: 50-200)
   * * `threshold` - Binary conversion threshold (0.0 = auto/median)
   */
  constructor(period: number, threshold: number);
  /**
   * Calculate complexity for next value
   *
   * Returns normalized complexity [0, 1] or null if insufficient data
   * - High (> 0.7): Random, chaotic
   * - Medium (0.4-0.7): Normal
   * - Low (< 0.4): Structured, repeating patterns
   */
  next(value: number): any;
  /**
   * Reset the indicator state
   */
  reset(): void;
  /**
   * Calculate Lempel-Ziv Complexity for array of values
   *
   * Returns JSON array of complexity values
   */
  static calculate(values: Float64Array, period: number, threshold: number): string;
}

export class WasmPermutationEntropy {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get current buffer length
   */
  len(): number;
  /**
   * Create a new Permutation Entropy indicator
   *
   * # Arguments
   * * `period` - Window size (recommended: 50-200)
   * * `embedding_dimension` - Pattern length (recommended: 3-5)
   * * `delay` - Time delay (recommended: 1)
   */
  constructor(period: number, embedding_dimension: number, delay: number);
  /**
   * Calculate permutation entropy for next value
   *
   * Returns normalized entropy [0, 1] or null if insufficient data
   * - High (> 0.8): Random, unpredictable
   * - Medium (0.4-0.8): Normal
   * - Low (< 0.4): Strong ordinal patterns
   */
  next(value: number): any;
  /**
   * Reset the indicator state
   */
  reset(): void;
  /**
   * Calculate Permutation Entropy for array of values
   *
   * Returns JSON array of entropy values
   */
  static calculate(values: Float64Array, period: number, embedding_dimension: number, delay: number): string;
}

export class WasmShannonEntropy {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get current buffer length
   */
  len(): number;
  /**
   * Create a new Shannon Entropy indicator
   *
   * # Arguments
   * * `period` - Window size (recommended: 14-50)
   * * `bins` - Number of histogram bins (recommended: 10-20)
   */
  constructor(period: number, bins: number);
  /**
   * Calculate entropy for next value
   *
   * Returns normalized entropy [0, 1] or null if insufficient data
   * - High (> 0.8): Random market
   * - Medium (0.4-0.8): Normal market
   * - Low (< 0.4): Structured market
   */
  next(value: number): any;
  /**
   * Reset the indicator state
   */
  reset(): void;
  /**
   * Calculate Shannon Entropy for array of values
   *
   * Returns JSON array of entropy values
   */
  static calculate(values: Float64Array, period: number, bins: number): string;
}

/**
 * Add an overlay indicator to the main chart (e.g., EMA, BB)
 * Set separate_scale=true for indicators like MFI (0-100) to map to price range
 */
export function add_chart_overlay(indicator_id: string, separate_scale: boolean): void;

/**
 * Add indicator to chart using chartcore's new indicator system
 *
 * # Parameters
 * - `indicator_type`: Indicator ID (e.g., "rsi", "sma", "macd")
 * - `params_json`: JSON string with indicator parameters (e.g., `{"period": 14}`)
 *
 * # Returns
 * - Indicator ID for later reference
 *
 * # Example
 * ```javascript
 * const id = await wasm.add_chartcore_indicator("rsi", '{"period": 14}');
 * ```
 */
export function add_chartcore_indicator(indicator_type: string, params_json: string): string;

/**
 * Add a dedicated indicator panel (e.g., RSI, MACD)
 * Returns panel ID as string
 */
export function add_indicator_panel(indicator_id: string, params_json: string): string;

/**
 * Check and clear last heartbeat reply from window object
 * Returns the ref ID if a heartbeat reply was received
 */
export function check_heartbeat_reply(): string | undefined;

/**
 * Collapse/minimize a panel (Task 5.3)
 */
export function collapse_panel(panel_id: string): void;

/**
 * Connect to data stream with properly wired callbacks
 */
export function connect(source: string, symbol: string, tf: string, ws_url: string): void;

/**
 * Disconnect from data stream (manual disconnect resets reconnection)
 */
export function disconnect(): void;

/**
 * Perform reconnection attempt
 */
export function do_reconnect(): void;

/**
 * Expand/restore a panel (Task 5.3)
 */
export function expand_panel(panel_id: string): void;

/**
 * Get all available indicator metadata as JSON
 */
export function getAllIndicators(): string;

/**
 * Get specific indicator metadata by ID as JSON
 */
export function getIndicatorMetadata(id: string): any;

/**
 * Get list of active indicators with their configurations
 *
 * Returns JSON array of indicator objects
 */
export function get_active_chartcore_indicators(): string;

/**
 * Get all available indicators with metadata
 * Returns JSON array of indicator metadata
 */
export function get_all_indicators(): string;

/**
 * Get list of all available indicator types
 *
 * Returns JSON array with indicator metadata (id, name, category, params)
 */
export function get_available_indicators(): string;

/**
 * Get current connection status
 */
export function get_connection_status(): string;

/**
 * Get metadata for a specific indicator by ID
 * Returns JSON object with indicator metadata, or null if not found
 */
export function get_indicator_metadata(indicator_id: string): any;

/**
 * Get last candle as JSON
 */
export function get_last_candle(): string | undefined;

/**
 * Get panel layout as JSON (for rendering)
 */
export function get_panel_layout(): string;

/**
 * Get reconnection delay (returns 0 if not in reconnecting state)
 * JS should call this, wait the returned ms, then call do_reconnect()
 */
export function get_reconnect_delay(): number;

/**
 * Initialize the WASM module with configuration
 */
export function init(config_json: string): void;

/**
 * Check if a panel is collapsed (Task 5.3)
 */
export function is_panel_collapsed(panel_id: string): boolean;

/**
 * Check if a panel is maximized (Task 5.3)
 */
export function is_panel_maximized(panel_id: string): boolean;

/**
 * Check if currently in reconnecting state
 */
export function is_reconnecting(): boolean;

/**
 * Load test data using chartcore generator
 *
 * # Parameters
 * - market_type: "crypto", "stock", "forex", "futures", "commodities"
 * - trend: "bullish_strong", "bullish_mild", "sideways", "bearish_mild", "bearish_strong"
 * - volatility: "low", "normal", "high", "extreme"
 * - count: number of candles to generate
 */
export function load_test_data(market_type: string, trend: string, volatility: string, count: number): void;

/**
 * Maximize a panel (collapse all others) (Task 5.3)
 */
export function maximize_panel(panel_id: string): void;

/**
 * Move panel to new position
 */
export function move_panel(panel_id: string, new_index: number): void;

/**
 * Remove an overlay from the main chart
 */
export function remove_chart_overlay(indicator_id: string): void;

/**
 * Remove indicator from chart
 */
export function remove_chartcore_indicator(indicator_id: string): void;

/**
 * Remove a panel by ID
 */
export function remove_panel(panel_id: string): void;

/**
 * Reorder panels by swapping two indices (Task 5.2)
 */
export function reorder_panels(from_index: number, to_index: number): void;

/**
 * Resize a panel (user dragged separator)
 */
export function resize_panel(panel_id: string, height: number): void;

/**
 * Restore all panels (expand all collapsed) (Task 5.3)
 */
export function restore_all_panels(): void;

/**
 * Restore panel layout from JSON (workspace persistence)
 */
export function restore_panel_layout(json: string): void;

/**
 * Send heartbeat - should be called periodically by JS (every 30s)
 * Returns true if heartbeat was sent, false if connection is not active
 */
export function send_heartbeat(): boolean;

/**
 * Set total container height (call from window resize)
 */
export function set_panel_container_height(height: number): void;

/**
 * Change symbol (triggers resync)
 */
export function set_symbol(symbol: string): void;

/**
 * Change timeframe (triggers resync)
 */
export function set_timeframe(tf: string): void;

/**
 * Toggle indicator on/off
 */
export function toggle_indicator(name: string, params_json: string, enabled: boolean): void;

/**
 * Update indicator parameters
 *
 * # Parameters
 * - `indicator_id`: ID returned from add_chartcore_indicator
 * - `params_json`: New parameters as JSON
 */
export function update_chartcore_indicator_params(indicator_id: string, params_json: string): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly add_chart_overlay: (a: number, b: number, c: number, d: number) => void;
  readonly add_chartcore_indicator: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly add_indicator_panel: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly check_heartbeat_reply: (a: number) => void;
  readonly collapse_panel: (a: number, b: number, c: number) => void;
  readonly connect: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
  readonly disconnect: (a: number) => void;
  readonly do_reconnect: (a: number) => void;
  readonly expand_panel: (a: number, b: number, c: number) => void;
  readonly get_active_chartcore_indicators: (a: number) => void;
  readonly get_all_indicators: (a: number) => void;
  readonly get_available_indicators: (a: number) => void;
  readonly get_connection_status: (a: number) => void;
  readonly get_indicator_metadata: (a: number, b: number) => number;
  readonly get_last_candle: (a: number) => void;
  readonly get_panel_layout: (a: number) => void;
  readonly get_reconnect_delay: (a: number) => void;
  readonly init: (a: number, b: number, c: number) => void;
  readonly is_panel_collapsed: (a: number, b: number, c: number) => void;
  readonly is_panel_maximized: (a: number, b: number, c: number) => void;
  readonly is_reconnecting: (a: number) => void;
  readonly load_test_data: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly maximize_panel: (a: number, b: number, c: number) => void;
  readonly move_panel: (a: number, b: number, c: number, d: number) => void;
  readonly remove_chart_overlay: (a: number, b: number, c: number) => void;
  readonly remove_chartcore_indicator: (a: number, b: number, c: number) => void;
  readonly remove_panel: (a: number, b: number, c: number) => void;
  readonly reorder_panels: (a: number, b: number, c: number) => void;
  readonly resize_panel: (a: number, b: number, c: number, d: number) => void;
  readonly restore_all_panels: (a: number) => void;
  readonly restore_panel_layout: (a: number, b: number, c: number) => void;
  readonly send_heartbeat: (a: number) => void;
  readonly set_panel_container_height: (a: number, b: number) => void;
  readonly set_symbol: (a: number, b: number, c: number) => void;
  readonly set_timeframe: (a: number, b: number, c: number) => void;
  readonly toggle_indicator: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly update_chartcore_indicator_params: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbg_wasmchart_free: (a: number, b: number) => void;
  readonly __wbg_wasmlempelzivcomplexity_free: (a: number, b: number) => void;
  readonly __wbg_wasmpermutationentropy_free: (a: number, b: number) => void;
  readonly __wbg_wasmshannonentropy_free: (a: number, b: number) => void;
  readonly getAllIndicators: (a: number) => void;
  readonly getIndicatorMetadata: (a: number, b: number) => number;
  readonly wasmchart_addCandle: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wasmchart_attachCanvas: (a: number, b: number, c: number) => void;
  readonly wasmchart_clearTools: (a: number, b: number) => void;
  readonly wasmchart_createHorizontalLine: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wasmchart_createTrendLine: (a: number, b: number, c: number, d: number, e: bigint, f: number, g: bigint, h: number) => void;
  readonly wasmchart_createVerticalLine: (a: number, b: number, c: number, d: number, e: bigint) => void;
  readonly wasmchart_endPriceScale: (a: number, b: number) => void;
  readonly wasmchart_exportState: (a: number, b: number) => void;
  readonly wasmchart_fitToData: (a: number) => void;
  readonly wasmchart_getCandleAtPosition: (a: number, b: number, c: number) => number;
  readonly wasmchart_getCandles: (a: number, b: number) => void;
  readonly wasmchart_getCrosshairInfo: (a: number) => number;
  readonly wasmchart_getOHLCFormatted: (a: number) => number;
  readonly wasmchart_getTools: (a: number, b: number) => void;
  readonly wasmchart_getViewportInfo: (a: number) => number;
  readonly wasmchart_importState: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_isDirty: (a: number) => number;
  readonly wasmchart_new: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wasmchart_onDoubleClick: (a: number, b: number, c: number) => void;
  readonly wasmchart_onKeyDown: (a: number, b: number, c: number) => void;
  readonly wasmchart_onMouseDown: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_onMouseLeave: (a: number) => void;
  readonly wasmchart_onMouseMove: (a: number, b: number, c: number) => void;
  readonly wasmchart_onMouseUp: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_onMouseWheel: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_onTouchEnd: (a: number, b: number, c: number) => void;
  readonly wasmchart_onTouchMove: (a: number, b: number, c: number) => void;
  readonly wasmchart_onTouchStart: (a: number, b: number, c: number) => void;
  readonly wasmchart_removeTool: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_render: (a: number, b: number) => void;
  readonly wasmchart_resetPriceScale: (a: number, b: number) => void;
  readonly wasmchart_resetTimeScale: (a: number, b: number) => void;
  readonly wasmchart_resize: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_scalePriceTo: (a: number, b: number, c: number) => void;
  readonly wasmchart_scaleTimeTo: (a: number, b: number, c: number) => void;
  readonly wasmchart_setCandleStyle: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_setCandles: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_startPriceScale: (a: number, b: number, c: number) => void;
  readonly wasmchart_startTimeScale: (a: number, b: number, c: number) => void;
  readonly wasmlempelzivcomplexity_calculate: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wasmlempelzivcomplexity_len: (a: number) => number;
  readonly wasmlempelzivcomplexity_new: (a: number, b: number) => number;
  readonly wasmlempelzivcomplexity_next: (a: number, b: number) => number;
  readonly wasmlempelzivcomplexity_reset: (a: number) => void;
  readonly wasmpermutationentropy_calculate: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly wasmpermutationentropy_len: (a: number) => number;
  readonly wasmpermutationentropy_new: (a: number, b: number, c: number) => number;
  readonly wasmpermutationentropy_next: (a: number, b: number) => number;
  readonly wasmpermutationentropy_reset: (a: number) => void;
  readonly wasmshannonentropy_calculate: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wasmshannonentropy_len: (a: number) => number;
  readonly wasmshannonentropy_new: (a: number, b: number) => number;
  readonly wasmshannonentropy_next: (a: number, b: number) => number;
  readonly wasmshannonentropy_reset: (a: number) => void;
  readonly wasmchart_endTimeScale: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_356: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_92: (a: number, b: number) => void;
  readonly __wasm_bindgen_func_elem_357: (a: number, b: number, c: number) => void;
  readonly __wbindgen_export: (a: number, b: number) => number;
  readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export3: (a: number) => void;
  readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
