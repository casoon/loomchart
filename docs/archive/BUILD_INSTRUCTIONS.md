# WASM Build Anleitung

Da bash commands in dieser Session nicht funktionieren, hier die manuelle Build-Anleitung:

## Option 1: Terminal direkt (EMPFOHLEN)

Öffne ein neues Terminal und führe aus:

```bash
cd /Users/jseidel/GitHub/loom/packages/wasm-core
wasm-pack build --target web --out-dir ../../apps/frontend/public/wasm
```

## Option 2: Via pnpm Script

Falls ein pnpm script existiert:

```bash
cd /Users/jseidel/GitHub/loom
pnpm wasm:build
```

## Prerequisites prüfen

### 1. Rust WASM Target installiert?

```bash
rustup target list --installed | grep wasm32
```

Falls nicht gefunden:

```bash
rustup target add wasm32-unknown-unknown
```

### 2. wasm-pack installiert?

```bash
wasm-pack --version
```

Falls nicht gefunden:

```bash
cargo install wasm-pack
```

## Mögliche Fehler & Lösungen

### Fehler: "chartcore not found"

**Problem:** Verzeichnis heißt `chartcore-indicators` aber Cargo.toml referenziert `chartcore`

**Lösung:** ✅ Bereits gefixt in Cargo.toml (zeigt jetzt auf `crates/chartcore-indicators`)

### Fehler: "chrono::DateTime::from_timestamp not found"

**Problem:** Zu alte chrono Version

**Lösung:** chrono 0.4 sollte die Methode haben. Prüfe Version:

```bash
cd packages/wasm-core
cargo tree | grep chrono
```

### Fehler: Type mismatch in load_test_data

**Lösung:** Stelle sicher dass chartcore alle exportierten Types hat:
- `CandleGenerator`
- `GeneratorConfig`
- `MarketType`
- `Trend`
- `VolatilityRegime`

## Expected Output

Nach erfolgreichem Build solltest du sehen:

```
packages/wasm-core/
  ├─ target/
  └─ ...

apps/frontend/public/wasm/
  ├─ trading_ui_bg.wasm
  ├─ trading_ui.js
  ├─ trading_ui.d.ts
  └─ package.json
```

## Test nach Build

```bash
# Prüfe ob WASM files existieren
ls -la apps/frontend/public/wasm/

# Starte Frontend
cd apps/frontend
pnpm dev
```

## Debugging

Falls der Build fehlschlägt, führe aus:

```bash
cd packages/wasm-core
cargo check --target wasm32-unknown-unknown 2>&1 | tee build-errors.txt
```

Dann schick mir den Inhalt von `build-errors.txt` und ich kann die Fehler fixen!

## Alternative: Ohne wasm-pack

Falls wasm-pack Probleme macht:

```bash
cd packages/wasm-core
cargo build --target wasm32-unknown-unknown --release
```

Das `.wasm` file ist dann unter `target/wasm32-unknown-unknown/release/trading_ui.wasm`

Allerdings fehlen dann die JavaScript bindings - wasm-pack ist daher empfohlen.
