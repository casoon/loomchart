/**
 * Layout Presets
 * Task 6.2: Predefined layout templates for common trading setups
 */

export interface LayoutPreset {
  id: string;
  name: string;
  description: string;
  icon: string; // SVG path data
  category: 'simple' | 'technical' | 'advanced';
  panels: Array<{
    type: 'Chart' | 'Indicator' | 'Volume';
    indicator_id?: string;
    params?: Record<string, any>;
    stretch_factor: number;
  }>;
}

/**
 * Available layout presets
 */
export const LAYOUT_PRESETS: LayoutPreset[] = [
  // Simple Layouts
  {
    id: 'simple-chart',
    name: 'Simple Chart',
    description: 'Just the price chart - clean and minimal',
    icon: 'M3 13h2l3-6 2 9 3-9 2 6h2',
    category: 'simple',
    panels: [
      {
        type: 'Chart',
        stretch_factor: 1.0,
      },
    ],
  },
  {
    id: 'chart-volume',
    name: 'Chart + Volume',
    description: 'Price chart with volume panel below',
    icon: 'M3 13h2l3-6 2 9 3-9 2 6h2M3 18v3M8 18v3M13 18v3M18 18v3',
    category: 'simple',
    panels: [
      {
        type: 'Chart',
        stretch_factor: 3.0,
      },
      {
        type: 'Volume',
        stretch_factor: 1.0,
      },
    ],
  },

  // Technical Analysis Layouts
  {
    id: 'momentum-trader',
    name: 'Momentum Trader',
    description: 'Chart with RSI and MACD indicators',
    icon: 'M3 13h2l3-6 2 9 3-9 2 6h2M3 18h18M3 22h18',
    category: 'technical',
    panels: [
      {
        type: 'Chart',
        stretch_factor: 3.0,
      },
      {
        type: 'Indicator',
        indicator_id: 'rsi',
        params: { period: 14 },
        stretch_factor: 1.0,
      },
      {
        type: 'Indicator',
        indicator_id: 'macd',
        params: { fast: 12, slow: 26, signal: 9 },
        stretch_factor: 1.0,
      },
    ],
  },
  {
    id: 'scalping-setup',
    name: 'Scalping Setup',
    description: 'Fast-paced trading with RSI, Stochastic, and Volume',
    icon: 'M3 13h2l3-6 2 9 3-9 2 6h2M3 18h18M8 22h8',
    category: 'technical',
    panels: [
      {
        type: 'Chart',
        stretch_factor: 2.5,
      },
      {
        type: 'Indicator',
        indicator_id: 'rsi',
        params: { period: 7 },
        stretch_factor: 0.8,
      },
      {
        type: 'Indicator',
        indicator_id: 'stochastic',
        params: { k_period: 14, d_period: 3 },
        stretch_factor: 0.8,
      },
      {
        type: 'Volume',
        stretch_factor: 0.9,
      },
    ],
  },
  {
    id: 'swing-trader',
    name: 'Swing Trader',
    description: 'Chart with MACD, Volume, and ADX for trend confirmation',
    icon: 'M3 8l3-3 3 3 3-3 3 3M3 14h18M3 20h18',
    category: 'technical',
    panels: [
      {
        type: 'Chart',
        stretch_factor: 3.0,
      },
      {
        type: 'Indicator',
        indicator_id: 'macd',
        params: { fast: 12, slow: 26, signal: 9 },
        stretch_factor: 1.2,
      },
      {
        type: 'Volume',
        stretch_factor: 0.8,
      },
      {
        type: 'Indicator',
        indicator_id: 'adx',
        params: { period: 14 },
        stretch_factor: 1.0,
      },
    ],
  },

  // Advanced Layouts
  {
    id: 'full-technical',
    name: 'Full Technical Analysis',
    description: 'Complete setup with RSI, MACD, Volume, and Stochastic',
    icon: 'M3 6h18M3 10h18M3 14h18M3 18h18',
    category: 'advanced',
    panels: [
      {
        type: 'Chart',
        stretch_factor: 3.0,
      },
      {
        type: 'Indicator',
        indicator_id: 'rsi',
        params: { period: 14 },
        stretch_factor: 1.0,
      },
      {
        type: 'Indicator',
        indicator_id: 'macd',
        params: { fast: 12, slow: 26, signal: 9 },
        stretch_factor: 1.0,
      },
      {
        type: 'Volume',
        stretch_factor: 0.8,
      },
      {
        type: 'Indicator',
        indicator_id: 'stochastic',
        params: { k_period: 14, d_period: 3 },
        stretch_factor: 0.8,
      },
    ],
  },
  {
    id: 'volume-analysis',
    name: 'Volume Analysis',
    description: 'Focus on volume with MFI and OBV indicators',
    icon: 'M3 18v-3M8 18v-6M13 18v-9M18 18v-12',
    category: 'advanced',
    panels: [
      {
        type: 'Chart',
        stretch_factor: 3.0,
      },
      {
        type: 'Volume',
        stretch_factor: 1.0,
      },
      {
        type: 'Indicator',
        indicator_id: 'mfi',
        params: { period: 14 },
        stretch_factor: 1.0,
      },
      {
        type: 'Indicator',
        indicator_id: 'obv',
        params: {},
        stretch_factor: 1.0,
      },
    ],
  },
  {
    id: 'multi-oscillator',
    name: 'Multi-Oscillator',
    description: 'RSI, CCI, Williams %R, and Stochastic for overbought/oversold',
    icon: 'M3 12h18M3 8l3-3 3 3 3-3 3 3 3-3M3 16l3 3 3-3 3 3 3-3 3 3',
    category: 'advanced',
    panels: [
      {
        type: 'Chart',
        stretch_factor: 2.5,
      },
      {
        type: 'Indicator',
        indicator_id: 'rsi',
        params: { period: 14 },
        stretch_factor: 0.8,
      },
      {
        type: 'Indicator',
        indicator_id: 'cci',
        params: { period: 14 },
        stretch_factor: 0.8,
      },
      {
        type: 'Indicator',
        indicator_id: 'williams_r',
        params: { period: 14 },
        stretch_factor: 0.8,
      },
      {
        type: 'Indicator',
        indicator_id: 'stochastic',
        params: { k_period: 14, d_period: 3 },
        stretch_factor: 0.8,
      },
    ],
  },
];

/**
 * Get presets by category
 */
export function getPresetsByCategory(category: LayoutPreset['category']): LayoutPreset[] {
  return LAYOUT_PRESETS.filter((preset) => preset.category === category);
}

/**
 * Get preset by ID
 */
export function getPresetById(id: string): LayoutPreset | undefined {
  return LAYOUT_PRESETS.find((preset) => preset.id === id);
}

/**
 * Apply a preset to create panel configuration
 */
export async function applyPreset(presetId: string): Promise<boolean> {
  const preset = getPresetById(presetId);
  if (!preset) {
    console.error(`Preset not found: ${presetId}`);
    return false;
  }

  const wasm = (window as any).getWasm?.();
  if (!wasm) {
    console.error('WASM not initialized');
    return false;
  }

  try {
    // Clear existing panels (except main chart which will be reconfigured)
    const currentLayout = JSON.parse(await wasm.get_panel_layout());
    const panelIds = currentLayout.panels
      .filter((p: any) => p.panel_type.type !== 'Chart')
      .map((p: any) => p.id);

    for (const panelId of panelIds) {
      try {
        await wasm.remove_panel(panelId);
      } catch (err) {
        console.warn(`Failed to remove panel ${panelId}:`, err);
      }
    }

    // Add panels from preset
    for (const panelConfig of preset.panels) {
      if (panelConfig.type === 'Chart') {
        // Main chart already exists, just configure it
        continue;
      }

      if (panelConfig.type === 'Volume') {
        // Add volume panel
        await wasm.add_volume_panel?.();
      } else if (panelConfig.type === 'Indicator' && panelConfig.indicator_id) {
        // Add indicator panel
        const params = JSON.stringify(panelConfig.params || {});
        await wasm.add_indicator_panel?.(panelConfig.indicator_id, params);
      }
    }

    // Refresh panels to show changes
    await (window as any).refreshPanels?.();

    console.log(`Applied preset: ${preset.name}`);
    return true;
  } catch (err) {
    console.error(`Failed to apply preset ${presetId}:`, err);
    return false;
  }
}
