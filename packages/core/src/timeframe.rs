//! Timeframe definitions and bucketing utilities.
//!
//! Supports standard trading timeframes with proper bucket alignment.

use core::fmt;
use core::str::FromStr;

#[cfg(not(feature = "std"))]
use alloc::string::String;

/// Trading timeframe
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Timeframe {
    /// 1 second
    S1,
    /// 5 seconds
    S5,
    /// 15 seconds
    S15,
    /// 30 seconds
    S30,
    /// 1 minute
    M1,
    /// 3 minutes
    M3,
    /// 5 minutes
    M5,
    /// 15 minutes
    M15,
    /// 30 minutes
    M30,
    /// 1 hour
    H1,
    /// 2 hours
    H2,
    /// 4 hours
    H4,
    /// 6 hours
    H6,
    /// 8 hours
    H8,
    /// 12 hours
    H12,
    /// 1 day
    D1,
    /// 3 days
    D3,
    /// 1 week
    W1,
    /// 1 month (approximated as 30 days)
    MN1,
}

impl Timeframe {
    /// All standard timeframes in ascending order
    pub const ALL: &'static [Timeframe] = &[
        Self::S1, Self::S5, Self::S15, Self::S30,
        Self::M1, Self::M3, Self::M5, Self::M15, Self::M30,
        Self::H1, Self::H2, Self::H4, Self::H6, Self::H8, Self::H12,
        Self::D1, Self::D3, Self::W1, Self::MN1,
    ];

    /// Duration in seconds
    #[inline]
    pub const fn as_seconds(&self) -> i64 {
        match self {
            Self::S1 => 1,
            Self::S5 => 5,
            Self::S15 => 15,
            Self::S30 => 30,
            Self::M1 => 60,
            Self::M3 => 180,
            Self::M5 => 300,
            Self::M15 => 900,
            Self::M30 => 1800,
            Self::H1 => 3600,
            Self::H2 => 7200,
            Self::H4 => 14400,
            Self::H6 => 21600,
            Self::H8 => 28800,
            Self::H12 => 43200,
            Self::D1 => 86400,
            Self::D3 => 259200,
            Self::W1 => 604800,
            Self::MN1 => 2592000, // 30 days approximation
        }
    }

    /// Duration in milliseconds
    #[inline]
    pub const fn as_millis(&self) -> i64 {
        self.as_seconds() * 1000
    }

    /// Round timestamp (in seconds) down to bucket start
    ///
    /// # Example
    /// ```
    /// use loom_core::Timeframe;
    ///
    /// let tf = Timeframe::H1;
    /// let ts = 1700001234; // Some timestamp
    /// let bucket = tf.bucket_start(ts);
    /// assert_eq!(bucket % 3600, 0); // Aligned to hour
    /// ```
    #[inline]
    pub const fn bucket_start(&self, timestamp_secs: i64) -> i64 {
        let period = self.as_seconds();
        (timestamp_secs / period) * period
    }

    /// Round timestamp (in milliseconds) down to bucket start
    #[inline]
    pub const fn bucket_start_ms(&self, timestamp_ms: i64) -> i64 {
        let period = self.as_millis();
        (timestamp_ms / period) * period
    }

    /// Get the bucket end time (exclusive) for a given timestamp
    #[inline]
    pub const fn bucket_end(&self, timestamp_secs: i64) -> i64 {
        self.bucket_start(timestamp_secs) + self.as_seconds()
    }

    /// Get the bucket end time in milliseconds
    #[inline]
    pub const fn bucket_end_ms(&self, timestamp_ms: i64) -> i64 {
        self.bucket_start_ms(timestamp_ms) + self.as_millis()
    }

    /// Check if timestamp is at bucket boundary
    #[inline]
    pub const fn is_bucket_start(&self, timestamp_secs: i64) -> bool {
        timestamp_secs % self.as_seconds() == 0
    }

    /// Get number of bars between two timestamps
    #[inline]
    pub const fn bars_between(&self, start_secs: i64, end_secs: i64) -> i64 {
        (end_secs - start_secs) / self.as_seconds()
    }

    /// Check if this timeframe is intraday
    #[inline]
    pub const fn is_intraday(&self) -> bool {
        self.as_seconds() < 86400
    }

    /// Check if this timeframe is sub-minute
    #[inline]
    pub const fn is_sub_minute(&self) -> bool {
        self.as_seconds() < 60
    }

    /// Get the canonical string representation
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::S1 => "1s",
            Self::S5 => "5s",
            Self::S15 => "15s",
            Self::S30 => "30s",
            Self::M1 => "1m",
            Self::M3 => "3m",
            Self::M5 => "5m",
            Self::M15 => "15m",
            Self::M30 => "30m",
            Self::H1 => "1h",
            Self::H2 => "2h",
            Self::H4 => "4h",
            Self::H6 => "6h",
            Self::H8 => "8h",
            Self::H12 => "12h",
            Self::D1 => "1d",
            Self::D3 => "3d",
            Self::W1 => "1w",
            Self::MN1 => "1M",
        }
    }

    /// Parse from various string formats
    ///
    /// Supports: "1m", "1min", "1M" (minute), "1h", "1H", "1d", "1D", "1w", "1W", "1M" (month)
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.trim();

        // Handle common aliases
        match s.to_lowercase().as_str() {
            "1s" | "s1" => return Some(Self::S1),
            "5s" | "s5" => return Some(Self::S5),
            "15s" | "s15" => return Some(Self::S15),
            "30s" | "s30" => return Some(Self::S30),
            "1m" | "m1" | "1min" => return Some(Self::M1),
            "3m" | "m3" | "3min" => return Some(Self::M3),
            "5m" | "m5" | "5min" => return Some(Self::M5),
            "15m" | "m15" | "15min" => return Some(Self::M15),
            "30m" | "m30" | "30min" => return Some(Self::M30),
            "1h" | "h1" | "60m" | "60min" => return Some(Self::H1),
            "2h" | "h2" | "120m" => return Some(Self::H2),
            "4h" | "h4" | "240m" => return Some(Self::H4),
            "6h" | "h6" => return Some(Self::H6),
            "8h" | "h8" => return Some(Self::H8),
            "12h" | "h12" => return Some(Self::H12),
            "1d" | "d1" | "d" | "daily" => return Some(Self::D1),
            "3d" | "d3" => return Some(Self::D3),
            "1w" | "w1" | "w" | "weekly" => return Some(Self::W1),
            _ => {}
        }

        // Handle "1M" for month (uppercase M)
        if s == "1M" || s == "M1" || s == "monthly" {
            return Some(Self::MN1);
        }

        None
    }

    /// Get the next larger timeframe
    pub const fn next_larger(&self) -> Option<Timeframe> {
        match self {
            Self::S1 => Some(Self::S5),
            Self::S5 => Some(Self::S15),
            Self::S15 => Some(Self::S30),
            Self::S30 => Some(Self::M1),
            Self::M1 => Some(Self::M5),
            Self::M3 => Some(Self::M5),
            Self::M5 => Some(Self::M15),
            Self::M15 => Some(Self::M30),
            Self::M30 => Some(Self::H1),
            Self::H1 => Some(Self::H4),
            Self::H2 => Some(Self::H4),
            Self::H4 => Some(Self::D1),
            Self::H6 => Some(Self::H12),
            Self::H8 => Some(Self::D1),
            Self::H12 => Some(Self::D1),
            Self::D1 => Some(Self::W1),
            Self::D3 => Some(Self::W1),
            Self::W1 => Some(Self::MN1),
            Self::MN1 => None,
        }
    }

    /// Check if this timeframe evenly divides into another
    pub const fn divides_into(&self, other: &Timeframe) -> bool {
        other.as_seconds() % self.as_seconds() == 0
    }
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error when parsing timeframe
#[derive(Debug, Clone)]
pub struct ParseTimeframeError {
    input: String,
}

impl fmt::Display for ParseTimeframeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid timeframe: '{}'", self.input)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseTimeframeError {}

impl FromStr for Timeframe {
    type Err = ParseTimeframeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Timeframe::from_str(s).ok_or_else(|| ParseTimeframeError {
            input: String::from(s),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_start() {
        let tf = Timeframe::H1;
        assert_eq!(tf.bucket_start(3661), 3600); // 1:01:01 -> 1:00:00
        assert_eq!(tf.bucket_start(7199), 3600); // 1:59:59 -> 1:00:00
        assert_eq!(tf.bucket_start(7200), 7200); // 2:00:00 -> 2:00:00
    }

    #[test]
    fn test_parse_timeframe() {
        assert_eq!(Timeframe::from_str("1m"), Some(Timeframe::M1));
        assert_eq!(Timeframe::from_str("1h"), Some(Timeframe::H1));
        assert_eq!(Timeframe::from_str("4h"), Some(Timeframe::H4));
        assert_eq!(Timeframe::from_str("1d"), Some(Timeframe::D1));
        assert_eq!(Timeframe::from_str("1w"), Some(Timeframe::W1));
        assert_eq!(Timeframe::from_str("1M"), Some(Timeframe::MN1));
    }

    #[test]
    fn test_bars_between() {
        let tf = Timeframe::M1;
        assert_eq!(tf.bars_between(0, 300), 5); // 5 minutes = 5 bars
    }
}
