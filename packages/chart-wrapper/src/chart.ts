import {
  createChart,
  type IChartApi,
  type ISeriesApi,
  type CandlestickSeriesOptions,
  type LineSeriesOptions,
  type HistogramSeriesOptions,
  type SeriesMarker,
  type Time,
} from 'lightweight-charts';

import { CHART_COLORS } from '@loom/shared';
import type {
  ChartConfig,
  SeriesConfig,
  ChartCandle,
  ChartLinePoint,
  ChartMarker,
} from './types.js';
import { createChartTheme, CHART_THEMES } from './theme.js';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnySeries = ISeriesApi<any>;

/**
 * LoomChart - Wrapper for lightweight-charts
 *
 * Provides a simple imperative API for WASM to control chart rendering:
 * - createChart(config)
 * - setCandles(seriesId, candles[])
 * - updateCandle(seriesId, candle)
 * - setLineSeries(seriesId, points[])
 * - updateLinePoint(seriesId, point)
 * - addMarker(seriesId, marker)
 */
export class LoomChart {
  private chart: IChartApi | null = null;
  private series: Map<string, AnySeries> = new Map();
  private container: HTMLElement | null = null;
  private resizeObserver: ResizeObserver | null = null;

  /**
   * Create and initialize chart
   */
  create(config: ChartConfig): void {
    const container = document.getElementById(config.containerId);
    if (!container) {
      throw new Error(`Container not found: ${config.containerId}`);
    }

    this.container = container;
    const theme = config.theme || 'dark';
    const themeOptions = createChartTheme(theme);

    this.chart = createChart(container, {
      ...themeOptions,
      width: config.width || container.clientWidth,
      height: config.height || container.clientHeight,
      handleScale: {
        axisPressedMouseMove: true,
      },
      handleScroll: {
        mouseWheel: true,
        pressedMouseMove: true,
        horzTouchDrag: true,
        vertTouchDrag: true,
      },
    });

    // Auto-resize
    if (config.autoSize !== false) {
      this.resizeObserver = new ResizeObserver(entries => {
        for (const entry of entries) {
          if (entry.target === container && this.chart) {
            this.chart.applyOptions({
              width: entry.contentRect.width,
              height: entry.contentRect.height,
            });
          }
        }
      });
      this.resizeObserver.observe(container);
    }
  }

  /**
   * Add a candlestick series
   */
  addCandlestickSeries(id: string, options?: Partial<CandlestickSeriesOptions>): void {
    if (!this.chart) throw new Error('Chart not initialized');

    const theme = CHART_THEMES.dark;
    const series = this.chart.addCandlestickSeries({
      upColor: theme.upColor,
      downColor: theme.downColor,
      borderUpColor: theme.upColor,
      borderDownColor: theme.downColor,
      wickUpColor: theme.upColor,
      wickDownColor: theme.downColor,
      ...options,
    });

    this.series.set(id, series);
  }

  /**
   * Add a line series (for indicators like EMA)
   */
  addLineSeries(id: string, options?: Partial<LineSeriesOptions>): void {
    if (!this.chart) throw new Error('Chart not initialized');

    const series = this.chart.addLineSeries({
      color: CHART_COLORS.ema21,
      lineWidth: 1,
      crosshairMarkerVisible: false,
      lastValueVisible: false,
      priceLineVisible: false,
      ...options,
    });

    this.series.set(id, series);
  }

  /**
   * Add a histogram series (for MACD histogram)
   */
  addHistogramSeries(id: string, options?: Partial<HistogramSeriesOptions>): void {
    if (!this.chart) throw new Error('Chart not initialized');

    const series = this.chart.addHistogramSeries({
      color: CHART_COLORS.macdHistPositive,
      priceLineVisible: false,
      lastValueVisible: false,
      ...options,
    });

    this.series.set(id, series);
  }

  /**
   * Set candle data for a series
   */
  setCandles(seriesId: string, candles: ChartCandle[]): void {
    const series = this.getSeries(seriesId);
    const data = candles.map(c => ({
      time: c.time as Time,
      open: c.open,
      high: c.high,
      low: c.low,
      close: c.close,
    }));
    series.setData(data);
  }

  /**
   * Update single candle (for live updates)
   */
  updateCandle(seriesId: string, candle: ChartCandle): void {
    const series = this.getSeries(seriesId);
    series.update({
      time: candle.time as Time,
      open: candle.open,
      high: candle.high,
      low: candle.low,
      close: candle.close,
    });
  }

  /**
   * Set line series data
   */
  setLineSeries(seriesId: string, points: ChartLinePoint[]): void {
    const series = this.getSeries(seriesId);
    const data = points.map(p => ({
      time: p.time as Time,
      value: p.value,
    }));
    series.setData(data);
  }

  /**
   * Update single point on line series
   */
  updateLinePoint(seriesId: string, point: ChartLinePoint): void {
    const series = this.getSeries(seriesId);
    series.update({
      time: point.time as Time,
      value: point.value,
    });
  }

  /**
   * Add marker to series
   */
  addMarker(seriesId: string, marker: ChartMarker): void {
    const series = this.getSeries(seriesId);
    const markers = (series as ISeriesApi<'Candlestick'>).markers() || [];

    const newMarker: SeriesMarker<Time> = {
      time: marker.time as Time,
      position: marker.position,
      color: marker.color,
      shape: marker.shape,
      text: marker.text,
    };

    markers.push(newMarker);
    (series as ISeriesApi<'Candlestick'>).setMarkers(markers);
  }

  /**
   * Clear all markers from series
   */
  clearMarkers(seriesId: string): void {
    const series = this.getSeries(seriesId);
    (series as ISeriesApi<'Candlestick'>).setMarkers([]);
  }

  /**
   * Remove a series
   */
  removeSeries(seriesId: string): void {
    const series = this.series.get(seriesId);
    if (series && this.chart) {
      this.chart.removeSeries(series);
      this.series.delete(seriesId);
    }
  }

  /**
   * Show/hide series
   */
  setSeriesVisible(seriesId: string, visible: boolean): void {
    const series = this.getSeries(seriesId);
    series.applyOptions({ visible });
  }

  /**
   * Scroll to latest data
   */
  scrollToRealTime(): void {
    this.chart?.timeScale().scrollToRealTime();
  }

  /**
   * Fit content to view
   */
  fitContent(): void {
    this.chart?.timeScale().fitContent();
  }

  /**
   * Destroy chart and cleanup
   */
  destroy(): void {
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = null;
    }

    if (this.chart) {
      this.chart.remove();
      this.chart = null;
    }

    this.series.clear();
    this.container = null;
  }

  private getSeries(id: string): AnySeries {
    const series = this.series.get(id);
    if (!series) {
      throw new Error(`Series not found: ${id}`);
    }
    return series;
  }
}
