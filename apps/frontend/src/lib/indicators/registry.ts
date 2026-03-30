/**
 * Scientific Indicators Registry
 *
 * Manages available indicators and their metadata.
 * Fetches metadata from WASM module.
 */

import type { IndicatorMetadata, IndicatorInstance, IndicatorConfig } from './types';

let wasmModule: any = null;
let indicatorCache: Map<string, IndicatorMetadata> = new Map();

/**
 * Initialize the indicator registry with WASM module
 */
export async function initializeRegistry(wasm: any): Promise<void> {
  wasmModule = wasm;

  // Fetch all indicator metadata from WASM
  try {
    const metadataJson = wasm.getAllIndicators();
    const metadata: IndicatorMetadata[] = JSON.parse(metadataJson);

    // Cache metadata
    for (const meta of metadata) {
      indicatorCache.set(meta.id, meta);
    }

    console.log(`[IndicatorRegistry] Loaded ${metadata.length} indicators:`,
      metadata.map(m => m.name).join(', '));
  } catch (error) {
    console.error('[IndicatorRegistry] Failed to load indicators:', error);
  }
}

/**
 * Get all available indicators
 */
export function getAllIndicators(): IndicatorMetadata[] {
  return Array.from(indicatorCache.values());
}

/**
 * Get indicators by category
 */
export function getIndicatorsByCategory(category: string): IndicatorMetadata[] {
  return getAllIndicators().filter(ind => ind.category === category);
}

/**
 * Get indicator metadata by ID
 */
export function getIndicatorMetadata(id: string): IndicatorMetadata | null {
  return indicatorCache.get(id) || null;
}

/**
 * Create indicator instance
 */
export function createIndicator(config: IndicatorConfig): IndicatorInstance | null {
  const metadata = getIndicatorMetadata(config.id);
  if (!metadata) {
    console.error(`[IndicatorRegistry] Unknown indicator: ${config.id}`);
    return null;
  }

  if (!wasmModule) {
    console.error('[IndicatorRegistry] WASM module not initialized');
    return null;
  }

  // Create WASM indicator instance based on type
  let wasmIndicator: any;

  try {
    switch (config.id) {
      case 'shannon_entropy': {
        const period = config.parameters.period || 20;
        const bins = config.parameters.bins || 10;
        wasmIndicator = new wasmModule.WasmShannonEntropy(period, bins);
        break;
      }

      case 'lempel_ziv': {
        const period = config.parameters.period || 100;
        const threshold = config.parameters.threshold || 0.0;
        wasmIndicator = new wasmModule.WasmLempelZivComplexity(period, threshold);
        break;
      }

      case 'permutation_entropy': {
        const period = config.parameters.period || 100;
        const dimension = config.parameters.dimension || 3;
        const delay = config.parameters.delay || 1;
        wasmIndicator = new wasmModule.WasmPermutationEntropy(period, dimension, delay);
        break;
      }

      default:
        console.error(`[IndicatorRegistry] Unsupported indicator: ${config.id}`);
        return null;
    }
  } catch (error) {
    console.error(`[IndicatorRegistry] Failed to create indicator ${config.id}:`, error);
    return null;
  }

  const instance: IndicatorInstance = {
    instanceId: `${config.id}_${Date.now()}`,
    metadata,
    config,
    wasmIndicator,
    values: [],
    lastValue: null,
  };

  console.log(`[IndicatorRegistry] Created indicator: ${metadata.name}`);
  return instance;
}

/**
 * Calculate indicator value for next price
 */
export function updateIndicator(instance: IndicatorInstance, price: number): number | null {
  if (!instance.wasmIndicator) {
    return null;
  }

  try {
    const result = instance.wasmIndicator.next(price);

    // WASM returns null if insufficient data
    if (result === null || result === undefined) {
      instance.values.push(null);
      return null;
    }

    instance.values.push(result);
    instance.lastValue = result;
    return result;
  } catch (error) {
    console.error(`[IndicatorRegistry] Update failed for ${instance.metadata.name}:`, error);
    return null;
  }
}

/**
 * Calculate indicator for array of prices
 */
export function calculateIndicator(
  config: IndicatorConfig,
  prices: number[]
): (number | null)[] {
  if (!wasmModule) {
    console.error('[IndicatorRegistry] WASM module not initialized');
    return [];
  }

  try {
    let resultJson: string;

    switch (config.id) {
      case 'shannon_entropy': {
        const period = config.parameters.period || 20;
        const bins = config.parameters.bins || 10;
        resultJson = wasmModule.WasmShannonEntropy.calculate(prices, period, bins);
        break;
      }

      case 'lempel_ziv': {
        const period = config.parameters.period || 100;
        const threshold = config.parameters.threshold || 0.0;
        resultJson = wasmModule.WasmLempelZivComplexity.calculate(prices, period, threshold);
        break;
      }

      case 'permutation_entropy': {
        const period = config.parameters.period || 100;
        const dimension = config.parameters.dimension || 3;
        const delay = config.parameters.delay || 1;
        resultJson = wasmModule.WasmPermutationEntropy.calculate(
          prices,
          period,
          dimension,
          delay
        );
        break;
      }

      default:
        console.error(`[IndicatorRegistry] Unsupported indicator: ${config.id}`);
        return [];
    }

    const result: number[] = JSON.parse(resultJson);

    // Convert NaN to null for TypeScript
    return result.map(v => (isNaN(v) ? null : v));
  } catch (error) {
    console.error(`[IndicatorRegistry] Calculation failed for ${config.id}:`, error);
    return [];
  }
}

/**
 * Reset indicator instance
 */
export function resetIndicator(instance: IndicatorInstance): void {
  if (instance.wasmIndicator?.reset) {
    instance.wasmIndicator.reset();
    instance.values = [];
    instance.lastValue = null;
  }
}

/**
 * Get default configuration for an indicator
 */
export function getDefaultConfig(indicatorId: string): IndicatorConfig | null {
  const metadata = getIndicatorMetadata(indicatorId);
  if (!metadata) {
    return null;
  }

  const parameters: Record<string, number | boolean | string> = {};

  // Extract default values from metadata
  for (const param of metadata.parameters) {
    const paramType = param.param_type;

    if ('Integer' in paramType) {
      parameters[param.id] = paramType.Integer.default;
    } else if ('Float' in paramType) {
      parameters[param.id] = paramType.Float.default;
    } else if ('Boolean' in paramType) {
      parameters[param.id] = paramType.Boolean.default;
    } else if ('Choice' in paramType) {
      parameters[param.id] = paramType.Choice.options[paramType.Choice.default];
    } else if ('Color' in paramType) {
      parameters[param.id] = paramType.Color.default;
    }
  }

  return {
    id: indicatorId,
    parameters,
  };
}

/**
 * Interpret indicator value
 */
export function interpretValue(
  metadata: IndicatorMetadata,
  value: number
): 'high' | 'medium' | 'low' {
  if (!metadata.interpretation) {
    return 'medium';
  }

  // Assume normalized [0, 1] range for scientific indicators
  if (value > 0.7) {
    return 'high';
  } else if (value < 0.4) {
    return 'low';
  } else {
    return 'medium';
  }
}

/**
 * Get interpretation text
 */
export function getInterpretationText(
  metadata: IndicatorMetadata,
  value: number
): string {
  if (!metadata.interpretation) {
    return '';
  }

  const level = interpretValue(metadata, value);

  switch (level) {
    case 'high':
      return metadata.interpretation.high;
    case 'medium':
      return metadata.interpretation.medium;
    case 'low':
      return metadata.interpretation.low;
  }
}
