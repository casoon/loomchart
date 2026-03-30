// Chart analysis plugins
//
// Plugins that draw on charts (not indicators)
// Examples: Pivots, Wyckoff, SMC, Elliott Waves, Support/Resistance
//
// Currently a placeholder - will be migrated from packages/chart

pub trait ChartPlugin {
    fn id(&self) -> &str;
    fn name(&self) -> &str;

    // More methods to be added during migration
}
