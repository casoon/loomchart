/**
 * Toast Notification System
 *
 * Features:
 * - Multiple toast types (success, error, warning, info)
 * - Auto-dismiss with configurable duration
 * - Manual dismiss
 * - Action buttons
 * - Queue management (max 3 visible at once)
 * - Smooth animations
 */

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface ToastAction {
  label: string;
  handler: () => void;
}

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration: number;
  action?: ToastAction;
  timestamp: number;
}

export interface ToastOptions {
  type?: ToastType;
  message: string;
  duration?: number; // milliseconds, 0 = no auto-dismiss
  action?: ToastAction;
}

class ToastManager {
  private toasts: Toast[] = [];
  private maxVisible: number = 3;
  private container: HTMLElement | null = null;

  constructor() {
    this.initContainer();
    this.listenToEvents();
  }

  /**
   * Create toast container
   */
  private initContainer(): void {
    // Check if container already exists
    if (document.getElementById('toast-container')) {
      this.container = document.getElementById('toast-container');
      return;
    }

    this.container = document.createElement('div');
    this.container.id = 'toast-container';
    this.container.className = 'fixed top-4 right-4 z-50 flex flex-col gap-2 pointer-events-none';

    // Wait for DOM to be ready
    if (document.body) {
      document.body.appendChild(this.container);
    } else {
      document.addEventListener('DOMContentLoaded', () => {
        document.body.appendChild(this.container!);
      });
    }
  }

  /**
   * Listen to global toast events
   */
  private listenToEvents(): void {
    window.addEventListener('showToast', ((event: CustomEvent<ToastOptions>) => {
      this.show(event.detail);
    }) as EventListener);
  }

  /**
   * Show a toast notification
   */
  show(options: ToastOptions): string {
    const toast: Toast = {
      id: this.generateId(),
      type: options.type || 'info',
      message: options.message,
      duration: options.duration !== undefined ? options.duration : 5000,
      action: options.action,
      timestamp: Date.now(),
    };

    // Add to queue
    this.toasts.push(toast);

    // Render
    this.render();

    // Auto-dismiss if duration > 0
    if (toast.duration > 0) {
      setTimeout(() => {
        this.dismiss(toast.id);
      }, toast.duration);
    }

    return toast.id;
  }

  /**
   * Dismiss a toast by ID
   */
  dismiss(id: string): void {
    const index = this.toasts.findIndex(t => t.id === id);
    if (index === -1) return;

    // Get toast element
    const element = document.getElementById(`toast-${id}`);
    if (element) {
      // Animate out
      element.classList.add('opacity-0', 'translate-x-full');

      // Remove after animation
      setTimeout(() => {
        this.toasts = this.toasts.filter(t => t.id !== id);
        this.render();
      }, 300);
    } else {
      this.toasts = this.toasts.filter(t => t.id !== id);
      this.render();
    }
  }

  /**
   * Dismiss all toasts
   */
  dismissAll(): void {
    this.toasts = [];
    this.render();
  }

  /**
   * Render all toasts
   */
  private render(): void {
    if (!this.container) return;

    // Clear container
    this.container.innerHTML = '';

    // Show only the most recent maxVisible toasts
    const visibleToasts = this.toasts.slice(-this.maxVisible);

    visibleToasts.forEach(toast => {
      const element = this.createToastElement(toast);
      this.container!.appendChild(element);
    });
  }

  /**
   * Create toast DOM element
   */
  private createToastElement(toast: Toast): HTMLElement {
    const div = document.createElement('div');
    div.id = `toast-${toast.id}`;
    div.className = `
      toast
      pointer-events-auto
      min-w-[320px] max-w-md
      px-4 py-3
      rounded-lg shadow-lg
      flex items-start gap-3
      transition-all duration-300 ease-out
      transform translate-x-0 opacity-100
      ${this.getToastStyles(toast.type)}
    `;

    // Icon
    const icon = document.createElement('div');
    icon.className = 'flex-shrink-0 mt-0.5';
    icon.innerHTML = this.getIcon(toast.type);
    div.appendChild(icon);

    // Content
    const content = document.createElement('div');
    content.className = 'flex-1 min-w-0';

    const message = document.createElement('p');
    message.className = 'text-sm font-medium leading-tight';
    message.textContent = toast.message;
    content.appendChild(message);

    // Action button
    if (toast.action) {
      const button = document.createElement('button');
      button.className = 'mt-2 text-xs font-semibold underline hover:no-underline';
      button.textContent = toast.action.label;
      button.onclick = () => {
        toast.action!.handler();
        this.dismiss(toast.id);
      };
      content.appendChild(button);
    }

    div.appendChild(content);

    // Close button
    const closeBtn = document.createElement('button');
    closeBtn.className = 'flex-shrink-0 text-current opacity-50 hover:opacity-100 transition-opacity';
    closeBtn.innerHTML = `
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
      </svg>
    `;
    closeBtn.onclick = () => this.dismiss(toast.id);
    div.appendChild(closeBtn);

    return div;
  }

  /**
   * Get toast background and text colors
   */
  private getToastStyles(type: ToastType): string {
    const styles = {
      success: 'bg-green-600 text-white',
      error: 'bg-red-600 text-white',
      warning: 'bg-yellow-500 text-gray-900',
      info: 'bg-blue-600 text-white',
    };
    return styles[type];
  }

  /**
   * Get toast icon SVG
   */
  private getIcon(type: ToastType): string {
    const icons = {
      success: `
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
        </svg>
      `,
      error: `
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
        </svg>
      `,
      warning: `
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
        </svg>
      `,
      info: `
        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
        </svg>
      `,
    };
    return icons[type];
  }

  /**
   * Generate unique toast ID
   */
  private generateId(): string {
    return `toast_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }
}

// Global instance
export const toastManager = new ToastManager();

// Helper functions for easy usage
export function showToast(message: string, type: ToastType = 'info', duration: number = 5000): string {
  return toastManager.show({ message, type, duration });
}

export function showSuccess(message: string, duration: number = 5000): string {
  return toastManager.show({ message, type: 'success', duration });
}

export function showError(message: string, duration: number = 5000): string {
  return toastManager.show({ message, type: 'error', duration });
}

export function showWarning(message: string, duration: number = 5000): string {
  return toastManager.show({ message, type: 'warning', duration });
}

export function showInfo(message: string, duration: number = 5000): string {
  return toastManager.show({ message, type: 'info', duration });
}

// Expose globally for event-based usage
(window as any).showToast = showToast;
(window as any).showSuccess = showSuccess;
(window as any).showError = showError;
(window as any).showWarning = showWarning;
(window as any).showInfo = showInfo;
