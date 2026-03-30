# Scientific Indicators Roadmap
## Advanced Technical Analysis für Loom Trading Platform

**Quelle:** advanced-technical-analysis-master (Python Reference)  
**Ziel:** Rust/WASM Implementation für Echtzeit-Trading  
**Status:** Planungsphase  
**Erstellt:** 2026-01-02

---

## Executive Summary

Das advanced-technical-analysis Repository bietet **wissenschaftliche Indikatoren** aus:
- **Information Theory** (Shannon Entropy, Approximate Entropy, Mutual Information)
- **Chaos Theory** (Lyapunov Exponent, RQA)
- **Fractal Analysis** (Multifractal Spectrum, Fractal Dimension)
- **Complexity Theory** (Lempel-Ziv, Permutation Entropy)
- **Network Theory** (Visibility Graph)

Diese Indikatoren sind in **Standard-Trading-Bibliotheken NICHT vorhanden** und bieten:
✅ **Market Regime Detection** - Automatische Erkennung von Marktphasen  
✅ **Predictability Assessment** - Vorhersagbarkeit messen  
✅ **Crisis Detection** - Frühwarnsignale für Marktkrisen  
✅ **Efficiency Measurement** - Markteffizienz quantifizieren

---

## Problem Statement

### Was Loom bereits hat (60+ Indikatoren)

**Trend:** SMA, EMA, MACD, ADX, Aroon, Ichimoku, Parabolic SAR, Supertrend  
**Momentum:** RSI, Stochastic, Williams %R, CCI, Ultimate, KST, TRIX  
**Volatility:** ATR, Bollinger Bands, Keltner, Chandelier  
**Volume:** OBV, Chaikin, CMF, VWAP, VWMA

### Was fehlt (Wissenschaftliche Indikatoren)

❌ **Keine Information-Theory Metriken** (Entropy, Complexity)  
❌ **Keine Chaos-Theory Analyse** (Lyapunov, RQA)  
❌ **Keine Fractal-Analyse** (Multifractal Spectrum)  
❌ **Keine Regime-Detection** außer Pattern Recognition

---

## Indicator Categories & Prioritization

### Tier 1: High Value, Medium Effort (🚀 Quick Wins)

#### 1. Shannon Entropy
**Purpose:** Misst Zufälligkeit/Unordnung in Preisbewegungen  
**Formula:** `H = -∑(pk * log₂(pk))`  
**Use Case:**
- Market Regime Detection (Random vs Trending)
- Volatility Forecasting
- Liquidity Assessment

**Implementation:**
```rust
pub struct ShannonEntropy {
    period: usize,
    bins: usize,  // Histogram bins (default: 10)
}

impl Next<f64> for ShannonEntropy {
    type Output = f64;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // 1. Maintain rolling window
        // 2. Create histogram with bins
        // 3. Calculate probabilities pk
        // 4. Compute H = -∑(pk * log2(pk))
        // 5. Return entropy value
    }
}
```

**Complexity:** O(n) time, O(n) space  
**Streaming:** ✅ Yes (with rolling window)  
**Estimated Effort:** 2-3 days  
**Priority:** ⭐⭐⭐⭐⭐ (HIGHEST)

**Benefits:**
- Simple to implement
- Low computational cost
- High trading value
- Real-time compatible

---

#### 2. Lempel-Ziv Complexity
**Purpose:** Misst Randomness via Datenkompression (LZ76 Algorithm)  
**Formula:** Counts distinct patterns in binary sequence  
**Use Case:**
- Market Efficiency Measurement
- Pattern Detection
- Regime Identification

**Implementation:**
```rust
pub struct LempelZivComplexity {
    period: usize,
    threshold: f64,  // For binarization
}

impl Next<f64> for LempelZivComplexity {
    type Output = f64;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // 1. Binarize price changes (above/below threshold)
        // 2. Apply LZ76 algorithm on sliding window
        // 3. Count unique patterns
        // 4. Normalize by theoretical maximum
        // 5. Return complexity ratio
    }
}
```

**Complexity:** O(n²) worst-case, O(n log n) average  
**Streaming:** ⚠️ Batch processing recommended  
**Estimated Effort:** 1-2 days  
**Priority:** ⭐⭐⭐⭐ (HIGH)

**Benefits:**
- Pattern-based approach
- Compression analogy intuitive
- Market efficiency proxy

---

### Tier 2: Medium Value, High Effort (📊 Advanced Tools)

#### 3. Approximate Entropy (ApEn)
**Purpose:** Robuste Entropy-Metrik, resistent gegen Noise  
**Parameters:**
- `m` - Embedding dimension (default: 2)
- `r` - Tolerance (default: 0.2 * std_dev)

**Formula:**
```
ApEn(m, r, N) = φ(m) - φ(m+1)
where φ(m) = (1/(N-m+1)) * ∑ log(Cm_i(r))
```

**Use Case:**
- Complexity Assessment
- Pattern Recognition
- More robust than Shannon for noisy data

**Implementation:**
```rust
pub struct ApproximateEntropy {
    period: usize,
    embedding_dim: usize,  // m
    tolerance: f64,        // r (as ratio of std_dev)
}

impl Next<f64> for ApproximateEntropy {
    type Output = f64;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // 1. Create embedded vectors (dimension m and m+1)
        // 2. Calculate distances between all pairs
        // 3. Count matches within tolerance r
        // 4. Compute φ(m) and φ(m+1)
        // 5. Return ApEn = φ(m) - φ(m+1)
    }
}
```

**Complexity:** O(n · m²) time  
**Streaming:** ❌ No (requires embedding)  
**Estimated Effort:** 3-4 days  
**Priority:** ⭐⭐⭐ (MEDIUM)

**Benefits:**
- More robust than Shannon
- Established in medical/financial literature
- Parameter-tunable

---

#### 4. Fractal Dimension
**Purpose:** Quantifiziert Komplexität von Preis-Patterns  
**Methods:**
- Box-Counting (einfachste)
- Correlation Dimension
- Higuchi Method

**Use Case:**
- Pattern Complexity Measurement
- Market State Classification
- Trend vs Range Detection

**Implementation (Box-Counting):**
```rust
pub struct FractalDimension {
    period: usize,
    min_box_size: usize,
    max_box_size: usize,
}

impl Next<f64> for FractalDimension {
    type Output = f64;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // 1. Normalize price series to [0,1] grid
        // 2. For each box size:
        //    - Count boxes needed to cover data
        // 3. Log-log regression: log(count) vs log(1/box_size)
        // 4. Slope = fractal dimension
        // 5. Return dimension value
    }
}
```

**Complexity:** O(n · log n) time  
**Streaming:** ⚠️ Batch recommended  
**Estimated Effort:** 4-5 days  
**Priority:** ⭐⭐⭐ (MEDIUM)

**Benefits:**
- Visual/intuitive interpretation
- Multiple methods available
- Research-backed

---

#### 5. Permutation Entropy
**Purpose:** Analysiert Ordinal Patterns in Zeitreihen  
**Characteristics:**
- Robust gegen Noise
- Scale-invariant
- Computationally efficient

**Use Case:**
- Microstructure Analysis
- HFT Signals
- Regime Changes

**Implementation:**
```rust
pub struct PermutationEntropy {
    period: usize,
    embedding_dim: usize,  // Order of permutations (default: 3)
}

impl Next<f64> for PermutationEntropy {
    type Output = f64;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // 1. Create overlapping windows of size embedding_dim
        // 2. Convert each window to ordinal pattern (rank order)
        // 3. Count frequency of each permutation
        // 4. Calculate Shannon entropy of permutation distribution
        // 5. Normalize by log(embedding_dim!)
    }
}
```

**Complexity:** O(n · m!) where m = embedding_dim  
**Streaming:** ✅ Yes (with rolling window)  
**Estimated Effort:** 2-3 days  
**Priority:** ⭐⭐⭐⭐ (HIGH)

**Benefits:**
- Fast computation
- Noise-resistant
- Natural streaming implementation

---

### Tier 3: Specialized, Very High Effort (🔬 Research Tools)

#### 6. Lyapunov Exponent
**Purpose:** Misst Sensitivität zu Anfangsbedingungen (Chaos)  
**Interpretation:**
- **Positive:** Chaotic behavior (unpredictable)
- **Negative:** Stable patterns (predictable)
- **Zero:** Periodic behavior

**Use Case:**
- Predictability Assessment
- Market Stability Analysis
- Crisis Detection

**Implementation:**
```rust
pub struct LyapunovExponent {
    period: usize,
    embedding_dim: usize,
    time_delay: usize,
    epsilon: f64,  // Initial distance threshold
}

impl Next<f64> for LyapunovExponent {
    type Output = f64;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // 1. Reconstruct phase space (embedding)
        // 2. Find nearest neighbors (distance < epsilon)
        // 3. Track divergence over time
        // 4. Linear regression on log(divergence) vs time
        // 5. Slope = Lyapunov exponent
    }
}
```

**Complexity:** O(n²) time, O(n) space  
**Streaming:** ❌ No (requires full history)  
**Estimated Effort:** 6-8 days  
**Priority:** ⭐⭐ (LOW-MEDIUM)

**Benefits:**
- Chaos theory foundation
- Predictability metric
- Academic rigor

**Challenges:**
- Parameter-sensitive
- Computationally expensive
- Interpretation complex

---

#### 7. Recurrence Quantification Analysis (RQA)
**Purpose:** Analysiert wiederkehrende Patterns in Phase Space  
**Metrics:**
- **RR (Recurrence Rate):** Pattern repetitiveness
- **DET (Determinism):** Predictability
- **LAM (Laminarity):** Stability

**Use Case:**
- Regime Detection
- Early Warning Signals
- Market Structure Analysis

**Implementation:**
```rust
pub struct RecurrenceQuantification {
    period: usize,
    embedding_dim: usize,  // Default: 2
    time_delay: usize,     // Default: 2
    radius: f64,           // Default: 0.65 (Euclidean metric)
}

impl Next<f64> for RecurrenceQuantification {
    type Output = RqaMetrics;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // 1. Reconstruct phase space (embedding + delay)
        // 2. Calculate distance matrix (all pairs)
        // 3. Create recurrence matrix (distances < radius)
        // 4. Compute RR, DET, LAM from matrix structure
        // 5. Return metrics struct
    }
}

pub struct RqaMetrics {
    recurrence_rate: f64,
    determinism: f64,
    laminarity: f64,
}
```

**Complexity:** O(n²) time, O(n²) space ⚠️  
**Streaming:** ❌ No (distance matrix required)  
**Estimated Effort:** 10+ days  
**Priority:** ⭐ (LOW - Research Only)

**Benefits:**
- Comprehensive market analysis
- Multiple metrics from single calculation
- Research-grade tool

**Challenges:**
- O(n²) space = memory intensive
- Not suitable for real-time
- Complex interpretation

---

#### 8. Multifractal Spectrum Analysis
**Purpose:** Assesses market structure complexity  
**Method:** Chhabra-Jensen implementation  
**Outputs:**
- **α (Hölder exponent):** Scaling behavior
- **f(α) (Hausdorff dimension):** Singularity spectrum
- **D(q) (Generalized fractal dimension):** q-dependent scaling

**Metrics:**
- **Δα (width):** Range of scaling behaviors
- **Δf(α) (height):** Complexity variation

**Use Case:**
- Market Efficiency Assessment
- Crisis Prediction
- Trading Optimization

**Implementation:**
```rust
pub struct MultifractalAnalysis {
    period: usize,
    q_range: (i32, i32),      // Default: (-40, 40)
    scale_range: (usize, usize), // Default: (1, 7) dyadic scales
}

impl Next<f64> for MultifractalAnalysis {
    type Output = MultifractalSpectrum;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // 1. Compute partition function for each q and scale
        // 2. Calculate mass exponents τ(q)
        // 3. Legendre transform: α(q) and f(α)
        // 4. Extract spectrum metrics (width, height)
        // 5. Return spectrum structure
    }
}

pub struct MultifractalSpectrum {
    alpha: Vec<f64>,          // Hölder exponents
    f_alpha: Vec<f64>,        // Spectrum values
    d_q: Vec<f64>,            // Generalized dimensions
    width: f64,               // Δα
    height: f64,              // Δf
}
```

**Complexity:** O(n · log² n) time  
**Streaming:** ❌ No (scale iterations required)  
**Estimated Effort:** 12+ days  
**Priority:** ⭐ (LOW - Research Only)

**Benefits:**
- Most comprehensive fractal analysis
- Crisis detection capability
- Academic research tool

**Challenges:**
- Extremely complex mathematics
- Parameter-sensitive
- Interpretation requires expertise
- Not for casual traders

---

## Integration Strategy

### Architecture Decision

**Problem:** Python library vs Rust/WASM streaming architecture

**Solution: Hybrid Approach**

```
┌─────────────────────────────────────────────────────────────┐
│  Frontend (TypeScript/WASM)                                 │
│  ├─ Real-time Indicators (Shannon, Permutation, LZ)        │
│  └─ Chart Visualization                                     │
└──────────────────────┬──────────────────────────────────────┘
                       │ WebSocket
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  Rust WASM Core                                             │
│  ├─ Streaming Indicators (Entropy, Complexity)             │
│  └─ Incremental Calculations                                │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTP/gRPC
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  Python Analysis Service (Optional)                         │
│  ├─ Batch Indicators (Lyapunov, RQA, Multifractal)        │
│  ├─ Scheduled Jobs (Daily/Hourly Analysis)                 │
│  └─ Research Tools (Deep Dive Analysis)                    │
└─────────────────────────────────────────────────────────────┘
```

**Phase 1:** Rust-only (Tier 1 + Tier 2)
- Implement streaming-compatible indicators
- Shannon, LZ, Permutation, ApEn, Fractal Dimension
- Full WASM integration

**Phase 2:** Hybrid (Tier 3 - Optional)
- Python service for research indicators
- Lyapunov, RQA, Multifractal
- Async task queue
- Results via WebSocket/REST

---

## Implementation Roadmap

### Sprint 1: Foundation (Week 1-2)
**Goal:** Tier 1 indicators operational

- [ ] Create `indicators/information/` module
- [ ] Implement Shannon Entropy (2 days)
- [ ] Implement Lempel-Ziv Complexity (2 days)
- [ ] Unit tests + integration tests (1 day)
- [ ] Documentation + usage examples (1 day)

**Deliverable:** 2 new indicators in Loom

---

### Sprint 2: Advanced Metrics (Week 3-4)
**Goal:** Tier 2 indicators operational

- [ ] Implement Permutation Entropy (2 days)
- [ ] Implement Approximate Entropy (3 days)
- [ ] Implement Fractal Dimension (4 days)
- [ ] Performance optimization (1 day)

**Deliverable:** 5 total scientific indicators

---

### Sprint 3: Visualization & UX (Week 5)
**Goal:** User-facing features

- [ ] Add indicators to indicator selector UI
- [ ] Create visualization templates
- [ ] Add tooltips/help text
- [ ] Parameter configuration UI
- [ ] Example strategies/tutorials

**Deliverable:** Full UI integration

---

### Sprint 4: Research Tools (Week 6+) - OPTIONAL
**Goal:** Advanced analysis capabilities

- [ ] Evaluate Python service architecture
- [ ] Implement Lyapunov Exponent (optional)
- [ ] Create async task queue
- [ ] WebSocket notification system

**Deliverable:** Research-grade tooling (if needed)

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_entropy_random() {
        // Random data should have high entropy (~1.0)
        let mut indicator = ShannonEntropy::new(100, 10);
        let random_data: Vec<f64> = (0..100).map(|_| rand::random()).collect();
        
        let entropy = random_data.iter()
            .map(|&x| indicator.next(x))
            .last()
            .unwrap();
        
        assert!(entropy > 0.9, "Random data entropy too low: {}", entropy);
    }

    #[test]
    fn test_shannon_entropy_constant() {
        // Constant data should have zero entropy
        let mut indicator = ShannonEntropy::new(100, 10);
        let constant_data = vec![42.0; 100];
        
        let entropy = constant_data.iter()
            .map(|&x| indicator.next(x))
            .last()
            .unwrap();
        
        assert!(entropy < 0.1, "Constant data entropy too high: {}", entropy);
    }
}
```

### Integration Tests
```rust
#[test]
fn test_entropy_on_real_market_data() {
    let data = load_btc_data("test_data/btc_2024.csv");
    let mut entropy = ShannonEntropy::new(30, 10);
    
    let results: Vec<f64> = data.iter()
        .map(|candle| entropy.next(candle.close))
        .collect();
    
    // During volatile periods, entropy should be high
    // During ranging periods, entropy should be low
    assert!(results.iter().any(|&e| e > 0.8));
    assert!(results.iter().any(|&e| e < 0.3));
}
```

### Performance Benchmarks
```rust
#[bench]
fn bench_shannon_entropy(b: &mut Bencher) {
    let mut indicator = ShannonEntropy::new(100, 10);
    let data = vec![42.0; 1000];
    
    b.iter(|| {
        for &value in &data {
            black_box(indicator.next(value));
        }
    });
}
```

---

## Documentation Requirements

### Per Indicator
- [ ] Mathematical formula (LaTeX if needed)
- [ ] Parameter descriptions
- [ ] Interpretation guide
- [ ] Trading strategy examples
- [ ] Performance characteristics
- [ ] References to academic papers

### Example (Shannon Entropy)
```markdown
# Shannon Entropy

## Overview
Shannon Entropy measures the randomness or disorder in price movements.

## Formula
```
H = -∑(pk * log₂(pk))
```
where pk is the probability of price being in bin k.

## Parameters
- **period:** Lookback window (default: 30)
- **bins:** Number of histogram bins (default: 10)

## Interpretation
- **High Entropy (> 0.8):** Market is random/volatile
- **Medium Entropy (0.4-0.8):** Mixed behavior
- **Low Entropy (< 0.4):** Market is trending/predictable

## Trading Strategy
1. **High Entropy → Range Trading:** Use Bollinger Bands, mean reversion
2. **Low Entropy → Trend Following:** Use moving averages, momentum

## Performance
- Time Complexity: O(n)
- Space Complexity: O(n)
- Suitable for streaming: ✓ Yes

## References
- Shannon, C. E. (1948). "A Mathematical Theory of Communication"
```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| **Complex Mathematics** | High | Medium | Use Python reference, peer review |
| **Performance Issues** | Medium | High | Benchmark early, optimize aggressively |
| **Parameter Sensitivity** | High | Medium | Provide defaults, parameter guides |
| **User Confusion** | High | Low | Extensive documentation, tooltips |
| **Limited Adoption** | Medium | Medium | Education, example strategies |

---

## Success Criteria

**Functional:**
- [ ] 5+ scientific indicators implemented
- [ ] All pass unit + integration tests
- [ ] Performance ≤ 1ms per indicator per candle
- [ ] UI fully integrated

**Quality:**
- [ ] Code review approved
- [ ] Documentation complete
- [ ] Example strategies provided
- [ ] Performance benchmarks published

**User Adoption:**
- [ ] Tutorial videos created
- [ ] Community feedback positive
- [ ] Used in ≥3 trading strategies

---

## References

### Academic Papers
- Shannon (1948) - Information Theory
- Pincus (1991) - Approximate Entropy
- Mandelbrot (1982) - Fractal Market Hypothesis
- Lempel & Ziv (1976) - Complexity Measurement

### Code References
- `/temp/advanced-technical-analysis-master/` - Python reference
- `/packages/indicators/src/indicators/` - Loom indicators
- TradingView Pine Script - Indicator patterns

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-02  
**Author:** Claude Assistant  
**Status:** DRAFT - Awaiting Technical Review
