import type { Candle, ChartCandle, Timeframe } from "../types/index.js";

/**
 * Convert ISO timestamp to bucket start based on timeframe
 */
export function alignToBucket(ts: string | Date, timeframe: Timeframe): Date {
  const date = typeof ts === "string" ? new Date(ts) : ts;
  const ms = date.getTime();
  const bucketMs = timeframeToDurationMs(timeframe);
  return new Date(Math.floor(ms / bucketMs) * bucketMs);
}

/**
 * Get duration in milliseconds for a timeframe
 */
export function timeframeToDurationMs(tf: Timeframe): number {
  const durations: Record<Timeframe, number> = {
    "1m": 60 * 1000,
    "5m": 5 * 60 * 1000,
    "15m": 15 * 60 * 1000,
    "30m": 30 * 60 * 1000,
    "1h": 60 * 60 * 1000,
    "4h": 4 * 60 * 60 * 1000,
    "1d": 24 * 60 * 60 * 1000,
    "1w": 7 * 24 * 60 * 60 * 1000,
  };
  return durations[tf];
}

/**
 * Convert Candle to ChartCandle format (for chart library)
 */
export function candleToChartCandle(candle: Candle): ChartCandle {
  return {
    time: new Date(candle.ts).getTime() / 1000, // unix seconds
    open: candle.o,
    high: candle.h,
    low: candle.l,
    close: candle.c,
    volume: candle.v,
  };
}

/**
 * Generate channel topic name
 */
export function makeChannelTopic(source: string, symbol: string, tf: Timeframe): string {
  return `candles:${source}:${symbol}:${tf}`;
}

/**
 * Parse channel topic
 */
export function parseChannelTopic(topic: string): { source: string; symbol: string; tf: Timeframe } | null {
  const match = topic.match(/^candles:([^:]+):([^:]+):([^:]+)$/);
  if (!match) return null;
  return {
    source: match[1],
    symbol: match[2],
    tf: match[3] as Timeframe,
  };
}

/**
 * Exponential backoff delay calculator
 */
export function calculateBackoffDelay(attempt: number, baseMs = 1000, maxMs = 30000): number {
  const delay = Math.min(baseMs * Math.pow(2, attempt), maxMs);
  // Add jitter (0-25% of delay)
  return delay + Math.random() * delay * 0.25;
}

/**
 * Format price with appropriate decimal places
 */
export function formatPrice(price: number, decimals = 5): string {
  return price.toFixed(decimals);
}

/**
 * Validate candle data
 */
export function isValidCandle(candle: Partial<Candle>): candle is Candle {
  return (
    typeof candle.source === "string" &&
    typeof candle.symbol === "string" &&
    typeof candle.tf === "string" &&
    typeof candle.ts === "string" &&
    typeof candle.o === "number" &&
    typeof candle.h === "number" &&
    typeof candle.l === "number" &&
    typeof candle.c === "number" &&
    typeof candle.v === "number" &&
    typeof candle.is_final === "boolean"
  );
}
