# Loom Trading Platform - Developer Guide

Complete guide for developers working on the Loom codebase.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Development Setup](#development-setup)
3. [Project Structure](#project-structure)
4. [Building & Running](#building--running)
5. [Testing](#testing)
6. [Code Patterns](#code-patterns)
7. [Performance Guidelines](#performance-guidelines)
8. [Contributing](#contributing)

---

## Architecture Overview

### Tech Stack

**Frontend**:
- **Astro**: Static site generation
- **Alpine.js**: Reactive UI framework (lightweight)
- **TypeScript**: Type-safe JavaScript
- **TailwindCSS**: Utility-first styling

**Chart Engine**:
- **Rust**: High-performance core logic
- **WebAssembly**: Compile Rust to WASM
- **Canvas API**: Direct rendering (60fps+)

**Backend**:
- **Elixir/Phoenix**: WebSocket server
- **PostgreSQL**: Time-series data storage
- **Ecto**: Database ORM

**Real-Time**:
- **Phoenix Channels**: WebSocket communication
- **Capital.com API**: Market data feed

### Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│                 Frontend (Astro)                 │
│  ┌────────────┐  ┌─────────────┐  ┌──────────┐ │
│  │  Alpine.js │──│ TypeScript  │──│ Tailwind │ │
│  └────────────┘  └─────────────┘  └──────────┘ │
│         │              │                │        │
│         └──────────────┼────────────────┘        │
│                        │                         │
│                   ┌────▼─────┐                   │
│                   │   WASM   │                   │
│                   │  Bridge  │                   │
│                   └────┬─────┘                   │
└────────────────────────┼─────────────────────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
  ┌─────▼──────┐  ┌──────▼──────┐  ┌─────▼─────┐
  │  ChartCore │  │  Indicators │  │  Drawing  │
  │   (Rust)   │  │   (Rust)    │  │   Tools   │
  └────────────┘  └─────────────┘  └───────────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
  ┌─────▼──────┐  ┌──────▼──────┐  ┌─────▼─────┐
  │  Phoenix   │  │   Ecto      │  │ Capital   │
  │  Channels  │  │ PostgreSQL  │  │ .com API  │
  └────────────┘  └─────────────┘  └───────────┘
```

---

## Development Setup

### Prerequisites

- **Node.js**: v18+ (for frontend)
- **Rust**: 1.75+ (for WASM compilation)
- **pnpm**: v8+ (package manager)
- **wasm-pack**: Latest (WASM tooling)
- **Elixir**: 1.15+ (for backend, optional)
- **PostgreSQL**: 15+ (for backend, optional)

### Installation

```bash
# Clone repository
git clone https://github.com/yourusername/loom.git
cd loom

# Install Node dependencies
pnpm install

# Install Rust toolchain
rustup target add wasm32-unknown-unknown
cargo install wasm-pack

# Build WASM modules
./build-wasm.sh

# Start development server
cd apps/frontend
pnpm dev
```

### IDE Setup

**VS Code** (Recommended):
- Install extensions:
  - `rust-analyzer` - Rust language support
  - `Astro` - Astro framework support
  - `Tailwind CSS IntelliSense`
  - `Error Lens` - Inline error display

**Settings** (`.vscode/settings.json`):
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

---

## Project Structure

### Monorepo Layout

```
loom/
├── apps/
│   ├── frontend/          # Astro web app
│   │   ├── src/
│   │   │   ├── components/   # UI components
│   │   │   ├── layouts/      # Page layouts
│   │   │   ├── lib/          # TypeScript utilities
│   │   │   └── pages/        # Astro pages
│   │   └── tests/            # Frontend tests
│   └── capital-feed/      # Elixir backend
│       └── src/
├── crates/
│   └── chartcore/         # Rust chart engine
│       ├── src/
│       │   ├── core/         # Core types & logic
│       │   ├── rendering/    # Rendering optimizations
│       │   ├── drawings/     # Drawing tools
│       │   └── plugins/      # Indicator plugins
│       └── tests/            # Rust tests
├── packages/
│   ├── chart-core/        # TypeScript chart wrapper
│   ├── indicators/        # Indicator definitions
│   ├── shared/            # Shared types
│   └── wasm-core/         # WASM bindings
├── docs/                  # Documentation
│   ├── api/
│   ├── completed/
│   └── archive/
└── scripts/               # Build scripts
```

### Key Files

**Frontend**:
- `apps/frontend/src/lib/app-rust.ts` - Main app logic
- `apps/frontend/src/lib/rust-chart.ts` - WASM chart wrapper
- `apps/frontend/src/lib/realtime-client.ts` - WebSocket client
- `apps/frontend/src/lib/error-boundary.ts` - Error handling
- `apps/frontend/src/lib/toast.ts` - Notifications
- `apps/frontend/src/lib/loading-state.ts` - Loading management

**Chart Engine**:
- `crates/chartcore/src/core/chart.rs` - Main chart struct
- `crates/chartcore/src/core/generator.rs` - Test data generator
- `crates/chartcore/src/core/types.rs` - Core data types
- `crates/chartcore/src/rendering/optimizations.rs` - Performance
- `crates/chartcore/src/plugins/` - Indicator system

**WASM Bridge**:
- `packages/wasm-core/src/lib.rs` - WASM entry point
- `packages/wasm-core/src/state.rs` - Global state management

---

## Building & Running

### Development Workflow

```bash
# Watch mode (auto-rebuild)
cd apps/frontend
pnpm dev

# In another terminal, watch WASM changes
./quick-rebuild-wasm.sh

# Run tests
pnpm test

# Run backend (optional)
cd apps/capital-feed
mix phx.server
```

### Production Build

```bash
# Build everything
./rebuild-and-start.sh

# Or manually:
# 1. Build WASM
cd packages/wasm-core
wasm-pack build --target web --out-dir ../../apps/frontend/src/wasm

# 2. Build frontend
cd apps/frontend
pnpm build

# 3. Serve
pnpm preview
```

### Environment Variables

Create `.env` files:

**Frontend** (`apps/frontend/.env`):
```env
PUBLIC_API_URL=https://loom-trading.fly.dev
PUBLIC_WS_URL=wss://loom-trading.fly.dev/socket
```

**Backend** (`apps/capital-feed/.env`):
```env
DATABASE_URL=postgresql://user:pass@localhost/loom_dev
CAPITAL_API_KEY=your_api_key
CAPITAL_API_SECRET=your_secret
SECRET_KEY_BASE=random_64_char_string
```

---

## Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test integration_test

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

### Frontend Tests

```bash
cd apps/frontend

# Run Vitest
pnpm test

# Watch mode
pnpm test:watch

# Coverage
pnpm test:coverage
```

### Integration Tests

**Chart Engine** (`crates/chartcore/tests/integration_test.rs`):
- Chart initialization
- Candle generator scenarios
- Streaming candles
- Trend behavior
- Market types
- Volatility regimes

**Frontend** (`apps/frontend/tests/integration.test.ts`):
- Error boundary system
- Toast notifications
- Loading state management
- Full workflow integration

### Test Coverage Goals

- **Rust**: > 80% line coverage
- **TypeScript**: > 70% line coverage
- **Integration**: All critical paths covered

---

## Code Patterns

### Error Handling

**Rust**:
```rust
use anyhow::{Result, Context};

pub fn process_candles(candles: Vec<Candle>) -> Result<Chart> {
    let chart = Chart::new()
        .context("Failed to create chart")?;
    
    for candle in candles {
        chart.push(candle);
    }
    
    Ok(chart)
}
```

**TypeScript**:
```typescript
import { errorBoundary } from './error-boundary';
import { loadingState, withLoading } from './loading-state';

async function fetchData() {
    return withLoading('fetch-data', 'Fetching candles', async () => {
        try {
            const response = await fetch('/api/candles');
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return await response.json();
        } catch (error) {
            errorBoundary.handleError(error as Error, 'Network');
            throw error;
        }
    });
}
```

### State Management

**Alpine.js Reactive State**:
```typescript
export function initTradingApp(): TradingAppState {
    return {
        // State
        loading: false,
        error: null,
        candles: [],
        
        // Methods
        async init() {
            try {
                await this.initChart();
                await this.fetchCandles();
            } catch (error) {
                errorBoundary.handleError(error, 'Initialization');
            }
        },
    };
}
```

### WASM Interop

**Rust → JavaScript**:
```rust
#[wasm_bindgen]
pub fn calculate_sma(prices: &[f64], period: usize) -> Vec<f64> {
    // Implementation
}
```

**JavaScript → Rust**:
```typescript
import init, { calculate_sma } from './wasm/chartcore.js';

await init();
const sma = calculate_sma(prices, 20);
```

### Rendering Optimization

**Viewport Culling**:
```rust
use crate::rendering::optimizations::cull_candles;

let visible = cull_candles(&all_candles, &viewport);
// Only render visible candles (99% reduction)
```

**Level of Detail**:
```rust
use crate::rendering::optimizations::should_render_detail;

match should_render_detail(bar_width) {
    RenderDetail::LineOnly => render_line(candle),
    RenderDetail::SimplifiedBars => render_simple(candle),
    RenderDetail::FullDetail => render_full(candle),
}
```

---

## Performance Guidelines

### Critical Paths

1. **Chart Rendering**: Must maintain 60fps
2. **WebSocket Updates**: < 10ms processing time
3. **WASM Initialization**: < 500ms
4. **Data Fetch**: < 1s for 1000 candles

### Optimization Checklist

**Rust**:
- ✅ Use `opt-level = "z"` for WASM builds
- ✅ Enable LTO (Link-Time Optimization)
- ✅ Strip debug symbols in release
- ✅ Use `#[inline]` for hot paths
- ✅ Avoid allocations in render loops
- ✅ Use `&[T]` slices instead of `Vec<T>` when possible

**TypeScript**:
- ✅ Minimize DOM manipulations
- ✅ Use `requestAnimationFrame` for animations
- ✅ Debounce expensive operations
- ✅ Lazy-load heavy components
- ✅ Use Web Workers for background tasks

**WASM**:
- ✅ Keep WASM bundle < 500KB gzipped
- ✅ Use `wasm-opt -Oz` for size optimization
- ✅ Minimize JS ↔ WASM boundary crossings
- ✅ Pass arrays by reference when possible

### Profiling

**Rust**:
```bash
# Profile with perf
cargo build --release
perf record --call-graph=dwarf ./target/release/app
perf report

# or use flamegraph
cargo install flamegraph
cargo flamegraph
```

**Browser**:
```typescript
// Performance monitoring
console.time('render');
renderChart();
console.timeEnd('render');

// Chrome DevTools → Performance tab
// Record → Analyze
```

---

## Contributing

### Git Workflow

```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes
git add .
git commit -m "feat: Add my feature"

# Push and create PR
git push origin feature/my-feature
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: Add streaming candle support
fix: Resolve WebSocket reconnection bug
docs: Update developer guide
perf: Optimize viewport culling
refactor: Simplify error boundary logic
test: Add integration tests for generator
```

### Code Review Checklist

- [ ] Tests pass locally
- [ ] No compiler warnings
- [ ] Code follows style guidelines
- [ ] Documentation updated
- [ ] Performance impact assessed
- [ ] No breaking changes (or documented)

### Release Process

1. Update version in `Cargo.toml` and `package.json`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v0.2.0`
4. Push tag: `git push origin v0.2.0`
5. CI/CD deploys automatically

---

## Debugging

### Common Issues

**WASM Won't Load**:
```bash
# Check WASM build
cd packages/wasm-core
wasm-pack build --dev --target web

# Check browser console for detailed error
```

**WebSocket Disconnects**:
```typescript
// Enable debug logging
localStorage.setItem('debug', 'loom:*');
// Reload page, check console
```

**Slow Rendering**:
```rust
// Enable FPS counter
chart.config.show_fps = true;

// Check viewport culling is active
println!("Rendering {} / {} candles", visible, total);
```

### Debug Tools

**Rust**:
```bash
# Debug build with symbols
cargo build

# GDB debugging
rust-gdb ./target/debug/app
```

**Browser**:
- Chrome DevTools → Sources → Breakpoints
- Performance Monitor: Ctrl+Shift+P → "Performance Monitor"
- WASM debugging: Enable DWARF support in Chrome

---

## API Reference

See `docs/api/` for detailed API documentation:
- `chart-api.md` - Chart engine API
- `indicators-api.md` - Indicator system
- `websocket-api.md` - Real-time API
- `wasm-api.md` - WASM bridge

---

**Version**: 0.1.0  
**Last Updated**: 2025-12-31  
**Maintainers**: Development Team
