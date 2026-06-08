/**
 * Crosshair/Viewport Bridge — a requestAnimationFrame loop that polls the
 * WASM-only `getCrosshairInfo()` / `getViewportInfo()` getters on the global
 * `window.rustChart` and writes the results into the Svelte stores
 * (src/stores/chart.ts), giving overlays a reactive data source instead of
 * direct DOM manipulation.
 *
 * Lifecycle:
 *   - initCrosshairBridge() waits for the `rustChartReady` event, then starts
 *     the rAF loop. Idempotent.
 *   - destroyCrosshairBridge() cancels the rAF and detaches the listener.
 *
 * Performance: the loop yields when the document is hidden, and only writes a
 * store when the polled value actually changed (cheap JSON compare), so idle
 * frames cost nothing downstream.
 */

import { crosshairInfo, viewportInfo } from '../stores/chart'
import type { RustChart } from './rust-chart'

let rafId: number | null = null
let listenerAttached = false
let lastCrosshair = ''
let lastViewport = ''

function getChart(): RustChart | null {
  return ((window as any).rustChart as RustChart | undefined) ?? null
}

function tick(): void {
  rafId = null

  // Pause work while the tab is hidden — nothing to render, no reason to poll.
  if (typeof document !== 'undefined' && document.hidden) {
    schedule()
    return
  }

  const chart = getChart()
  if (chart) {
    const crosshair = chart.getCrosshairInfo()
    const crosshairKey = crosshair ? JSON.stringify(crosshair) : ''
    if (crosshairKey !== lastCrosshair) {
      lastCrosshair = crosshairKey
      crosshairInfo.set(crosshair)
    }

    const viewport = chart.getViewportInfo()
    const viewportKey = viewport ? JSON.stringify(viewport) : ''
    if (viewportKey !== lastViewport) {
      lastViewport = viewportKey
      viewportInfo.set(viewport)
    }
  }

  schedule()
}

function schedule(): void {
  if (rafId === null && typeof requestAnimationFrame !== 'undefined') {
    rafId = requestAnimationFrame(tick)
  }
}

function start(): void {
  if (rafId !== null) return
  schedule()
}

export function initCrosshairBridge(): void {
  // If the chart is already up, start immediately; otherwise wait for the event.
  if (getChart()) {
    start()
    return
  }
  if (listenerAttached) return
  listenerAttached = true
  window.addEventListener('rustChartReady', start, { once: true })
}

export function destroyCrosshairBridge(): void {
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
  if (listenerAttached) {
    window.removeEventListener('rustChartReady', start)
    listenerAttached = false
  }
  lastCrosshair = ''
  lastViewport = ''
}
