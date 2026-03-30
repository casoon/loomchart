// Indicator Configurations and Presets
// Each indicator has default settings and multiple presets

export interface IndicatorParameter {
  name: string;
  label: string;
  type: 'number' | 'select' | 'color' | 'boolean';
  default: any;
  min?: number;
  max?: number;
  step?: number;
  options?: { label: string; value: any }[];
  description?: string;
}

export interface IndicatorPreset {
  id: string;
  name: string;
  description: string;
  params: Record<string, any>;
}

export interface IndicatorConfig {
  id: string;
  name: string;
  category: string;
  description: string;
  supportsOverlay: boolean;
  requiresPanel: boolean;
  defaultScale?: { min: number; max: number };
  parameters: IndicatorParameter[];
  presets: IndicatorPreset[];
  defaultColors?: {
    line?: string;
    up?: string;
    down?: string;
    signal?: string;
    histogram?: string;
  };
}

export const INDICATOR_CONFIGS: Record<string, IndicatorConfig> = {
  // === EMA ===
  ema9: {
    id: 'ema9',
    name: 'EMA 9',
    category: 'Trend',
    description: 'Exponential Moving Average',
    supportsOverlay: true,
    requiresPanel: false,
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 9, min: 2, max: 500, step: 1 },
      { name: 'color', label: 'Color', type: 'color', default: '#3b82f6' },
      { name: 'lineWidth', label: 'Line Width', type: 'number', default: 2, min: 1, max: 5, step: 1 },
    ],
    presets: [
      { id: 'fast', name: 'Fast (9)', description: 'Quick response', params: { period: 9 } },
      { id: 'medium', name: 'Medium (21)', description: 'Balanced', params: { period: 21 } },
      { id: 'slow', name: 'Slow (50)', description: 'Trend following', params: { period: 50 } },
      { id: 'very_slow', name: 'Very Slow (200)', description: 'Long-term trend', params: { period: 200 } },
    ],
    defaultColors: { line: '#3b82f6' },
  },

  ema21: {
    id: 'ema21',
    name: 'EMA 21',
    category: 'Trend',
    description: 'Exponential Moving Average',
    supportsOverlay: true,
    requiresPanel: false,
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 21, min: 2, max: 500, step: 1 },
      { name: 'color', label: 'Color', type: 'color', default: '#8b5cf6' },
      { name: 'lineWidth', label: 'Line Width', type: 'number', default: 2, min: 1, max: 5, step: 1 },
    ],
    presets: [
      { id: 'fast', name: 'Fast (9)', description: 'Quick response', params: { period: 9 } },
      { id: 'medium', name: 'Medium (21)', description: 'Balanced', params: { period: 21 } },
      { id: 'slow', name: 'Slow (50)', description: 'Trend following', params: { period: 50 } },
    ],
    defaultColors: { line: '#8b5cf6' },
  },

  // === RSI ===
  rsi: {
    id: 'rsi',
    name: 'RSI',
    category: 'Momentum',
    description: 'Relative Strength Index',
    supportsOverlay: true,
    requiresPanel: false,
    defaultScale: { min: 0, max: 100 },
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 14, min: 2, max: 100, step: 1 },
      { name: 'overbought', label: 'Overbought', type: 'number', default: 70, min: 50, max: 90, step: 5 },
      { name: 'oversold', label: 'Oversold', type: 'number', default: 30, min: 10, max: 50, step: 5 },
      { name: 'color', label: 'Line Color', type: 'color', default: '#f59e0b' },
      { name: 'showLevels', label: 'Show Levels', type: 'boolean', default: true },
    ],
    presets: [
      { id: 'standard', name: 'Standard (14)', description: 'Default settings', params: { period: 14, overbought: 70, oversold: 30 } },
      { id: 'fast', name: 'Fast (7)', description: 'More sensitive', params: { period: 7, overbought: 75, oversold: 25 } },
      { id: 'slow', name: 'Slow (21)', description: 'Smoother', params: { period: 21, overbought: 65, oversold: 35 } },
      { id: 'extreme', name: 'Extreme (14)', description: 'Tighter levels', params: { period: 14, overbought: 80, oversold: 20 } },
    ],
    defaultColors: { line: '#f59e0b' },
  },

  rsi14: {
    id: 'rsi14',
    name: 'RSI 14',
    category: 'Momentum',
    description: 'RSI with period 14',
    supportsOverlay: true,
    requiresPanel: false,
    defaultScale: { min: 0, max: 100 },
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 14, min: 2, max: 100, step: 1 },
      { name: 'overbought', label: 'Overbought', type: 'number', default: 70, min: 50, max: 90, step: 5 },
      { name: 'oversold', label: 'Oversold', type: 'number', default: 30, min: 10, max: 50, step: 5 },
      { name: 'color', label: 'Line Color', type: 'color', default: '#f59e0b' },
      { name: 'showLevels', label: 'Show Levels', type: 'boolean', default: true },
    ],
    presets: [
      { id: 'standard', name: 'Standard', description: 'Default RSI 14', params: { period: 14, overbought: 70, oversold: 30 } },
      { id: 'fast', name: 'Fast (7)', description: 'More sensitive', params: { period: 7, overbought: 75, oversold: 25 } },
    ],
    defaultColors: { line: '#f59e0b' },
  },

  // === MACD ===
  macd: {
    id: 'macd',
    name: 'MACD',
    category: 'Momentum',
    description: 'Moving Average Convergence Divergence',
    supportsOverlay: false,
    requiresPanel: true,
    parameters: [
      { name: 'fastPeriod', label: 'Fast Period', type: 'number', default: 12, min: 2, max: 50, step: 1 },
      { name: 'slowPeriod', label: 'Slow Period', type: 'number', default: 26, min: 2, max: 100, step: 1 },
      { name: 'signalPeriod', label: 'Signal Period', type: 'number', default: 9, min: 2, max: 50, step: 1 },
      { name: 'macdColor', label: 'MACD Color', type: 'color', default: '#3b82f6' },
      { name: 'signalColor', label: 'Signal Color', type: 'color', default: '#ef4444' },
      { name: 'histogramUp', label: 'Histogram Up', type: 'color', default: '#10b981' },
      { name: 'histogramDown', label: 'Histogram Down', type: 'color', default: '#ef4444' },
    ],
    presets: [
      { id: 'standard', name: 'Standard (12,26,9)', description: 'Default MACD', params: { fastPeriod: 12, slowPeriod: 26, signalPeriod: 9 } },
      { id: 'fast', name: 'Fast (6,13,5)', description: 'Quick signals', params: { fastPeriod: 6, slowPeriod: 13, signalPeriod: 5 } },
      { id: 'slow', name: 'Slow (19,39,9)', description: 'Smoother signals', params: { fastPeriod: 19, slowPeriod: 39, signalPeriod: 9 } },
    ],
    defaultColors: { line: '#3b82f6', signal: '#ef4444', up: '#10b981', down: '#ef4444' },
  },

  // === MFI ===
  mfi: {
    id: 'mfi',
    name: 'MFI',
    category: 'Volume',
    description: 'Money Flow Index',
    supportsOverlay: true,
    requiresPanel: false,
    defaultScale: { min: 0, max: 100 },
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 14, min: 2, max: 100, step: 1 },
      { name: 'overbought', label: 'Overbought', type: 'number', default: 80, min: 50, max: 95, step: 5 },
      { name: 'oversold', label: 'Oversold', type: 'number', default: 20, min: 5, max: 50, step: 5 },
      { name: 'color', label: 'Line Color', type: 'color', default: '#06b6d4' },
      { name: 'showLevels', label: 'Show Levels', type: 'boolean', default: true },
    ],
    presets: [
      { id: 'standard', name: 'Standard (14)', description: 'Default MFI', params: { period: 14, overbought: 80, oversold: 20 } },
      { id: 'sensitive', name: 'Sensitive (10)', description: 'More reactive', params: { period: 10, overbought: 75, oversold: 25 } },
    ],
    defaultColors: { line: '#06b6d4' },
  },

  // === Bollinger Bands ===
  bb: {
    id: 'bb',
    name: 'Bollinger Bands',
    category: 'Volatility',
    description: 'Price volatility indicator',
    supportsOverlay: true,
    requiresPanel: false,
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 20, min: 2, max: 100, step: 1 },
      { name: 'stdDev', label: 'Std Deviation', type: 'number', default: 2, min: 0.5, max: 4, step: 0.5 },
      { name: 'basisColor', label: 'Basis Color', type: 'color', default: '#64748b' },
      { name: 'upperColor', label: 'Upper Band', type: 'color', default: '#ef4444' },
      { name: 'lowerColor', label: 'Lower Band', type: 'color', default: '#10b981' },
      { name: 'fillBands', label: 'Fill Between Bands', type: 'boolean', default: true },
    ],
    presets: [
      { id: 'standard', name: 'Standard (20,2)', description: 'Default BB', params: { period: 20, stdDev: 2 } },
      { id: 'tight', name: 'Tight (20,1.5)', description: 'Narrower bands', params: { period: 20, stdDev: 1.5 } },
      { id: 'wide', name: 'Wide (20,3)', description: 'Wider bands', params: { period: 20, stdDev: 3 } },
    ],
    defaultColors: { line: '#64748b', up: '#ef4444', down: '#10b981' },
  },

  // === Stochastic ===
  stoch: {
    id: 'stoch',
    name: 'Stochastic',
    category: 'Momentum',
    description: 'Stochastic Oscillator',
    supportsOverlay: true,
    requiresPanel: false,
    defaultScale: { min: 0, max: 100 },
    parameters: [
      { name: 'kPeriod', label: '%K Period', type: 'number', default: 14, min: 2, max: 100, step: 1 },
      { name: 'dPeriod', label: '%D Period', type: 'number', default: 3, min: 2, max: 50, step: 1 },
      { name: 'smooth', label: 'Smooth', type: 'number', default: 3, min: 1, max: 10, step: 1 },
      { name: 'overbought', label: 'Overbought', type: 'number', default: 80, min: 50, max: 95, step: 5 },
      { name: 'oversold', label: 'Oversold', type: 'number', default: 20, min: 5, max: 50, step: 5 },
      { name: 'kColor', label: '%K Color', type: 'color', default: '#3b82f6' },
      { name: 'dColor', label: '%D Color', type: 'color', default: '#ef4444' },
    ],
    presets: [
      { id: 'standard', name: 'Standard (14,3,3)', description: 'Default stochastic', params: { kPeriod: 14, dPeriod: 3, smooth: 3 } },
      { id: 'fast', name: 'Fast (5,3,3)', description: 'Quick signals', params: { kPeriod: 5, dPeriod: 3, smooth: 3 } },
      { id: 'slow', name: 'Slow (21,5,5)', description: 'Smoother', params: { kPeriod: 21, dPeriod: 5, smooth: 5 } },
    ],
    defaultColors: { line: '#3b82f6', signal: '#ef4444' },
  },

  // === ATR ===
  atr: {
    id: 'atr',
    name: 'ATR',
    category: 'Volatility',
    description: 'Average True Range',
    supportsOverlay: false,
    requiresPanel: true,
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 14, min: 2, max: 100, step: 1 },
      { name: 'color', label: 'Color', type: 'color', default: '#8b5cf6' },
      { name: 'lineWidth', label: 'Line Width', type: 'number', default: 2, min: 1, max: 5, step: 1 },
    ],
    presets: [
      { id: 'standard', name: 'Standard (14)', description: 'Default ATR', params: { period: 14 } },
      { id: 'short', name: 'Short (7)', description: 'Quick volatility', params: { period: 7 } },
      { id: 'long', name: 'Long (21)', description: 'Smoothed volatility', params: { period: 21 } },
    ],
    defaultColors: { line: '#8b5cf6' },
  },

  // === OBV ===
  obv: {
    id: 'obv',
    name: 'OBV',
    category: 'Volume',
    description: 'On-Balance Volume',
    supportsOverlay: false,
    requiresPanel: true,
    parameters: [
      { name: 'color', label: 'Color', type: 'color', default: '#06b6d4' },
      { name: 'lineWidth', label: 'Line Width', type: 'number', default: 2, min: 1, max: 5, step: 1 },
      { name: 'showMA', label: 'Show MA', type: 'boolean', default: false },
      { name: 'maPeriod', label: 'MA Period', type: 'number', default: 20, min: 2, max: 100, step: 1 },
    ],
    presets: [
      { id: 'standard', name: 'Standard', description: 'OBV only', params: { showMA: false } },
      { id: 'with_ma', name: 'With MA (20)', description: 'OBV with moving average', params: { showMA: true, maPeriod: 20 } },
    ],
    defaultColors: { line: '#06b6d4' },
  },

  // === CCI ===
  cci: {
    id: 'cci',
    name: 'CCI',
    category: 'Momentum',
    description: 'Commodity Channel Index',
    supportsOverlay: true,
    requiresPanel: false,
    defaultScale: { min: -200, max: 200 },
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 20, min: 2, max: 100, step: 1 },
      { name: 'overbought', label: 'Overbought', type: 'number', default: 100, min: 50, max: 200, step: 10 },
      { name: 'oversold', label: 'Oversold', type: 'number', default: -100, min: -200, max: -50, step: 10 },
      { name: 'color', label: 'Color', type: 'color', default: '#f97316' },
    ],
    presets: [
      { id: 'standard', name: 'Standard (20)', description: 'Default CCI', params: { period: 20, overbought: 100, oversold: -100 } },
      { id: 'fast', name: 'Fast (14)', description: 'Quick signals', params: { period: 14, overbought: 100, oversold: -100 } },
    ],
    defaultColors: { line: '#f97316' },
  },

  // === SMA ===
  sma: {
    id: 'sma',
    name: 'SMA',
    category: 'Trend',
    description: 'Simple Moving Average',
    supportsOverlay: true,
    requiresPanel: false,
    parameters: [
      { name: 'period', label: 'Period', type: 'number', default: 20, min: 2, max: 500, step: 1 },
      { name: 'color', label: 'Color', type: 'color', default: '#10b981' },
      { name: 'lineWidth', label: 'Line Width', type: 'number', default: 2, min: 1, max: 5, step: 1 },
    ],
    presets: [
      { id: 'fast', name: 'Fast (20)', description: 'Short-term', params: { period: 20 } },
      { id: 'medium', name: 'Medium (50)', description: 'Medium-term', params: { period: 50 } },
      { id: 'slow', name: 'Slow (200)', description: 'Long-term trend', params: { period: 200 } },
    ],
    defaultColors: { line: '#10b981' },
  },
};

// Helper to get indicator config
export function getIndicatorConfig(id: string): IndicatorConfig | undefined {
  return INDICATOR_CONFIGS[id];
}

// Helper to get default parameters
export function getDefaultParams(indicatorId: string): Record<string, any> {
  const config = INDICATOR_CONFIGS[indicatorId];
  if (!config) return {};

  const params: Record<string, any> = {};
  config.parameters.forEach(param => {
    params[param.name] = param.default;
  });
  return params;
}

// Helper to apply preset
export function applyPreset(indicatorId: string, presetId: string): Record<string, any> | null {
  const config = INDICATOR_CONFIGS[indicatorId];
  if (!config) return null;

  const preset = config.presets.find(p => p.id === presetId);
  if (!preset) return null;

  // Merge preset params with defaults
  const defaults = getDefaultParams(indicatorId);
  return { ...defaults, ...preset.params };
}
