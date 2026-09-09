/**
 * English UI strings.
 * Keys use dot-notation: section.item or section.item.detail
 * Add a new locale file (e.g. de.ts) with the same keys to support more languages.
 */
export const en = {

  // ── Timeframes ────────────────────────────────────────────────
  'tf.1m':  '1 minute',
  'tf.5m':  '5 minutes',
  'tf.15m': '15 minutes',
  'tf.1h':  '1 hour',
  'tf.4h':  '4 hours',
  'tf.1d':  '1 day',

  // ── Chart types ───────────────────────────────────────────────
  'chart.candlestick':     'Candlestick',
  'chart.candlestick.tip': 'Classic OHLC candlestick chart',
  'chart.ohlc':            'OHLC',
  'chart.ohlc.tip':        'Open-High-Low-Close bar chart',
  'chart.hollow':          'Hollow',
  'chart.hollow.tip':      'Hollow-body candlestick — body is filled only on bearish candles',
  'chart.ha':              'Heikin Ashi',
  'chart.ha.tip':          'Heikin Ashi — smoothed candles that reduce noise',
  'chart.line':            'Line',
  'chart.line.tip':        'Line chart connecting close prices',
  'chart.area':            'Area',
  'chart.area.tip':        'Filled area chart below the close price line',
  'chart.footprint':       'Footprint',
  'chart.footprint.tip':   'Footprint candles — bid/ask volume at each price level',
  'chart.renko':           'Renko',
  'chart.renko.tip':       'Renko bricks — each brick represents a fixed price move, time is ignored',

  // ── Scale controls ────────────────────────────────────────────
  'scale.fit':      'Fit',
  'scale.fit.tip':  'Fit all candles into view (F)',
  'scale.log':      'Log',
  'scale.log.tip':  'Toggle logarithmic price scale (L) — useful for long-term charts',
  'scale.lock':     'Axis',
  'scale.lock.tip': 'Lock price axis (A) — prevents auto-fit from adjusting the price range',

  // ── Volume Profile ────────────────────────────────────────────
  'vp.off':      'VP off',
  'vp.off.tip':  'Volume Profile disabled',
  'vp.vpvr':     'VPVR',
  'vp.vpvr.tip': 'Volume Profile Visible Range — volume distribution for the currently visible bars',
  'vp.vpsv':     'VPSV',
  'vp.vpsv.tip': 'Volume Profile Session Volume — volume distribution for the last 24 h trading session',

  // ── Indicators panel ──────────────────────────────────────────
  'indicators':                  'Indicators',
  'indicators.tip':              'Add and manage chart indicators',
  'indicators.on_chart':         'On chart',
  'indicators.on_chart.empty':   'No indicators active',
  'indicators.catalog':          'Catalog',
  'indicators.catalog.overlays': 'Overlays',
  'indicators.catalog.osc':      'Oscillators',
  'indicators.catalog.compare':  'Compare',

  // Indicator catalog entries
  'ind.strategy.name':   'Strategy',
  'ind.strategy.desc':   'SMA crossover — entry/exit signals on chart',
  'ind.divergence.name': 'Divergence',
  'ind.divergence.desc': 'RSI divergence — price vs. momentum mismatches',
  'ind.patterns.name':   'Patterns',
  'ind.patterns.desc':   'Double top / double bottom detection',
  'ind.mtf.name':        'Multi-TF SMA',
  'ind.mtf.desc':        'SMA from one timeframe higher overlaid on chart',
  'ind.news.name':       'News Events',
  'ind.news.desc':       'Macro events and economic releases on chart',

  // Oscillators
  'osc.rsi.tip':   'Relative Strength Index — overbought above 70, oversold below 30',
  'osc.macd.tip':  'MACD histogram — trend-following momentum indicator',
  'osc.stoch.tip': 'Stochastic — compares close to high-low range over N periods',
  'osc.cci.tip':   'Commodity Channel Index — deviation from statistical mean',
  'osc.wr.tip':    'Williams %R — momentum oscillator, similar to stochastic',

  // ── Drawing tools ─────────────────────────────────────────────
  'tool.pointer':    'Pointer',
  'tool.pointer.tip':'Select & pan — drag to pan, scroll to zoom (Esc)',
  'tool.hline':      'Horizontal line',
  'tool.hline.tip':  'Place a horizontal price level line — click to set price',
  'tool.vline':      'Vertical line',
  'tool.vline.tip':  'Place a vertical time marker — click to set time',
  'tool.trend':      'Trend line',
  'tool.trend.tip':  'Draw a trend line — click start, click end',
  'tool.rect':       'Rectangle',
  'tool.rect.tip':   'Draw a price/time rectangle — click two corners',
  'tool.fib':        'Fibonacci',
  'tool.fib.tip':    'Fibonacci retracement — click swing high and swing low',
  'tool.text':       'Text label',
  'tool.text.tip':   'Add a text annotation to the chart',
  'tool.alert':      'Price alert',
  'tool.alert.tip':  'Set a price alert — fires when the last candle crosses the level',
  'tool.clear':      'Clear drawings',
  'tool.clear.tip':  'Remove all drawings from the chart',

  // ── Playback ──────────────────────────────────────────────────
  'playback.static':   'Static',
  'playback.simulate': 'Simulate',
  'playback.replay':   'Replay',
  'playback.static.tip':   'Historical static data — no updates',
  'playback.simulate.tip': 'Live simulation — new candles generated every 1.5 s',
  'playback.replay.tip':   'Step-by-step replay — scrub through history candle by candle',

} as const;

export type TranslationKey = keyof typeof en;
