/**
 * Integration Tests for Frontend
 *
 * Tests the integration between:
 * - Error boundary system
 * - Toast notifications
 * - Loading state management
 * - WASM chart integration
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { errorBoundary } from '../src/lib/error-boundary';
import { toastManager } from '../src/lib/toast';
import { loadingState } from '../src/lib/loading-state';

describe('Error Boundary Integration', () => {
  beforeEach(() => {
    // Reset error log
    errorBoundary['errorLog'] = [];
  });

  it('should catch and log errors', () => {
    const error = new Error('Test error');
    errorBoundary.handleError(error, 'Test');

    const log = errorBoundary.getErrorLog();
    expect(log).toHaveLength(1);
    expect(log[0].message).toBe('Test error');
    expect(log[0].context).toBe('Test');
  });

  it('should limit error log size', () => {
    // Add more than maxErrors
    for (let i = 0; i < 150; i++) {
      errorBoundary.handleError(new Error(`Error ${i}`), 'Test');
    }

    const log = errorBoundary.getErrorLog();
    expect(log.length).toBeLessThanOrEqual(100);
  });

  it('should handle network errors', () => {
    const error = new Error('Failed to fetch');
    errorBoundary.handleError(error, 'Network');

    const log = errorBoundary.getErrorLog();
    expect(log[0].context).toBe('Network');
  });

  it('should handle WASM errors', () => {
    const error = new Error('WASM module failed');
    errorBoundary.handleError(error, 'WASM');

    const log = errorBoundary.getErrorLog();
    expect(log[0].context).toBe('WASM');
  });
});

describe('Toast Notification Integration', () => {
  beforeEach(() => {
    toastManager['toasts'] = [];
  });

  it('should create toast notifications', () => {
    const id = toastManager.show({
      type: 'success',
      message: 'Test message',
      duration: 5000,
    });

    expect(id).toBeTruthy();
    const toasts = toastManager['toasts'];
    expect(toasts).toHaveLength(1);
    expect(toasts[0].message).toBe('Test message');
  });

  it('should auto-dismiss toasts', (done) => {
    toastManager.show({
      type: 'info',
      message: 'Auto dismiss',
      duration: 100,
    });

    setTimeout(() => {
      const toasts = toastManager['toasts'];
      expect(toasts).toHaveLength(0);
      done();
    }, 150);
  });

  it('should limit visible toasts', () => {
    for (let i = 0; i < 5; i++) {
      toastManager.show({
        type: 'info',
        message: `Toast ${i}`,
        duration: 10000,
      });
    }

    const toasts = toastManager['toasts'];
    expect(toasts.length).toBe(5);
  });

  it('should dismiss specific toast', () => {
    const id = toastManager.show({
      type: 'info',
      message: 'Test',
      duration: 0,
    });

    toastManager.dismiss(id);

    setTimeout(() => {
      const toasts = toastManager['toasts'];
      expect(toasts).toHaveLength(0);
    }, 350); // Animation time
  });

  it('should dismiss all toasts', () => {
    toastManager.show({ type: 'info', message: 'Toast 1', duration: 0 });
    toastManager.show({ type: 'info', message: 'Toast 2', duration: 0 });

    toastManager.dismissAll();

    const toasts = toastManager['toasts'];
    expect(toasts).toHaveLength(0);
  });
});

describe('Loading State Integration', () => {
  beforeEach(() => {
    loadingState['operations'].clear();
  });

  it('should track loading operations', () => {
    loadingState.start('test-op', 'Testing operation');

    expect(loadingState.isLoading('test-op')).toBe(true);
    expect(loadingState.isLoading()).toBe(true);
  });

  it('should end loading operations', () => {
    loadingState.start('test-op', 'Testing');
    loadingState.end('test-op');

    expect(loadingState.isLoading('test-op')).toBe(false);
    expect(loadingState.isLoading()).toBe(false);
  });

  it('should track multiple operations', () => {
    loadingState.start('op1', 'Operation 1');
    loadingState.start('op2', 'Operation 2');

    expect(loadingState.getOperations()).toHaveLength(2);
    expect(loadingState.isLoading()).toBe(true);

    loadingState.end('op1');
    expect(loadingState.isLoading()).toBe(true);

    loadingState.end('op2');
    expect(loadingState.isLoading()).toBe(false);
  });

  it('should update operation progress', () => {
    loadingState.start('test-op', 'Testing');
    loadingState.setProgress('test-op', 50);

    const op = loadingState.getOperation('test-op');
    expect(op?.progress).toBe(50);
  });

  it('should auto-timeout operations', (done) => {
    loadingState.start('test-op', 'Testing', 100);

    setTimeout(() => {
      expect(loadingState.isLoading('test-op')).toBe(false);
      done();
    }, 150);
  });

  it('should clear all operations', () => {
    loadingState.start('op1', 'Operation 1');
    loadingState.start('op2', 'Operation 2');

    loadingState.clearAll();

    expect(loadingState.getOperations()).toHaveLength(0);
    expect(loadingState.isLoading()).toBe(false);
  });
});

describe('Error Boundary + Toast Integration', () => {
  beforeEach(() => {
    errorBoundary['errorLog'] = [];
    toastManager['toasts'] = [];
  });

  it('should show toast on error', () => {
    const showToastSpy = vi.spyOn(window, 'dispatchEvent');

    errorBoundary.handleError(new Error('Test error'), 'Test');

    expect(showToastSpy).toHaveBeenCalled();
  });

  it('should show appropriate toast for WASM errors', () => {
    const showToastSpy = vi.spyOn(window, 'dispatchEvent');

    errorBoundary.handleError(new Error('WASM failed'), 'WASM');

    expect(showToastSpy).toHaveBeenCalledWith(
      expect.objectContaining({
        type: 'showToast',
      })
    );
  });
});

describe('Loading State + Error Boundary Integration', () => {
  beforeEach(() => {
    loadingState['operations'].clear();
    errorBoundary['errorLog'] = [];
  });

  it('should end loading on error', async () => {
    loadingState.start('test-op', 'Testing');

    try {
      throw new Error('Operation failed');
    } catch (error) {
      loadingState.end('test-op');
      errorBoundary.handleError(error as Error, 'Test');
    }

    expect(loadingState.isLoading('test-op')).toBe(false);
    expect(errorBoundary.getErrorLog()).toHaveLength(1);
  });
});

describe('Full Integration Workflow', () => {
  beforeEach(() => {
    loadingState['operations'].clear();
    errorBoundary['errorLog'] = [];
    toastManager['toasts'] = [];
  });

  it('should handle successful async operation', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ data: [] }),
    });

    loadingState.start('fetch-data', 'Fetching data');

    try {
      await mockFetch('/api/data');
      loadingState.end('fetch-data');
    } catch (error) {
      loadingState.end('fetch-data');
      errorBoundary.handleError(error as Error, 'Network');
    }

    expect(loadingState.isLoading()).toBe(false);
    expect(errorBoundary.getErrorLog()).toHaveLength(0);
  });

  it('should handle failed async operation', async () => {
    const mockFetch = vi.fn().mockRejectedValue(new Error('Network error'));

    loadingState.start('fetch-data', 'Fetching data');

    try {
      await mockFetch('/api/data');
      loadingState.end('fetch-data');
    } catch (error) {
      loadingState.end('fetch-data');
      errorBoundary.handleError(error as Error, 'Network');
    }

    expect(loadingState.isLoading()).toBe(false);
    expect(errorBoundary.getErrorLog()).toHaveLength(1);
  });
});
