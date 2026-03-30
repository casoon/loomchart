/**
 * Market Regime Analysis
 *
 * Combines multiple entropy indicators to classify market regimes.
 * Uses the "Entropy Triangle" methodology.
 */

import type { MarketRegime, RegimeAnalysis, IndicatorInstance } from './types';

/**
 * Classify market regime based on three entropy indicators
 */
export function classifyRegime(
  shannonEntropy: number | null,
  lempelZiv: number | null,
  permutationEntropy: number | null
): RegimeAnalysis {
  // Default to transitioning if we don't have all values
  if (shannonEntropy === null && lempelZiv === null && permutationEntropy === null) {
    return {
      regime: 'Transitioning' as MarketRegime,
      confidence: 0.0,
      color: '#808080',
      description: 'Insufficient data for regime classification',
    };
  }

  const sh = shannonEntropy ?? 0.5;
  const lz = lempelZiv ?? 0.5;
  const pe = permutationEntropy ?? 0.5;

  // Calculate average for confidence
  const avg = (sh + lz + pe) / 3;
  const variance = ((sh - avg) ** 2 + (lz - avg) ** 2 + (pe - avg) ** 2) / 3;
  const confidence = Math.max(0, 1 - variance * 4); // Higher variance = lower confidence

  // Regime classification logic

  // HighlyStructured: all indicators low
  if (sh < 0.4 && lz < 0.4 && pe < 0.4) {
    return {
      regime: 'HighlyStructured' as MarketRegime,
      confidence,
      shannon: sh,
      lempelZiv: lz,
      permutation: pe,
      color: '#22c55e', // Green
      description: 'Highly predictable market with clear patterns. Ideal for pattern-based strategies.',
    };
  }

  // Random: all indicators high
  if (sh > 0.7 && lz > 0.7 && pe > 0.7) {
    return {
      regime: 'Random' as MarketRegime,
      confidence,
      shannon: sh,
      lempelZiv: lz,
      permutation: pe,
      color: '#ef4444', // Red
      description: 'Random, unpredictable market. Avoid directional strategies, use market-neutral approaches.',
    };
  }

  // Trending: low permutation, medium-high Shannon
  if (pe < 0.5 && sh > 0.4 && sh < 0.8) {
    return {
      regime: 'Trending' as MarketRegime,
      confidence,
      shannon: sh,
      lempelZiv: lz,
      permutation: pe,
      color: '#3b82f6', // Blue
      description: 'Trending market with momentum. Use trend-following strategies.',
    };
  }

  // Ranging: high Shannon, low LZ (repeating patterns)
  if (sh > 0.6 && lz < 0.5) {
    return {
      regime: 'Ranging' as MarketRegime,
      confidence,
      shannon: sh,
      lempelZiv: lz,
      permutation: pe,
      color: '#f59e0b', // Amber
      description: 'Range-bound market with repeating patterns. Use mean-reversion strategies.',
    };
  }

  // Default: Transitioning
  return {
    regime: 'Transitioning' as MarketRegime,
    confidence,
    shannon: sh,
    lempelZiv: lz,
    permutation: pe,
    color: '#8b5cf6', // Purple
    description: 'Market regime is changing. Reduce position sizes and wait for clarity.',
  };
}

/**
 * Analyze regime with multiple indicator instances
 */
export function analyzeRegime(
  shannonInstance: IndicatorInstance | null,
  lempelZivInstance: IndicatorInstance | null,
  permutationInstance: IndicatorInstance | null
): RegimeAnalysis {
  const sh = shannonInstance?.lastValue ?? null;
  const lz = lempelZivInstance?.lastValue ?? null;
  const pe = permutationInstance?.lastValue ?? null;

  return classifyRegime(sh, lz, pe);
}

/**
 * Get regime color
 */
export function getRegimeColor(regime: MarketRegime): string {
  switch (regime) {
    case 'HighlyStructured':
      return '#22c55e'; // Green
    case 'Trending':
      return '#3b82f6'; // Blue
    case 'Ranging':
      return '#f59e0b'; // Amber
    case 'Random':
      return '#ef4444'; // Red
    case 'Transitioning':
      return '#8b5cf6'; // Purple
    default:
      return '#808080'; // Gray
  }
}

/**
 * Get regime description
 */
export function getRegimeDescription(regime: MarketRegime): string {
  switch (regime) {
    case 'HighlyStructured':
      return 'Highly predictable patterns - use aggressive pattern-based strategies';
    case 'Trending':
      return 'Strong momentum - use trend-following strategies';
    case 'Ranging':
      return 'Range-bound - use mean-reversion strategies';
    case 'Random':
      return 'Random walk - avoid directional bets, use market-neutral strategies';
    case 'Transitioning':
      return 'Regime change in progress - reduce exposure and wait';
    default:
      return 'Unknown regime';
  }
}

/**
 * Get recommended strategy for regime
 */
export function getRecommendedStrategy(regime: MarketRegime): string[] {
  switch (regime) {
    case 'HighlyStructured':
      return [
        'Use all pattern-based strategies',
        'Aggressive position sizing',
        'Tight stops',
        'High confidence trades',
      ];
    case 'Trending':
      return [
        'Trend-following strategies',
        'Momentum indicators (MACD, RSI)',
        'Trail stops aggressively',
        'Add to winners',
      ];
    case 'Ranging':
      return [
        'Mean-reversion strategies',
        'Support/resistance trading',
        'Fade extremes',
        'Quick profit-taking',
      ];
    case 'Random':
      return [
        'Reduce directional exposure',
        'Market-making strategies',
        'Options selling',
        'Conservative position sizing',
      ];
    case 'Transitioning':
      return [
        'Reduce overall position size',
        'Widen stops',
        'Wait for clear regime signal',
        'Monitor entropy changes closely',
      ];
    default:
      return ['No recommendation available'];
  }
}

/**
 * Format entropy value for display
 */
export function formatEntropyValue(value: number | null): string {
  if (value === null) {
    return 'N/A';
  }
  return value.toFixed(3);
}

/**
 * Get value color based on interpretation
 */
export function getValueColor(value: number | null): string {
  if (value === null) {
    return '#808080'; // Gray
  }

  if (value > 0.7) {
    return '#ef4444'; // Red (high/random)
  } else if (value < 0.4) {
    return '#22c55e'; // Green (low/structured)
  } else {
    return '#f59e0b'; // Amber (medium/normal)
  }
}
