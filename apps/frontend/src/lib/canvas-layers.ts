/**
 * Canvas Layer Manager
 *
 * Manages multiple canvas layers for efficient rendering.
 * Each layer can be independently marked as dirty and re-rendered.
 * Layers are composited in z-index order for final output.
 */

export interface Layer {
  id: string;
  canvas: HTMLCanvasElement;
  context: CanvasRenderingContext2D;
  visible: boolean;
  alpha: number;
  zIndex: number;
  dirty: boolean;
}

export class CanvasLayerManager {
  private layers: Map<string, Layer> = new Map();
  private container: HTMLElement;
  private width: number = 0;
  private height: number = 0;
  private pixelRatio: number = 1;

  constructor(container: HTMLElement) {
    this.container = container;
    this.pixelRatio = window.devicePixelRatio || 1;
  }

  /**
   * Create a new layer
   */
  createLayer(
    id: string,
    zIndex: number,
    options: { visible?: boolean; alpha?: number } = {},
  ): CanvasRenderingContext2D {
    if (this.layers.has(id)) {
      console.warn(`[CanvasLayers] Layer ${id} already exists, returning existing layer`);
      return this.layers.get(id)!.context;
    }

    // Create canvas
    const canvas = document.createElement("canvas");
    canvas.id = `layer-${id}`;
    canvas.style.position = "absolute";
    canvas.style.top = "0";
    canvas.style.left = "0";
    canvas.style.pointerEvents = "none";
    canvas.style.zIndex = String(zIndex);

    // Set dimensions if already set
    if (this.width > 0 && this.height > 0) {
      this.setCanvasDimensions(canvas, this.width, this.height);
    }

    // Get context
    const context = canvas.getContext("2d");
    if (!context) {
      throw new Error(`[CanvasLayers] Failed to get 2d context for layer ${id}`);
    }

    // Create layer object
    const layer: Layer = {
      id,
      canvas,
      context,
      visible: options.visible !== undefined ? options.visible : true,
      alpha: options.alpha !== undefined ? options.alpha : 1.0,
      zIndex,
      dirty: true, // Initially dirty
    };

    this.layers.set(id, layer);

    // Append to container
    this.container.appendChild(canvas);

    console.log(`[CanvasLayers] Created layer: ${id} (z-index: ${zIndex})`);
    return context;
  }

  /**
   * Get layer by ID
   */
  getLayer(id: string): Layer | undefined {
    return this.layers.get(id);
  }

  /**
   * Get layer context
   */
  getContext(id: string): CanvasRenderingContext2D | undefined {
    return this.layers.get(id)?.context;
  }

  /**
   * Mark layer as dirty (needs redraw)
   */
  markDirty(id: string): void {
    const layer = this.layers.get(id);
    if (layer) {
      layer.dirty = true;
    }
  }

  /**
   * Mark layer as clean (up to date)
   */
  markClean(id: string): void {
    const layer = this.layers.get(id);
    if (layer) {
      layer.dirty = false;
    }
  }

  /**
   * Check if layer is dirty
   */
  isDirty(id: string): boolean {
    const layer = this.layers.get(id);
    return layer ? layer.dirty : false;
  }

  /**
   * Mark all layers as dirty
   */
  markAllDirty(): void {
    for (const layer of this.layers.values()) {
      layer.dirty = true;
    }
  }

  /**
   * Set layer visibility
   */
  setVisible(id: string, visible: boolean): void {
    const layer = this.layers.get(id);
    if (layer) {
      layer.visible = visible;
      layer.canvas.style.display = visible ? "block" : "none";
    }
  }

  /**
   * Set layer alpha (opacity)
   */
  setAlpha(id: string, alpha: number): void {
    const layer = this.layers.get(id);
    if (layer) {
      layer.alpha = Math.max(0, Math.min(1, alpha));
      layer.canvas.style.opacity = String(layer.alpha);
    }
  }

  /**
   * Clear a specific layer
   */
  clearLayer(id: string): void {
    const layer = this.layers.get(id);
    if (layer) {
      layer.context.clearRect(0, 0, layer.canvas.width, layer.canvas.height);
    }
  }

  /**
   * Clear all layers
   */
  clearAll(): void {
    for (const layer of this.layers.values()) {
      layer.context.clearRect(0, 0, layer.canvas.width, layer.canvas.height);
    }
  }

  /**
   * Resize all layers
   */
  resize(width: number, height: number): void {
    this.width = width;
    this.height = height;

    for (const layer of this.layers.values()) {
      this.setCanvasDimensions(layer.canvas, width, height);
    }

    // Mark all layers dirty after resize
    this.markAllDirty();

    console.log(`[CanvasLayers] Resized all layers to ${width}x${height}`);
  }

  /**
   * Set canvas dimensions with pixel ratio
   */
  private setCanvasDimensions(
    canvas: HTMLCanvasElement,
    width: number,
    height: number,
  ): void {
    // Set physical size (with pixel ratio)
    canvas.width = width * this.pixelRatio;
    canvas.height = height * this.pixelRatio;

    // Set CSS size (without pixel ratio)
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;

    // Scale context to account for pixel ratio
    const ctx = canvas.getContext("2d");
    if (ctx) {
      ctx.scale(this.pixelRatio, this.pixelRatio);
    }
  }

  /**
   * Get list of layers sorted by z-index
   */
  getLayersSorted(): Layer[] {
    return Array.from(this.layers.values()).sort((a, b) => a.zIndex - b.zIndex);
  }

  /**
   * Remove a layer
   */
  removeLayer(id: string): void {
    const layer = this.layers.get(id);
    if (layer) {
      layer.canvas.remove();
      this.layers.delete(id);
      console.log(`[CanvasLayers] Removed layer: ${id}`);
    }
  }

  /**
   * Remove all layers
   */
  removeAll(): void {
    for (const layer of this.layers.values()) {
      layer.canvas.remove();
    }
    this.layers.clear();
    console.log("[CanvasLayers] Removed all layers");
  }

  /**
   * Get rendering statistics
   */
  getStats(): {
    totalLayers: number;
    visibleLayers: number;
    dirtyLayers: number;
  } {
    let visible = 0;
    let dirty = 0;

    for (const layer of this.layers.values()) {
      if (layer.visible) visible++;
      if (layer.dirty) dirty++;
    }

    return {
      totalLayers: this.layers.size,
      visibleLayers: visible,
      dirtyLayers: dirty,
    };
  }

  /**
   * Cleanup
   */
  destroy(): void {
    this.removeAll();
    console.log("[CanvasLayers] Destroyed");
  }
}
