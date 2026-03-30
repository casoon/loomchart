// Color type for visualization
use serde::{Deserialize, Serialize};

/// RGBA Color
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create color from hex string (e.g., "#FF5722" or "#FF5722AA")
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        let (r, g, b, a) = match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
                (r, g, b, 1.0)
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
                let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
                let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
                let a_int = u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?;
                let a = (a_int as f32) / 255.0;
                (r, g, b, a)
            }
            _ => return Err(format!("Invalid hex color: {}", hex)),
        };

        Ok(Self { r, g, b, a })
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha.clamp(0.0, 1.0);
        self
    }

    /// Create a lighter version of this color
    pub fn lighten(&self, amount: f32) -> Self {
        let factor = 1.0 + amount;
        Self {
            r: ((self.r as f32 * factor).min(255.0)) as u8,
            g: ((self.g as f32 * factor).min(255.0)) as u8,
            b: ((self.b as f32 * factor).min(255.0)) as u8,
            a: self.a,
        }
    }

    /// Create a darker version of this color
    pub fn darken(&self, amount: f32) -> Self {
        let factor = 1.0 - amount;
        Self {
            r: ((self.r as f32 * factor).max(0.0)) as u8,
            g: ((self.g as f32 * factor).max(0.0)) as u8,
            b: ((self.b as f32 * factor).max(0.0)) as u8,
            a: self.a,
        }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_rgba_string(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
    }

    /// Alias for to_rgba_string() for backward compatibility
    pub fn to_css(&self) -> String {
        self.to_rgba_string()
    }

    // Predefined colors
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const RED: Color = Color::rgb(239, 83, 80);
    pub const GREEN: Color = Color::rgb(38, 166, 154);
    pub const BLUE: Color = Color::rgb(66, 165, 245);
    pub const YELLOW: Color = Color::rgb(255, 235, 59);
    pub const ORANGE: Color = Color::rgb(255, 152, 0);
    pub const PURPLE: Color = Color::rgb(171, 71, 188);
    pub const CYAN: Color = Color::rgb(0, 188, 212);
    pub const GRAY: Color = Color::rgb(158, 158, 158);

    // Trading specific
    pub const BULLISH: Color = Color::rgb(38, 166, 154);
    pub const BEARISH: Color = Color::rgb(239, 83, 80);
    pub const NEUTRAL: Color = Color::rgb(158, 158, 158);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLUE
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<(u8, u8, u8, f32)> for Color {
    fn from((r, g, b, a): (u8, u8, u8, f32)) -> Self {
        Self::rgba(r, g, b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::rgb(255, 0, 0);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_alpha() {
        let color = Color::rgb(255, 0, 0).with_alpha(0.5);
        assert_eq!(color.a, 0.5);
    }

    #[test]
    fn test_color_hex() {
        let color = Color::rgb(255, 128, 64);
        assert_eq!(color.to_hex(), "#ff8040");
    }

    #[test]
    fn test_color_rgba_string() {
        let color = Color::rgba(255, 128, 64, 0.5);
        assert_eq!(color.to_rgba_string(), "rgba(255,128,64,0.5)");
    }

    #[test]
    fn test_color_lighten() {
        let color = Color::rgb(100, 100, 100);
        let lighter = color.lighten(0.5);
        assert!(lighter.r > 100);
        assert!(lighter.g > 100);
        assert!(lighter.b > 100);
        assert_eq!(lighter.a, 1.0); // Alpha unchanged
    }

    #[test]
    fn test_color_darken() {
        let color = Color::rgb(200, 200, 200);
        let darker = color.darken(0.5);
        assert!(darker.r < 200);
        assert!(darker.g < 200);
        assert!(darker.b < 200);
        assert_eq!(darker.a, 1.0); // Alpha unchanged
    }

    #[test]
    fn test_color_lighten_clamp() {
        let color = Color::rgb(200, 200, 200);
        let lighter = color.lighten(2.0); // Should clamp to 255
        assert_eq!(lighter.r, 255);
        assert_eq!(lighter.g, 255);
        assert_eq!(lighter.b, 255);
    }

    #[test]
    fn test_color_darken_clamp() {
        let color = Color::rgb(50, 50, 50);
        let darker = color.darken(2.0); // Should clamp to 0
        assert_eq!(darker.r, 0);
        assert_eq!(darker.g, 0);
        assert_eq!(darker.b, 0);
    }
}
