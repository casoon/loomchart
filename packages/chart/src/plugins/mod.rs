//! Built-in chart plugins.
//!
//! Ready-to-use plugins for common trading visualizations.

mod pivots;
mod wyckoff;
mod smc;
mod elliott;
mod support_resistance;
mod trendlines;

pub use pivots::{PivotPlugin, PivotConfig, PivotType};
pub use wyckoff::{WyckoffPlugin, WyckoffVisualConfig};
pub use smc::{SmcPlugin, SmcVisualConfig};
pub use elliott::{ElliottPlugin, ElliottVisualConfig};
pub use support_resistance::SupportResistancePlugin;
pub use trendlines::TrendlinePlugin;
