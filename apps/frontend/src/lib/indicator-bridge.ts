/**
 * Indicator Bridge — taps the indicator window events emitted by
 * lib/indicators/chart-integration.ts and mirrors the active-indicator set into
 * the `activeIndicators` Svelte store (src/stores/chart.ts).
 *
 * Events:
 *   - indicator-added    { instance }              → append
 *   - indicator-removed  { instanceId }            → drop
 *   - indicator-updated  { instanceId, value, ... } → patch lastValue in place
 *
 * Call initIndicatorBridge() once after WASM init. Idempotent — safe to call
 * multiple times. Hydrates the store from the current integration state so a
 * reload with persisted indicators is reflected immediately.
 */

import { activeIndicators } from '../stores/chart'
import { getActiveIndicators } from './indicators/chart-integration'
import type { IndicatorInstance } from './indicators/types'

let initialized = false

export function initIndicatorBridge(): void {
  if (initialized) return
  initialized = true

  // Hydrate from whatever the integration already holds (restored state).
  activeIndicators.set(getActiveIndicators())

  window.addEventListener(
    'indicator-added',
    ((e: CustomEvent<{ instance: IndicatorInstance }>) => {
      const instance = e.detail?.instance
      if (!instance) return
      activeIndicators.update((list) => {
        if (list.some((i) => i.instanceId === instance.instanceId)) {
          return list.map((i) =>
            i.instanceId === instance.instanceId ? instance : i,
          )
        }
        return [...list, instance]
      })
    }) as EventListener,
  )

  window.addEventListener(
    'indicator-removed',
    ((e: CustomEvent<{ instanceId: string }>) => {
      const instanceId = e.detail?.instanceId
      if (!instanceId) return
      activeIndicators.update((list) =>
        list.filter((i) => i.instanceId !== instanceId),
      )
    }) as EventListener,
  )

  window.addEventListener(
    'indicator-updated',
    ((e: CustomEvent<{ instanceId: string; value: number }>) => {
      const { instanceId, value } = e.detail ?? {}
      if (!instanceId) return
      // Patch the existing entry in place — no full replace.
      activeIndicators.update((list) =>
        list.map((i) =>
          i.instanceId === instanceId ? { ...i, lastValue: value } : i,
        ),
      )
    }) as EventListener,
  )
}

/** Reset for testing. Not needed in production. */
export function _resetIndicatorBridge(): void {
  initialized = false
}
