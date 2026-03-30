// WASM Loader - Exposes WASM module to window
let wasmModule = null;
let wasmModuleLoaded = false;
let wasmCoreInitialized = false;

async function initWasm() {
  console.log(
    "[WASM Loader] initWasm called, loaded:",
    wasmModuleLoaded,
    "initialized:",
    wasmCoreInitialized,
  );

  // If already loaded, return the cached module
  if (wasmModuleLoaded && wasmModule) {
    console.log("[WASM Loader] Returning cached module");
    return wasmModule;
  }

  // Import the WASM module (only happens once)
  console.log("[WASM Loader] Loading WASM module...");
  const wasm = await import("/wasm/trading_ui.js");
  await wasm.default();

  wasmModule = wasm;
  wasmModuleLoaded = true;
  console.log("[WASM Loader] WASM module loaded successfully");

  return wasm;
}

// Initialize the WASM core (separate from module loading)
async function initWasmCore(config) {
  console.log(
    "[WASM Loader] initWasmCore called, core initialized:",
    wasmCoreInitialized,
  );

  // Ensure module is loaded first
  const wasm = await initWasm();

  // Only initialize once
  if (wasmCoreInitialized) {
    console.log("[WASM Loader] Core already initialized, skipping");
    return true;
  }

  try {
    console.log("[WASM Loader] Initializing WASM core with config:", config);
    wasm.init(JSON.stringify(config));
    wasmCoreInitialized = true;
    console.log("[WASM Loader] WASM core initialized successfully");
    return true;
  } catch (e) {
    console.error("[WASM Loader] Failed to initialize WASM core:", e);
    // Check if already initialized by trying a test call
    try {
      const status = wasm.get_connection_status();
      console.log(
        "[WASM Loader] Core was already initialized, status:",
        status,
      );
      wasmCoreInitialized = true;
      return true;
    } catch (e2) {
      console.error("[WASM Loader] Core initialization failed:", e2);
      return false;
    }
  }
}

// Check if core is initialized
function isWasmCoreInitialized() {
  return wasmCoreInitialized;
}

// Expose to window
window.loadWasm = initWasm;
window.initWasmCore = initWasmCore;
window.isWasmCoreInitialized = isWasmCoreInitialized;
window.getWasm = () => wasmModule;
