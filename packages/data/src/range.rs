//! Time range specification for data queries.

use loom_core::Timestamp;
use chrono::{DateTime, Utc, Duration};

/// Time range for data queries
#[derive(Debug, Clone, Copy)]
pub struct TimeRange {
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
    pub limit: Option<usize>,
}

impl TimeRange {
    /// Create a range with specific start and end
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
            limit: None,
        }
    }

    /// All available data
    pub fn all() -> Self {
        Self {
            start: None,
            end: None,
            limit: None,
        }
    }

    /// Last N candles
    pub fn last_n(n: usize) -> Self {
        Self {
            start: None,
            end: None,
            limit: Some(n),
        }
    }

    /// From a specific time to now
    pub fn from(start: Timestamp) -> Self {
        Self {
            start: Some(start),
            end: None,
            limit: None,
        }
    }

    /// From start to specific end time
    pub fn until(end: Timestamp) -> Self {
        Self {
            start: None,
            end: Some(end),
            limit: None,
        }
    }

    /// Last N days
    pub fn last_days(days: i64) -> Self {
        let now = Utc::now();
        let start = now - Duration::days(days);
        Self {
            start: Some(start.timestamp_millis()),
            end: Some(now.timestamp_millis()),
            limit: None,
        }
    }

    /// Last N hours
    pub fn last_hours(hours: i64) -> Self {
        let now = Utc::now();
        let start = now - Duration::hours(hours);
        Self {
            start: Some(start.timestamp_millis()),
            end: Some(now.timestamp_millis()),
            limit: None,
        }
    }

    /// Today's data
    pub fn today() -> Self {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        Self {
            start: Some(DateTime::<Utc>::from_naive_utc_and_offset(start, Utc).timestamp_millis()),
            end: Some(now.timestamp_millis()),
            limit: None,
        }
    }

    /// This week's data
    pub fn this_week() -> Self {
        Self::last_days(7)
    }

    /// This month's data
    pub fn this_month() -> Self {
        Self::last_days(30)
    }

    /// This year's data
    pub fn this_year() -> Self {
        Self::last_days(365)
    }

    /// With a limit applied
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if a timestamp is within range
    pub fn contains(&self, timestamp: Timestamp) -> bool {
        if let Some(start) = self.start {
            if timestamp < start {
                return false;
            }
        }
        if let Some(end) = self.end {
            if timestamp > end {
                return false;
            }
        }
        true
    }

    /// Filter candles by range
    pub fn filter<T: HasTimestamp>(&self, items: Vec<T>) -> Vec<T> {
        let filtered: Vec<T> = items
            .into_iter()
            .filter(|item| self.contains(item.timestamp()))
            .collect();

        if let Some(limit) = self.limit {
            filtered.into_iter().rev().take(limit).rev().collect()
        } else {
            filtered
        }
    }
}

/// Trait for items with a timestamp
pub trait HasTimestamp {
    fn timestamp(&self) -> Timestamp;
}

impl HasTimestamp for loom_core::Candle {
    fn timestamp(&self) -> Timestamp {
        self.time
    }
}

impl Default for TimeRange {
    fn default() -> Self {
        Self::all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_range() {
        let range = TimeRange::new(1000, 2000);
        assert!(range.contains(1000));
        assert!(range.contains(1500));
        assert!(range.contains(2000));
        assert!(!range.contains(999));
        assert!(!range.contains(2001));
    }

    #[test]
    fn test_last_n() {
        let range = TimeRange::last_n(10);
        assert!(range.limit == Some(10));
    }
}
