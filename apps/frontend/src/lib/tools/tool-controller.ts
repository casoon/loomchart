/**
 * Tool Controller - Manages drawing tools interaction
 * Coordinates between user input and Rust WASM drawing tools
 */

export enum ToolType {
  Cursor = "cursor",
  TrendLine = "trendline",
  HorizontalLine = "horizontal",
  VerticalLine = "vertical",
}

export interface ToolDefinition {
  type: ToolType;
  name: string;
  icon: string;
  cursor: string;
}

export interface ToolNode {
  time: number;
  price: number;
}

export class ToolController {
  private activeTool: ToolType = ToolType.Cursor;
  private wasmChart: any;
  private chartContainer: HTMLElement;
  private isPlacing: boolean = false;
  private currentToolId: string | null = null;
  private firstNode: ToolNode | null = null;
  private toolRegistry: Map<ToolType, ToolDefinition>;
  private listeners: Map<string, (e: MouseEvent) => void> = new Map();

  constructor(wasmChart: any, chartContainer: HTMLElement) {
    this.wasmChart = wasmChart;
    this.chartContainer = chartContainer;

    // Register available tools
    this.toolRegistry = new Map([
      [
        ToolType.Cursor,
        {
          type: ToolType.Cursor,
          name: "Cursor",
          icon: "⌖",
          cursor: "default",
        },
      ],
      [
        ToolType.TrendLine,
        {
          type: ToolType.TrendLine,
          name: "Trend Line",
          icon: "📈",
          cursor: "crosshair",
        },
      ],
      [
        ToolType.HorizontalLine,
        {
          type: ToolType.HorizontalLine,
          name: "Horizontal Line",
          icon: "—",
          cursor: "crosshair",
        },
      ],
      [
        ToolType.VerticalLine,
        {
          type: ToolType.VerticalLine,
          name: "Vertical Line",
          icon: "|",
          cursor: "crosshair",
        },
      ],
    ]);

    this.setupEventListeners();
  }

  private setupEventListeners(): void {
    const clickHandler = (e: MouseEvent) => this.handleClick(e);
    const moveHandler = (e: MouseEvent) => this.handleMove(e);

    this.chartContainer.addEventListener("click", clickHandler);
    this.chartContainer.addEventListener("mousemove", moveHandler);

    this.listeners.set("click", clickHandler);
    this.listeners.set("mousemove", moveHandler);
  }

  /**
   * Activate a drawing tool
   */
  public activateTool(type: ToolType): void {
    this.activeTool = type;
    this.isPlacing = false;
    this.currentToolId = null;

    const toolDef = this.toolRegistry.get(type);
    if (toolDef) {
      this.chartContainer.style.cursor = toolDef.cursor;
    }

    // Emit tool change event
    this.emitToolChange(type);
  }

  /**
   * Deactivate current tool and return to cursor
   */
  public deactivateTool(): void {
    this.activateTool(ToolType.Cursor);
  }

  /**
   * Get currently active tool
   */
  public getActiveTool(): ToolType {
    return this.activeTool;
  }

  /**
   * Handle mouse click on chart
   */
  private handleClick(e: MouseEvent): void {
    if (this.activeTool === ToolType.Cursor) {
      // TODO: Implement tool selection/editing
      return;
    }

    const rect = this.chartContainer.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    // Convert screen coordinates to time/price using viewport
    const viewportJson = this.wasmChart.getViewportInfo();
    if (!viewportJson) return;

    const viewport = JSON.parse(viewportJson);

    const time = this.screenToTime(x, viewport);
    const price = this.screenToPrice(y, viewport);

    this.placeTool(time, price);
  }

  /**
   * Handle mouse move on chart
   */
  private handleMove(e: MouseEvent): void {
    if (!this.isPlacing) return;

    const rect = this.chartContainer.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const viewportJson = this.wasmChart.getViewportInfo();
    if (!viewportJson) return;

    const viewport = JSON.parse(viewportJson);

    const time = this.screenToTime(x, viewport);
    const price = this.screenToPrice(y, viewport);

    // Update preview of tool being placed
    this.updateToolPreview(time, price);
  }

  /**
   * Place a tool node
   */
  private placeTool(time: number, price: number): void {
    switch (this.activeTool) {
      case ToolType.TrendLine:
        this.placeTrendLine(time, price);
        break;
      case ToolType.HorizontalLine:
        this.placeHorizontalLine(time, price);
        break;
      case ToolType.VerticalLine:
        this.placeVerticalLine(time, price);
        break;
    }
  }

  /**
   * Place trend line (requires 2 clicks)
   */
  private placeTrendLine(time: number, price: number): void {
    if (!this.isPlacing) {
      // First click - start placing
      this.currentToolId = `trendline_${Date.now()}`;
      this.isPlacing = true;
      this.firstNode = { time, price };

      console.log(`TrendLine started at time=${time}, price=${price}`);
    } else {
      // Second click - complete tool
      if (this.currentToolId && this.firstNode) {
        try {
          this.wasmChart.createTrendLine(
            this.currentToolId,
            BigInt(this.firstNode.time),
            this.firstNode.price,
            BigInt(time),
            price,
          );
          console.log(`TrendLine created: ${this.currentToolId}`);
        } catch (error) {
          console.error("Failed to create trendline:", error);
        }
      }

      this.isPlacing = false;
      this.firstNode = null;

      // Return to cursor after placing
      this.deactivateTool();
    }
  }

  /**
   * Place horizontal line (requires 1 click)
   */
  private placeHorizontalLine(time: number, price: number): void {
    const toolId = `horizontal_${Date.now()}`;

    try {
      this.wasmChart.createHorizontalLine(toolId, price);
      console.log(`HorizontalLine created at price=${price}`);
    } catch (error) {
      console.error("Failed to create horizontal line:", error);
    }

    // Return to cursor after placing
    this.deactivateTool();
  }

  /**
   * Place vertical line (requires 1 click)
   */
  private placeVerticalLine(time: number, price: number): void {
    const toolId = `vertical_${Date.now()}`;

    try {
      this.wasmChart.createVerticalLine(toolId, BigInt(time));
      console.log(`VerticalLine created at time=${time}`);
    } catch (error) {
      console.error("Failed to create vertical line:", error);
    }

    // Return to cursor after placing
    this.deactivateTool();
  }

  /**
   * Update tool preview while placing
   */
  private updateToolPreview(time: number, price: number): void {
    // TODO: Update preview in WASM
    console.log(`Preview update: time=${time}, price=${price}`);
  }

  /**
   * Convert screen X coordinate to time
   */
  private screenToTime(x: number, viewport: any): number {
    const { time, dimensions } = viewport;
    const { start, end } = time;
    const width = dimensions.width;

    return Math.floor(start + (x / width) * (end - start));
  }

  /**
   * Convert screen Y coordinate to price
   */
  private screenToPrice(y: number, viewport: any): number {
    const { price, dimensions } = viewport;
    const { min, max } = price;
    const height = dimensions.height;

    return max - (y / height) * (max - min);
  }

  /**
   * Emit tool change event
   */
  private emitToolChange(type: ToolType): void {
    const event = new CustomEvent("toolchange", {
      detail: { tool: type },
    });
    this.chartContainer.dispatchEvent(event);
  }

  /**
   * Clean up event listeners
   */
  public destroy(): void {
    this.listeners.forEach((handler, event) => {
      this.chartContainer.removeEventListener(event, handler);
    });
    this.listeners.clear();
  }

  /**
   * Get all tool definitions
   */
  public getTools(): ToolDefinition[] {
    return Array.from(this.toolRegistry.values());
  }

  /**
   * Cancel current tool placement
   */
  public cancel(): void {
    this.isPlacing = false;
    this.currentToolId = null;
    this.deactivateTool();
  }
}
