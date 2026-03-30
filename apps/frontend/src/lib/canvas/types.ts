/**
 * Canvas DPI Management - TypeScript Types
 *
 * Type-safe coordinate space types matching the Rust implementation.
 * Prevents mixing CSS pixels with device pixels at compile time.
 */

/**
 * CSS Pixels - coordinates in DOM/layout space
 * Used for: mouse events, element positioning, logical dimensions
 */
export type CssPixels = number & { readonly __brand: 'CssPixels' };

/**
 * Device Pixels - coordinates in canvas bitmap space
 * Used for: actual canvas rendering, physical pixel operations
 */
export type DevicePixels = number & { readonly __brand: 'DevicePixels' };

/**
 * Pixel Ratio - ratio between device pixels and CSS pixels
 * Common values: 1.0 (standard), 1.5, 2.0 (Retina), 3.0 (high-DPI mobile)
 */
export type PixelRatio = number & { readonly __brand: 'PixelRatio' };

/**
 * Coordinate Scope - identifies which coordinate system is being used
 */
export enum CoordinateScope {
  /** CSS pixel coordinates (layout space) */
  Media = 'media',
  /** Device pixel coordinates (bitmap space) */
  Bitmap = 'bitmap',
}

/**
 * Size in a specific coordinate space
 */
export interface Size<T extends CssPixels | DevicePixels = CssPixels> {
  width: T;
  height: T;
}

/**
 * Point in a specific coordinate space
 */
export interface Point<T extends CssPixels | DevicePixels = CssPixels> {
  x: T;
  y: T;
}

/**
 * Rectangle in a specific coordinate space
 */
export interface Rect<T extends CssPixels | DevicePixels = CssPixels> {
  x: T;
  y: T;
  width: T;
  height: T;
}

// Helper functions to create type-safe values
export const cssPixels = (value: number): CssPixels => value as CssPixels;
export const devicePixels = (value: number): DevicePixels => value as DevicePixels;
export const pixelRatio = (value: number): PixelRatio => Math.max(1.0, value) as PixelRatio;

/**
 * Get the current device pixel ratio
 */
export function getDevicePixelRatio(): PixelRatio {
  return pixelRatio(window.devicePixelRatio || 1.0);
}

/**
 * Convert CSS pixels to device pixels
 */
export function toDevicePixels(css: CssPixels, ratio: PixelRatio): DevicePixels {
  return devicePixels(css * ratio);
}

/**
 * Convert device pixels to CSS pixels
 */
export function toCssPixels(device: DevicePixels, ratio: PixelRatio): CssPixels {
  return cssPixels(device / ratio);
}

/**
 * Convert a size from CSS to device pixels
 */
export function toDeviceSize(size: Size<CssPixels>, ratio: PixelRatio): Size<DevicePixels> {
  return {
    width: toDevicePixels(size.width, ratio),
    height: toDevicePixels(size.height, ratio),
  };
}

/**
 * Convert a size from device to CSS pixels
 */
export function toCssSize(size: Size<DevicePixels>, ratio: PixelRatio): Size<CssPixels> {
  return {
    width: toCssPixels(size.width, ratio),
    height: toCssPixels(size.height, ratio),
  };
}

/**
 * Convert a point from CSS to device pixels
 */
export function toDevicePoint(point: Point<CssPixels>, ratio: PixelRatio): Point<DevicePixels> {
  return {
    x: toDevicePixels(point.x, ratio),
    y: toDevicePixels(point.y, ratio),
  };
}

/**
 * Convert a point from device to CSS pixels
 */
export function toCssPoint(point: Point<DevicePixels>, ratio: PixelRatio): Point<CssPixels> {
  return {
    x: toCssPixels(point.x, ratio),
    y: toCssPixels(point.y, ratio),
  };
}
