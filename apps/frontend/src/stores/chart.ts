import { writable, derived } from 'svelte/store'
import type { ConnectionStatus } from '@loom/shared'
import type { CrosshairInfo, ViewportInfo } from '../lib/rust-chart'
import type { IndicatorInstance } from '../lib/indicators/types'

// WASM-facing reactive state — the single source of truth for data that flows from
// the Rust/WASM layer into the UI. Alpine mirrors these via subscriptions in app-rust.ts;
// future Svelte components can consume them directly with the $store syntax.

/** Chart-internal OHLCV candle. Not the same as @loom/shared Candle (which carries wire metadata). */
export interface OhlcvCandle {
  time: number // unix seconds
  o: number
  h: number
  l: number
  c: number
  v: number
}

export const connectionStatus = writable<ConnectionStatus>('disconnected')
export const lastCandle = writable<OhlcvCandle | null>(null)
export const candles = writable<OhlcvCandle[]>([])
export const candleCount = writable<number>(0)

export const priceChangePercent = derived(
  lastCandle,
  ($c) => ($c ? (($c.c - $c.o) / $c.o) * 100 : 0),
)

export const isLive = derived(
  connectionStatus,
  ($s) => $s === 'connected' || $s === 'syncing',
)

// Crosshair/viewport — fed by the rAF polling loop in lib/crosshair-bridge.ts.
// Null when the pointer is not over the chart.
export const crosshairInfo = writable<CrosshairInfo | null>(null)
export const viewportInfo = writable<ViewportInfo | null>(null)

// Active indicators — fed by lib/indicator-bridge.ts from the indicator-* window events.
export const activeIndicators = writable<IndicatorInstance[]>([])

export const hasActiveIndicators = derived(
  activeIndicators,
  ($i) => $i.length > 0,
)
