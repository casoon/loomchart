/**
 * Loading State Manager
 *
 * Manages loading states for async operations with:
 * - Global loading indicator
 * - Operation-specific loading states
 * - Loading progress tracking
 * - Automatic timeout handling
 */

export interface LoadingOperation {
  id: string;
  label: string;
  progress?: number; // 0-100
  startTime: number;
  timeout?: number; // milliseconds
}

class LoadingStateManager {
  private operations: Map<string, LoadingOperation> = new Map();
  private globalLoading: boolean = false;

  /**
   * Start a loading operation
   */
  start(id: string, label: string, timeout: number = 30000): void {
    const operation: LoadingOperation = {
      id,
      label,
      startTime: Date.now(),
      timeout,
    };

    this.operations.set(id, operation);
    this.updateGlobalState();
    this.dispatchUpdate();

    // Auto-timeout
    if (timeout > 0) {
      setTimeout(() => {
        if (this.operations.has(id)) {
          console.warn(`[Loading] Operation "${label}" timed out after ${timeout}ms`);
          this.end(id);
        }
      }, timeout);
    }
  }

  /**
   * Update operation progress
   */
  setProgress(id: string, progress: number): void {
    const operation = this.operations.get(id);
    if (!operation) return;

    operation.progress = Math.max(0, Math.min(100, progress));
    this.dispatchUpdate();
  }

  /**
   * End a loading operation
   */
  end(id: string): void {
    if (!this.operations.has(id)) return;

    this.operations.delete(id);
    this.updateGlobalState();
    this.dispatchUpdate();
  }

  /**
   * Check if specific operation is loading
   */
  isLoading(id?: string): boolean {
    if (id) {
      return this.operations.has(id);
    }
    return this.globalLoading;
  }

  /**
   * Get all active operations
   */
  getOperations(): LoadingOperation[] {
    return Array.from(this.operations.values());
  }

  /**
   * Get specific operation
   */
  getOperation(id: string): LoadingOperation | undefined {
    return this.operations.get(id);
  }

  /**
   * Clear all operations
   */
  clearAll(): void {
    this.operations.clear();
    this.updateGlobalState();
    this.dispatchUpdate();
  }

  /**
   * Update global loading state
   */
  private updateGlobalState(): void {
    const wasLoading = this.globalLoading;
    this.globalLoading = this.operations.size > 0;

    if (wasLoading !== this.globalLoading) {
      this.dispatchGlobalStateChange();
    }
  }

  /**
   * Dispatch loading state update event
   */
  private dispatchUpdate(): void {
    window.dispatchEvent(new CustomEvent('loadingStateUpdate', {
      detail: {
        operations: this.getOperations(),
        isLoading: this.globalLoading,
      }
    }));
  }

  /**
   * Dispatch global state change event
   */
  private dispatchGlobalStateChange(): void {
    window.dispatchEvent(new CustomEvent('globalLoadingChange', {
      detail: { isLoading: this.globalLoading }
    }));
  }
}

// Global instance
export const loadingState = new LoadingStateManager();

/**
 * Wrapper function for async operations with automatic loading state
 */
export async function withLoading<T>(
  id: string,
  label: string,
  operation: () => Promise<T>,
  timeout: number = 30000
): Promise<T> {
  loadingState.start(id, label, timeout);

  try {
    const result = await operation();
    loadingState.end(id);
    return result;
  } catch (error) {
    loadingState.end(id);
    throw error;
  }
}

/**
 * Decorator for async functions to automatically track loading state
 */
export function tracked(id: string, label: string, timeout: number = 30000) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      loadingState.start(id, label, timeout);

      try {
        const result = await originalMethod.apply(this, args);
        loadingState.end(id);
        return result;
      } catch (error) {
        loadingState.end(id);
        throw error;
      }
    };

    return descriptor;
  };
}

// Expose globally for debugging
(window as any).loadingState = loadingState;
