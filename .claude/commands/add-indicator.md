# Add Indicator

Guide for adding a new technical indicator to LoomChart.

## Steps

### 1. Implement the indicator logic (if not already in `packages/indicators/`)

Add a new struct in `packages/indicators/src/` implementing the `Indicator` trait:
- Stateless math function for one-shot calculations
- Stateful struct with `update(&mut self, value: f64) -> f64` for streaming use

### 2. Register in chartcore

In `crates/chartcore/src/indicators/`:
- `builtin/mod.rs` — add the indicator variant and its calculation logic
- `metadata.rs` — add metadata: display name, parameter definitions, output series, group
- `registry.rs` — register the indicator so it appears in the selector

### 3. Rebuild WASM

```bash
cd packages/wasm-core
wasm-pack build --target web --out-dir ../../apps/frontend/src/wasm
```

### 4. Test

- Add a unit test in `packages/indicators/src/` verifying output against known values
- Run `cargo test --workspace` to confirm nothing is broken

## Naming Conventions

- Rust struct: `PascalCase` (e.g. `RelativeStrengthIndex`)
- Registry key: `SCREAMING_SNAKE_CASE` (e.g. `RSI`)
- Display name: Human-readable (e.g. `"Relative Strength Index (RSI)"`)
