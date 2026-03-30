/**
 * Canvas Binding
 *
 * Manages canvas lifecycle with DPI-aware rendering.
 * Handles resize, pixel ratio changes, and WASM chart integration.
 */

import type { CssPixels, DevicePixels, PixelRatio, Size } from './types';
import { CanvasResizeHandler, type ResizeInfo } from './resize-handler';
import { cssPixels, devicePixels, getDevicePixelRatio } from './types';

export interface CanvasBindingConfig {
  /** Canvas element */
  canvas: HTMLCanvasElement;
  /** WASM chart instance */
  wasmChart: any;
  /** Called when canvas is resized */
  onResize?: (info: ResizeInfo) => void;
  /** Called when pixel ratio changes */
  onPixelRatioChange?: (ratio: PixelRatio) => void;
}

/**
 * Canvas Binding - manages canvas lifecycle with DPI awareness
 */
export class CanvasBinding {
  private canvas: HTMLCanvasElement;
  private wasmChart: any;
  private resizeHandler: CanvasResizeHandler;
  private currentPixelRatio: PixelRatio;
  private config: CanvasBindingConfig;
  private mediaQueryList: MediaQueryList | null = null;

  constructor(config: CanvasBindingConfig) {
    this.config = config;
    this.canvas = config.canvas;
    this.wasmChart = config.wasmChart;
    this.currentPixelRatio = getDevicePixelRatio();

    // Create resize handler
    this.resizeHandler = new CanvasResizeHandler(this.canvas, (info) => {
      this.handleResize(info);
    });

    // Watch for pixel ratio changes (e.g., moving window between displays)
    this.setupPixelRatioChangeDetection();
  }

  /**
   * Initialize the canvas binding and start observing
   */
  initialize(): void {
    console.log('[CanvasBinding] Initializing with pixel ratio:', this.currentPixelRatio);

    // Start resize observation
    this.resizeHandler.start();
  }

  /**
   * Handle canvas resize
   */
  private handleResize(info: ResizeInfo): void {
    console.log('[CanvasBinding] Resize:', {
      css: `${info.cssSize.width}x${info.cssSize.height}`,
      device: `${info.deviceSize.width}x${info.deviceSize.height}`,
      ratio: info.pixelRatio,
    });

    // Check if pixel ratio changed
    if (info.pixelRatio !== this.currentPixelRatio) {
      this.handlePixelRatioChange(info.pixelRatio);
    }

    // Update canvas physical size (device pixels)
    this.updateCanvasSize(info.cssSize, info.deviceSize, info.pixelRatio);

    // Notify WASM chart
    if (this.wasmChart?.resize) {
      try {
        this.wasmChart.resize(
          Math.floor(info.cssSize.width),
          Math.floor(info.cssSize.height)
        );
      } catch (error) {
        console.error('[CanvasBinding] Failed to resize WASM chart:', error);
      }
    }

    // Call user callback
    if (this.config.onResize) {
      this.config.onResize(info);
    }
  }

  /**
   * Update canvas size with proper DPI scaling
   */
  private updateCanvasSize(
    cssSize: Size<CssPixels>,
    deviceSize: Size<DevicePixels>,
    pixelRatio: PixelRatio
  ): void {
    // Set canvas internal resolution (device pixels)
    this.canvas.width = Math.floor(deviceSize.width);
    this.canvas.height = Math.floor(deviceSize.height);

    // CSS size is controlled by container, don't set it here
    // to avoid ResizeObserver loops

    // Get 2D context and scale
    const ctx = this.canvas.getContext('2d');
    if (ctx) {
      // Reset transform
      ctx.setTransform(1, 0, 0, 1, 0, 0);

      // Scale context to match pixel ratio
      // This allows drawing in CSS pixels while rendering at device pixels
      ctx.scale(pixelRatio, pixelRatio);
    }
  }

  /**
   * Setup detection for pixel ratio changes (e.g., moving between displays)
   */
  private setupPixelRatioChangeDetection(): void {
    // Use matchMedia to detect devicePixelRatio changes
    if (!window.matchMedia) {
      return;
    }

    const updatePixelRatio = () => {
      const newRatio = getDevicePixelRatio();
      if (newRatio !== this.currentPixelRatio) {
        this.handlePixelRatioChange(newRatio);
      }
    };

    // Create media query for current pixel ratio
    const createMediaQuery = (ratio: PixelRatio) => {
      // Remove old listener
      if (this.mediaQueryList) {
        this.mediaQueryList.removeEventListener?.('change', updatePixelRatio);
        this.mediaQueryList.removeListener?.(updatePixelRatio); // Fallback for older browsers
      }

      // Create new media query
      const mql = window.matchMedia(`(resolution: ${ratio}dppx)`);

      // Add listener
      if (mql.addEventListener) {
        mql.addEventListener('change', updatePixelRatio);
      } else {
        // Fallback for older browsers
        mql.addListener(updatePixelRatio);
      }

      this.mediaQueryList = mql;
    };

    createMediaQuery(this.currentPixelRatio);
  }

  /**
   * Handle pixel ratio change
   */
  private handlePixelRatioChange(newRatio: PixelRatio): void {
    console.log('[CanvasBinding] Pixel ratio changed:', {
      old: this.currentPixelRatio,
      new: newRatio,
    });

    this.currentPixelRatio = newRatio;

    // Notify user callback
    if (this.config.onPixelRatioChange) {
      this.config.onPixelRatioChange(newRatio);
    }

    // Setup new media query listener
    this.setupPixelRatioChangeDetection();
  }

  /**
   * Get current pixel ratio
   */
  getPixelRatio(): PixelRatio {
    return this.currentPixelRatio;
  }

  /**
   * Get canvas size in CSS pixels
   */
  getCssSize(): Size<CssPixels> {
    const rect = this.canvas.getBoundingClientRect();
    return {
      width: cssPixels(rect.width),
      height: cssPixels(rect.height),
    };
  }

  /**
   * Get canvas size in device pixels
   */
  getDeviceSize(): Size<DevicePixels> {
    return {
      width: devicePixels(this.canvas.width),
      height: devicePixels(this.canvas.height),
    };
  }

  /**
   * Cleanup and stop observing
   */
  destroy(): void {
    console.log('[CanvasBinding] Destroying');

    // Stop resize handler
    this.resizeHandler.stop();

    // Remove pixel ratio change listener
    if (this.mediaQueryList) {
      const updatePixelRatio = () => {}; // Placeholder
      this.mediaQueryList.removeEventListener?.('change', updatePixelRatio);
      this.mediaQueryList.removeListener?.(updatePixelRatio);
      this.mediaQueryList = null;
    }
  }
}
