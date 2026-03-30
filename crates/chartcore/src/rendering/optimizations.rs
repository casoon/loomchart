//! Rendering optimizations for better performance
//!
//! This module provides various optimization techniques to improve rendering performance:
//! - Viewport culling (only render visible candles)
//! - Level-of-detail rendering (skip details when zoomed out)
//! - Command batching (combine similar draw operations)

use crate::core::{Candle, Viewport};

/// Viewport culling - only return candles that are visible in the current viewport
///
/// This is a critical optimization that prevents rendering thousands of off-screen candles.
/// With 10,000 candles but only 100 visible, this reduces rendering work by 99%.
///
/// # Arguments
/// * `candles` - Full candle dataset
/// * `viewport` - Current viewport (time range)
///
/// # Returns
/// Slice of candles that are visible in the viewport
///
/// # Performance
/// O(log n) binary search to find start index, then O(1) to calculate end index
pub fn cull_candles<'a>(candles: &'a [Candle], viewport: &Viewport) -> &'a [Candle] {
    if candles.is_empty() {
        return &[];
    }

    let start_time = viewport.time_start();
    let end_time = viewport.time_end();

    // Binary search for first visible candle
    let start_idx = candles
        .binary_search_by_key(&start_time, |c| c.time)
        .unwrap_or_else(|idx| idx.saturating_sub(1)); // Include one before for continuity

    // Binary search for last visible candle
    let end_idx = candles
        .binary_search_by_key(&end_time, |c| c.time)
        .unwrap_or_else(|idx| idx.min(candles.len() - 1))
        .saturating_add(1); // Include one after for continuity

    // Ensure valid range
    let start = start_idx.min(candles.len());
    let end = end_idx.min(candles.len()).max(start);

    &candles[start..end]
}

/// Level-of-detail rendering - determine if detailed rendering should be used
///
/// When bars are very narrow (zoomed out), skip rendering wicks and just show
/// simplified bars. This improves performance when viewing large time ranges.
///
/// # Arguments
/// * `bar_width` - Width of a single candle bar in pixels
///
/// # Returns
/// `true` if detailed rendering should be used (show wicks, shadows, etc.)
///
/// # Thresholds
/// - < 2px: Show only close prices as line
/// - 2-4px: Show simplified bars (no wicks)
/// - > 4px: Show full detail (wicks, shadows, etc.)
pub fn should_render_detail(bar_width: f64) -> RenderDetail {
    if bar_width < 2.0 {
        RenderDetail::LineOnly
    } else if bar_width < 4.0 {
        RenderDetail::SimplifiedBars
    } else {
        RenderDetail::FullDetail
    }
}

/// Level of detail for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDetail {
    /// Only render close price as a line (< 2px bar width)
    LineOnly,
    /// Render simplified bars without wicks (2-4px bar width)
    SimplifiedBars,
    /// Render full detail including wicks and shadows (> 4px bar width)
    FullDetail,
}

/// Calculate visible candle range from viewport
///
/// Given a viewport and candle spacing, calculate how many candles can fit
/// on screen. This is used for pre-allocation and optimization.
///
/// # Arguments
/// * `viewport` - Current viewport
/// * `bar_width` - Width of each bar in pixels
/// * `bar_spacing` - Space between bars in pixels
///
/// # Returns
/// Number of visible candles (approximately)
pub fn calculate_visible_count(viewport: &Viewport, bar_width: f64, bar_spacing: f64) -> usize {
    let total_bar_width = bar_width + bar_spacing;
    if total_bar_width <= 0.0 {
        return 0;
    }

    let viewport_width = viewport.width() as f64;
    let visible_count = (viewport_width / total_bar_width).ceil() as usize;

    // Add some padding for smooth scrolling
    visible_count + 20
}

/// Determine if indicator should be rendered based on zoom level
///
/// Some indicators are expensive to calculate and render. When zoomed way out,
/// they may not be visible or useful, so we can skip them.
///
/// # Arguments
/// * `visible_candles` - Number of candles currently visible
/// * `indicator_complexity` - Complexity score (higher = more expensive)
///
/// # Returns
/// `true` if the indicator should be rendered
pub fn should_render_indicator(visible_candles: usize, indicator_complexity: u8) -> bool {
    match indicator_complexity {
        // Simple indicators (SMA, EMA): always render
        0..=2 => true,
        // Medium complexity (RSI, MACD): skip when > 1000 candles visible
        3..=5 => visible_candles <= 1000,
        // High complexity (Ichimoku, custom): skip when > 500 candles visible
        6..=8 => visible_candles <= 500,
        // Very high complexity: skip when > 200 candles visible
        _ => visible_candles <= 200,
    }
}

/// Pre-calculate indicator complexity score
///
/// This helps determine which indicators to skip when performance is critical.
///
/// # Complexity Factors
/// - Number of data series (1-5 lines)
/// - Calculation complexity (simple average vs recursive)
/// - Memory usage (small lookback vs large)
pub fn calculate_indicator_complexity(
    num_series: usize,
    has_recursion: bool,
    lookback_period: usize,
) -> u8 {
    let mut score = 0u8;

    // Series count (0-3 points)
    score += match num_series {
        1 => 0,
        2..=3 => 1,
        4..=5 => 2,
        _ => 3,
    };

    // Recursion adds complexity (0-2 points)
    if has_recursion {
        score += 2;
    }

    // Large lookback period (0-3 points)
    score += match lookback_period {
        0..=50 => 0,
        51..=100 => 1,
        101..=200 => 2,
        _ => 3,
    };

    score.min(10) // Cap at 10
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candles(count: usize) -> Vec<Candle> {
        (0..count)
            .map(|i| Candle {
                time: i as i64 * 60000, // 1 minute intervals
                o: 100.0,
                h: 105.0,
                l: 95.0,
                c: 102.0,
                v: 1000.0,
            })
            .collect()
    }

    #[test]
    fn test_viewport_culling() {
        let candles = create_test_candles(1000);

        // Create viewport showing candles 100-200
        let mut viewport = Viewport::new(800, 600);
        viewport.time.start = 100 * 60000; // start time
        viewport.time.end = 200 * 60000; // end time

        let visible = cull_candles(&candles, &viewport);

        // Should return approximately 100 candles plus padding
        assert!(visible.len() >= 100, "Should have at least 100 candles");
        assert!(visible.len() <= 120, "Should not have way more than needed");
    }

    #[test]
    fn test_level_of_detail() {
        assert_eq!(should_render_detail(1.0), RenderDetail::LineOnly);
        assert_eq!(should_render_detail(3.0), RenderDetail::SimplifiedBars);
        assert_eq!(should_render_detail(5.0), RenderDetail::FullDetail);
    }

    #[test]
    fn test_visible_count_calculation() {
        let viewport = Viewport::new(800, 600);
        let count = calculate_visible_count(&viewport, 8.0, 2.0);

        // 800px / 10px per bar = 80 bars + 20 padding = 100
        assert_eq!(count, 100);
    }

    #[test]
    fn test_indicator_complexity() {
        // Simple SMA: 1 series, no recursion, 20 period
        let simple = calculate_indicator_complexity(1, false, 20);
        assert!(simple <= 2, "Simple indicator should have low complexity");

        // Complex Ichimoku: 5 series, no recursion, 52 period
        let complex = calculate_indicator_complexity(5, false, 52);
        assert!(complex >= 3, "Ichimoku should have higher complexity");

        // Recursive indicator: 2 series, recursive, 14 period
        let recursive = calculate_indicator_complexity(2, true, 14);
        assert!(
            recursive >= 3,
            "Recursive indicators should have higher complexity"
        );
    }

    #[test]
    fn test_should_render_indicator() {
        // Simple indicator - always render
        assert!(should_render_indicator(2000, 1));

        // Medium complexity - skip when > 1000 visible
        assert!(should_render_indicator(500, 4));
        assert!(!should_render_indicator(1500, 4));

        // High complexity - skip when > 500 visible
        assert!(should_render_indicator(300, 7));
        assert!(!should_render_indicator(600, 7));
    }
}
