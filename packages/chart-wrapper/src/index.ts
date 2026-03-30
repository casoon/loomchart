/**
 * @loom/chart-wrapper
 *
 * Thin wrapper around lightweight-charts library.
 * Provides a simple API for WASM to control chart rendering.
 */

export { LoomChart } from './chart.js';
export type { ChartConfig, SeriesConfig, LineSeriesConfig } from './types.js';
export { createChartTheme, CHART_THEMES } from './theme.js';
