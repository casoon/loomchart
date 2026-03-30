# Indicator Migration Plan: plugins/ → indicators/

**Strategy:** Option A - Complete Migration  
**Status:** 20/70 Complete (29%)  
**Estimated Time:** 16-20 hours total (~6 hours invested)  

## Objective
Migrate all 70 indicators from old `plugins/builtin/` system to new `indicators/builtin/` system with standardized IndicatorOutput interface.

---

## Why Migrate?

### Old System (`plugins/`)
❌ Complex plugin architecture  
❌ `IndicatorResult` not render-ready  
❌ Requires conversion layer for rendering  
❌ Trait object overhead  
❌ Harder to test and maintain  

### New System (`indicators/`)
✅ Simple, clear `Indicator` trait  
✅ `IndicatorOutput` enum (render-ready)  
✅ Direct integration with ChartRenderer  
✅ Type-safe with pattern matching  
✅ Easy to test and extend  

---

## Migration Progress

### ✅ Completed (20/70)
1. **RSI** - Relative Strength Index (SingleLine)
2. **SMA** - Simple Moving Average (SingleLine)
3. **EMA** - Exponential Moving Average (SingleLine)
4. **MACD** - Moving Average Convergence Divergence (MultiLine)
5. **Bollinger Bands** - Volatility bands (Bands)
6. **Stochastic** - Momentum oscillator (MultiLine)
7. **ATR** - Average True Range (SingleLine)
8. **Williams %R** - Momentum indicator (SingleLine)
9. **ADX** - Average Directional Index (MultiLine)
10. **HMA** - Hull Moving Average (SingleLine)
11. **WMA** - Weighted Moving Average (SingleLine)
12. **VWMA** - Volume Weighted MA (SingleLine)
13. **Keltner Channels** - Volatility bands (Bands) ✨ NEW
14. **Donchian Channels** - Breakout indicator (Bands) ✨ NEW
15. **CCI** - Commodity Channel Index (SingleLine) ✨ NEW
16. **MFI** - Money Flow Index (SingleLine) ✨ NEW
17. **OBV** - On Balance Volume (SingleLine) ✨ NEW
18. **ROC** - Rate of Change (SingleLine) ✨ NEW
19. **Momentum** - Price momentum (SingleLine) ✨ NEW
20. **Stochastic** - Already migrated

### ✅ Priority Tier 1: COMPLETE! 🎉
- Volume handled by volume_pane.rs

### 📋 Priority Tier 2: Common Indicators (4 remaining)
21. Ichimoku - Cloud indicator
22. Parabolic SAR - Stop and Reverse
23. Supertrend - Trend indicator
24. Aroon - Trend strength
25. DMI - Directional Movement Index

### 📋 Priority Tier 3: Advanced Indicators (20)
26. DEMA - Double EMA
27. TEMA - Triple EMA
28. SMMA - Smoothed MA
29. RMA - Rolling MA
30. ALMA - Arnaud Legoux MA
31. JMA - Jurik MA
32. McGinley Dynamic
33. LSMA - Least Squares MA
34. Median Price
35. Envelope - Price envelope
36. Stochastic RSI - Stochastic of RSI
37. TSI - True Strength Index
38. Ultimate Oscillator
39. Chande Momentum
40. Coppock Curve
41. Fisher Transform
42. RVI - Relative Vigor Index
43. Vortex Indicator
44. Choppiness Index
45. Mass Index

### 📋 Priority Tier 4: Specialized (25)
46. Bull/Bear Power
47. Elder Force Index
48. Klinger Oscillator
49. Chaikin Oscillator
50. Volume Oscillator
51. Price Oscillator
52. EOM - Ease of Movement
53. PVT - Price Volume Trend
54. CVD - Cumulative Volume Delta
55. ADR - Average Daily Range
56. Historical Volatility
57. Standard Deviation
58. DPO - Detrended Price
59. TRIX - Triple EMA Oscillator
60. RCI - Rank Correlation Index
61. SMI Ergodic
62. Trend Strength
63. Woodies CCI
64. Chande Kroll Stop
65. ZigZag
66. MA Cross
67. MA Ribbon
68. Alligator (Williams)
69. BOP - Balance of Power
70. Awesome Oscillator

---

## Migration Template

For each indicator:

```rust
/// [NAME] Indicator
///
/// [DESCRIPTION]

use crate::core::Candle;
use crate::indicators::output::*;
use crate::renderers::Color;
use serde_json::json;

pub struct [IndicatorName] {
    // Parameters
    period: usize,
    color: Color,
    // ... other params
}

impl [IndicatorName] {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            color: Color::rgb(r, g, b),
        }
    }
    
    fn compute_values(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        // Implementation from plugins/builtin/[name].rs
    }
}

impl Indicator for [IndicatorName] {
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput {
        // Return appropriate variant
    }
    
    fn get_scale_range(&self, candles: &[Candle]) -> Option<(f64, f64)> {
        // Auto-scale range or None for price scale
    }
    
    fn supports_overlay(&self) -> bool {
        // true if overlays on price chart
    }
    
    fn name(&self) -> &str { "[NAME]" }
    fn id(&self) -> String { format!("[id]_{}", self.period) }
    
    fn get_params(&self) -> serde_json::Value {
        json!({ "period": self.period })
    }
    
    fn set_params(&mut self, params: serde_json::Value) -> Result<(), String> {
        // Validate and update params
    }
    
    fn required_candles(&self) -> usize { self.period }
}

#[cfg(test)]
mod tests {
    // Comprehensive tests
}
```

---

## IndicatorOutput Mapping

| Indicator Type | Output Variant | Examples |
|----------------|----------------|----------|
| Simple Moving Average | SingleLine | SMA, EMA, RSI, ATR, Williams %R |
| Multiple Lines | MultiLine | MACD, Stochastic, ADX, Ichimoku |
| Bands/Channels | Bands | Bollinger, Keltner, Donchian |
| Volume/Histogram | Histogram | Volume, MACD Histogram |
| Cloud/Area | CloudArea | Ichimoku Cloud |
| Levels/Points | Scatter | Pivot Points, Support/Resistance |

---

## Migration Workflow

### Step 1: Copy Core Logic
```bash
# Copy calculation logic from old indicator
cp crates/chartcore/src/plugins/builtin/[name].rs /tmp/
# Extract calculation function
```

### Step 2: Create New File
```bash
# Create in new location
touch crates/chartcore/src/indicators/builtin/[name].rs
```

### Step 3: Implement Indicator Trait
- Adapt calculate() function
- Choose correct IndicatorOutput variant
- Implement all trait methods
- Add comprehensive tests

### Step 4: Update mod.rs
```rust
pub mod [name];
pub use [name]::[IndicatorName];
```

### Step 5: Test
```bash
cargo test -p chartcore --lib indicators::builtin::[name]
```

### Step 6: Mark Old as Deprecated
```rust
#[deprecated(note = "Use indicators::builtin::[name] instead")]
pub use old_implementation;
```

### Step 7: Delete After Full Migration
```bash
rm crates/chartcore/src/plugins/builtin/[name].rs
```

---

## Batch Processing

### Batch 1: Top 10 (Week 1)
- Days 1-2: Stochastic, ATR, Volume
- Days 3-4: ADX, Williams %R
- Day 5: Testing and refinement

### Batch 2: Common (Week 2)
- Days 1-3: HMA, WMA, VWMA, Ichimoku, Parabolic SAR
- Days 4-5: Supertrend, Keltner, Donchian, CCI, MFI

### Batch 3: Common cont. (Week 3)
- Days 1-3: OBV, ROC, Momentum, Aroon, DMI
- Days 4-5: Testing and integration

### Batch 4: Advanced (Week 4)
- Days 1-5: 20 advanced indicators

### Batch 5: Specialized (Week 5)
- Days 1-5: 25 specialized indicators

---

## Success Metrics

- ✅ All 70 indicators migrated
- ✅ Old `plugins/builtin/` deleted
- ✅ All tests passing
- ✅ No breaking changes in public API
- ✅ Documentation updated
- ✅ ChartRenderer integration working

---

## Risks & Mitigation

### Risk: Breaking Changes
**Mitigation:** Keep old system until full migration, use feature flags

### Risk: Calculation Differences
**Mitigation:** Port exact calculation logic, add comparison tests

### Risk: Time Overrun
**Mitigation:** Prioritize top 20 indicators, batch migration template

### Risk: Missing Edge Cases
**Mitigation:** Comprehensive test suite, visual verification

---

## Next Steps

1. ✅ Complete Top 5 (RSI, SMA, EMA, MACD, BB)
2. 🚧 Start Tier 1: Stochastic, ATR, Volume
3. ⏳ Complete Tier 1 (Top 10)
4. ⏳ Integrate with ChartRenderer
5. ⏳ Continue Tier 2 migration
6. ⏳ Delete old system after full migration

---

## Timeline

| Week | Task | Indicators | Status |
|------|------|------------|--------|
| Week 1 | Top 10 | 10 | 10/10 ✅ |
| Week 2 | Common 1 | 15 | 10/15 🚧 |
| Week 3 | Common 2 | 15 | 0/15 ⏳ |
| Week 4 | Advanced | 20 | 0/20 ⏳ |
| Week 5 | Specialized | 25 | 0/25 ⏳ |
| **Total** | **All** | **70** | **20/70 (29%)** |

---

## Current Session Goal

Complete Tier 1 (Top 10):
- [x] RSI
- [x] SMA
- [x] EMA
- [x] MACD
- [x] Bollinger Bands
- [ ] Stochastic ← NEXT
- [ ] ATR
- [ ] Volume
- [ ] ADX
- [ ] Williams %R

**Ready to continue with Stochastic!** 🚀
