/**
 * Chart Integration for Scientific Indicators
 *
 * This module integrates the scientific indicators with the main chart,
 * handling data updates, event dispatching, and visualization coordination.
 */

import type { IndicatorInstance, IndicatorConfig } from "./types";
import { createIndicator } from "./registry";

interface ChartIntegration {
  activeIndicators: Map<string, IndicatorInstance>;
  chartInstance: any; // RustChart instance
  updateInterval: number | null;
}

const integration: ChartIntegration = {
  activeIndicators: new Map(),
  chartInstance: null,
  updateInterval: null,
};

/**
 * Initialize the indicator integration with the chart
 */
export function initializeChartIntegration(chartInstance: any): void {
  integration.chartInstance = chartInstance;

  // Start update loop for real-time indicator calculations
  startUpdateLoop();

  console.log("[IndicatorIntegration] Initialized with chart instance");
}

/**
 * Add an indicator to the chart
 */
export async function addIndicatorToChart(
  config: IndicatorConfig,
): Promise<IndicatorInstance | null> {
  // Try to add via WASM first (for new scientific indicators)
  try {
    const wasmModule = await import("../../../public/wasm/trading_ui.js");

    // Check if this is a WASM indicator
    const metadataJson = wasmModule.get_all_indicators();
    const allIndicators = JSON.parse(metadataJson);
    const isWasmIndicator = allIndicators.some(
      (ind: any) => ind.id === config.id,
    );

    if (isWasmIndicator) {
      console.log(
        "[IndicatorIntegration] Adding WASM indicator:",
        config.id,
        "with params:",
        config.parameters,
      );

      const metadata = allIndicators.find((ind: any) => ind.id === config.id);
      const instanceId = `${config.id}_${Date.now()}`;

      // Create the WASM indicator instance based on type
      let wasmIndicator: any = null;
      try {
        switch (config.id) {
          case "shannon_entropy":
            const se_period = config.parameters?.period || 20;
            const se_bins = config.parameters?.bins || 10;
            wasmIndicator = new wasmModule.WasmShannonEntropy(
              se_period,
              se_bins,
            );
            console.log("[IndicatorIntegration] Created WasmShannonEntropy:", {
              period: se_period,
              bins: se_bins,
            });
            break;
          case "lempel_ziv":
            const lz_period = config.parameters?.period || 100;
            const lz_threshold = config.parameters?.threshold || 0.0;
            wasmIndicator = new wasmModule.WasmLempelZivComplexity(
              lz_period,
              lz_threshold,
            );
            console.log(
              "[IndicatorIntegration] Created WasmLempelZivComplexity:",
              { period: lz_period, threshold: lz_threshold },
            );
            break;
          case "permutation_entropy":
            const pe_period = config.parameters?.period || 20;
            const pe_embedding_dimension =
              config.parameters?.embedding_dimension || 3;
            const pe_delay = config.parameters?.delay || 1;
            wasmIndicator = new wasmModule.WasmPermutationEntropy(
              pe_period,
              pe_embedding_dimension,
              pe_delay,
            );
            console.log(
              "[IndicatorIntegration] Created WasmPermutationEntropy:",
              {
                period: pe_period,
                embedding_dimension: pe_embedding_dimension,
                delay: pe_delay,
              },
            );
            break;
        }
      } catch (error) {
        console.error(
          "[IndicatorIntegration] Failed to create WASM indicator instance:",
          error,
        );
        throw error;
      }

      // Create indicator instance with WASM calculation function
      const instance: IndicatorInstance = {
        instanceId,
        metadata,
        config,
        lastValue: null,
        values: [],
        calculate: (candles: any[]) => {
          if (!wasmIndicator) return null;

          // Extract close prices for calculation
          const closes = candles.map((c: any) => c.c);

          try {
            // Call the calculate method on the WASM indicator
            const result = wasmIndicator.calculate(new Float64Array(closes));
            return result;
          } catch (error) {
            console.error(
              "[IndicatorIntegration] WASM calculation error:",
              error,
            );
            return null;
          }
        },
      };

      integration.activeIndicators.set(instance.instanceId, instance);

      // Calculate initial values if we have chart data
      if (integration.chartInstance) {
        updateIndicatorValues(instance);
      }

      // Dispatch event
      window.dispatchEvent(
        new CustomEvent("indicator-added", {
          detail: { instance },
        }),
      );

      console.log(
        "[IndicatorIntegration] Added WASM indicator:",
        metadata.name,
      );
      return instance;
    }
  } catch (error) {
    console.warn("[IndicatorIntegration] Error loading WASM indicator:", error);
    // Fall through to TypeScript indicator system
  }

  // Fallback to TypeScript indicator system
  const instance = createIndicator(config);

  if (!instance) {
    console.error(
      "[IndicatorIntegration] Failed to create indicator:",
      config.id,
    );
    return null;
  }

  integration.activeIndicators.set(instance.instanceId, instance);

  // Calculate initial values if we have chart data
  if (integration.chartInstance) {
    updateIndicatorValues(instance);
  }

  // Dispatch event
  window.dispatchEvent(
    new CustomEvent("indicator-added", {
      detail: { instance },
    }),
  );

  console.log(
    "[IndicatorIntegration] Added indicator:",
    instance.metadata.name,
  );
  return instance;
}

/**
 * Remove an indicator from the chart
 */
export function removeIndicatorFromChart(instanceId: string): boolean {
  const instance = integration.activeIndicators.get(instanceId);

  if (!instance) {
    return false;
  }

  integration.activeIndicators.delete(instanceId);
  removeIndicator(instanceId);

  // Dispatch event
  window.dispatchEvent(
    new CustomEvent("indicator-removed", {
      detail: { instanceId },
    }),
  );

  console.log(
    "[IndicatorIntegration] Removed indicator:",
    instance.metadata.name,
  );
  return true;
}

/**
 * Update indicator configuration
 */
export function updateIndicatorConfig(
  instanceId: string,
  newConfig: Partial<IndicatorConfig>,
): boolean {
  const instance = integration.activeIndicators.get(instanceId);

  if (!instance) {
    return false;
  }

  // Update the configuration
  const updated = updateIndicator(instanceId, newConfig);

  if (updated) {
    // Recalculate values
    updateIndicatorValues(updated);

    // Dispatch update event
    window.dispatchEvent(
      new CustomEvent("indicator-config-updated", {
        detail: { instance: updated },
      }),
    );
  }

  return updated !== null;
}

/**
 * Calculate indicator values based on current chart data
 */
function updateIndicatorValues(instance: IndicatorInstance): void {
  if (!integration.chartInstance) {
    return;
  }

  try {
    // Get price data from chart
    const candles = integration.chartInstance.getCandles?.();

    if (!candles || candles.length === 0) {
      return;
    }

    // Extract close prices
    const prices = candles.map((c: any) => c.c);

    // Calculate indicator values incrementally
    const { period, bins, dimension, delay } = instance.config.parameters;

    let value: number | null = null;

    switch (instance.metadata.id) {
      case "shannon_entropy":
        if (prices.length >= period) {
          const window = prices.slice(-period);
          const result = instance.wasmIndicator.constructor.calculate(
            window,
            period,
            bins || 10,
          );
          const parsed = JSON.parse(result);
          value = parsed[parsed.length - 1];
        }
        break;

      case "lempel_ziv":
        if (prices.length >= period) {
          const window = prices.slice(-period);
          const result = instance.wasmIndicator.constructor.calculate(
            window,
            period,
            bins || 10,
          );
          const parsed = JSON.parse(result);
          value = parsed[parsed.length - 1];
        }
        break;

      case "permutation_entropy":
        if (prices.length >= period) {
          const window = prices.slice(-period);
          const result = instance.wasmIndicator.constructor.calculate(
            window,
            dimension || 3,
            delay || 1,
          );
          const parsed = JSON.parse(result);
          value = parsed[parsed.length - 1];
        }
        break;
    }

    // Update instance
    if (value !== null && !isNaN(value)) {
      instance.lastValue = value;
      instance.values.push(value);

      // Keep only last 1000 values to prevent memory issues
      if (instance.values.length > 1000) {
        instance.values.shift();
      }

      // Dispatch update event
      window.dispatchEvent(
        new CustomEvent("indicator-updated", {
          detail: {
            instanceId: instance.instanceId,
            value: value,
            metadata: instance.metadata,
          },
        }),
      );
    }
  } catch (error) {
    console.error(
      "[IndicatorIntegration] Error updating indicator values:",
      error,
    );
  }
}

/**
 * Update all active indicators
 */
export function updateAllIndicators(): void {
  integration.activeIndicators.forEach((instance) => {
    updateIndicatorValues(instance);
  });
}

/**
 * Start the update loop for real-time calculations
 */
function startUpdateLoop(): void {
  // Update indicators every 1 second
  integration.updateInterval = window.setInterval(() => {
    if (integration.activeIndicators.size > 0) {
      updateAllIndicators();
    }
  }, 1000);

  console.log("[IndicatorIntegration] Started update loop");
}

/**
 * Stop the update loop
 */
export function stopUpdateLoop(): void {
  if (integration.updateInterval !== null) {
    window.clearInterval(integration.updateInterval);
    integration.updateInterval = null;
    console.log("[IndicatorIntegration] Stopped update loop");
  }
}

/**
 * Get all active indicators
 */
export function getActiveIndicators(): IndicatorInstance[] {
  return Array.from(integration.activeIndicators.values());
}

/**
 * Get a specific active indicator by instance ID
 */
export function getActiveIndicator(
  instanceId: string,
): IndicatorInstance | undefined {
  return integration.activeIndicators.get(instanceId);
}

/**
 * Clear all indicators
 */
export function clearAllIndicators(): void {
  const instanceIds = Array.from(integration.activeIndicators.keys());

  instanceIds.forEach((id) => {
    removeIndicatorFromChart(id);
  });

  console.log("[IndicatorIntegration] Cleared all indicators");
}

/**
 * Handle new candle data
 */
export function onCandleAdded(candle: any): void {
  // Update all indicators when a new candle arrives
  if (integration.activeIndicators.size > 0) {
    updateAllIndicators();
  }
}

/**
 * Handle chart data reset
 */
export function onChartReset(): void {
  // Reset all indicator values
  integration.activeIndicators.forEach((instance) => {
    instance.values = [];
    instance.lastValue = null;
  });

  console.log("[IndicatorIntegration] Reset all indicator values");
}

/**
 * Export current state for persistence
 */
export function exportIndicatorState(): any {
  const state = {
    version: "1.0.0",
    indicators: Array.from(integration.activeIndicators.values()).map(
      (instance) => ({
        instanceId: instance.instanceId,
        config: instance.config,
        values: instance.values,
        lastValue: instance.lastValue,
      }),
    ),
  };

  return state;
}

/**
 * Import state from persistence
 */
export function importIndicatorState(state: any): void {
  if (!state || state.version !== "1.0.0") {
    console.warn("[IndicatorIntegration] Invalid state version");
    return;
  }

  // Clear existing indicators
  clearAllIndicators();

  // Restore indicators
  state.indicators.forEach((saved: any) => {
    const instance = addIndicatorToChart(saved.config);

    if (instance) {
      instance.values = saved.values || [];
      instance.lastValue = saved.lastValue;
    }
  });

  console.log(
    "[IndicatorIntegration] Imported indicator state:",
    state.indicators.length,
    "indicators",
  );
}

// Export integration object for debugging
if (typeof window !== "undefined") {
  (window as any).__indicatorIntegration = integration;
}
