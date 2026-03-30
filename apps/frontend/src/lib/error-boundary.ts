/**
 * Error Boundary - Global error handling system
 *
 * Catches and handles all errors gracefully:
 * - Unhandled JavaScript errors
 * - Unhandled promise rejections
 * - WASM errors
 * - WebSocket errors
 * - Network errors
 */

export interface ErrorReport {
    message: string;
    stack?: string;
    timestamp: number;
    context: string;
    userAgent?: string;
}

export type ErrorContext =
    | 'window.onerror'
    | 'unhandledrejection'
    | 'WASM'
    | 'WebSocket'
    | 'Network'
    | 'Chart'
    | 'Indicator'
    | 'Drawing'
    | 'Storage';

export class ErrorBoundary {
    private errorLog: ErrorReport[] = [];
    private maxErrors: number = 100;
    private enabled: boolean = true;

    /**
     * Initialize error boundary
     */
    init(): void {
        if (!this.enabled) return;

        console.log('[ErrorBoundary] Initializing global error handling');

        // Catch unhandled errors
        window.addEventListener('error', (event) => {
            this.handleError(event.error || new Error(event.message), 'window.onerror');
            event.preventDefault(); // Prevent default error logging
        });

        // Catch unhandled promise rejections
        window.addEventListener('unhandledrejection', (event) => {
            this.handleError(
                event.reason instanceof Error ? event.reason : new Error(String(event.reason)),
                'unhandledrejection'
            );
            event.preventDefault();
        });

        console.log('[ErrorBoundary] Error handlers registered');
    }

    /**
     * Handle an error with context
     */
    handleError(error: Error | unknown, context: ErrorContext): void {
        const errorObj = error instanceof Error ? error : new Error(String(error));

        console.error(`[ErrorBoundary] ${context}:`, errorObj);

        // Log error
        const report: ErrorReport = {
            message: errorObj.message || String(error),
            stack: errorObj.stack,
            timestamp: Date.now(),
            context,
            userAgent: navigator.userAgent,
        };

        this.errorLog.push(report);

        // Limit log size
        if (this.errorLog.length > this.maxErrors) {
            this.errorLog.shift();
        }

        // Show user-friendly notification
        this.showErrorNotification(errorObj, context);

        // Report to monitoring service (if configured)
        this.reportToMonitoring(report);
    }

    /**
     * Show user-friendly error notification
     */
    private showErrorNotification(error: Error, context: ErrorContext): void {
        let userMessage = 'An error occurred';
        let action: { label: string; handler: () => void } | undefined;

        // Customize message based on error type
        if (context === 'WASM') {
            userMessage = 'Chart rendering error - please refresh the page';
            action = {
                label: 'Refresh',
                handler: () => window.location.reload(),
            };
        } else if (context === 'WebSocket') {
            userMessage = 'Connection error - will attempt to reconnect';
            // No action needed, reconnection is automatic
        } else if (context === 'Network') {
            userMessage = 'Network error - please check your connection';
            action = {
                label: 'Retry',
                handler: () => window.location.reload(),
            };
        } else if (error.message.includes('quota')) {
            userMessage = 'Storage quota exceeded - clearing cache';
            action = {
                label: 'Clear Cache',
                handler: () => this.clearStorage(),
            };
        } else if (context === 'Chart' || context === 'Indicator') {
            userMessage = `${context} error - some features may not work`;
        } else if (error.message.includes('fetch')) {
            userMessage = 'Failed to load data - please try again';
            action = {
                label: 'Retry',
                handler: () => window.location.reload(),
            };
        }

        // Dispatch toast notification
        window.dispatchEvent(
            new CustomEvent('showToast', {
                detail: {
                    type: 'error',
                    message: userMessage,
                    duration: 5000,
                    action,
                },
            })
        );
    }

    /**
     * Clear all browser storage
     */
    private async clearStorage(): Promise<void> {
        try {
            // Clear IndexedDB
            const databases = await indexedDB.databases();
            for (const db of databases) {
                if (db.name) {
                    indexedDB.deleteDatabase(db.name);
                }
            }

            // Clear localStorage
            localStorage.clear();

            // Clear sessionStorage
            sessionStorage.clear();

            console.log('[ErrorBoundary] Storage cleared');

            // Reload page
            window.location.reload();
        } catch (err) {
            console.error('[ErrorBoundary] Failed to clear storage:', err);
        }
    }

    /**
     * Report error to monitoring service
     *
     * In production, this would send to Sentry, LogRocket, or similar service.
     * For now, just log to console.
     */
    private reportToMonitoring(report: ErrorReport): void {
        // In production, send to monitoring service:
        // if (window.Sentry) {
        //     Sentry.captureException(new Error(report.message), {
        //         contexts: {
        //             error: report
        //         }
        //     });
        // }

        console.log('[ErrorBoundary] Error reported:', report);
    }

    /**
     * Get error log for debugging
     */
    getErrorLog(): ErrorReport[] {
        return [...this.errorLog];
    }

    /**
     * Clear error log
     */
    clearErrorLog(): void {
        this.errorLog = [];
    }

    /**
     * Enable/disable error boundary
     */
    setEnabled(enabled: boolean): void {
        this.enabled = enabled;
    }

    /**
     * Wrap a function with error handling
     */
    wrap<T extends (...args: any[]) => any>(
        fn: T,
        context: ErrorContext
    ): (...args: Parameters<T>) => ReturnType<T> | null {
        return (...args: Parameters<T>): ReturnType<T> | null => {
            try {
                return fn(...args);
            } catch (err) {
                this.handleError(err, context);
                return null;
            }
        };
    }

    /**
     * Wrap an async function with error handling
     */
    wrapAsync<T extends (...args: any[]) => Promise<any>>(
        fn: T,
        context: ErrorContext
    ): (...args: Parameters<T>) => Promise<Awaited<ReturnType<T>> | null> {
        return async (...args: Parameters<T>): Promise<Awaited<ReturnType<T>> | null> => {
            try {
                return await fn(...args);
            } catch (err) {
                this.handleError(err, context);
                return null;
            }
        };
    }
}

// Create singleton instance
export const errorBoundary = new ErrorBoundary();

// Auto-initialize on load
if (typeof window !== 'undefined') {
    errorBoundary.init();
}

// Export helper functions
export function reportError(error: Error | unknown, context: ErrorContext): void {
    errorBoundary.handleError(error, context);
}

export function wrapFunction<T extends (...args: any[]) => any>(
    fn: T,
    context: ErrorContext
): (...args: Parameters<T>) => ReturnType<T> | null {
    return errorBoundary.wrap(fn, context);
}

export function wrapAsync<T extends (...args: any[]) => Promise<any>>(
    fn: T,
    context: ErrorContext
): (...args: Parameters<T>) => Promise<Awaited<ReturnType<T>> | null> {
    return errorBoundary.wrapAsync(fn, context);
}
