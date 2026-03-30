//! Panel system for multi-pane chart layouts
//!
//! Supports:
//! - Multiple resizable panels (main chart, indicators, volume)
//! - Flexible height distribution with stretch factors
//! - Multi-scale overlays (e.g., MFI 0-100 overlaid on price chart)
//! - Automatic scale normalization and mapping

pub mod manager;
pub mod panel;
pub mod scale;

pub use manager::PanelManager;
pub use panel::{OverlayConfig, Panel, PanelConfig, PanelId, PanelType, PriceScale};
pub use scale::{OverlayScale, ScaleMapper, ScaleRange};
