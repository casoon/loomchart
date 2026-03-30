//! Time bucket utilities for candle aggregation.
//!
//! Handles proper time alignment including DST considerations.

use crate::{Timeframe, Timestamp};

/// Time bucket for candle aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeBucket {
    /// Start timestamp (milliseconds)
    pub start: Timestamp,
    /// End timestamp (exclusive, milliseconds)
    pub end: Timestamp,
    /// Timeframe
    pub timeframe: Timeframe,
}

impl TimeBucket {
    /// Create a new bucket for timestamp
    pub fn new(timestamp_ms: Timestamp, timeframe: Timeframe) -> Self {
        let start = timeframe.bucket_start_ms(timestamp_ms);
        let end = start + timeframe.as_millis();
        Self { start, end, timeframe }
    }

    /// Check if timestamp is in this bucket
    #[inline]
    pub fn contains(&self, timestamp_ms: Timestamp) -> bool {
        timestamp_ms >= self.start && timestamp_ms < self.end
    }

    /// Get the next bucket
    #[inline]
    pub fn next(&self) -> Self {
        Self {
            start: self.end,
            end: self.end + self.timeframe.as_millis(),
            timeframe: self.timeframe,
        }
    }

    /// Get the previous bucket
    #[inline]
    pub fn prev(&self) -> Self {
        Self {
            start: self.start - self.timeframe.as_millis(),
            end: self.start,
            timeframe: self.timeframe,
        }
    }

    /// Progress through bucket (0.0 to 1.0)
    #[inline]
    pub fn progress(&self, timestamp_ms: Timestamp) -> f64 {
        if timestamp_ms <= self.start {
            0.0
        } else if timestamp_ms >= self.end {
            1.0
        } else {
            (timestamp_ms - self.start) as f64 / (self.end - self.start) as f64
        }
    }

    /// Milliseconds remaining in bucket
    #[inline]
    pub fn remaining_ms(&self, timestamp_ms: Timestamp) -> i64 {
        (self.end - timestamp_ms).max(0)
    }

    /// Duration in milliseconds
    #[inline]
    pub fn duration_ms(&self) -> i64 {
        self.end - self.start
    }
}

/// Iterator over time buckets
pub struct BucketIterator {
    current: TimeBucket,
    end: Timestamp,
}

impl BucketIterator {
    /// Create iterator from start to end
    pub fn new(start_ms: Timestamp, end_ms: Timestamp, timeframe: Timeframe) -> Self {
        Self {
            current: TimeBucket::new(start_ms, timeframe),
            end: end_ms,
        }
    }
}

impl Iterator for BucketIterator {
    type Item = TimeBucket;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.start >= self.end {
            None
        } else {
            let bucket = self.current;
            self.current = bucket.next();
            Some(bucket)
        }
    }
}

/// Utility functions for bucket operations
pub mod utils {
    use super::*;

    /// Align timestamp to bucket boundary
    #[inline]
    pub fn align(timestamp_ms: Timestamp, timeframe: Timeframe) -> Timestamp {
        timeframe.bucket_start_ms(timestamp_ms)
    }

    /// Get bucket index from a reference point
    #[inline]
    pub fn bucket_index(timestamp_ms: Timestamp, reference_ms: Timestamp, timeframe: Timeframe) -> i64 {
        let aligned_ts = align(timestamp_ms, timeframe);
        let aligned_ref = align(reference_ms, timeframe);
        (aligned_ts - aligned_ref) / timeframe.as_millis()
    }

    /// Generate bucket timestamps for a range
    pub fn generate_buckets(
        start_ms: Timestamp,
        end_ms: Timestamp,
        timeframe: Timeframe,
    ) -> impl Iterator<Item = Timestamp> {
        BucketIterator::new(start_ms, end_ms, timeframe).map(|b| b.start)
    }

    /// Check if two timestamps are in the same bucket
    #[inline]
    pub fn same_bucket(ts1: Timestamp, ts2: Timestamp, timeframe: Timeframe) -> bool {
        align(ts1, timeframe) == align(ts2, timeframe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bucket_contains() {
        let bucket = TimeBucket::new(3600000, Timeframe::H1); // 1 hour mark
        assert!(bucket.contains(3600000));
        assert!(bucket.contains(3600001));
        assert!(bucket.contains(7199999));
        assert!(!bucket.contains(7200000)); // Next hour
    }

    #[test]
    fn test_bucket_progress() {
        let bucket = TimeBucket::new(0, Timeframe::M1);
        assert_eq!(bucket.progress(0), 0.0);
        assert_eq!(bucket.progress(30000), 0.5);
        assert_eq!(bucket.progress(60000), 1.0);
    }

    #[test]
    fn test_bucket_iterator() {
        let buckets: Vec<_> = BucketIterator::new(0, 180000, Timeframe::M1).collect();
        assert_eq!(buckets.len(), 3);
        assert_eq!(buckets[0].start, 0);
        assert_eq!(buckets[1].start, 60000);
        assert_eq!(buckets[2].start, 120000);
    }
}
