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
   * Remove an indicator pane by pane ID.
   */
  removePane(pane_id: string): boolean;
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
   * Query current log scale mode
   */
  isLogScale(): boolean;
  /**
   * Handle touch end
   */
  onTouchEnd(x: number, y: number): void;
  /**
   * Set trading session configurations from JSON array.
   * Each session: `{ name, open_utc: [h,m], close_utc: [h,m], color: [r,g,b,a], show_open, show_close }`
   * Pass an empty array `[]` to clear sessions.
   * Pass `"default"` as the string to load NYSE, London, Tokyo, Sydney presets.
   */
  setSessions(sessions_json: string): void;
  /**
   * Set timezone offset in minutes from UTC.
   * Examples: 60 = UTC+1, -300 = UTC-5, 540 = UTC+9, 0 = UTC
   */
  setTimezone(offset_minutes: number): void;
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
   * Toggle logarithmic price scale
   */
  setLogScale(enabled: boolean): void;
  /**
   * Merge new candles into the existing dataset (delegates to CandleBuffer::append).
   *
   * Existing candles are kept; incoming candles are sorted and deduped.
   * Duplicate timestamps are overwritten by the incoming value.
   */
  appendCandles(candles_json: string): void;
  /**
   * Create an ellipse drawing tool (bounding box defined by two corner points)
   */
  createEllipse(id: string, t1: bigint, p1: number, t2: bigint, p2: number): void;
  /**
   * End time scaling - user released mouse
   */
  endTimeScale(): void;
  /**
   * Get current scale mode as string
   */
  getScaleMode(): string;
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
   * Set price scale display mode.
   * `mode` must be one of: "price", "log", "percent", "indexed"
   */
  setScaleMode(mode: string): void;
  /**
   * End price scaling - user released mouse
   */
  endPriceScale(): void;
  /**
   * Get current bar spacing extra value
   */
  getBarSpacing(): number;
  /**
   * Get current magnet mode as string
   */
  getMagnetMode(): string;
  /**
   * Return pane layout as JSON with main + indicator fractions.
   */
  getPaneLayout(): string;
  /**
   * Query current price axis lock state
   */
  isPriceLocked(): boolean;
  /**
   * Handle double click event
   */
  onDoubleClick(x: number, y: number): void;
  /**
   * Set additional bar spacing in CSS pixels (positive = wider bars, negative = narrower)
   */
  setBarSpacing(extra_px: number): void;
  /**
   * Set the magnet/snap mode for drawing tool placement.
   * `mode` must be one of: "off", "weak", "strong"
   */
  setMagnetMode(mode: string): void;
  /**
   * Create a Fibonacci retracement drawing tool
   */
  createFibonacci(id: string, t1: bigint, p1: number, t2: bigint, p2: number): void;
  /**
   * Create a rectangle drawing tool
   */
  createRectangle(id: string, t1: bigint, p1: number, t2: bigint, p2: number): void;
  /**
   * Reset time scale to fit all data (double-click)
   */
  resetTimeScale(): void;
  /**
   * Set candle rendering style
   */
  setCandleStyle(style: string): void;
  /**
   * Lock or unlock the price axis. When locked, fit_to_data() and reloading
   * candles will leave the price range unchanged.
   */
  setPriceLocked(locked: boolean): void;
  /**
   * Start time scaling - user clicked on time axis
   */
  startTimeScale(x: number): void;
  /**
   * Create a text label drawing tool
   */
  createTextLabel(id: string, time: bigint, price: number, text: string): void;
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
   * Select drawing at canvas position. If additive is true, toggles membership.
   */
  selectDrawingAt(x: number, y: number, additive: boolean): boolean;
  /**
   * Replace all candles (delegates to CandleBuffer::snapshot).
   *
   * Backward-compatible alias for `setCandles`; both methods accept the
   * same JSON format.
   */
  setCandlesBatch(candles_json: string): void;
  /**
   * Show or hide session marker lines
   */
  setShowSessions(show: boolean): void;
  /**
   * Start price scaling - user pressed mouse on price axis
   */
  startPriceScale(y: number): void;
  /**
   * Add or replace a comparison symbol rendered as normalized percent performance.
   */
  addCompareSymbol(symbol: string, candles_json: string, color: string): void;
  /**
   * Create or replace an indicator pane. Returns the pane ID.
   */
  addIndicatorPane(indicator_id: string, params_json: string): string;
  /**
   * Get crosshair position as JSON
   */
  getCrosshairInfo(): any;
  /**
   * Get OHLC formatted string at crosshair
   */
  getOHLCFormatted(): any;
  /**
   * Replace footprint candle data.
   */
  setFootprintData(candles_json: string): void;
  /**
   * Return active comparison symbols as JSON.
   */
  getCompareSymbols(): string;
  /**
   * Get current timezone offset in minutes
   */
  getTimezoneOffset(): number;
  /**
   * Set bar width ratio (0.0 = auto, 0.1–0.95 = explicit fraction of slot)
   */
  setBarWidthRatio(ratio: number): void;
  /**
   * Snap a (time, price) coordinate to the nearest OHLC point when magnet is active.
   * Returns JSON: `{ time: i64, price: f64, snapped: bool }`
   */
  snapToCandle(time: bigint, price: number): any;
  /**
   * Append or replace one footprint candle by timestamp.
   */
  addFootprintCandle(candle_json: string): void;
  /**
   * Create a new vertical line tool
   */
  createVerticalLine(id: string, time: bigint): void;
  /**
   * Set candle style, including Renko with brick size.
   * For renko: pass "renko" and provide brick_size > 0.
   */
  setRenkoBrickSize(brick_size: number): void;
  /**
   * Return selected drawing IDs as JSON.
   */
  getSelectedDrawings(): string;
  /**
   * Remove a comparison symbol.
   */
  removeCompareSymbol(symbol: string): void;
  /**
   * Enable or disable footprint rendering.
   */
  setFootprintEnabled(enabled: boolean): void;
  /**
   * Upsert a single candle by timestamp (delegates to CandleBuffer::update_running).
   *
   * Accepts a JSON object representing one candle.  If a candle with the
   * same `time` already exists it is replaced in-place; otherwise it is
   * inserted at the correct sorted position.
   */
  updateRunningCandle(candle_json: string): void;
  /**
   * Create a new horizontal line tool
   */
  createHorizontalLine(id: string, price: number): void;
  /**
   * Get candle at position (with hit-testing)
   */
  getCandleAtPosition(x: number, y: number): any;
  /**
   * Select all drawings whose nodes are fully inside a screen-space rectangle.
   */
  selectDrawingsInRect(x1: number, y1: number, x2: number, y2: number, additive: boolean): string;
  /**
   * Delete all selected drawings as one undoable operation.
   */
  deleteSelectedDrawings(): number;
  /**
   * Set one pane height fraction, then normalize all panes.
   */
  setPaneHeightFraction(pane_id: string, fraction: number): void;
  /**
   * Move all selected drawings to follow the current canvas position.
   */
  dragSelectedDrawingsTo(x: number, y: number): void;
  /**
   * End a bulk drawing drag.
   */
  endSelectedDrawingsDrag(): void;
  /**
   * Start bulk-dragging selected drawings from a canvas position.
   */
  startSelectedDrawingsDrag(x: number, y: number): boolean;
  /**
   * Create a new chart instance
   */
  constructor(width: number, height: number, timeframe: string);
  /**
   * Redo the last undone drawing action. Returns true if there was something to redo.
   */
  redo(): boolean;
  /**
   * Undo the last drawing action. Returns true if there was something to undo.
   */
  undo(): boolean;
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
  /**
   * Switch between dark and light theme
   */
  setTheme(dark: boolean): void;
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
 * Get all available indicator metadata as JSON
 */
export function getAllIndicators(): string;

/**
 * Get specific indicator metadata by ID as JSON
 */
export function getIndicatorMetadata(id: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmchart_free: (a: number, b: number) => void;
  readonly __wbg_wasmlempelzivcomplexity_free: (a: number, b: number) => void;
  readonly __wbg_wasmpermutationentropy_free: (a: number, b: number) => void;
  readonly __wbg_wasmshannonentropy_free: (a: number, b: number) => void;
  readonly getAllIndicators: () => [number, number];
  readonly getIndicatorMetadata: (a: number, b: number) => any;
  readonly wasmchart_addCandle: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wasmchart_addCompareSymbol: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly wasmchart_addFootprintCandle: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_addIndicatorPane: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly wasmchart_appendCandles: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_attachCanvas: (a: number, b: any) => [number, number];
  readonly wasmchart_clearTools: (a: number) => [number, number];
  readonly wasmchart_createEllipse: (a: number, b: number, c: number, d: bigint, e: number, f: bigint, g: number) => [number, number];
  readonly wasmchart_createFibonacci: (a: number, b: number, c: number, d: bigint, e: number, f: bigint, g: number) => [number, number];
  readonly wasmchart_createHorizontalLine: (a: number, b: number, c: number, d: number) => [number, number];
  readonly wasmchart_createRectangle: (a: number, b: number, c: number, d: bigint, e: number, f: bigint, g: number) => [number, number];
  readonly wasmchart_createTextLabel: (a: number, b: number, c: number, d: bigint, e: number, f: number, g: number) => [number, number];
  readonly wasmchart_createTrendLine: (a: number, b: number, c: number, d: bigint, e: number, f: bigint, g: number) => [number, number];
  readonly wasmchart_createVerticalLine: (a: number, b: number, c: number, d: bigint) => [number, number];
  readonly wasmchart_deleteSelectedDrawings: (a: number) => number;
  readonly wasmchart_dragSelectedDrawingsTo: (a: number, b: number, c: number) => void;
  readonly wasmchart_endPriceScale: (a: number) => [number, number];
  readonly wasmchart_endSelectedDrawingsDrag: (a: number) => void;
  readonly wasmchart_exportState: (a: number) => [number, number, number, number];
  readonly wasmchart_fitToData: (a: number) => void;
  readonly wasmchart_getBarSpacing: (a: number) => number;
  readonly wasmchart_getCandleAtPosition: (a: number, b: number, c: number) => any;
  readonly wasmchart_getCandles: (a: number) => [number, number];
  readonly wasmchart_getCompareSymbols: (a: number) => [number, number];
  readonly wasmchart_getCrosshairInfo: (a: number) => any;
  readonly wasmchart_getMagnetMode: (a: number) => [number, number];
  readonly wasmchart_getOHLCFormatted: (a: number) => any;
  readonly wasmchart_getPaneLayout: (a: number) => [number, number];
  readonly wasmchart_getScaleMode: (a: number) => [number, number];
  readonly wasmchart_getSelectedDrawings: (a: number) => [number, number];
  readonly wasmchart_getTimezoneOffset: (a: number) => number;
  readonly wasmchart_getTools: (a: number) => [number, number];
  readonly wasmchart_getViewportInfo: (a: number) => any;
  readonly wasmchart_importState: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_isDirty: (a: number) => number;
  readonly wasmchart_isLogScale: (a: number) => number;
  readonly wasmchart_isPriceLocked: (a: number) => number;
  readonly wasmchart_new: (a: number, b: number, c: number, d: number) => [number, number, number];
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
  readonly wasmchart_redo: (a: number) => number;
  readonly wasmchart_removeCompareSymbol: (a: number, b: number, c: number) => void;
  readonly wasmchart_removePane: (a: number, b: number, c: number) => number;
  readonly wasmchart_removeTool: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_render: (a: number) => [number, number];
  readonly wasmchart_resetPriceScale: (a: number) => [number, number];
  readonly wasmchart_resetTimeScale: (a: number) => [number, number];
  readonly wasmchart_resize: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_scalePriceTo: (a: number, b: number) => [number, number];
  readonly wasmchart_scaleTimeTo: (a: number, b: number) => [number, number];
  readonly wasmchart_selectDrawingAt: (a: number, b: number, c: number, d: number) => number;
  readonly wasmchart_selectDrawingsInRect: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly wasmchart_setBarSpacing: (a: number, b: number) => void;
  readonly wasmchart_setBarWidthRatio: (a: number, b: number) => void;
  readonly wasmchart_setCandleStyle: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_setCandles: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_setCandlesBatch: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_setFootprintData: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_setFootprintEnabled: (a: number, b: number) => void;
  readonly wasmchart_setLogScale: (a: number, b: number) => void;
  readonly wasmchart_setMagnetMode: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_setPaneHeightFraction: (a: number, b: number, c: number, d: number) => void;
  readonly wasmchart_setPriceLocked: (a: number, b: number) => void;
  readonly wasmchart_setRenkoBrickSize: (a: number, b: number) => [number, number];
  readonly wasmchart_setScaleMode: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_setSessions: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_setShowSessions: (a: number, b: number) => void;
  readonly wasmchart_setTheme: (a: number, b: number) => void;
  readonly wasmchart_setTimezone: (a: number, b: number) => void;
  readonly wasmchart_snapToCandle: (a: number, b: bigint, c: number) => any;
  readonly wasmchart_startPriceScale: (a: number, b: number) => [number, number];
  readonly wasmchart_startSelectedDrawingsDrag: (a: number, b: number, c: number) => number;
  readonly wasmchart_startTimeScale: (a: number, b: number) => [number, number];
  readonly wasmchart_undo: (a: number) => number;
  readonly wasmchart_updateRunningCandle: (a: number, b: number, c: number) => [number, number];
  readonly wasmlempelzivcomplexity_calculate: (a: number, b: number, c: number, d: number) => [number, number];
  readonly wasmlempelzivcomplexity_len: (a: number) => number;
  readonly wasmlempelzivcomplexity_new: (a: number, b: number) => number;
  readonly wasmlempelzivcomplexity_next: (a: number, b: number) => any;
  readonly wasmlempelzivcomplexity_reset: (a: number) => void;
  readonly wasmpermutationentropy_calculate: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly wasmpermutationentropy_len: (a: number) => number;
  readonly wasmpermutationentropy_new: (a: number, b: number, c: number) => number;
  readonly wasmpermutationentropy_next: (a: number, b: number) => any;
  readonly wasmpermutationentropy_reset: (a: number) => void;
  readonly wasmshannonentropy_calculate: (a: number, b: number, c: number, d: number) => [number, number];
  readonly wasmshannonentropy_new: (a: number, b: number) => number;
  readonly wasmshannonentropy_next: (a: number, b: number) => any;
  readonly wasmshannonentropy_reset: (a: number) => void;
  readonly wasmchart_endTimeScale: (a: number) => [number, number];
  readonly wasmshannonentropy_len: (a: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
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
