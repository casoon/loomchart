/**
 * Chart Tooltip Handler
 *
 * Handles mouse events to show OHLC tooltip when hovering over candles.
 * Uses WASM getCandleAtPosition API for hit-testing.
 */

import type { Chart } from "./wasm-chart-wrapper";

interface TooltipElements {
  container: HTMLElement;
  time: HTMLElement;
  open: HTMLElement;
  high: HTMLElement;
  low: HTMLElement;
  close: HTMLElement;
  volume: HTMLElement;
}

export class ChartTooltipHandler {
  private chart: Chart | null = null;
  private canvas: HTMLCanvasElement | null = null;
  private elements: TooltipElements | null = null;
  private isVisible = false;

  constructor() {
    this.initElements();
  }

  /**
   * Initialize tooltip DOM elements
   */
  private initElements(): void {
    const container = document.getElementById("chart-tooltip");
    const time = document.getElementById("tooltip-time");
    const open = document.getElementById("tooltip-open");
    const high = document.getElementById("tooltip-high");
    const low = document.getElementById("tooltip-low");
    const close = document.getElementById("tooltip-close");
    const volume = document.getElementById("tooltip-volume");

    if (!container || !time || !open || !high || !low || !close || !volume) {
      console.warn("[ChartTooltip] Tooltip elements not found");
      return;
    }

    this.elements = {
      container,
      time,
      open,
      high,
      low,
      close,
      volume,
    };
  }

  /**
   * Attach tooltip to chart
   */
  attach(chart: Chart, canvas: HTMLCanvasElement): void {
    this.chart = chart;
    this.canvas = canvas;

    // Add mouse event listeners
    canvas.addEventListener("mousemove", this.handleMouseMove);
    canvas.addEventListener("mouseleave", this.handleMouseLeave);

    console.log("[ChartTooltip] Attached to canvas");
  }

  /**
   * Detach tooltip from chart
   */
  detach(): void {
    if (this.canvas) {
      this.canvas.removeEventListener("mousemove", this.handleMouseMove);
      this.canvas.removeEventListener("mouseleave", this.handleMouseLeave);
    }

    this.hide();
    this.chart = null;
    this.canvas = null;

    console.log("[ChartTooltip] Detached from canvas");
  }

  /**
   * Handle mouse move event
   */
  private handleMouseMove = (event: MouseEvent): void => {
    if (!this.chart || !this.elements || !this.canvas) return;

    // Get mouse position relative to canvas
    const rect = this.canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;

    // Get candle at position from WASM
    try {
      const wasmChart = (this.chart as any).wasmChart;
      if (!wasmChart?.getCandleAtPosition) {
        console.warn("[ChartTooltip] WASM chart not available");
        return;
      }

      const result = wasmChart.getCandleAtPosition(x, y);

      if (result) {
        // Parse JSON result
        const candleInfo = typeof result === "string" ? JSON.parse(result) : result;

        if (candleInfo && candleInfo.time) {
          this.updateTooltip(candleInfo, event.clientX, event.clientY);
          this.show();
        } else {
          this.hide();
        }
      } else {
        this.hide();
      }
    } catch (error) {
      console.error("[ChartTooltip] Error getting candle at position:", error);
      this.hide();
    }
  };

  /**
   * Handle mouse leave event
   */
  private handleMouseLeave = (): void => {
    this.hide();
  };

  /**
   * Update tooltip content
   */
  private updateTooltip(
    candle: {
      time: number;
      open: number;
      high: number;
      low: number;
      close: number;
      volume: number;
    },
    mouseX: number,
    mouseY: number,
  ): void {
    if (!this.elements) return;

    // Format time
    const date = new Date(candle.time * 1000);
    const timeStr = date.toLocaleString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });

    // Update content
    this.elements.time.textContent = timeStr;
    this.elements.open.textContent = candle.open.toFixed(2);
    this.elements.high.textContent = candle.high.toFixed(2);
    this.elements.low.textContent = candle.low.toFixed(2);
    this.elements.close.textContent = candle.close.toFixed(2);

    // Format volume (e.g., 1234.56 -> "1.2k")
    const volK = candle.volume / 1000;
    this.elements.volume.textContent =
      volK >= 1 ? `${volK.toFixed(1)}k` : candle.volume.toFixed(2);

    // Position tooltip
    this.positionTooltip(mouseX, mouseY);
  }

  /**
   * Position tooltip near mouse cursor
   */
  private positionTooltip(mouseX: number, mouseY: number): void {
    if (!this.elements) return;

    const tooltip = this.elements.container;
    const offset = 15; // Offset from cursor

    // Get viewport dimensions
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    // Get tooltip dimensions
    const tooltipRect = tooltip.getBoundingClientRect();
    const tooltipWidth = tooltipRect.width;
    const tooltipHeight = tooltipRect.height;

    // Calculate position (default: bottom-right of cursor)
    let left = mouseX + offset;
    let top = mouseY + offset;

    // Flip horizontal if too close to right edge
    if (left + tooltipWidth > viewportWidth - 10) {
      left = mouseX - tooltipWidth - offset;
    }

    // Flip vertical if too close to bottom edge
    if (top + tooltipHeight > viewportHeight - 10) {
      top = mouseY - tooltipHeight - offset;
    }

    // Ensure tooltip stays within viewport
    left = Math.max(10, Math.min(left, viewportWidth - tooltipWidth - 10));
    top = Math.max(10, Math.min(top, viewportHeight - tooltipHeight - 10));

    tooltip.style.left = `${left}px`;
    tooltip.style.top = `${top}px`;
  }

  /**
   * Show tooltip
   */
  private show(): void {
    if (!this.elements || this.isVisible) return;

    this.elements.container.classList.remove("hidden");
    this.isVisible = true;
  }

  /**
   * Hide tooltip
   */
  private hide(): void {
    if (!this.elements || !this.isVisible) return;

    this.elements.container.classList.add("hidden");
    this.isVisible = false;
  }
}

// Global tooltip instance
let tooltipHandler: ChartTooltipHandler | null = null;

/**
 * Initialize chart tooltip
 */
export function initChartTooltip(chart: Chart, canvas: HTMLCanvasElement): void {
  if (tooltipHandler) {
    tooltipHandler.detach();
  }

  tooltipHandler = new ChartTooltipHandler();
  tooltipHandler.attach(chart, canvas);

  console.log("[ChartTooltip] Initialized");
}

/**
 * Dispose chart tooltip
 */
export function disposeChartTooltip(): void {
  if (tooltipHandler) {
    tooltipHandler.detach();
    tooltipHandler = null;
  }

  console.log("[ChartTooltip] Disposed");
}
