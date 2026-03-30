/**
 * Separate Chart Axes Components
 *
 * These render independently from the main chart canvas
 * and can be positioned top/bottom/left/right as needed.
 */

export interface AxisConfig {
  position: "top" | "bottom" | "left" | "right";
  width?: number; // for vertical axes
  height?: number; // for horizontal axes
  labelCount?: number;
}

export interface ViewportInfo {
  time: { start: number; end: number };
  price: { min: number; max: number };
  dimensions: { width: number; height: number };
}

/**
 * Price Axis (vertical)
 */
export class PriceAxis {
  private container: HTMLDivElement;
  private config: AxisConfig;
  private currentPriceLine: HTMLDivElement | null = null;
  private currentPriceLabel: HTMLDivElement | null = null;
  private wasmChart: any = null;
  private isScaling: boolean = false;
  private chartCanvas: HTMLCanvasElement | null = null;

  constructor(parentElement: HTMLElement, config: Partial<AxisConfig> = {}) {
    this.config = {
      position: config.position || "right",
      width: config.width || 70,
      labelCount: config.labelCount || 8,
    };

    // Create container
    this.container = document.createElement("div");
    this.container.className = "price-axis";
    this.container.style.cssText = `
      position: absolute;
      ${this.config.position}: 0;
      top: 0;
      bottom: 30px;
      width: ${this.config.width}px;
      background: rgba(20, 24, 28, 0.9);
      border-${this.config.position === "left" ? "right" : "left"}: 1px solid rgba(45, 54, 64, 0.5);
      display: flex;
      flex-direction: column;
      justify-content: space-between;
      padding: 8px 4px;
      font-size: 11px;
      color: #e7e9ea;
      font-family: monospace;
      overflow: visible;
      z-index: 100;
      pointer-events: auto;
    `;

    parentElement.appendChild(this.container);
  }

  update(viewport: ViewportInfo, currentPrice?: number): void {
    const { min, max } = viewport.price;
    const step = (max - min) / (this.config.labelCount! - 1);

    // Clear and regenerate labels
    this.container.innerHTML = "";

    for (let i = 0; i < this.config.labelCount!; i++) {
      const price = max - i * step; // Top to bottom
      const label = document.createElement("div");
      label.textContent = price.toFixed(2);
      label.style.cssText = `
        text-align: ${this.config.position === "right" ? "right" : "left"};
        padding: 2px 6px;
        white-space: nowrap;
      `;
      this.container.appendChild(label);
    }

    // Add current price line and label if provided
    if (
      currentPrice !== undefined &&
      currentPrice >= min &&
      currentPrice <= max
    ) {
      this.drawCurrentPriceLine(currentPrice, min, max);
    }
  }

  private drawCurrentPriceLine(price: number, min: number, max: number): void {
    const parent = this.container.parentElement;
    if (!parent) return;

    // Calculate position (percentage from bottom)
    const priceRange = max - min;
    const priceFromBottom = price - min;
    const percentFromBottom = (priceFromBottom / priceRange) * 100;
    const percentFromTop = 100 - percentFromBottom;

    // Create or update horizontal line
    if (!this.currentPriceLine) {
      this.currentPriceLine = document.createElement("div");
      this.currentPriceLine.className = "current-price-line";
      parent.appendChild(this.currentPriceLine);
    }

    this.currentPriceLine.style.cssText = `
      position: absolute;
      left: 0;
      right: ${this.config.width}px;
      top: ${percentFromTop}%;
      height: 1px;
      background: rgba(59, 130, 246, 0.5);
      pointer-events: none;
      z-index: 10;
    `;

    // Create or update price label
    if (!this.currentPriceLabel) {
      this.currentPriceLabel = document.createElement("div");
      this.currentPriceLabel.className = "current-price-label";
      parent.appendChild(this.currentPriceLabel);
    }

    this.currentPriceLabel.textContent = price.toFixed(2);
    this.currentPriceLabel.style.cssText = `
      position: absolute;
      ${this.config.position}: 0;
      top: ${percentFromTop}%;
      transform: translateY(-50%);
      width: ${this.config.width}px;
      background: rgba(59, 130, 246, 0.9);
      color: white;
      font-size: 11px;
      font-family: monospace;
      font-weight: bold;
      text-align: center;
      padding: 2px 4px;
      border-radius: 2px;
      pointer-events: none;
      z-index: 11;
    `;
  }

  /**
   * Connect to WASM chart for interactive scaling
   */
  connectToChart(wasmChart: any, chartCanvas: HTMLCanvasElement): void {
    console.log("[PriceAxis] Connecting to WASM chart", {
      hasWasm: !!wasmChart,
    });
    this.wasmChart = wasmChart;
    this.chartCanvas = chartCanvas;

    // Verify WASM methods exist
    if (wasmChart) {
      console.log("[PriceAxis] WASM methods available:", {
        startPriceScale: typeof wasmChart.startPriceScale,
        scalePriceTo: typeof wasmChart.scalePriceTo,
        endPriceScale: typeof wasmChart.endPriceScale,
        resetPriceScale: typeof wasmChart.resetPriceScale,
      });
    }

    this.setupInteraction();
  }

  /**
   * Setup mouse event handlers for price scaling
   */
  private setupInteraction(): void {
    console.log("[PriceAxis] Setting up interaction handlers");

    // Mouse down - start scaling
    this.container.addEventListener("mousedown", (e) => {
      console.log("[PriceAxis] mousedown event", {
        hasWasm: !!this.wasmChart,
        button: e.button,
      });
      if (!this.wasmChart || !this.chartCanvas || e.button !== 0) return; // Only left click

      e.preventDefault();
      this.isScaling = true;

      // Get Y relative to the chart canvas, not the price axis container
      const canvasRect = this.chartCanvas.getBoundingClientRect();
      const y = e.clientY - canvasRect.top;

      console.log(
        "[PriceAxis] Starting price scale at y:",
        y,
        "canvas height:",
        this.chartCanvas.height / window.devicePixelRatio,
      );

      try {
        this.wasmChart.startPriceScale(y);
        console.log("[PriceAxis] startPriceScale called successfully");
      } catch (error) {
        console.error("[PriceAxis] Failed to start price scale:", error);
      }

      // Change cursor
      this.container.style.cursor = "ns-resize";
    });

    // Mouse move - apply scaling
    this.container.addEventListener("mousemove", (e) => {
      if (!this.wasmChart || !this.chartCanvas) return;

      if (this.isScaling) {
        // Get Y relative to the chart canvas
        const canvasRect = this.chartCanvas.getBoundingClientRect();
        const y = e.clientY - canvasRect.top;

        try {
          this.wasmChart.scalePriceTo(y);
        } catch (error) {
          console.error("[PriceAxis] Failed to scale price:", error);
        }
      } else {
        // Show resize cursor on hover
        this.container.style.cursor = "ns-resize";
      }
    });

    // Mouse up - end scaling
    const handleMouseUp = () => {
      if (!this.wasmChart || !this.isScaling) return;

      console.log("[PriceAxis] Ending price scale");
      this.isScaling = false;

      try {
        this.wasmChart.endPriceScale();
        console.log("[PriceAxis] endPriceScale called successfully");
      } catch (error) {
        console.error("[PriceAxis] Failed to end price scale:", error);
      }
    };

    this.container.addEventListener("mouseup", handleMouseUp);
    document.addEventListener("mouseup", handleMouseUp); // Catch mouse up outside

    // Mouse leave - reset cursor
    this.container.addEventListener("mouseleave", () => {
      if (!this.isScaling) {
        this.container.style.cursor = "default";
      }
    });

    // Double click - reset to auto-fit
    this.container.addEventListener("dblclick", (e) => {
      if (!this.wasmChart) return;

      e.preventDefault();

      try {
        this.wasmChart.resetPriceScale();
      } catch (error) {
        console.error("Failed to reset price scale:", error);
      }
    });
  }

  destroy(): void {
    this.currentPriceLine?.remove();
    this.currentPriceLabel?.remove();
    this.container.remove();
  }
}

/**
 * Time Axis (horizontal)
 */
export class TimeAxis {
  private container: HTMLDivElement;
  private config: AxisConfig;
  private wasmChart: any = null;
  private isScaling: boolean = false;
  private chartCanvas: HTMLCanvasElement | null = null;

  constructor(parentElement: HTMLElement, config: Partial<AxisConfig> = {}) {
    this.config = {
      position: config.position || "bottom",
      height: config.height || 25,
      labelCount: config.labelCount || 6,
    };

    // Create container
    this.container = document.createElement("div");
    this.container.className = "time-axis";
    this.container.style.cssText = `
      position: absolute;
      ${this.config.position}: 0;
      left: 0;
      right: 70px;
      height: ${this.config.height}px;
      background: rgba(20, 24, 28, 0.9);
      border-${this.config.position === "top" ? "bottom" : "top"}: 1px solid rgba(45, 54, 64, 0.5);
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 0 8px;
      font-size: 11px;
      color: #e7e9ea;
      font-family: monospace;
      z-index: 100;
      pointer-events: auto;
    `;

    parentElement.appendChild(this.container);
  }

  update(viewport: ViewportInfo): void {
    const { start, end } = viewport.time;
    const step = (end - start) / (this.config.labelCount! - 1);

    // Clear and regenerate labels
    this.container.innerHTML = "";

    for (let i = 0; i < this.config.labelCount!; i++) {
      const timestamp = start + i * step;
      const date = new Date(timestamp * 1000); // Convert seconds to milliseconds
      const label = document.createElement("div");

      // Format as HH:MM
      const hours = date.getHours().toString().padStart(2, "0");
      const minutes = date.getMinutes().toString().padStart(2, "0");
      label.textContent = `${hours}:${minutes}`;

      label.style.cssText = `
        text-align: center;
        padding: 2px 4px;
        white-space: nowrap;
      `;
      this.container.appendChild(label);
    }
  }

  /**
   * Connect to WASM chart for interactive scaling
   */
  connectToChart(wasmChart: any, chartCanvas: HTMLCanvasElement): void {
    console.log("[TimeAxis] Connecting to WASM chart", {
      hasWasm: !!wasmChart,
    });
    this.wasmChart = wasmChart;
    this.chartCanvas = chartCanvas;

    // Verify WASM methods exist
    if (wasmChart) {
      console.log("[TimeAxis] WASM methods available:", {
        startTimeScale: typeof wasmChart.startTimeScale,
        scaleTimeTo: typeof wasmChart.scaleTimeTo,
        endTimeScale: typeof wasmChart.endTimeScale,
        resetTimeScale: typeof wasmChart.resetTimeScale,
      });
    }

    this.setupInteraction();
  }

  /**
   * Setup mouse event handlers for time scaling
   */
  private setupInteraction(): void {
    console.log("[TimeAxis] Setting up interaction handlers");

    // Mouse down - start scaling
    this.container.addEventListener("mousedown", (e) => {
      console.log("[TimeAxis] mousedown event", {
        hasWasm: !!this.wasmChart,
        button: e.button,
      });
      if (!this.wasmChart || !this.chartCanvas || e.button !== 0) return; // Only left click

      e.preventDefault();
      this.isScaling = true;

      // Get X relative to the chart canvas
      const canvasRect = this.chartCanvas.getBoundingClientRect();
      const x = e.clientX - canvasRect.left;

      console.log(
        "[TimeAxis] Starting time scale at x:",
        x,
        "canvas width:",
        this.chartCanvas.width / window.devicePixelRatio,
      );

      try {
        this.wasmChart.startTimeScale(x);
        console.log("[TimeAxis] startTimeScale called successfully");
      } catch (error) {
        console.error("[TimeAxis] Failed to start time scale:", error);
      }

      // Change cursor
      this.container.style.cursor = "ew-resize";
    });

    // Mouse move - apply scaling
    this.container.addEventListener("mousemove", (e) => {
      if (!this.wasmChart || !this.chartCanvas) return;

      if (this.isScaling) {
        // Get X relative to the chart canvas
        const canvasRect = this.chartCanvas.getBoundingClientRect();
        const x = e.clientX - canvasRect.left;

        try {
          this.wasmChart.scaleTimeTo(x);
        } catch (error) {
          console.error("[TimeAxis] Failed to scale time:", error);
        }
      } else {
        // Show resize cursor on hover
        this.container.style.cursor = "ew-resize";
      }
    });

    // Mouse up - end scaling
    const handleMouseUp = () => {
      if (!this.wasmChart || !this.isScaling) return;

      console.log("[TimeAxis] Ending time scale");
      this.isScaling = false;

      try {
        this.wasmChart.endTimeScale();
        console.log("[TimeAxis] endTimeScale called successfully");
      } catch (error) {
        console.error("[TimeAxis] Failed to end time scale:", error);
      }
    };

    this.container.addEventListener("mouseup", handleMouseUp);
    document.addEventListener("mouseup", handleMouseUp); // Catch mouse up outside

    // Mouse leave - reset cursor
    this.container.addEventListener("mouseleave", () => {
      if (!this.isScaling) {
        this.container.style.cursor = "default";
      }
    });

    // Double click - reset to auto-fit
    this.container.addEventListener("dblclick", (e) => {
      if (!this.wasmChart) return;

      e.preventDefault();

      try {
        this.wasmChart.resetTimeScale();
      } catch (error) {
        console.error("[TimeAxis] Failed to reset time scale:", error);
      }
    });
  }

  destroy(): void {
    this.container.remove();
  }
}
