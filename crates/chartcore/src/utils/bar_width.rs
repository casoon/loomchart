/// Optimal bar width calculation inspired by TradingView Lightweight Charts
///
/// These functions calculate pixel-perfect bar widths that look good at any zoom level
/// and properly account for device pixel ratio.

/// Calculate optimal bar width for OHLC bars
///
/// Uses 30% of bar spacing as a baseline
pub fn optimal_bar_width(bar_spacing: f64, pixel_ratio: f64) -> f64 {
    (bar_spacing * 0.3 * pixel_ratio).floor()
}

/// Calculate optimal candlestick width with special handling for different zoom levels
///
/// This implements TradingView's sophisticated algorithm that:
/// - Uses fixed width (3 * pixelRatio) for bar spacing between 2.5-4 pixels
/// - Gradually reduces coefficient from 1.0 to 0.8 as bars get wider
/// - Ensures bars are never smaller than 1 physical pixel
/// - Keeps bars symmetric with grid lines
pub fn optimal_candlestick_width(bar_spacing: f64, pixel_ratio: f64) -> f64 {
    // Special case: very compressed view (2.5 to 4 pixels per bar)
    const SPECIAL_CASE_FROM: f64 = 2.5;
    const SPECIAL_CASE_TO: f64 = 4.0;
    const SPECIAL_CASE_COEFF: f64 = 3.0;

    if bar_spacing >= SPECIAL_CASE_FROM && bar_spacing <= SPECIAL_CASE_TO {
        return (SPECIAL_CASE_COEFF * pixel_ratio).floor();
    }

    // For wider spacing, use a coefficient that reduces from 1.0 to 0.8
    // This prevents bars from looking too thick when zoomed out
    const REDUCING_COEFF: f64 = 0.2;
    let adjusted_spacing = (bar_spacing - SPECIAL_CASE_TO).max(0.0);
    let atan_factor = adjusted_spacing.atan() / (std::f64::consts::PI * 0.5);
    let coeff = 1.0 - REDUCING_COEFF * atan_factor;

    let calculated = (bar_spacing * coeff * pixel_ratio).floor();
    let scaled_spacing = (bar_spacing * pixel_ratio).floor();
    let optimal = calculated.min(scaled_spacing);

    // Never go below 1 physical pixel
    optimal.max(pixel_ratio.floor())
}

/// Calculate bar line width ensuring symmetry with grid lines
///
/// Ensures that bar width has the same parity (odd/even) as grid line width
/// for pixel-perfect alignment and symmetric crosshair rendering
pub fn symmetric_bar_width(bar_width: f64, pixel_ratio: f64, thin_bars: bool) -> (f64, f64) {
    let mut adjusted_width = bar_width;

    // Grid and crosshair have line width = floor(pixelRatio)
    // Make bar width match parity for symmetric rendering
    if adjusted_width >= 2.0 {
        let line_width = pixel_ratio.floor().max(1.0);
        let bar_width_int = adjusted_width as i32;
        let line_width_int = line_width as i32;

        // If parity doesn't match, adjust bar width
        if (bar_width_int % 2) != (line_width_int % 2) {
            adjusted_width -= 1.0;
        }
    }

    // Calculate actual line width for drawing
    let bar_line_width = if thin_bars {
        adjusted_width.min(pixel_ratio.floor())
    } else {
        adjusted_width
    };

    (adjusted_width, bar_line_width)
}

/// Calculate if open/close ticks should be drawn based on available space
///
/// Open/close ticks need at least 1.5 pixels of spacing to be readable
pub fn should_draw_open_close(bar_spacing: f64, bar_width: f64, pixel_ratio: f64) -> bool {
    let min_spacing = (1.5 * pixel_ratio).floor();
    bar_width <= bar_spacing && bar_spacing >= min_spacing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimal_bar_width() {
        // Standard spacing at 1x pixel ratio
        assert_eq!(optimal_bar_width(10.0, 1.0), 3.0);

        // High DPI (2x)
        assert_eq!(optimal_bar_width(10.0, 2.0), 6.0);

        // Very compressed
        assert_eq!(optimal_bar_width(3.0, 1.0), 0.0);
    }

    #[test]
    fn test_optimal_candlestick_width_special_case() {
        // Special case range: should return 3 * pixelRatio
        assert_eq!(optimal_candlestick_width(3.0, 1.0), 3.0);
        assert_eq!(optimal_candlestick_width(3.0, 2.0), 6.0);
    }

    #[test]
    fn test_optimal_candlestick_width_normal() {
        // Normal spacing: should use coefficient scaling
        let width = optimal_candlestick_width(10.0, 1.0);
        assert!(width >= 1.0); // Never less than 1 pixel
        assert!(width <= 10.0); // Never more than spacing
    }

    #[test]
    fn test_symmetric_bar_width() {
        // Even pixel ratio, even bar width -> should stay even
        let (adjusted, _) = symmetric_bar_width(4.0, 2.0, false);
        assert_eq!(adjusted as i32 % 2, 0);

        // Odd pixel ratio, even bar width -> should become odd
        let (adjusted, _) = symmetric_bar_width(4.0, 1.0, false);
        assert_eq!(adjusted as i32 % 2, 1);
    }

    #[test]
    fn test_thin_bars() {
        // Thin bars should cap at pixel ratio
        let (bar_width, line_width) = symmetric_bar_width(10.0, 2.0, true);
        assert!(line_width <= 2.0);
        assert_eq!(line_width, bar_width.min(2.0));
    }

    #[test]
    fn test_should_draw_open_close() {
        // Enough spacing
        assert!(should_draw_open_close(5.0, 3.0, 1.0));

        // Not enough spacing
        assert!(!should_draw_open_close(1.0, 3.0, 1.0));

        // Bar too wide
        assert!(!should_draw_open_close(5.0, 6.0, 1.0));
    }
}
