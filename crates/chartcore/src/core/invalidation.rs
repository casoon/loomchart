/// Invalidation level for different parts of the chart
///
/// This allows us to optimize rendering by only updating what changed.
/// For example, if only the crosshair moved, we don't need to redraw
/// all the candles and indicators.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InvalidationLevel {
    /// Nothing needs to be redrawn
    None = 0,

    /// Only the cursor/crosshair moved (cheapest update)
    Cursor = 1,

    /// Light update - new candle added or single data point changed
    /// Can use incremental rendering techniques
    Light = 2,

    /// Full redraw needed - viewport changed, zoom, pan, or major state change
    Full = 3,
}

impl InvalidationLevel {
    /// Get the maximum (most severe) of two invalidation levels
    pub fn max(self, other: InvalidationLevel) -> InvalidationLevel {
        if self > other {
            self
        } else {
            other
        }
    }
}

/// Tracks which parts of the chart need to be redrawn
///
/// This is the core of our optimized rendering system. By tracking
/// invalidation separately for each component, we can skip expensive
/// operations when they're not needed.
///
/// # Example
///
/// ```
/// use chartcore::core::{InvalidationMask, InvalidationLevel};
///
/// let mut mask = InvalidationMask::new();
///
/// // User moved the mouse - only crosshair needs update
/// mask.invalidate_crosshair();
/// assert!(mask.is_cursor_only());
///
/// // New candle arrived - light update
/// mask.invalidate_candles(InvalidationLevel::Light);
/// assert!(!mask.is_cursor_only());
/// assert!(mask.needs_render());
///
/// // After rendering, reset
/// mask.reset();
/// assert!(!mask.needs_render());
/// ```
#[derive(Debug, Clone)]
pub struct InvalidationMask {
    /// Invalidation level for candlestick data
    candles: InvalidationLevel,

    /// Invalidation level for technical indicators
    indicators: InvalidationLevel,

    /// Invalidation level for drawing tools (trendlines, etc.)
    drawings: InvalidationLevel,

    /// Invalidation level for price/time axes
    axes: InvalidationLevel,

    /// Invalidation level for crosshair cursor
    crosshair: InvalidationLevel,

    /// Invalidation level for grid lines
    grid: InvalidationLevel,

    /// Invalidation level for volume pane (if separate)
    volume: InvalidationLevel,
}

impl InvalidationMask {
    /// Create a new invalidation mask with nothing invalidated
    pub fn new() -> Self {
        Self {
            candles: InvalidationLevel::None,
            indicators: InvalidationLevel::None,
            drawings: InvalidationLevel::None,
            axes: InvalidationLevel::None,
            crosshair: InvalidationLevel::None,
            grid: InvalidationLevel::None,
            volume: InvalidationLevel::None,
        }
    }

    /// Mark all components for invalidation at the given level
    ///
    /// Use this for major changes like viewport resize, zoom, or pan.
    pub fn invalidate_all(&mut self, level: InvalidationLevel) {
        self.candles = level;
        self.indicators = level;
        self.drawings = level;
        self.axes = level;
        self.grid = level;
        self.volume = level;

        // Crosshair gets at least cursor level
        if level != InvalidationLevel::None {
            self.crosshair = level;
        }
    }

    /// Mark candles for invalidation
    ///
    /// Use Light for new candle added, Full for viewport change.
    pub fn invalidate_candles(&mut self, level: InvalidationLevel) {
        self.candles = self.candles.max(level);
    }

    /// Mark indicators for invalidation
    pub fn invalidate_indicators(&mut self, level: InvalidationLevel) {
        self.indicators = self.indicators.max(level);
    }

    /// Mark drawing tools for invalidation
    pub fn invalidate_drawings(&mut self, level: InvalidationLevel) {
        self.drawings = self.drawings.max(level);
    }

    /// Mark axes for invalidation
    pub fn invalidate_axes(&mut self, level: InvalidationLevel) {
        self.axes = self.axes.max(level);
    }

    /// Mark grid for invalidation
    pub fn invalidate_grid(&mut self, level: InvalidationLevel) {
        self.grid = self.grid.max(level);
    }

    /// Mark volume pane for invalidation
    pub fn invalidate_volume(&mut self, level: InvalidationLevel) {
        self.volume = self.volume.max(level);
    }

    /// Mark only the crosshair as needing update
    ///
    /// This is the cheapest invalidation - we can just redraw
    /// the crosshair lines on top of the existing frame.
    pub fn invalidate_crosshair(&mut self) {
        self.crosshair = InvalidationLevel::Cursor;
    }

    /// Check if any component needs rendering
    pub fn needs_render(&self) -> bool {
        self.candles != InvalidationLevel::None
            || self.indicators != InvalidationLevel::None
            || self.drawings != InvalidationLevel::None
            || self.axes != InvalidationLevel::None
            || self.crosshair != InvalidationLevel::None
            || self.grid != InvalidationLevel::None
            || self.volume != InvalidationLevel::None
    }

    /// Check if a full re-render is needed
    ///
    /// Returns true if any component is marked as Full invalidation.
    pub fn needs_full_render(&self) -> bool {
        self.candles == InvalidationLevel::Full
            || self.indicators == InvalidationLevel::Full
            || self.drawings == InvalidationLevel::Full
            || self.axes == InvalidationLevel::Full
            || self.grid == InvalidationLevel::Full
            || self.volume == InvalidationLevel::Full
    }

    /// Check if only the cursor needs updating
    ///
    /// When true, we can use a fast path that only redraws the crosshair,
    /// potentially saving 90%+ of rendering time.
    pub fn is_cursor_only(&self) -> bool {
        self.crosshair != InvalidationLevel::None
            && self.candles == InvalidationLevel::None
            && self.indicators == InvalidationLevel::None
            && self.drawings == InvalidationLevel::None
            && self.axes == InvalidationLevel::None
            && self.grid == InvalidationLevel::None
            && self.volume == InvalidationLevel::None
    }

    /// Check if this is a light update (incremental)
    ///
    /// Light updates can often use incremental techniques like
    /// only rendering the new candle instead of all candles.
    pub fn is_light_update(&self) -> bool {
        let max_level = self.get_max_level();
        max_level == InvalidationLevel::Light
    }

    /// Get the maximum invalidation level across all components
    pub fn get_max_level(&self) -> InvalidationLevel {
        [
            self.candles,
            self.indicators,
            self.drawings,
            self.axes,
            self.crosshair,
            self.grid,
            self.volume,
        ]
        .iter()
        .copied()
        .max()
        .unwrap_or(InvalidationLevel::None)
    }

    /// Reset all invalidation flags to None
    ///
    /// Call this after rendering to mark everything as clean.
    pub fn reset(&mut self) {
        self.candles = InvalidationLevel::None;
        self.indicators = InvalidationLevel::None;
        self.drawings = InvalidationLevel::None;
        self.axes = InvalidationLevel::None;
        self.crosshair = InvalidationLevel::None;
        self.grid = InvalidationLevel::None;
        self.volume = InvalidationLevel::None;
    }

    /// Get individual component levels (for debugging/monitoring)
    pub fn get_levels(&self) -> InvalidationLevels {
        InvalidationLevels {
            candles: self.candles,
            indicators: self.indicators,
            drawings: self.drawings,
            axes: self.axes,
            crosshair: self.crosshair,
            grid: self.grid,
            volume: self.volume,
        }
    }
}

impl Default for InvalidationMask {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of invalidation levels for all components
#[derive(Debug, Clone, Copy)]
pub struct InvalidationLevels {
    pub candles: InvalidationLevel,
    pub indicators: InvalidationLevel,
    pub drawings: InvalidationLevel,
    pub axes: InvalidationLevel,
    pub crosshair: InvalidationLevel,
    pub grid: InvalidationLevel,
    pub volume: InvalidationLevel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalidation_level_ordering() {
        assert!(InvalidationLevel::None < InvalidationLevel::Cursor);
        assert!(InvalidationLevel::Cursor < InvalidationLevel::Light);
        assert!(InvalidationLevel::Light < InvalidationLevel::Full);
    }

    #[test]
    fn test_invalidation_level_max() {
        assert_eq!(
            InvalidationLevel::Light.max(InvalidationLevel::Cursor),
            InvalidationLevel::Light
        );
        assert_eq!(
            InvalidationLevel::Cursor.max(InvalidationLevel::Full),
            InvalidationLevel::Full
        );
    }

    #[test]
    fn test_new_mask_clean() {
        let mask = InvalidationMask::new();
        assert!(!mask.needs_render());
        assert!(!mask.needs_full_render());
        assert!(!mask.is_cursor_only());
    }

    #[test]
    fn test_invalidate_all() {
        let mut mask = InvalidationMask::new();
        mask.invalidate_all(InvalidationLevel::Full);

        assert!(mask.needs_render());
        assert!(mask.needs_full_render());
        assert!(!mask.is_cursor_only());
    }

    #[test]
    fn test_cursor_only() {
        let mut mask = InvalidationMask::new();
        mask.invalidate_crosshair();

        assert!(mask.needs_render());
        assert!(!mask.needs_full_render());
        assert!(mask.is_cursor_only());
    }

    #[test]
    fn test_light_update() {
        let mut mask = InvalidationMask::new();
        mask.invalidate_candles(InvalidationLevel::Light);

        assert!(mask.needs_render());
        assert!(!mask.needs_full_render());
        assert!(!mask.is_cursor_only());
        assert!(mask.is_light_update());
    }

    #[test]
    fn test_mixed_levels() {
        let mut mask = InvalidationMask::new();
        mask.invalidate_candles(InvalidationLevel::Light);
        mask.invalidate_crosshair();

        assert!(mask.needs_render());
        assert!(!mask.needs_full_render());
        assert!(!mask.is_cursor_only()); // Not cursor-only because candles are also invalidated
        assert!(mask.is_light_update());
    }

    #[test]
    fn test_progressive_invalidation() {
        let mut mask = InvalidationMask::new();

        // Start with cursor
        mask.invalidate_crosshair();
        assert_eq!(mask.get_max_level(), InvalidationLevel::Cursor);

        // Add light invalidation
        mask.invalidate_candles(InvalidationLevel::Light);
        assert_eq!(mask.get_max_level(), InvalidationLevel::Light);

        // Escalate to full
        mask.invalidate_indicators(InvalidationLevel::Full);
        assert_eq!(mask.get_max_level(), InvalidationLevel::Full);
        assert!(mask.needs_full_render());
    }

    #[test]
    fn test_reset() {
        let mut mask = InvalidationMask::new();
        mask.invalidate_all(InvalidationLevel::Full);

        assert!(mask.needs_render());

        mask.reset();

        assert!(!mask.needs_render());
        assert!(!mask.needs_full_render());
    }

    #[test]
    fn test_max_level_escalation() {
        let mut mask = InvalidationMask::new();

        // Adding lower level shouldn't downgrade
        mask.invalidate_candles(InvalidationLevel::Full);
        mask.invalidate_candles(InvalidationLevel::Light);

        assert_eq!(mask.candles, InvalidationLevel::Full);
    }
}
