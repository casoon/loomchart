## 🦀 Rust-Based Indicator Plugin System

### Architecture Overview

```
┌───────────────────────────────────────────────────────┐
│  TypeScript/JavaScript (Thin UI Layer)                │
│  - Input controls generation                          │
│  - Plot visualization                                 │
└────────────────────┬──────────────────────────────────┘
                     │ WASM Bridge (minimal)
                     ▼
┌───────────────────────────────────────────────────────┐
│  Rust Core (wasm-core)                                │
│  ┌─────────────────────────────────────────────────┐  │
│  │ Plugin Registry                                 │  │
│  │ - Manages all plugins (built-in + WASM)        │  │
│  └─────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────┐  │
│  │ TA Calculation Library                          │  │
│  │ - Moving Averages (SMA, EMA, WMA, RMA, etc.)   │  │
│  │ - Statistics (StdDev, Correlation, LinReg)     │  │
│  │ - Momentum (RSI, CCI, Williams %R)             │  │
│  └─────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────┐  │
│  │ Built-in Plugins (Rust)                        │  │
│  │ - RSI, EMA, SMA, MACD, Bollinger Bands        │  │
│  └─────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────────────┐  │
│  │ WASM Plugin Loader (Future)                    │  │
│  │ - Load external .wasm indicators               │  │
│  │ - wasmtime/wasmer integration                  │  │
│  └─────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────┘
```

### Key Principles

1. **Rust First**: Core plugin system and calculations in Rust for performance
2. **WASM Target**: External plugins compile to WASM for sandboxing
3. **Testable**: Pure Rust testing, no WASM needed during development
4. **Shared Library**: Built-in and external plugins use same TA functions

### Plugin Development Workflow

#### 1. Create a Plugin in Rust

```rust
// my-indicator/src/lib.rs

use loom_wasm_core::{plugins::*, ta, types::Candle};

#[derive(Default)]
pub struct MyCustomRSI;

impl IndicatorPlugin for MyCustomRSI {
    fn id(&self) -> &str {
        "my-custom-rsi"
    }

    fn name(&self) -> &str {
        "My Custom RSI"
    }

    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }

    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("period", "Period", 14).min(1).max(500),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("rsi", "RSI", "#7E57C2")]
    }

    fn calculate(&self, ctx: &CalculationContext) -> IndicatorResult {
        let period = ctx.input_int("period").unwrap_or(14) as usize;
        let source_type = ctx.input_source("source").unwrap_or(SourceType::Close);

        // Get source data
        let source = ctx.source(source_type);

        // Use shared TA library!
        let rsi = ta::rsi(&source, period);

        IndicatorResult::new("My Custom RSI", "MCRSI", false)
            .add_plot("rsi", rsi)
    }
}
```

#### 2. Test in Pure Rust (No WASM!)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_indicator() {
        let indicator = MyCustomRSI::default();

        // Create test candles
        let candles: Vec<Candle> = (0..30)
            .map(|i| Candle {
                time: i * 60,
                o: 100.0 + (i as f64 * 0.5),
                h: 101.0 + (i as f64 * 0.5),
                l: 99.0 + (i as f64 * 0.5),
                c: 100.0 + (i as f64 * 0.5),
                v: 1000.0,
            })
            .collect();

        // Create context
        let context = CalculationContext::new(&candles)
            .with_input("period", InputValue::Int(14))
            .with_input("source", InputValue::Source(SourceType::Close));

        // Calculate
        let result = indicator.calculate(&context);

        // Assert
        assert!(result.plots.contains_key("rsi"));
        assert_eq!(result.plots["rsi"].len(), 30);

        for (i, &val) in result.plots["rsi"].iter().enumerate() {
            if let Some(rsi_val) = val {
                assert!(
                    rsi_val >= 0.0 && rsi_val <= 100.0,
                    "RSI at {} = {}",
                    i,
                    rsi_val
                );
            }
        }
    }
}
```

Run tests with:
```bash
cargo test
```

**✅ No WASM needed! Just pure Rust!**

#### 3. Build for WASM (Production)

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]

[dependencies]
loom-wasm-core = "0.1"
```

Build to WASM:
```bash
cargo build --target wasm32-unknown-unknown --release

# Optimize
wasm-opt -Oz -o my-indicator.wasm \
  target/wasm32-unknown-unknown/release/my_indicator.wasm
```

#### 4. Load Plugin Dynamically

```typescript
// In your app
import { loadPlugin } from '@loom/chart-core';

// Load from URL
await loadPlugin('https://plugins.example.com/my-indicator.wasm');

// Or from file
const fileInput = document.getElementById('plugin-upload');
const file = fileInput.files[0];
const bytes = await file.arrayBuffer();
await loadPlugin(new Uint8Array(bytes));
```

### Built-in Plugins (Current)

Located in `packages/wasm-core/src/plugins/builtin/`:

1. **RSI** (`rsi.rs`) - Relative Strength Index
2. **EMA** (`ema.rs`) - Exponential Moving Average (with incremental calculation!)
3. **SMA** (`sma.rs`) - Simple Moving Average

All use the shared **TA Library** (`src/ta/`).

### TA Calculation Library

Located in `packages/wasm-core/src/ta/`:

#### Moving Averages (`moving_averages.rs`):
- `sma(source, period)` - Simple Moving Average
- `ema(source, period)` - Exponential Moving Average
- `wma(source, period)` - Weighted Moving Average
- `rma(source, period)` - Wilder's Smoothing (RMA)
- `dema(source, period)` - Double EMA
- `tema(source, period)` - Triple EMA
- `hma(source, period)` - Hull Moving Average
- `vwma(source, volume, period)` - Volume Weighted MA

#### Statistics (`statistics.rs`):
- `stdev(source, mean, period)` - Standard Deviation
- `highest(source, period)` - Highest value
- `lowest(source, period)` - Lowest value
- `sum(source, period)` - Sum over period
- `change(source, length)` - Price change
- `roc(source, period)` - Rate of Change
- `correlation(series1, series2, period)` - Correlation Coefficient
- `linreg(source, period)` - Linear Regression

#### Momentum (`momentum.rs`):
- `rsi(source, period)` - Relative Strength Index
- `stochastic_k(high, low, close, period)` - Stochastic %K
- `cci(high, low, close, period)` - Commodity Channel Index
- `williams_r(high, low, close, period)` - Williams %R

**All functions are tested** with `#[cfg(test)]` blocks!

### Plugin Trait

```rust
pub trait IndicatorPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn category(&self) -> IndicatorCategory;
    fn description(&self) -> &str { "" }
    fn overlay(&self) -> bool { false }
    
    fn inputs(&self) -> Vec<InputConfig>;
    fn plots(&self) -> Vec<PlotConfig>;
    
    fn calculate(&self, context: &CalculationContext) -> IndicatorResult;
    
    // Optional: For real-time optimization
    fn calculate_incremental(
        &self,
        context: &CalculationContext,
        new_candle: &Candle,
        previous_result: &IndicatorResult,
    ) -> Option<IndicatorResult> {
        None
    }
    
    // Optional: Input validation
    fn validate_inputs(&self, inputs: &HashMap<String, InputValue>) 
        -> Result<(), Vec<String>> {
        Ok(())
    }
    
    // Optional: Dependencies
    fn dependencies(&self, inputs: &HashMap<String, InputValue>) -> Vec<String> {
        Vec::new()
    }
}
```

### Plugin Registry

```rust
use loom_wasm_core::plugins::{PluginRegistry, CalculationContext, InputValue};

// Create registry with built-ins
let registry = PluginRegistry::default();

// Or manually register
let registry = PluginRegistry::new();
registry.register_builtins();

// List all plugins
let ids = registry.list_ids(); // ["rsi", "ema", "sma"]

// Get a plugin
let rsi = registry.get("rsi").unwrap();

// Calculate
let context = CalculationContext::new(&candles)
    .with_input("length", InputValue::Int(14));
let result = registry.calculate("rsi", &context).unwrap();
```

### WASM Plugin Loader (Future)

```rust
use loom_wasm_core::plugins::WasmPluginLoader;

let mut loader = WasmPluginLoader::new();

// Load WASM plugin
let wasm_bytes = std::fs::read("my-indicator.wasm").unwrap();
let plugin_id = loader.load_from_bytes(&wasm_bytes).unwrap();

// Plugin is now in registry
let plugin = loader.get(&plugin_id).unwrap();
```

**Status**: API defined, implementation pending (needs wasmtime/wasmer integration).

### Performance

#### Built-in Plugins (Rust)
- **Compiled with optimizations** (`--release`)
- **SIMD auto-vectorization** where applicable
- **Zero-copy** where possible
- **Incremental calculation** for real-time updates

Example: EMA incremental update is **O(1)** instead of O(n)!

#### External Plugins (WASM)
- **Sandboxed** for security
- **Near-native performance** (WASM is fast!)
- **Shareable** via NPM or URLs
- **Same TA library** as built-ins

### Comparison: TypeScript vs Rust Plugin System

| Feature | TypeScript (Old) | Rust (New) |
|---------|------------------|------------|
| **Performance** | Slow (interpreted) | Fast (compiled) |
| **Type Safety** | Runtime checks | Compile-time |
| **Testing** | Needs browser/WASM | Pure Rust tests |
| **Calculation Library** | JS (duplicated) | Rust (shared) |
| **External Plugins** | Dynamic import | WASM modules |
| **Security** | Limited | Sandboxed (WASM) |
| **SIMD** | No | Yes (auto) |
| **Memory Safety** | No | Yes (Rust) |

### Roadmap

#### Phase 1 (Current) ✅
- [x] Plugin trait in Rust
- [x] TA calculation library (16 functions)
- [x] Built-in plugins (RSI, EMA, SMA)
- [x] Plugin registry
- [x] Pure Rust testing

#### Phase 2 (Next)
- [ ] WASM plugin loader (wasmtime/wasmer)
- [ ] External plugin API crate (`loom-plugin-api`)
- [ ] Plugin macro for easy export
- [ ] More built-in plugins (MACD, BB, Stoch)
- [ ] Advanced TA functions (ATR, ADX, etc.)

#### Phase 3 (Future)
- [ ] Plugin marketplace integration
- [ ] Plugin verification/signing
- [ ] Hot-reload support
- [ ] Multi-threaded calculation
- [ ] GPU acceleration (via wgpu)

### Example: Full Plugin Project Structure

```
my-custom-indicator/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Plugin implementation
│   └── tests.rs        # Pure Rust tests
├── examples/
│   └── standalone.rs   # Test without WASM
└── README.md
```

**Cargo.toml**:
```toml
[package]
name = "my-custom-indicator"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]  # WASM + Rust library

[dependencies]
loom-wasm-core = "0.1"

[dev-dependencies]
# For testing
```

**lib.rs**:
```rust
use loom_wasm_core::plugins::*;
use loom_wasm_core::ta;

#[derive(Default)]
pub struct MyIndicator;

impl IndicatorPlugin for MyIndicator {
    // ... implementation
}

// Optional: Export for WASM
#[cfg(target_arch = "wasm32")]
pub fn _export_plugin() -> Box<dyn IndicatorPlugin> {
    Box::new(MyIndicator::default())
}
```

### Benefits

✅ **Fast**: Compiled Rust, SIMD, zero-copy  
✅ **Safe**: Memory-safe Rust, sandboxed WASM  
✅ **Testable**: Pure Rust tests, no browser needed  
✅ **Shareable**: WASM modules via NPM/URLs  
✅ **Reusable**: Shared TA library for all plugins  
✅ **Production-Ready**: Type-safe, tested, optimized  

### Next Steps

1. ✅ Implement TA library (16 functions)
2. ✅ Implement Plugin trait
3. ✅ Create 3 built-in plugins (RSI, EMA, SMA)
4. ✅ Create Plugin Registry
5. ⏳ Integrate wasmtime for WASM loading
6. ⏳ Create `loom-plugin-api` crate
7. ⏳ Write external plugin tutorial
8. ⏳ Build plugin marketplace

---

**Status**: Core system implemented, WASM loading next!
