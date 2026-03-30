/**
 * Drawing Interaction Handler
 * Phase 4: Task 8.2 - Mouse interaction for drawing tools
 */

export interface ChartPoint {
  timestamp: number;
  price: number;
}

export class DrawingInteraction {
  private canvas: HTMLCanvasElement;
  private mode: string = "cursor";
  private activeDrawingId: string | null = null;
  private isDragging: boolean = false;

  constructor(canvas: HTMLCanvasElement) {
    this.canvas = canvas;
    this.setupEventListeners();
    this.setupKeyboardShortcuts();
  }

  /**
   * Set the current drawing mode
   */
  setMode(mode: string): void {
    this.mode = mode;
    this.updateCursor();
  }

  /**
   * Set up all mouse event listeners
   */
  private setupEventListeners(): void {
    this.canvas.addEventListener("mousedown", (e) => this.handleMouseDown(e));
    this.canvas.addEventListener("mousemove", (e) => this.handleMouseMove(e));
    this.canvas.addEventListener("mouseup", (e) => this.handleMouseUp(e));
    this.canvas.addEventListener("mouseleave", () => this.handleMouseLeave());

    // Listen for tool changes
    window.addEventListener("drawing-tool-changed", ((e: CustomEvent) => {
      this.setMode(e.detail.tool);
    }) as EventListener);
  }

  /**
   * Set up keyboard shortcuts for undo/redo
   */
  private setupKeyboardShortcuts(): void {
    document.addEventListener("keydown", (e) => {
      // Ctrl+Z or Cmd+Z for undo
      if ((e.ctrlKey || e.metaKey) && e.key === "z" && !e.shiftKey) {
        e.preventDefault();
        this.handleUndo();
      }
      // Ctrl+Shift+Z or Cmd+Shift+Z for redo
      else if ((e.ctrlKey || e.metaKey) && e.key === "z" && e.shiftKey) {
        e.preventDefault();
        this.handleRedo();
      }
      // Ctrl+Y or Cmd+Y for redo (alternative)
      else if ((e.ctrlKey || e.metaKey) && e.key === "y") {
        e.preventDefault();
        this.handleRedo();
      }
      // Delete or Backspace to remove selected drawing
      else if (
        (e.key === "Delete" || e.key === "Backspace") &&
        !e.ctrlKey &&
        !e.metaKey
      ) {
        e.preventDefault();
        this.handleDelete();
      }
    });
  }

  /**
   * Handle undo operation
   */
  private handleUndo(): void {
    const wasm = (window as any).getWasm?.();
    if (!wasm) return;

    try {
      const canUndo = wasm.can_undo?.();
      if (!canUndo) {
        (window as any).showToast?.("info", "Nothing to undo", 1500);
        return;
      }

      wasm.undo_drawing();
      (window as any).requestChartRender?.();
      (window as any).showToast?.("success", "Undo", 1500);

      console.log("[DrawingInteraction] Undo successful");
    } catch (err) {
      console.error("Failed to undo:", err);
      (window as any).showToast?.("error", "Failed to undo");
    }
  }

  /**
   * Handle redo operation
   */
  private handleRedo(): void {
    const wasm = (window as any).getWasm?.();
    if (!wasm) return;

    try {
      const canRedo = wasm.can_redo?.();
      if (!canRedo) {
        (window as any).showToast?.("info", "Nothing to redo", 1500);
        return;
      }

      wasm.redo_drawing();
      (window as any).requestChartRender?.();
      (window as any).showToast?.("success", "Redo", 1500);

      console.log("[DrawingInteraction] Redo successful");
    } catch (err) {
      console.error("Failed to redo:", err);
      (window as any).showToast?.("error", "Failed to redo");
    }
  }

  /**
   * Handle delete operation for selected drawing
   */
  private handleDelete(): void {
    const wasm = (window as any).getWasm?.();
    if (!wasm) return;

    try {
      const selectedId = wasm.get_selected_drawing_id?.();
      if (!selectedId) {
        return; // Nothing selected, ignore
      }

      wasm.remove_drawing(selectedId);
      (window as any).requestChartRender?.();
      (window as any).showToast?.("success", "Drawing deleted", 1500);

      console.log("[DrawingInteraction] Deleted drawing:", selectedId);
    } catch (err) {
      console.error("Failed to delete drawing:", err);
      (window as any).showToast?.("error", "Failed to delete drawing");
    }
  }

  /**
   * Handle mouse down event
   */
  private handleMouseDown(e: MouseEvent): void {
    e.preventDefault();

    const point = this.getChartPoint(e);
    if (!point) return;

    const wasm = (window as any).getWasm?.();
    if (!wasm) return;

    if (this.mode === "cursor") {
      // Selection mode - check if clicking on existing drawing
      this.handleSelection(point, wasm);
    } else {
      // Drawing mode - start new drawing
      this.startDrawing(point, wasm);
    }

    this.isDragging = true;
  }

  /**
   * Handle mouse move event
   */
  private handleMouseMove(e: MouseEvent): void {
    if (!this.isDragging || !this.activeDrawingId) return;

    const point = this.getChartPoint(e);
    if (!point) return;

    const wasm = (window as any).getWasm?.();
    if (!wasm) return;

    try {
      // Update the active drawing with current mouse position
      wasm.update_active_drawing(JSON.stringify(point));

      // Request re-render
      (window as any).requestChartRender?.();
    } catch (err) {
      console.error("Failed to update drawing:", err);
    }
  }

  /**
   * Handle mouse up event
   */
  private handleMouseUp(e: MouseEvent): void {
    if (!this.isDragging) return;

    this.isDragging = false;

    if (!this.activeDrawingId) return;

    const wasm = (window as any).getWasm?.();
    if (!wasm) return;

    try {
      // Finalize the drawing
      const drawingId = wasm.finalize_drawing();

      console.log("[DrawingInteraction] Finalized drawing:", drawingId);

      // Show success toast
      (window as any).showToast?.("success", "Drawing created", 2000);

      // Return to cursor mode after drawing
      this.setMode("cursor");

      // Update toolbar
      window.dispatchEvent(
        new CustomEvent("drawing-tool-changed", {
          detail: { tool: "cursor" },
        }),
      );

      // Request re-render
      (window as any).requestChartRender?.();
    } catch (err) {
      console.error("Failed to finalize drawing:", err);

      // Cancel the drawing on error
      try {
        wasm.cancel_drawing();
      } catch (cancelErr) {
        console.error("Failed to cancel drawing:", cancelErr);
      }

      (window as any).showToast?.("error", "Failed to create drawing");
    } finally {
      this.activeDrawingId = null;
    }
  }

  /**
   * Handle mouse leave event
   */
  private handleMouseLeave(): void {
    // Cancel active drawing if mouse leaves canvas
    if (this.activeDrawingId) {
      const wasm = (window as any).getWasm?.();
      if (wasm) {
        try {
          wasm.cancel_drawing();
        } catch (err) {
          console.error("Failed to cancel drawing:", err);
        }
      }
      this.activeDrawingId = null;
    }
    this.isDragging = false;
  }

  /**
   * Start a new drawing
   */
  private startDrawing(point: ChartPoint, wasm: any): void {
    try {
      // Start drawing in WASM
      this.activeDrawingId = wasm.start_drawing(
        this.mode,
        JSON.stringify(point),
      );

      console.log(
        "[DrawingInteraction] Started drawing:",
        this.activeDrawingId,
        this.mode,
      );
    } catch (err) {
      console.error("Failed to start drawing:", err);
      (window as any).showToast?.("error", "Failed to start drawing");
    }
  }

  /**
   * Handle selection of existing drawings
   */
  private handleSelection(point: ChartPoint, wasm: any): void {
    try {
      // Convert to screen coordinates for hit testing
      const rect = this.canvas.getBoundingClientRect();
      const screenPoint = this.chartToScreen(point);

      if (!screenPoint) return;

      // Hit test with 10px tolerance
      const hitId = wasm.hit_test_drawing?.(screenPoint.x, screenPoint.y, 10.0);

      if (hitId) {
        // Select the drawing
        wasm.select_drawing(hitId);
        console.log("[DrawingInteraction] Selected drawing:", hitId);

        // Request re-render to show selection
        (window as any).requestChartRender?.();
      } else {
        // Deselect if clicking on empty space
        wasm.deselect_drawing?.();
        (window as any).requestChartRender?.();
      }
    } catch (err) {
      console.error("Failed to handle selection:", err);
    }
  }

  /**
   * Convert mouse event to chart point (logical coordinates)
   */
  private getChartPoint(e: MouseEvent): ChartPoint | null {
    const wasm = (window as any).getWasm?.();
    if (!wasm) return null;

    try {
      const rect = this.canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      // Convert screen coordinates to chart coordinates
      const pointJson = wasm.screen_to_chart_point?.(x, y);
      if (!pointJson) return null;

      return JSON.parse(pointJson);
    } catch (err) {
      console.error("Failed to get chart point:", err);
      return null;
    }
  }

  /**
   * Convert chart point to screen coordinates
   */
  private chartToScreen(point: ChartPoint): { x: number; y: number } | null {
    const wasm = (window as any).getWasm?.();
    if (!wasm) return null;

    try {
      const screenJson = wasm.chart_point_to_screen?.(JSON.stringify(point));
      if (!screenJson) return null;

      return JSON.parse(screenJson);
    } catch (err) {
      console.error("Failed to convert to screen coords:", err);
      return null;
    }
  }

  /**
   * Update cursor style based on mode
   */
  private updateCursor(): void {
    if (this.mode === "cursor") {
      this.canvas.style.cursor = "default";
    } else {
      this.canvas.style.cursor = "crosshair";
    }
  }

  /**
   * Clean up event listeners
   */
  destroy(): void {
    // Remove event listeners if needed
    this.canvas.style.cursor = "default";
  }
}

/**
 * Initialize drawing interaction for a canvas
 */
export function initDrawingInteraction(
  canvas: HTMLCanvasElement,
): DrawingInteraction {
  return new DrawingInteraction(canvas);
}
