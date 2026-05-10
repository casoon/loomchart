import { RustChart } from "./rust-chart";
import type { Candle as SharedCandle } from "@loom/shared";

const charts = new Map<string, RustChart>();

function sharedToChartCandle(c: SharedCandle) {
  return {
    time: Math.floor(new Date(c.ts).getTime() / 1000),
    o: c.o,
    h: c.h,
    l: c.l,
    c: c.c,
    v: c.v,
  };
}

export async function initChartBridge(containerId: string): Promise<void> {
  const container = document.getElementById(containerId);
  if (!container) throw new Error(`Container #${containerId} not found`);

  if (charts.has(containerId)) {
    charts.get(containerId)!.destroy();
    charts.delete(containerId);
  }

  const chart = new RustChart(container);
  charts.set(containerId, chart);
  await chart.initialize();
}

export function updateChartCandles(containerId: string, candles: SharedCandle[]): void {
  const chart = charts.get(containerId);
  if (!chart) return;
  chart.setCandles(candles.map(sharedToChartCandle));
}

export function disposeChart(containerId: string): void {
  const chart = charts.get(containerId);
  if (!chart) return;
  chart.destroy();
  charts.delete(containerId);
}
