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

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wasmchart_free: (a: number, b: number) => void;
  readonly wasmchart_addCandle: (a: number, b: bigint, c: number, d: number, e: number, f: number, g: number) => void;
  readonly wasmchart_attachCanvas: (a: number, b: any) => [number, number];
  readonly wasmchart_clearTools: (a: number) => [number, number];
  readonly wasmchart_createHorizontalLine: (a: number, b: number, c: number, d: number) => [number, number];
  readonly wasmchart_createTrendLine: (a: number, b: number, c: number, d: bigint, e: number, f: bigint, g: number) => [number, number];
  readonly wasmchart_createVerticalLine: (a: number, b: number, c: number, d: bigint) => [number, number];
  readonly wasmchart_endPriceScale: (a: number) => [number, number];
  readonly wasmchart_exportState: (a: number) => [number, number, number, number];
  readonly wasmchart_fitToData: (a: number) => void;
  readonly wasmchart_getCandleAtPosition: (a: number, b: number, c: number) => any;
  readonly wasmchart_getCrosshairInfo: (a: number) => any;
  readonly wasmchart_getOHLCFormatted: (a: number) => any;
  readonly wasmchart_getTools: (a: number) => [number, number];
  readonly wasmchart_getViewportInfo: (a: number) => any;
  readonly wasmchart_importState: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_isDirty: (a: number) => number;
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
  readonly wasmchart_removeTool: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_render: (a: number) => [number, number];
  readonly wasmchart_resetPriceScale: (a: number) => [number, number];
  readonly wasmchart_resetTimeScale: (a: number) => [number, number];
  readonly wasmchart_resize: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_scalePriceTo: (a: number, b: number) => [number, number];
  readonly wasmchart_scaleTimeTo: (a: number, b: number) => [number, number];
  readonly wasmchart_setCandleStyle: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_setCandles: (a: number, b: number, c: number) => [number, number];
  readonly wasmchart_startPriceScale: (a: number, b: number) => [number, number];
  readonly wasmchart_startTimeScale: (a: number, b: number) => [number, number];
  readonly wasmchart_endTimeScale: (a: number) => [number, number];
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
