/**
 * Notification Bridge — taps the `showToast` window event and pushes onto the
 * central notification store in src/stores/notifications.ts.
 *
 * Existing `window.dispatchEvent(new CustomEvent('showToast', ...))` callers stay
 * unchanged; the bridge is purely additive. Call initNotificationBridge() once,
 * early in the app lifecycle. Idempotent — safe to call multiple times.
 */

import { push, type ToastType } from '../stores/notifications'

let initialized = false

interface ShowToastDetail {
  type?: ToastType
  message: string
  duration?: number
}

export function initNotificationBridge(): void {
  if (initialized) return
  initialized = true

  window.addEventListener(
    'showToast',
    ((e: CustomEvent<ShowToastDetail>) => {
      const detail = e.detail
      if (!detail || typeof detail.message !== 'string') return
      push({
        type: detail.type ?? 'info',
        message: detail.message,
        duration: detail.duration,
      })
    }) as EventListener,
  )
}

/** Reset for testing. Not needed in production. */
export function _resetNotificationBridge(): void {
  initialized = false
}
