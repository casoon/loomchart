/**
 * WASM Chart Wrapper - Provides Chart API using Rust chartcore WASM
 *
 * This wrapper provides the same API as @loom/chart-core (TypeScript),
 * but uses the Rust chartcore WASM module for all rendering and calculations.
 *
 * Migration from TypeScript chart-core to Rust chartcore WASM.
 */

import type { WasmChart } from "../../public/wasm/trading_ui";
import { initChartTooltip, disposeChartTooltip } from "./chart-tooltip";

export interface ChartCandle {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export interface ChartOptions {
  width?: number;
  height?: number;
  autoScale?: boolean;
  crosshair?: boolean;
  grid?: boolean;
  timeframe?: string;
}

export interface IndicatorConfig {
  id: string;
  name: string;
  type: "line" | "histogram" | "overlay";
  color?: string;
  overlay?: boolean;
}

/**
 * Chart wrapper class that uses WASM chartcore
 */
export class Chart {
  public wasmChart: WasmChart | null = null; // Public for tooltip access
  private canvas: HTMLCanvasElement | null = null;
  private container: HTMLElement;
  private options: Required<ChartOptions>;
  private candles: ChartCandle[] = [];
  private indicators: Map<string, IndicatorConfig> = new Map();
  private wasmModule: any = null;

  constructor(container: HTMLElement, options: ChartOptions = {}) {
    this.container = container;
    this.options = {
      width: options.width ?? container.clientWidth,
      height: options.height ?? container.clientHeight,
      autoScale: options.autoScale ?? true,
      crosshair: options.crosshair ?? true,
      grid: options.grid ?? true,
      timeframe: options.timeframe ?? "1h",
    };
  }

  /**
   * Initialize the chart (async - loads WASM module)
   */
  async init(): Promise<void> {
    try {
      // Dynamically import the WASM module
      this.wasmModule = await import("../../public/wasm/trading_ui.js");

      // Create canvas
      this.canvas = document.createElement("canvas");
      this.canvas.width = this.options.width;
      this.canvas.height = this.options.height;
      this.canvas.style.width = `${this.options.width}px`;
      this.canvas.style.height = `${this.options.height}px`;
      this.container.appendChild(this.canvas);

      // Create WASM chart instance
      this.wasmChart = new this.wasmModule.WasmChart(
        this.options.width,
        this.options.height,
        this.options.timeframe,
      );

      // Attach canvas for rendering
      this.wasmChart.attachCanvas(this.canvas);

      // Set up event listeners
      this.setupEventListeners();

      // Initialize tooltip
      initChartTooltip(this, this.canvas);

      console.log(
        "[WasmChartWrapper] Initialized with dimensions:",
        this.options.width,
        "x",
        this.options.height,
      );
    } catch (error) {
      console.error("[WasmChartWrapper] Failed to initialize:", error);
      throw error;
    }
  }

  /**
   * Set up canvas event listeners
   */
  private setupEventListeners(): void {
    if (!this.canvas || !this.wasmChart) return;

    // Mouse events
    this.canvas.addEventListener("mousedown", (e) => {
      const rect = this.canvas!.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart!.onMouseDown(x, y, e.button);
    });

    this.canvas.addEventListener("mousemove", (e) => {
      const rect = this.canvas!.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart!.onMouseMove(x, y);
    });

    this.canvas.addEventListener("mouseup", (e) => {
      const rect = this.canvas!.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart!.onMouseUp(x, y, e.button);
    });

    this.canvas.addEventListener("mouseleave", () => {
      this.wasmChart!.onMouseLeave();
    });

    this.canvas.addEventListener("wheel", (e) => {
      e.preventDefault();
      const rect = this.canvas!.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart!.onMouseWheel(x, y, e.deltaY);
    });

    this.canvas.addEventListener("dblclick", (e) => {
      const rect = this.canvas!.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart!.onDoubleClick(x, y);
    });

    // Touch events
    this.canvas.addEventListener("touchstart", (e) => {
      e.preventDefault();
      if (e.touches.length > 0) {
        const rect = this.canvas!.getBoundingClientRect();
        const touch = e.touches[0];
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        this.wasmChart!.onTouchStart(x, y);
      }
    });

    this.canvas.addEventListener("touchmove", (e) => {
      e.preventDefault();
      if (e.touches.length > 0) {
        const rect = this.canvas!.getBoundingClientRect();
        const touch = e.touches[0];
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        this.wasmChart!.onTouchMove(x, y);
      }
    });

    this.canvas.addEventListener("touchend", (e) => {
      e.preventDefault();
      if (e.changedTouches.length > 0) {
        const rect = this.canvas!.getBoundingClientRect();
        const touch = e.changedTouches[0];
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        this.wasmChart!.onTouchEnd(x, y);
      }
    });

    // Keyboard events
    window.addEventListener("keydown", (e) => {
      this.wasmChart!.onKeyDown(e.key);
    });
  }

  /**
   * Set candle data (replaces existing)
   */
  setData(candles: ChartCandle[]): void {
    if (!this.wasmChart) {
      console.error("[WasmChartWrapper] Chart not initialized");
      return;
    }

    this.candles = candles;

    // Convert to JSON for WASM
    const candlesJson = JSON.stringify(candles);
    this.wasmChart.setCandles(candlesJson);

    // Fit viewport to data if autoScale is enabled
    if (this.options.autoScale) {
      this.wasmChart.fitToData();
    }

    // Render
    this.wasmChart.render();

    console.log("[WasmChartWrapper] Set", candles.length, "candles");
  }

  /**
   * Update single candle (for real-time updates)
   */
  updateCandle(candle: ChartCandle): void {
    if (!this.wasmChart) {
      console.error("[WasmChartWrapper] Chart not initialized");
      return;
    }

    // Update in local cache
    if (this.candles.length > 0) {
      const lastCandle = this.candles[this.candles.length - 1];
      if (lastCandle.time === candle.time) {
        this.candles[this.candles.length - 1] = candle;
      } else {
        this.candles.push(candle);
      }
    }

    // Update in WASM
    this.wasmChart.addCandle(
      BigInt(candle.time),
      candle.open,
      candle.high,
      candle.low,
      candle.close,
      candle.volume,
    );

    // Render
    this.wasmChart.render();
  }

  /**
   * Add indicator
   */
  addIndicator(config: IndicatorConfig): void {
    this.indicators.set(config.id, config);
    console.log("[WasmChartWrapper] Added indicator:", config.id);
    // TODO: Implement indicator support in WASM
  }

  /**
   * Update indicator data
   */
  updateIndicator(
    id: string,
    points: Array<{ time: number; value: number }>,
  ): void {
    console.log("[WasmChartWrapper] Updated indicator:", id, points.length);
    // TODO: Implement indicator support in WASM
  }

  /**
   * Remove indicator
   */
  removeIndicator(id: string): void {
    this.indicators.delete(id);
    console.log("[WasmChartWrapper] Removed indicator:", id);
    // TODO: Implement indicator support in WASM
  }

  /**
   * Get indicator config
   */
  getIndicatorConfig(id: string): IndicatorConfig | undefined {
    return this.indicators.get(id);
  }

  /**
   * Resize chart
   */
  resize(width: number, height: number): void {
    if (!this.wasmChart || !this.canvas) return;

    this.options.width = width;
    this.options.height = height;

    this.canvas.width = width;
    this.canvas.height = height;
    this.canvas.style.width = `${width}px`;
    this.canvas.style.height = `${height}px`;

    this.wasmChart.resize(width, height);
    this.wasmChart.render();

    console.log("[WasmChartWrapper] Resized to:", width, "x", height);
  }

  /**
   * Get viewport info
   */
  getViewportInfo(): any {
    if (!this.wasmChart) return null;
    return this.wasmChart.getViewportInfo();
  }

  /**
   * Get crosshair info
   */
  getCrosshairInfo(): any {
    if (!this.wasmChart) return null;
    return this.wasmChart.getCrosshairInfo();
  }

  /**
   * Dispose chart and cleanup
   */
  dispose(): void {
    // Dispose tooltip
    disposeChartTooltip();

    if (this.wasmChart) {
      this.wasmChart.free();
      this.wasmChart = null;
    }

    if (this.canvas && this.canvas.parentNode) {
      this.canvas.parentNode.removeChild(this.canvas);
      this.canvas = null;
    }

    this.candles = [];
    this.indicators.clear();

    console.log("[WasmChartWrapper] Disposed");
  }
}
