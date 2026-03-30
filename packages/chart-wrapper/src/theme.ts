import type { DeepPartial, ChartOptions } from 'lightweight-charts';
import { ColorType } from 'lightweight-charts';

export interface ChartTheme {
  backgroundColor: string;
  textColor: string;
  gridColor: string;
  borderColor: string;
  upColor: string;
  downColor: string;
  crosshairColor: string;
}

export const CHART_THEMES: Record<'dark' | 'light', ChartTheme> = {
  dark: {
    backgroundColor: '#0f1419',
    textColor: '#8b98a5',
    gridColor: '#1a1f26',
    borderColor: '#2d3640',
    upColor: '#26a69a',
    downColor: '#ef5350',
    crosshairColor: '#758696',
  },
  light: {
    backgroundColor: '#ffffff',
    textColor: '#333333',
    gridColor: '#f0f0f0',
    borderColor: '#e0e0e0',
    upColor: '#26a69a',
    downColor: '#ef5350',
    crosshairColor: '#758696',
  },
};

export function createChartTheme(theme: 'dark' | 'light' = 'dark'): DeepPartial<ChartOptions> {
  const t = CHART_THEMES[theme];

  return {
    layout: {
      background: { type: ColorType.Solid, color: t.backgroundColor },
      textColor: t.textColor,
    },
    grid: {
      vertLines: { color: t.gridColor },
      horzLines: { color: t.gridColor },
    },
    crosshair: {
      mode: 1, // Normal
      vertLine: {
        color: t.crosshairColor,
        width: 1,
        style: 3, // Dashed
        labelBackgroundColor: t.borderColor,
      },
      horzLine: {
        color: t.crosshairColor,
        width: 1,
        style: 3,
        labelBackgroundColor: t.borderColor,
      },
    },
    timeScale: {
      borderColor: t.borderColor,
      timeVisible: true,
      secondsVisible: false,
    },
    rightPriceScale: {
      borderColor: t.borderColor,
    },
    leftPriceScale: {
      borderColor: t.borderColor,
    },
  };
}
