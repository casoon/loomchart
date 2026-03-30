# Build WASM

Build the Rust chart engine and WASM bindings, then copy the output to the frontend.

## Steps

1. Run `wasm-pack build --target web --out-dir ../../apps/frontend/src/wasm` in `packages/wasm-core/`
2. Verify the generated files exist: `trading_ui.js`, `trading_ui_bg.wasm`, `trading_ui.d.ts`, `trading_ui_bg.wasm.d.ts`
3. Report any build errors clearly with the relevant Rust compiler output

If the build fails with a linker error, check that `wasm-pack` and the `wasm32-unknown-unknown` target are installed:
```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```
