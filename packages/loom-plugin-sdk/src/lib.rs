//! LoomChart Plugin SDK
//!
//! Import this crate in your plugin and use [`export_indicator!`] to generate
//! the required WASM exports automatically.
//!
//! # Minimal example
//!
//! ```rust,ignore
//! use loom_plugin_sdk::prelude::*;
//!
//! pub struct MyIndicator;
//!
//! impl IndicatorPlugin for MyIndicator {
//!     fn id(&self) -> &str { "my_indicator" }
//!     fn name(&self) -> &str { "My Indicator" }
//!     fn inputs(&self) -> Vec<InputConfig> { vec![] }
//!     fn plots(&self) -> Vec<PlotConfig> {
//!         vec![PlotConfig {
//!             id: "value".into(),
//!             title: "Value".into(),
//!             color: Color::rgb(100, 200, 100),
//!             line_width: 2,
//!             style: PlotStyle::Line,
//!         }]
//!     }
//!     fn calculate(&self, ctx: &CalculationContext) -> IndicatorResult {
//!         let values: Vec<Option<f64>> = ctx.candles.iter()
//!             .map(|c| Some(c.c))
//!             .collect();
//!         IndicatorResult::new("My Indicator", false).add_plot("value", values)
//!     }
//! }
//!
//! export_indicator!(
//!     plugin: MyIndicator,
//!     api_version: "1",
//!     id: "my_indicator",
//!     display_name: "My Indicator",
//! );
//! ```

pub use loom_plugin_api::*;

pub mod prelude {
    pub use loom_plugin_api::manifest::{Capability, PluginManifest, PluginType};
    pub use loom_plugin_api::traits::{
        CalculationContext, DataSourcePlugin, IndicatorPlugin, IndicatorResult, RendererPlugin,
        StrategyPlugin, StrategyContext, Viewport,
    };
    pub use loom_plugin_api::types::{
        Candle, Color, InputConfig, InputType, InputValue, PlotConfig, PlotStyle, RenderCommand,
        Signal, SignalKind, SourceType,
    };
}

// ---------------------------------------------------------------------------
// WASM ABI helpers (only compiled for wasm32 targets)
// ---------------------------------------------------------------------------

/// Allocate a byte buffer in WASM memory. Required export for the host ABI.
///
/// # Safety
/// This is an ABI function called by the host — do not call from Rust code.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe extern "C" fn loom_alloc(size: usize) -> *mut u8 {
    let layout = std::alloc::Layout::from_size_align(size, 1).expect("invalid layout");
    std::alloc::alloc(layout)
}

/// Free a buffer allocated with `loom_alloc`.
///
/// # Safety
/// Caller must pass a valid (ptr, size) pair from `loom_alloc`.
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe extern "C" fn loom_dealloc(ptr: *mut u8, size: usize) {
    if size > 0 {
        let layout = std::alloc::Layout::from_size_align(size, 1).expect("invalid layout");
        std::alloc::dealloc(ptr, layout);
    }
}

// ---------------------------------------------------------------------------
// Export macro
// ---------------------------------------------------------------------------

/// Generate all required WASM ABI exports for an `IndicatorPlugin` impl.
///
/// # Parameters
///
/// - `plugin`: the type (must implement `IndicatorPlugin + Default`)
/// - `api_version`: always `"1"` for now
/// - `id`: machine-readable identifier (lowercase, no spaces)
/// - `display_name`: human-readable name
/// - `description`: optional, defaults to `""`
/// - `version`: optional semver, defaults to `"0.1.0"`
/// - `capabilities`: optional `[]`
#[macro_export]
macro_rules! export_indicator {
    (
        plugin: $plugin:ty,
        api_version: $api:expr,
        id: $id:expr,
        display_name: $display:expr
        $(, description: $desc:expr)?
        $(, version: $ver:expr)?
        $(, capabilities: [$($cap:expr),*])?
        $(,)?
    ) => {
        use $crate::prelude::*;

        // ---------------------------------------------------------------
        // Thread-local plugin instance (WASM is single-threaded)
        // ---------------------------------------------------------------
        #[cfg(target_arch = "wasm32")]
        thread_local! {
            static __PLUGIN: std::cell::RefCell<$plugin> =
                std::cell::RefCell::new(<$plugin as Default>::default());
        }

        // ---------------------------------------------------------------
        // Static JSON payloads (computed once at startup)
        // ---------------------------------------------------------------
        #[cfg(target_arch = "wasm32")]
        static __MANIFEST_JSON: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        #[cfg(target_arch = "wasm32")]
        static __INPUTS_JSON: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        #[cfg(target_arch = "wasm32")]
        static __PLOTS_JSON: std::sync::OnceLock<String> = std::sync::OnceLock::new();

        #[cfg(target_arch = "wasm32")]
        fn __manifest_str() -> &'static str {
            __MANIFEST_JSON.get_or_init(|| {
                let m = $crate::manifest::PluginManifest {
                    api_version: $api.to_string(),
                    plugin_type: $crate::manifest::PluginType::Indicator,
                    id: $id.to_string(),
                    display_name: $display.to_string(),
                    description: { #[allow(unused)] let d = ""; $(let d = $desc;)? d.to_string() },
                    version: { #[allow(unused)] let v = "0.1.0"; $(let v = $ver;)? v.to_string() },
                    capabilities: vec![$($($cap),*)?],
                    entry: "plugin_entry".to_string(),
                };
                serde_json::to_string(&m).expect("manifest serialization failed")
            })
        }

        #[cfg(target_arch = "wasm32")]
        fn __inputs_str() -> &'static str {
            __INPUTS_JSON.get_or_init(|| {
                __PLUGIN.with(|p| {
                    serde_json::to_string(&p.borrow().inputs())
                        .expect("inputs serialization failed")
                })
            })
        }

        #[cfg(target_arch = "wasm32")]
        fn __plots_str() -> &'static str {
            __PLOTS_JSON.get_or_init(|| {
                __PLUGIN.with(|p| {
                    serde_json::to_string(&p.borrow().plots())
                        .expect("plots serialization failed")
                })
            })
        }

        // ---------------------------------------------------------------
        // Required exports
        // ---------------------------------------------------------------

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn loom_manifest_ptr() -> *const u8 {
            __manifest_str().as_ptr()
        }

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn loom_manifest_len() -> usize {
            __manifest_str().len()
        }

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn loom_inputs_ptr() -> *const u8 {
            __inputs_str().as_ptr()
        }

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn loom_inputs_len() -> usize {
            __inputs_str().len()
        }

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn loom_plots_ptr() -> *const u8 {
            __plots_str().as_ptr()
        }

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn loom_plots_len() -> usize {
            __plots_str().len()
        }

        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn loom_reset() {
            __PLUGIN.with(|p| p.borrow_mut().reset());
        }

        /// # Safety
        /// ctx_ptr/ctx_len must point to valid UTF-8 JSON in WASM linear memory.
        /// Returns packed i64: (result_ptr << 32) | result_len.
        /// Caller is responsible for calling `loom_dealloc` on the result buffer.
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub unsafe extern "C" fn loom_calculate(ctx_ptr: *const u8, ctx_len: usize) -> i64 {
            #[derive(serde::Deserialize)]
            struct CalcRequest {
                candles: Vec<$crate::types::Candle>,
                inputs: std::collections::HashMap<String, $crate::types::InputValue>,
            }

            #[derive(serde::Serialize)]
            struct CalcResponse {
                plots: std::collections::HashMap<String, Vec<Option<f64>>>,
                overlay: bool,
                title: String,
            }

            let ctx_bytes = std::slice::from_raw_parts(ctx_ptr, ctx_len);
            let ctx_str = match std::str::from_utf8(ctx_bytes) {
                Ok(s) => s,
                Err(_) => return pack_error("invalid UTF-8 context"),
            };
            let req: CalcRequest = match serde_json::from_str(ctx_str) {
                Ok(r) => r,
                Err(e) => return pack_error(&e.to_string()),
            };

            // Build internal CalculationContext from API types.
            let ctx = {
                use $crate::traits::CalculationContext;
                let mut c = CalculationContext::new(&req.candles);
                for (k, v) in &req.inputs {
                    c = c.with_input(k, v.clone());
                }
                c
            };

            let result = __PLUGIN.with(|p| p.borrow().calculate(&ctx));

            let resp = CalcResponse {
                plots: result.plots,
                overlay: result.overlay,
                title: result.title,
            };
            let json = match serde_json::to_string(&resp) {
                Ok(j) => j,
                Err(e) => return pack_error(&e.to_string()),
            };

            let bytes = json.into_bytes();
            let len = bytes.len();
            let ptr = loom_alloc(len);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
            ((ptr as i64) << 32) | (len as i64)
        }

        #[cfg(target_arch = "wasm32")]
        unsafe fn pack_error(msg: &str) -> i64 {
            // Return error as JSON so the host can surface it.
            let json = format!("{{\"error\":\"{msg}\"}}");
            let bytes = json.into_bytes();
            let len = bytes.len();
            let ptr = loom_alloc(len);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
            ((ptr as i64) << 32) | (len as i64)
        }

        /// Entry point referenced in the plugin manifest.
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn plugin_entry() {}
    };
}
