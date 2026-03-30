/**
 * Scientific Indicators - TypeScript Types
 *
 * Type definitions matching Rust indicator metadata system.
 */

/** Indicator category for UI organization */
export enum IndicatorCategory {
  Momentum = 'Momentum',
  Trend = 'Trend',
  Volatility = 'Volatility',
  Volume = 'Volume',
  Scientific = 'Scientific',
  Custom = 'Custom',
}

/** Parameter type definition */
export type ParameterType =
  | {
      type: 'Integer';
      min: number;
      max: number;
      default: number;
      step: number;
    }
  | {
      type: 'Float';
      min: number;
      max: number;
      default: number;
      step: number;
    }
  | {
      type: 'Boolean';
      default: boolean;
    }
  | {
      type: 'Choice';
      options: string[];
      default: number;
    }
  | {
      type: 'Color';
      default: string;
    };

/** Parameter definition */
export interface ParameterDefinition {
  /** Parameter identifier (e.g., "period", "bins") */
  id: string;
  /** Display name (e.g., "Period") */
  name: string;
  /** Detailed description */
  description: string;
  /** Parameter type and constraints */
  param_type: ParameterType;
  /** Tooltip help text */
  tooltip?: string;
}

/** Interpretation guide for indicator values */
export interface InterpretationGuide {
  /** High value interpretation */
  high: string;
  /** Medium value interpretation */
  medium: string;
  /** Low value interpretation */
  low: string;
}

/** Complete indicator metadata */
export interface IndicatorMetadata {
  /** Unique identifier */
  id: string;
  /** Display name */
  name: string;
  /** Short description (1 line) */
  short_description: string;
  /** Long description (multiple paragraphs) */
  long_description: string;
  /** Category */
  category: IndicatorCategory;
  /** Parameter definitions */
  parameters: ParameterDefinition[];
  /** Interpretation guide */
  interpretation?: InterpretationGuide;
  /** Recommended use cases */
  use_cases: string[];
  /** Output range [min, max] */
  output_range: [number, number];
  /** Whether output is normalized */
  normalized: boolean;
  /** Recommended timeframes */
  recommended_timeframes: string[];
  /** Algorithm complexity description */
  complexity: string;
  /** Related indicator IDs */
  related: string[];
}

/** Parameter value (runtime) */
export type ParameterValue =
  | { type: 'Integer'; value: number }
  | { type: 'Float'; value: number }
  | { type: 'Boolean'; value: boolean }
  | { type: 'Choice'; value: string }
  | { type: 'Color'; value: string };

/** Indicator configuration (user settings) */
export interface IndicatorConfig {
  /** Indicator ID */
  id: string;
  /** User-provided parameters */
  parameters: Record<string, number | boolean | string>;
  /** Display settings */
  display?: {
    color?: string;
    lineWidth?: number;
    opacity?: number;
  };
}

/** Indicator instance with state */
export interface IndicatorInstance {
  /** Unique instance ID */
  instanceId: string;
  /** Indicator metadata */
  metadata: IndicatorMetadata;
  /** Current configuration */
  config: IndicatorConfig;
  /** WASM indicator instance */
  wasmIndicator: any;
  /** Current values (time series) */
  values: (number | null)[];
  /** Last calculated value */
  lastValue: number | null;
}

/** Regime classification */
export enum MarketRegime {
  HighlyStructured = 'HighlyStructured',
  Trending = 'Trending',
  Ranging = 'Ranging',
  Random = 'Random',
  Transitioning = 'Transitioning',
}

/** Regime analysis result */
export interface RegimeAnalysis {
  /** Current regime */
  regime: MarketRegime;
  /** Confidence [0, 1] */
  confidence: number;
  /** Shannon Entropy value */
  shannon?: number;
  /** Lempel-Ziv Complexity value */
  lempelZiv?: number;
  /** Permutation Entropy value */
  permutation?: number;
  /** Regime color for visualization */
  color: string;
  /** Human-readable description */
  description: string;
}
