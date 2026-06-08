import { writable, derived } from 'svelte/store'

// Central notification store. The single source of truth for toasts that are
// dispatched as `showToast` window events from many places (errorBoundary,
// stateManager, WASM). lib/notification-bridge.ts taps those events and calls
// push(); Svelte components subscribe to `notifications` instead of registering
// their own window listeners.

export type ToastType = 'info' | 'success' | 'warning' | 'error'

export interface Toast {
  id: string
  type: ToastType
  message: string
  /** Auto-dismiss delay in ms. When set, the toast removes itself after the delay. */
  duration?: number
}

/** Input accepted by push() — id is generated, everything else passes through. */
export type ToastInput = Omit<Toast, 'id'> & { id?: string }

export const notifications = writable<Toast[]>([])

export const hasNotifications = derived(notifications, ($n) => $n.length > 0)

let counter = 0
function nextId(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) {
    return crypto.randomUUID()
  }
  counter += 1
  return `toast-${counter}`
}

/** Add a toast. Generates an id and arms an auto-dismiss timer when `duration` is set. */
export function push(toast: ToastInput): string {
  const id = toast.id ?? nextId()
  const entry: Toast = {
    id,
    type: toast.type ?? 'info',
    message: toast.message,
    duration: toast.duration,
  }

  notifications.update((list) => [...list, entry])

  if (entry.duration && entry.duration > 0 && typeof window !== 'undefined') {
    window.setTimeout(() => dismiss(id), entry.duration)
  }

  return id
}

/** Remove a toast by id. */
export function dismiss(id: string): void {
  notifications.update((list) => list.filter((t) => t.id !== id))
}

/** Remove all toasts. */
export function clearNotifications(): void {
  notifications.set([])
}
