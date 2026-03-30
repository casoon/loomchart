import type { ChartCandle, ChartLinePoint, ChartMarker } from '@loom/shared';

export interface ChartConfig {
  containerId: string;
  width?: number;
  height?: number;
  theme?: 'dark' | 'light';
  autoSize?: boolean;
}

export interface SeriesConfig {
  id: string;
  type: 'candlestick' | 'line' | 'histogram';
  color?: string;
  lineWidth?: number;
  priceScaleId?: 'left' | 'right';
  visible?: boolean;
}

export interface LineSeriesConfig extends SeriesConfig {
  type: 'line';
  color: string;
  lineWidth?: number;
}

export interface HistogramSeriesConfig extends SeriesConfig {
  type: 'histogram';
  baseValue?: number;
}

export interface UpdateCandleData {
  seriesId: string;
  candle: ChartCandle;
}

export interface UpdateLineData {
  seriesId: string;
  point: ChartLinePoint;
}

export interface MarkerData {
  seriesId: string;
  marker: ChartMarker;
}

// Re-export shared types for convenience
export type { ChartCandle, ChartLinePoint, ChartMarker };
