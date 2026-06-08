/**
 * WASM Bridge — taps window events emitted by the Rust/realtime layer and
 * writes them into the Svelte stores in src/stores/chart.ts.
 *
 * Responsibilities:
 *   - Subscribe to all WASM custom events exactly once
 *   - Normalise the raw server candle format to the internal Candle shape
 *   - Update stores; no chart side-effects (those stay in app-rust.ts)
 *
 * Call initWasmBridge() once, early in the app lifecycle (before realtime
 * events can fire). Idempotent — safe to call multiple times.
 */

import {
  connectionStatus,
  lastCandle,
  candles,
  candleCount,
  type OhlcvCandle,
} from '../stores/chart'
import type { ConnectionStatus } from '@loom/shared'

let initialized = false

/** Normalise the server/API wire format to the chart-internal OHLCV shape. */
export function convertCandle(c: {
  ts?: string | null
  o: number
  h: number
  l: number
  c: number
  v?: number
}): OhlcvCandle {
  return {
    time: c.ts ? Math.floor(new Date(c.ts).getTime() / 1000) : 0,
    o: c.o,
    h: c.h,
    l: c.l,
    c: c.c,
    v: c.v ?? 0,
  }
}

export function initWasmBridge(): void {
  if (initialized) return
  initialized = true

  window.addEventListener(
    'connectionStatusChanged',
    ((e: CustomEvent<{ status: ConnectionStatus }>) => {
      connectionStatus.set(e.detail.status)
    }) as EventListener,
  )

  window.addEventListener(
    'candleSnapshot',
    ((e: CustomEvent<{ candles: any[] }>) => {
      const converted = e.detail.candles.map(convertCandle)
      candles.set(converted)
      lastCandle.set(converted.at(-1) ?? null)
      candleCount.set(converted.length)
    }) as EventListener,
  )

  window.addEventListener(
    'candleBackfill',
    ((e: CustomEvent<{ candles: any[] }>) => {
      const prepended = e.detail.candles.map(convertCandle)
      candles.update((existing) => [...prepended, ...existing])
      candleCount.update((n) => n + prepended.length)
    }) as EventListener,
  )
}

/** Reset for testing. Not needed in production. */
export function _resetWasmBridge(): void {
  initialized = false
}
