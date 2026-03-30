/// Render commands for command-pattern rendering
///
/// This allows us to:
/// 1. Generate rendering instructions without executing them (testable)
/// 2. Batch similar operations for performance
/// 3. Serialize commands for WebWorker rendering (future)
/// 4. Record and replay rendering (debugging)
use crate::primitives::Color;
use serde::{Deserialize, Serialize};

/// A single render command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderCommand {
    /// Clear the entire canvas
    Clear { color: Color },

    /// Draw a line from (x1,y1) to (x2,y2)
    DrawLine {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
        width: f64,
    },

    /// Draw a rectangle
    DrawRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: Option<Color>,
        stroke: Option<Color>,
        stroke_width: f64,
    },

    /// Draw text
    DrawText {
        text: String,
        x: f64,
        y: f64,
        font: String,
        color: Color,
        align: TextAlign,
    },

    /// Draw a single candlestick
    DrawCandle {
        x: f64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        body_width: f64,
        wick_width: f64,
        bullish_color: Color,
        bearish_color: Color,
    },

    /// Draw multiple candlesticks in one batch (optimized)
    DrawCandlesBatch {
        candles: Vec<CandleData>,
        bullish_color: Color,
        bearish_color: Color,
    },

    /// Draw an indicator line (optimized for many points)
    DrawIndicatorLine {
        points: Vec<(f64, f64)>,
        color: Color,
        width: f64,
        style: LineStyle,
    },

    /// Set clipping region
    SetClip {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },

    /// Clear clipping region
    ClearClip,
}

/// Data for a single candle in batch rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleData {
    pub x: f64,
    pub open_y: f64,
    pub high_y: f64,
    pub low_y: f64,
    pub close_y: f64,
    pub width: f64,
}

impl CandleData {
    /// Check if this is a bullish candle
    pub fn is_bullish(&self) -> bool {
        self.close_y <= self.open_y // Note: Y is inverted (0 at top)
    }

    /// Get the top of the candle body
    pub fn body_top(&self) -> f64 {
        self.open_y.min(self.close_y)
    }

    /// Get the height of the candle body
    pub fn body_height(&self) -> f64 {
        (self.open_y - self.close_y).abs().max(1.0) // Min 1px for doji
    }
}

/// Line style for indicators and drawings
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed { dash_length: u32, gap_length: u32 },
    Dotted,
}

/// Text alignment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// A buffer of render commands for a single frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderCommandBuffer {
    pub commands: Vec<RenderCommand>,
    pub frame_id: u64,
    #[cfg(target_arch = "wasm32")]
    pub timestamp: f64,
    #[cfg(not(target_arch = "wasm32"))]
    pub timestamp: u128,
}

impl RenderCommandBuffer {
    /// Create a new command buffer
    pub fn new(frame_id: u64) -> Self {
        #[cfg(target_arch = "wasm32")]
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        #[cfg(not(target_arch = "wasm32"))]
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        Self {
            commands: Vec::with_capacity(1000),
            frame_id,
            timestamp,
        }
    }

    /// Add a command to the buffer
    pub fn push(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }

    /// Clear all commands
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    /// Get the number of commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Optimize the command buffer by batching similar commands
    pub fn optimize(&mut self) {
        // TODO: Implement batching optimization
        // - Combine consecutive DrawLine with same color/width
        // - Merge DrawRect with same fill color
        // - Batch DrawCandle into DrawCandlesBatch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_candle_data_bullish() {
        let candle = CandleData {
            x: 100.0,
            open_y: 150.0,
            high_y: 100.0,
            low_y: 180.0,
            close_y: 120.0, // Close above open (in screen coords, Y inverted)
            width: 8.0,
        };
        assert!(candle.is_bullish());
    }

    #[test]
    fn test_candle_data_bearish() {
        let candle = CandleData {
            x: 100.0,
            open_y: 120.0,
            high_y: 100.0,
            low_y: 180.0,
            close_y: 150.0, // Close below open
            width: 8.0,
        };
        assert!(!candle.is_bullish());
    }

    #[test]
    fn test_render_buffer_creation() {
        let buffer = RenderCommandBuffer::new(1);
        assert_eq!(buffer.frame_id, 1);
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_render_buffer_push() {
        let mut buffer = RenderCommandBuffer::new(1);
        buffer.push(RenderCommand::Clear {
            color: Color::rgb(0, 0, 0),
        });
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }
}
