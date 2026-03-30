// Chart buffer - ring buffer for candles

use super::types::Candle;
use std::collections::VecDeque;

/// Ring buffer for candle data with fixed capacity
#[derive(Debug, Clone)]
pub struct ChartBuffer {
    candles: VecDeque<Candle>,
    capacity: usize,
}

impl ChartBuffer {
    /// Create new buffer with given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            candles: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Create buffer with default capacity (2000 candles)
    pub fn default() -> Self {
        Self::new(2000)
    }

    /// Push a new candle to the buffer
    /// If buffer is full, removes the oldest candle
    pub fn push(&mut self, candle: Candle) {
        if self.candles.len() >= self.capacity {
            self.candles.pop_front();
        }
        self.candles.push_back(candle);
    }

    /// Update the last candle (for real-time updates)
    /// If buffer is empty, pushes the candle instead
    pub fn update_last(&mut self, candle: Candle) {
        if self.candles.is_empty() {
            self.push(candle);
        } else {
            if let Some(last) = self.candles.back_mut() {
                *last = candle;
            }
        }
    }

    /// Get the last candle
    pub fn last(&self) -> Option<&Candle> {
        self.candles.back()
    }

    /// Get candle at index (0 = oldest)
    pub fn get(&self, index: usize) -> Option<&Candle> {
        self.candles.get(index)
    }

    /// Get all candles as slice (copies to vec for contiguous access)
    pub fn as_slice(&self) -> Vec<Candle> {
        self.candles.iter().cloned().collect()
    }

    /// Get number of candles in buffer
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    /// Get buffer capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all candles
    pub fn clear(&mut self) {
        self.candles.clear();
    }

    /// Set new capacity and truncate if necessary
    pub fn set_capacity(&mut self, new_capacity: usize) {
        self.capacity = new_capacity;

        // Truncate from front if over capacity
        while self.candles.len() > new_capacity {
            self.candles.pop_front();
        }

        // Shrink allocation if significantly smaller
        if new_capacity < self.candles.capacity() / 2 {
            self.candles.shrink_to(new_capacity);
        }
    }

    /// Get iterator over candles
    pub fn iter(&self) -> impl Iterator<Item = &Candle> {
        self.candles.iter()
    }

    /// Get mutable iterator over candles
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Candle> {
        self.candles.iter_mut()
    }

    /// Get range of candles (from..to)
    pub fn range(&self, from: usize, to: usize) -> Option<Vec<Candle>> {
        if to > self.candles.len() {
            return None;
        }
        Some(
            self.candles
                .iter()
                .skip(from)
                .take(to - from)
                .cloned()
                .collect(),
        )
    }

    /// Get last N candles
    pub fn last_n(&self, n: usize) -> Vec<Candle> {
        let start = self.candles.len().saturating_sub(n);
        self.candles.iter().skip(start).cloned().collect()
    }

    /// Find candle by timestamp (exact match)
    pub fn find_by_time(&self, time: i64) -> Option<(usize, &Candle)> {
        self.candles
            .iter()
            .enumerate()
            .find(|(_, c)| c.time == time)
    }

    /// Find nearest candle to given timestamp
    pub fn find_nearest(&self, time: i64) -> Option<(usize, &Candle)> {
        if self.is_empty() {
            return None;
        }

        let mut nearest_idx = 0;
        let mut nearest_diff = i64::MAX;

        for (idx, candle) in self.candles.iter().enumerate() {
            let diff = (candle.time - time).abs();
            if diff < nearest_diff {
                nearest_diff = diff;
                nearest_idx = idx;
            }
        }

        self.candles.get(nearest_idx).map(|c| (nearest_idx, c))
    }
}

impl Default for ChartBuffer {
    fn default() -> Self {
        Self::new(2000)
    }
}

impl From<Vec<Candle>> for ChartBuffer {
    fn from(candles: Vec<Candle>) -> Self {
        let capacity = candles.len().max(2000);
        let mut buffer = Self::new(capacity);
        for candle in candles {
            buffer.push(candle);
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candle(time: i64) -> Candle {
        Candle::new(time, 100.0, 101.0, 99.0, 100.5, 1000.0)
    }

    #[test]
    fn test_buffer_push() {
        let mut buffer = ChartBuffer::new(3);

        buffer.push(create_test_candle(1000));
        buffer.push(create_test_candle(2000));
        buffer.push(create_test_candle(3000));

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.last().unwrap().time, 3000);

        // Push beyond capacity
        buffer.push(create_test_candle(4000));
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get(0).unwrap().time, 2000); // Oldest is now 2000
        assert_eq!(buffer.last().unwrap().time, 4000);
    }

    #[test]
    fn test_buffer_update_last() {
        let mut buffer = ChartBuffer::new(10);

        buffer.push(create_test_candle(1000));
        buffer.push(create_test_candle(2000));

        let updated = Candle::new(2000, 105.0, 106.0, 104.0, 105.5, 2000.0);
        buffer.update_last(updated.clone());

        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.last().unwrap().o, 105.0);
        assert_eq!(buffer.last().unwrap().v, 2000.0);
    }

    #[test]
    fn test_buffer_last_n() {
        let mut buffer = ChartBuffer::new(10);

        for i in 0..5 {
            buffer.push(create_test_candle(i * 1000));
        }

        let last_3 = buffer.last_n(3);
        assert_eq!(last_3.len(), 3);
        assert_eq!(last_3[0].time, 2000);
        assert_eq!(last_3[2].time, 4000);
    }

    #[test]
    fn test_buffer_find() {
        let mut buffer = ChartBuffer::new(10);

        buffer.push(create_test_candle(1000));
        buffer.push(create_test_candle(2000));
        buffer.push(create_test_candle(3000));

        let (idx, candle) = buffer.find_by_time(2000).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(candle.time, 2000);

        assert!(buffer.find_by_time(9999).is_none());
    }

    #[test]
    fn test_buffer_find_nearest() {
        let mut buffer = ChartBuffer::new(10);

        buffer.push(create_test_candle(1000));
        buffer.push(create_test_candle(5000));
        buffer.push(create_test_candle(9000));

        let (idx, candle) = buffer.find_nearest(4500).unwrap();
        assert_eq!(idx, 1);
        assert_eq!(candle.time, 5000);

        let (idx, candle) = buffer.find_nearest(500).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(candle.time, 1000);
    }

    #[test]
    fn test_buffer_capacity_change() {
        let mut buffer = ChartBuffer::new(5);

        for i in 0..5 {
            buffer.push(create_test_candle(i * 1000));
        }

        assert_eq!(buffer.len(), 5);

        // Reduce capacity
        buffer.set_capacity(3);
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.get(0).unwrap().time, 2000); // Oldest kept
        assert_eq!(buffer.last().unwrap().time, 4000);
    }
}
