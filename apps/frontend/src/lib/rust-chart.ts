/**
 * Rust Chart Engine - Minimal TypeScript Wrapper
 *
 * This is a thin wrapper around the Rust WASM chart engine.
 * All core logic (viewport, rendering, events) is in Rust.
 */

import { PriceAxis, TimeAxis } from "./chart-axes";
import { initChartTooltip, disposeChartTooltip } from "./chart-tooltip";
import { HighLowOverlay } from "./chart-highlow";
import { ToolController } from "./tools/tool-controller";
import { CanvasBinding } from "./canvas";
import {
  initializeChartIntegration,
  onCandleAdded,
  onChartReset,
} from "./indicators/chart-integration";

export interface Candle {
  time: number;
  o: number;
  h: number;
  l: number;
  c: number;
  v: number;
}

export interface CrosshairInfo {
  time: number;
  price: number;
  x: number;
  y: number;
  ohlcv?: {
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
  };
}

export interface ViewportInfo {
  time: {
    start: number;
    end: number;
  };
  price: {
    min: number;
    max: number;
  };
  dimensions: {
    width: number;
    height: number;
    pixelRatio: number;
  };
  visibleBars: number;
  barWidth: number;
}

/**
 * Rust Chart - Minimal wrapper around WASM chart engine
 */
export class RustChart {
  private wasmChart: any = null;
  private canvas: HTMLCanvasElement;
  private animationFrameId: number | null = null;
  private canvasBinding: CanvasBinding | null = null;
  private priceAxis: PriceAxis;
  private timeAxis: TimeAxis;
  private highLowOverlay: HighLowOverlay;
  private toolController: ToolController | null = null;

  constructor(
    private container: HTMLElement,
    private timeframe: string = "5m",
  ) {
    // Create canvas
    this.canvas = document.createElement("canvas");
    this.canvas.style.cssText = `
      position: absolute;
      top: 0;
      left: 0;
      right: 70px;
      bottom: 30px;
      pointer-events: auto;
    `;
    this.container.appendChild(this.canvas);

    // Create separate axis components
    this.priceAxis = new PriceAxis(container, { position: "right" });
    this.timeAxis = new TimeAxis(container, { position: "bottom" });
    this.highLowOverlay = new HighLowOverlay(container);

    // Setup event listeners
    this.setupEventListeners();
  }

  /**
   * Initialize WASM module and create chart
   */
  async initialize(): Promise<void> {
    // Load WASM module from public/wasm (trading_ui)
    const module = await import("../../public/wasm/trading_ui.js");
    await module.default();

    // Get canvas dimensions (container size minus axes)
    const containerRect = this.container.getBoundingClientRect();
    const width = Math.floor(containerRect.width - 70); // Subtract price axis width
    const height = Math.floor(containerRect.height - 30); // Subtract time axis height

    console.log("[RustChart] Initializing with size:", { width, height });

    // Create WASM chart instance with CSS dimensions
    this.wasmChart = new module.WasmChart(width, height, this.timeframe);
    this.wasmChart.attachCanvas(this.canvas);

    // Setup canvas binding for DPI-aware rendering and resize handling
    this.canvasBinding = new CanvasBinding({
      canvas: this.canvas,
      wasmChart: this.wasmChart,
      onResize: (info) => {
        console.log("[RustChart] Canvas resized:", {
          css: `${info.cssSize.width}x${info.cssSize.height}`,
          device: `${info.deviceSize.width}x${info.deviceSize.height}`,
          ratio: info.pixelRatio,
        });
      },
      onPixelRatioChange: (ratio) => {
        console.log("[RustChart] Pixel ratio changed:", ratio);
      },
    });

    // Initialize canvas binding (starts resize observation)
    this.canvasBinding.initialize();

    // Initialize tooltip
    initChartTooltip(this as any, this.canvas);

    // Connect price axis to WASM chart for interactive scaling
    this.priceAxis.connectToChart(this.wasmChart, this.canvas);

    // Connect time axis to WASM chart for interactive scaling
    this.timeAxis.connectToChart(this.wasmChart, this.canvas);

    // Initialize tool controller
    this.toolController = new ToolController(this.wasmChart, this.container);

    // Make tool controller available to toolbar
    if ((window as any).initDrawingToolbar) {
      (window as any).initDrawingToolbar(this.toolController);
    }

    // Initialize indicator integration
    initializeChartIntegration(this);

    // Start render loop
    this.startRenderLoop();
  }

  /**
   * Set candle data
   */
  setCandles(candles: Candle[]): void {
    if (!this.wasmChart) return;
    this.wasmChart.setCandles(JSON.stringify(candles));
    onChartReset();
  }

  /**
   * Add single candle (for real-time updates)
   */
  addCandle(candle: Candle): void {
    if (!this.wasmChart) return;
    this.wasmChart.addCandle(
      BigInt(candle.time),
      candle.o,
      candle.h,
      candle.l,
      candle.c,
      candle.v,
    );
    onCandleAdded(candle);
  }

  /**
   * Get all candles (for indicator calculations)
   */
  getCandles(): Candle[] {
    if (!this.wasmChart) return [];
    const candlesJson = this.wasmChart.getCandles();
    if (!candlesJson) return [];
    return JSON.parse(candlesJson);
  }

  /**
   * Fit viewport to data
   */
  fitToData(): void {
    if (!this.wasmChart) return;
    this.wasmChart.fitToData();
  }

  /**
   * Get crosshair info
   */
  getCrosshairInfo(): CrosshairInfo | null {
    if (!this.wasmChart) return null;
    const info = this.wasmChart.getCrosshairInfo();
    if (!info) return null;
    return JSON.parse(info);
  }

  /**
   * Get viewport info
   */
  getViewportInfo(): ViewportInfo | null {
    if (!this.wasmChart) return null;
    const info = this.wasmChart.getViewportInfo();
    if (!info) return null;
    return JSON.parse(info);
  }

  /**
   * Setup event listeners for mouse/touch/keyboard
   */
  private setupEventListeners(): void {
    // Mouse events
    this.canvas.addEventListener("mousedown", (e) => {
      if (!this.wasmChart) return;
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart.onMouseDown(x, y, e.button);
    });

    this.canvas.addEventListener("mouseup", (e) => {
      if (!this.wasmChart) return;
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart.onMouseUp(x, y, e.button);
    });

    this.canvas.addEventListener("mousemove", (e) => {
      if (!this.wasmChart) return;
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart.onMouseMove(x, y);
    });

    this.canvas.addEventListener(
      "wheel",
      (e) => {
        if (!this.wasmChart) return;
        e.preventDefault();
        const rect = this.canvas.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        this.wasmChart.onMouseWheel(x, y, e.deltaY);
      },
      { passive: false },
    );

    this.canvas.addEventListener("mouseleave", () => {
      if (!this.wasmChart) return;
      this.wasmChart.onMouseLeave();
    });

    this.canvas.addEventListener("dblclick", (e) => {
      if (!this.wasmChart) return;
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      this.wasmChart.onDoubleClick(x, y);
    });

    // Touch events
    this.canvas.addEventListener(
      "touchstart",
      (e) => {
        if (!this.wasmChart || e.touches.length === 0) return;
        e.preventDefault();
        const rect = this.canvas.getBoundingClientRect();
        const touch = e.touches[0];
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        this.wasmChart.onTouchStart(x, y);
      },
      { passive: false },
    );

    this.canvas.addEventListener(
      "touchmove",
      (e) => {
        if (!this.wasmChart || e.touches.length === 0) return;
        e.preventDefault();
        const rect = this.canvas.getBoundingClientRect();
        const touch = e.touches[0];
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        this.wasmChart.onTouchMove(x, y);
      },
      { passive: false },
    );

    this.canvas.addEventListener(
      "touchend",
      (e) => {
        if (!this.wasmChart) return;
        e.preventDefault();
        const rect = this.canvas.getBoundingClientRect();
        const touch = e.changedTouches[0];
        const x = touch.clientX - rect.left;
        const y = touch.clientY - rect.top;
        this.wasmChart.onTouchEnd(x, y);
      },
      { passive: false },
    );

    // Keyboard events
    window.addEventListener("keydown", (e) => {
      if (!this.wasmChart) return;
      if (
        document.activeElement === document.body ||
        document.activeElement === this.canvas
      ) {
        this.wasmChart.onKeyDown(e.key);
      }
    });
  }

  /**
   * Get current price from last candle
   */
  private getCurrentPrice(): number | undefined {
    const crosshair = this.getCrosshairInfo();
    if (crosshair && crosshair.ohlcv) {
      return crosshair.ohlcv.close;
    }
    return undefined;
  }

  /**
   * Start render loop
   */
  private startRenderLoop(): void {
    const render = () => {
      if (this.wasmChart) {
        // Always render (don't check isDirty)
        // The Rust side will handle optimization
        this.wasmChart.render();

        // Update axes with current viewport and current price
        const viewport = this.getViewportInfo();
        if (viewport) {
          // Try to get current price from app state
          const app = (window as any).Alpine?.raw?.tradingApp;
          const currentPrice = app?.lastCandle?.c;

          this.priceAxis.update(viewport, currentPrice);
          this.timeAxis.update(viewport);
          this.highLowOverlay.update(viewport, currentPrice);
        }
      }
      this.animationFrameId = requestAnimationFrame(render);
    };
    render();
  }

  /**
   * Cleanup
   */
  destroy(): void {
    // Dispose tooltip
    disposeChartTooltip();

    if (this.animationFrameId !== null) {
      cancelAnimationFrame(this.animationFrameId);
    }

    // Destroy canvas binding
    if (this.canvasBinding) {
      this.canvasBinding.destroy();
      this.canvasBinding = null;
    }

    if (this.wasmChart) {
      this.wasmChart.free();
      this.wasmChart = null;
    }

    // Destroy axes and overlays
    this.priceAxis.destroy();
    this.timeAxis.destroy();
    this.highLowOverlay.destroy();

    // Destroy tool controller
    if (this.toolController) {
      this.toolController.destroy();
      this.toolController = null;
    }

    this.canvas.remove();
  }
}
