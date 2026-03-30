/**
 * Canvas Resize Handler
 *
 * Handles canvas resizing with device-pixel-content-box support for accurate DPI handling.
 * Uses ResizeObserver with device-pixel-content-box when available, falls back to regular resize.
 */

import type { CssPixels, DevicePixels, PixelRatio, Size } from './types';
import { cssPixels, devicePixels, getDevicePixelRatio } from './types';

export interface ResizeInfo {
  /** Size in CSS pixels (logical size) */
  cssSize: Size<CssPixels>;
  /** Size in device pixels (physical size) */
  deviceSize: Size<DevicePixels>;
  /** Current pixel ratio */
  pixelRatio: PixelRatio;
}

export type ResizeCallback = (info: ResizeInfo) => void;

/**
 * Canvas Resize Handler with device-pixel-content-box support
 */
export class CanvasResizeHandler {
  private observer: ResizeObserver | null = null;
  private canvas: HTMLCanvasElement;
  private callback: ResizeCallback;
  private supportsDevicePixelContentBox: boolean = false;

  constructor(canvas: HTMLCanvasElement, callback: ResizeCallback) {
    this.canvas = canvas;
    this.callback = callback;

    // Check if device-pixel-content-box is supported
    this.supportsDevicePixelContentBox = this.checkDevicePixelContentBoxSupport();
  }

  /**
   * Check if ResizeObserver supports device-pixel-content-box
   */
  private checkDevicePixelContentBoxSupport(): boolean {
    if (typeof ResizeObserver === 'undefined') {
      return false;
    }

    // Try to create a test observer with device-pixel-content-box
    try {
      const testObserver = new ResizeObserver(() => {});
      testObserver.observe(document.body, { box: 'device-pixel-content-box' });
      testObserver.disconnect();
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Start observing canvas size changes
   */
  start(): void {
    if (typeof ResizeObserver === 'undefined') {
      console.warn('[CanvasResizeHandler] ResizeObserver not supported, using fallback');
      this.startFallback();
      return;
    }

    this.observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        this.handleResize(entry);
      }
    });

    // Use device-pixel-content-box if supported for accurate device pixel dimensions
    const options = this.supportsDevicePixelContentBox
      ? { box: 'device-pixel-content-box' as ResizeObserverBoxOptions }
      : undefined;

    this.observer.observe(this.canvas, options);

    // Trigger initial resize
    this.triggerInitialResize();
  }

  /**
   * Handle resize observer entry
   */
  private handleResize(entry: ResizeObserverEntry): void {
    let deviceWidth: DevicePixels;
    let deviceHeight: DevicePixels;
    let cssWidth: CssPixels;
    let cssHeight: CssPixels;

    if (this.supportsDevicePixelContentBox && entry.devicePixelContentBoxSize?.length) {
      // Use device-pixel-content-box for accurate device pixel dimensions
      const deviceBox = entry.devicePixelContentBoxSize[0];
      deviceWidth = devicePixels(deviceBox.inlineSize);
      deviceHeight = devicePixels(deviceBox.blockSize);

      // Calculate CSS size from device pixels
      const ratio = getDevicePixelRatio();
      cssWidth = cssPixels(deviceWidth / ratio);
      cssHeight = cssPixels(deviceHeight / ratio);
    } else if (entry.contentBoxSize?.length) {
      // Fallback to content-box (CSS pixels)
      const contentBox = entry.contentBoxSize[0];
      cssWidth = cssPixels(contentBox.inlineSize);
      cssHeight = cssPixels(contentBox.blockSize);

      // Calculate device pixels from CSS pixels
      const ratio = getDevicePixelRatio();
      deviceWidth = devicePixels(cssWidth * ratio);
      deviceHeight = devicePixels(cssHeight * ratio);
    } else {
      // Older browsers fallback
      const rect = this.canvas.getBoundingClientRect();
      cssWidth = cssPixels(rect.width);
      cssHeight = cssPixels(rect.height);

      const ratio = getDevicePixelRatio();
      deviceWidth = devicePixels(cssWidth * ratio);
      deviceHeight = devicePixels(cssHeight * ratio);
    }

    const pixelRatio = getDevicePixelRatio();

    this.callback({
      cssSize: { width: cssWidth, height: cssHeight },
      deviceSize: { width: deviceWidth, height: deviceHeight },
      pixelRatio,
    });
  }

  /**
   * Trigger initial resize to set canvas size
   */
  private triggerInitialResize(): void {
    const rect = this.canvas.getBoundingClientRect();
    const cssWidth = cssPixels(rect.width);
    const cssHeight = cssPixels(rect.height);
    const ratio = getDevicePixelRatio();
    const deviceWidth = devicePixels(cssWidth * ratio);
    const deviceHeight = devicePixels(cssHeight * ratio);

    this.callback({
      cssSize: { width: cssWidth, height: cssHeight },
      deviceSize: { width: deviceWidth, height: deviceHeight },
      pixelRatio: ratio,
    });
  }

  /**
   * Fallback for browsers without ResizeObserver
   */
  private startFallback(): void {
    // Use window resize event as fallback
    const handleWindowResize = () => {
      this.triggerInitialResize();
    };

    window.addEventListener('resize', handleWindowResize);

    // Initial trigger
    this.triggerInitialResize();

    // Store cleanup function
    this.stop = () => {
      window.removeEventListener('resize', handleWindowResize);
    };
  }

  /**
   * Stop observing canvas size changes
   */
  stop(): void {
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }
  }
}
