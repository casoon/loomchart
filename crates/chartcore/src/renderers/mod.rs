// Renderers - output adapters for different platforms
//
// Supports command-pattern rendering for testability and future WebWorker support

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod canvas2d;
pub mod commands;

// Re-exports removed - this module is internal only

/// Renderer trait for abstracting output
pub trait Renderer {
    /// Initialize renderer
    fn init(&mut self);

    /// Render frame
    fn render(&mut self);

    /// Clear canvas
    fn clear(&mut self);
}

/// No-op renderer for headless operation (testing, server-side)
pub struct NoopRenderer;

impl Renderer for NoopRenderer {
    fn init(&mut self) {}
    fn render(&mut self) {}
    fn clear(&mut self) {}
}
