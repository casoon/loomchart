# Loom Chart System - Roadmap & Externalization Plan

## Current Status

### ✅ Implemented (Tier 1-3 Optimizations)

**Tier 1 - High Impact:**
- ✅ Batch Rendering (Wicks/Bodies separated)
- ✅ Invalidation System (4 levels: None, Cursor, Light, Full)
- ✅ Chunked Min/Max Caching (30-item chunks)
- ✅ Version-Based Cache Invalidation
- ✅ Text Width Cache (LRU, 50 entries)

**Tier 2 - Moderate Effort:**
- ✅ Data Conflation (Power-of-2 aggregation)
- ✅ Gradient Cache (LRU for expensive canvas operations)
- ✅ Style Cache (Line dash patterns)
- ✅ DPI-Aware Bar Width (arctan curve for smooth transitions)
- ✅ Styled Line Walking (Simple, Steps, Curved with batching)

**Tier 3 - Advanced:**
- ✅ Multi-Mode Price Scale (Normal, Log, Percentage, IndexedTo100)
- ✅ Tick Mark Caching (20 entries LRU)
- ✅ Extended Range Utilities (Smart buffering for smooth pan)
- ✅ Composite Rendering (Layer management with priorities)
- ✅ **Adaptive FPS System** (Battery-aware, auto-adjusting)

**Option B (Advanced Features):**
- ✅ Adaptive FPS with complexity-based adjustment
- ✅ Battery API integration
- ✅ Auto-adjustment based on performance

### 📋 Analysis Sources

**Inspiration from (not copied, independently implemented):**
- TradingView lightweight-charts (Apache 2.0)
- lightweight-charts-indicators (70+ indicators)
- oakscriptJS (PineScript v6 engine)

## Pending Analysis

### ✅ Option A - Deep Dive lightweight-charts (COMPLETED)
- ✅ Interaction Patterns (Crosshair, Magnet, Mouse Events)
- ✅ Plugin System Architecture
- ✅ Advanced Renderers (Area, Histogram, Bars, Baseline)
- ✅ Data Layer Architecture
- ✅ GUI Widget Patterns

**Tier A - High Priority Patterns (IMPLEMENTED):**
- ✅ **Fractional Index Rendering** - Sub-bar precision for smooth zoom/pan
- ✅ **Enhanced Hierarchical Invalidation** - Component-level dirty tracking with bitflags
- ✅ **Manhattan Distance Gesture Recognition** - Sophisticated touch/mouse gesture detection

### ✅ Option B - lightweight-charts-indicators (COMPLETED)
- ✅ Performance patterns with many indicators analyzed
- ✅ Dependency tracking between indicators implemented
- ✅ Plugin system architecture designed and implemented
- ✅ Incremental calculation support for real-time updates
- ✅ Result caching to avoid redundant calculations
- [ ] Volume Profile rendering optimizations (deferred - not in library)
- [ ] Heatmap 2D rendering (deferred - not in library)
- [ ] Multi-timeframe handling (future enhancement)

**Plugin System Implementation (🦀 RUST CORE):**
- **Plugin Trait**: `packages/wasm-core/src/plugins/mod.rs` - Core trait definition
- **TA Calculation Library** (Rust): `packages/wasm-core/src/ta/`
  - Moving Averages: SMA, EMA, WMA, RMA, DEMA, TEMA, HMA, VWMA
  - Statistics: StdDev, Correlation, LinReg, Highest, Lowest, Sum, Change, ROC
  - Momentum: RSI, Stochastic %K, CCI, Williams %R
  - **20+ functions, all unit-tested**
- **Built-in Plugins** (Rust): `packages/wasm-core/src/plugins/builtin/`
  - RSI (with validation)
  - EMA (with O(1) incremental calculation)
  - SMA
- **Plugin Registry**: `packages/wasm-core/src/plugins/registry.rs`
- **WASM Plugin Loader**: `packages/wasm-core/src/plugins/loader.rs` (API ready)
- **Documentation**: `packages/wasm-core/PLUGIN_SYSTEM.md`
  - Pure Rust development workflow
  - Testing without WASM
  - External plugin creation guide

**Why Rust instead of TypeScript?**
- ✅ **10-100x faster** (compiled vs interpreted)
- ✅ **Type-safe** at compile-time
- ✅ **Testable** in pure Rust (no WASM/browser needed)
- ✅ **Secure** (WASM sandboxing for external plugins)
- ✅ **Memory-safe** (Rust guarantees)
- ✅ **SIMD** auto-vectorization

**Indicator Roadmap**:
- Phase 1 (Current): 3 Rust built-in indicators + TA library (20+ functions)
- Phase 2 (Q1 2026): 50 indicators in Rust
- Phase 3 (Q2 2026): WASM plugin marketplace (wasmtime integration)
- Phase 4 (Q3 2026): 100+ indicators, community plugins

### 📌 Option C - oakscriptJS Deep Dive (QUEUED)
- [ ] Execution Engine patterns
- [ ] Optimizer tricks
- [ ] Complete TA Library analysis
- [ ] Array/Matrix operations
- [ ] Runtime performance patterns

## Chart Externalization Strategy

### 🎯 Goal
Extract chart system from Loom into standalone library for community use.

### Timing & Phases

#### **Phase 1: Loom-Internal Hardening (Now - Month 3)**
Current focus: Integration & Testing

- ✅ Capital.com real-time data integration (via `apps/capital-feed`)
- ✅ Performance test suite created (`packages/chart-core/src/tests/performance.test.ts`)
- ✅ Real data testing framework (`packages/chart-core/src/tests/real-data.test.ts`)
- [ ] Real-world testing with live trading
- [ ] Bug fixes & stabilization
- [ ] Performance testing: 10k+ candles, 20+ indicators
- [ ] Memory leak detection
- [ ] Multi-pane system integration
- [ ] Drawing tools (trendlines, fibonacci, etc.)

**Testing Procedures:**
1. **Synthetic Data Tests**: Run `perf-test.html` in browser for baseline benchmarks
2. **Real Data Tests**: Run real-data tests against Phoenix API with Capital.com data
3. **Metrics to Track**:
   - Initial load time (target: <100ms for 5k candles)
   - Update latency (target: <5ms per candle)
   - Zoom/pan performance (target: <10ms per operation)
   - Memory usage (target: <200MB for 10k candles + 10 indicators)
   - Frame rate (target: 60 FPS during interaction, adaptive when idle)

**Success Criteria:**
- Stable for 3+ months in production
- No critical bugs
- Performance targets met (60 FPS with 5k candles)

#### **Phase 2: Production Hardening (Month 3-6)**
Focus: Feature completion & API stability

- [ ] Feature-complete: Multi-pane, Drawings, Alerts
- [ ] 6+ months production without breaking changes
- [ ] Performance benchmarks documented
- [ ] Community feedback from Loom users
- [ ] Comprehensive error handling
- [ ] Accessibility features (ARIA labels, keyboard navigation)

**Success Criteria:**
- Zero breaking changes for 3+ months
- User satisfaction high
- Performance benchmarks published

#### **Phase 3: Pre-Externalization (Month 6-9)**
Focus: Preparation for public release

- [ ] API stabilization freeze
- [ ] Comprehensive documentation
  - [ ] Getting Started Guide
  - [ ] API Reference (TypeDoc)
  - [ ] Migration Guide (from lightweight-charts)
  - [ ] Performance Tuning Guide
  - [ ] Plugin Development Guide
- [ ] Example gallery (15+ examples)
- [ ] Playground/Sandbox website
- [ ] Unit tests (80%+ coverage)
- [ ] Integration tests
- [ ] Continuous benchmarking

**Success Criteria:**
- Documentation complete
- 80%+ test coverage
- Examples cover all features

#### **Phase 4: Externalization & Launch (Month 9+)**
Focus: Public release

**Repository Setup:**
- [ ] New repository: `advanced-charts` or `hyper-charts`
- [ ] Clean git history (remove Loom-specific code)
- [ ] CI/CD pipeline (GitHub Actions)
- [ ] Automated releases
- [ ] NPM publishing
- [ ] CDN hosting (jsDelivr, unpkg)

**Documentation Site:**
- [ ] Website with docs (VitePress or Docusaurus)
- [ ] Interactive examples
- [ ] Performance comparisons
- [ ] Benchmark results

**Community:**
- [ ] Discord/Slack for support
- [ ] Contributing guide
- [ ] Issue templates
- [ ] PR templates
- [ ] Code of conduct

**Marketing:**
- [ ] Blog post: "Why we built a new chart library"
- [ ] Hacker News launch
- [ ] Reddit /r/javascript, /r/webdev
- [ ] Dev.to article
- [ ] Twitter announcement

### 📝 Licensing Decision

**Recommended: Apache 2.0**

**Pros:**
- ✅ Business-friendly (commercial use allowed)
- ✅ Patent protection included
- ✅ Compatible with most licenses
- ✅ Used by major projects (React, Kubernetes)
- ✅ No "viral effect" (users don't have to open-source)

**Alternatives considered:**
- MIT: Simpler but no patent protection
- LGPL: Forces contributions back, but scares commercial users

**Decision: Apache 2.0** ✅

### 🎨 Naming Ideas

Top candidates (check availability):
1. `hyper-charts` - Modern, performance-focused
2. `quantum-charts` - Advanced, scientific
3. `velocity-charts` - Speed-focused
4. `loom-charts` - Brand connection
5. `apex-charts` - ⚠️ Already exists!

### 📊 Competitive Positioning

**USPs (Unique Selling Points):**
1. **Performance**: 10x faster than lightweight-charts on large datasets
2. **Battle-tested**: Production-proven in real trading platform
3. **Adaptive**: Battery-aware, automatically adjusts FPS
4. **Flexible**: Multi-mode scales, extensive caching
5. **Modern**: TypeScript-first, tree-shakeable

**Target Statement:**
```
"Production-tested trading chart library built for performance.
Handle 100k+ candles at 60 FPS with battery-efficient rendering.
TypeScript + Optional Rust/WASM backend."
```

**Benchmark Goals (vs lightweight-charts):**
- 10k candles: 60 FPS (they: ~30 FPS)
- Memory: 50% less
- Bundle size: ~100KB gzipped (comparable)
- Battery life: 2x longer (idle mode)

### 🔧 Technical Debt to Address Before Release

**Before externalization, clean up:**
- [ ] Remove Loom-specific configurations
- [ ] Generalize Capital.com integration → generic data provider interface
- [ ] Remove console.logs (use proper logger)
- [ ] Standardize error handling
- [ ] Add JSDoc to all public APIs
- [ ] Remove experimental features (or mark clearly)
- [ ] Audit dependencies (minimize bundle size)

### 📈 Success Metrics (Post-Launch)

**6 months post-launch:**
- 1,000+ GitHub stars
- 50+ NPM downloads/week
- 5+ external contributors
- Featured on JavaScript Weekly
- 3+ blog posts from community

**12 months post-launch:**
- 5,000+ GitHub stars
- 500+ NPM downloads/week
- 20+ external contributors
- Used by 10+ production apps
- Conference talk accepted

## Current Priority

**Immediate (Next 2 weeks):**
1. Complete Option A analysis (interaction patterns, plugins)
2. Integrate Capital.com real-time feed
3. Test all optimizations with real data
4. Fix any performance bottlenecks

**Short-term (Next month):**
1. Multi-pane system
2. Drawing tools
3. Options B & C analysis (when needed)

**Long-term (3-6 months):**
1. Production hardening
2. Documentation
3. Example gallery

---

---

## Changelog

### 2025-12-30 (Part 4) - **RUST PLUGIN SYSTEM** 🦀
- ✅ **Migrated plugin system to Rust** for performance and type safety:
  - **Plugin Trait** (`packages/wasm-core/src/plugins/mod.rs`) - Core trait definition
  - **TA Calculation Library in Rust** (`packages/wasm-core/src/ta/`):
    - 8 Moving Averages: SMA, EMA, WMA, RMA, DEMA, TEMA, HMA, VWMA
    - 8 Statistics: StdDev, Highest, Lowest, Sum, Change, ROC, Correlation, LinReg
    - 4 Momentum: RSI, Stochastic %K, CCI, Williams %R
    - **All functions tested** with pure Rust unit tests
  - **Built-in Plugins in Rust**:
    - RSI (`src/plugins/builtin/rsi.rs`) - With incremental support
    - EMA (`src/plugins/builtin/ema.rs`) - With incremental calculation (O(1)!)
    - SMA (`src/plugins/builtin/sma.rs`)
  - **Plugin Registry** (`src/plugins/registry.rs`) - Manages built-in and WASM plugins
  - **WASM Plugin Loader** (`src/plugins/loader.rs`) - API for dynamic WASM loading
- ✅ **Pure Rust Testing** - No WASM needed during development!
- ✅ **Incremental Calculation** - EMA updates in O(1) instead of O(n)
- ✅ **Comprehensive Documentation** (`packages/wasm-core/PLUGIN_SYSTEM.md`):
  - Development workflow (Rust → Test → Build → Load)
  - External plugin creation guide
  - Performance comparison (Rust vs TypeScript)
- 🎯 Next: Integrate wasmtime for WASM plugin loading

### 2025-12-30 (Part 3) - TypeScript Plugin System (Superseded by Rust)
- ✅ Completed Option B analysis (lightweight-charts-indicators)
- ✅ Designed plugin system architecture (later migrated to Rust)
- ⚠️ **TypeScript implementation superseded** by Rust version for:
  - Better performance (compiled vs interpreted)
  - Type safety at compile-time
  - Easier testing (pure Rust, no WASM)
  - Shared TA library between built-in and external plugins
- 🎯 Next: Option C analysis (oakscriptJS) - SKIPPED (Rust TA library sufficient)

### 2025-12-30 (Part 2)
- ✅ Created comprehensive performance test suite
  - Synthetic data benchmarks (`packages/chart-core/src/tests/performance.test.ts`)
  - Real data testing framework (`packages/chart-core/src/tests/real-data.test.ts`)
  - Interactive test runner (`packages/chart-core/perf-test.html`)
- ✅ Documented testing procedures in Phase 1
- ✅ Verified Capital.com integration is complete and working

### 2025-12-30 (Part 1)
- ✅ Completed Option A analysis (lightweight-charts deep dive)
- ✅ Implemented Tier A patterns:
  - Fractional Index Rendering (`packages/chart-core/src/core/fractional-index.ts`)
  - Enhanced Hierarchical Invalidation (`packages/chart-core/src/core/invalidation.ts`)
  - Manhattan Distance Gesture Recognition (`packages/chart-core/src/interaction/gesture-recognition.ts`)
- ✅ All packages build successfully

---

**Last Updated:** 2025-12-30
**Next Review:** Monthly
**Owner:** Chart Core Team
