# Run Tests

Run the full test suite for the project.

## Steps

1. Run `cargo test --workspace` from the repo root — this covers all Rust crates and packages
2. Run `cd apps/frontend && pnpm check` for TypeScript / Astro type checking
3. If any test fails, show the full failure output and identify the root cause before attempting a fix

## Notes

- Integration tests live in `tests/` and require no external services
- The WASM crate (`packages/wasm-core`) is excluded from `cargo test` unless you pass `--target wasm32-unknown-unknown` — normal `cargo test` runs the native target
- Do not mock the database in integration tests; use the real schema defined in `apps/phoenix/priv/repo/migrations/`
