//! Trading session and market calendar.
//!
//! Handles market hours, sessions, and holidays.

use crate::Timestamp;

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionState {
    /// Market is open for trading
    Open,
    /// Market is closed
    Closed,
    /// Pre-market session
    PreMarket,
    /// After-hours/post-market session
    AfterHours,
    /// Market is halted
    Halted,
    /// Auction period
    Auction,
}

impl SessionState {
    /// Can place orders
    pub const fn can_trade(&self) -> bool {
        matches!(self, Self::Open | Self::PreMarket | Self::AfterHours)
    }

    /// Is regular session
    pub const fn is_regular(&self) -> bool {
        matches!(self, Self::Open)
    }
}

/// Trading session definition
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TradingSession {
    /// Session name
    pub name: String,
    /// Start time (seconds from midnight UTC)
    pub start: u32,
    /// End time (seconds from midnight UTC)
    pub end: u32,
    /// Days of week (0 = Sunday, 6 = Saturday)
    pub days: [bool; 7],
    /// Timezone offset in seconds (for display)
    pub tz_offset: i32,
}

impl TradingSession {
    /// Create a 24/7 session (crypto)
    pub fn always_open() -> Self {
        Self {
            name: String::from("24/7"),
            start: 0,
            end: 86400,
            days: [true; 7],
            tz_offset: 0,
        }
    }

    /// NYSE regular hours (9:30 AM - 4:00 PM ET)
    pub fn nyse() -> Self {
        Self {
            name: String::from("NYSE"),
            start: 14 * 3600 + 30 * 60, // 14:30 UTC = 9:30 ET
            end: 21 * 3600,             // 21:00 UTC = 16:00 ET
            days: [false, true, true, true, true, true, false], // Mon-Fri
            tz_offset: -5 * 3600,
        }
    }

    /// London session (8:00 AM - 4:30 PM GMT)
    pub fn london() -> Self {
        Self {
            name: String::from("London"),
            start: 8 * 3600,
            end: 16 * 3600 + 30 * 60,
            days: [false, true, true, true, true, true, false],
            tz_offset: 0,
        }
    }

    /// Tokyo session (9:00 AM - 3:00 PM JST)
    pub fn tokyo() -> Self {
        Self {
            name: String::from("Tokyo"),
            start: 0,      // 00:00 UTC = 9:00 JST
            end: 6 * 3600, // 06:00 UTC = 15:00 JST
            days: [false, true, true, true, true, true, false],
            tz_offset: 9 * 3600,
        }
    }

    /// Check if timestamp is within session
    pub fn is_open(&self, timestamp_secs: i64) -> bool {
        // Get day of week (0 = Sunday)
        let days_since_epoch = timestamp_secs / 86400;
        let day_of_week = ((days_since_epoch + 4) % 7) as usize; // Jan 1 1970 was Thursday

        if !self.days[day_of_week] {
            return false;
        }

        // Get seconds since midnight UTC
        let secs_today = (timestamp_secs % 86400) as u32;

        if self.start < self.end {
            // Normal session (same day)
            secs_today >= self.start && secs_today < self.end
        } else {
            // Overnight session
            secs_today >= self.start || secs_today < self.end
        }
    }

    /// Get seconds until session opens (0 if already open)
    pub fn seconds_until_open(&self, timestamp_secs: i64) -> u32 {
        if self.is_open(timestamp_secs) {
            return 0;
        }

        let secs_today = (timestamp_secs % 86400) as u32;

        if secs_today < self.start {
            self.start - secs_today
        } else {
            // Next day
            86400 - secs_today + self.start
        }
    }

    /// Get seconds until session closes
    pub fn seconds_until_close(&self, timestamp_secs: i64) -> u32 {
        if !self.is_open(timestamp_secs) {
            return 0;
        }

        let secs_today = (timestamp_secs % 86400) as u32;

        if self.start < self.end {
            self.end - secs_today
        } else if secs_today >= self.start {
            86400 - secs_today + self.end
        } else {
            self.end - secs_today
        }
    }
}

/// Market calendar with holidays
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MarketCalendar {
    /// Regular trading session
    pub regular: TradingSession,
    /// Pre-market session (optional)
    pub pre_market: Option<TradingSession>,
    /// After-hours session (optional)
    pub after_hours: Option<TradingSession>,
    /// Holiday dates (days since Unix epoch)
    pub holidays: Vec<i32>,
    /// Early close dates (days since Unix epoch, end time in seconds)
    pub early_closes: Vec<(i32, u32)>,
}

impl MarketCalendar {
    /// Create a 24/7 calendar (crypto)
    pub fn always_open() -> Self {
        Self {
            regular: TradingSession::always_open(),
            pre_market: None,
            after_hours: None,
            holidays: Vec::new(),
            early_closes: Vec::new(),
        }
    }

    /// Create NYSE calendar
    pub fn nyse() -> Self {
        Self {
            regular: TradingSession::nyse(),
            pre_market: Some(TradingSession {
                name: String::from("Pre-Market"),
                start: 9 * 3600,  // 4:00 AM ET = 09:00 UTC
                end: 14 * 3600 + 30 * 60,
                days: [false, true, true, true, true, true, false],
                tz_offset: -5 * 3600,
            }),
            after_hours: Some(TradingSession {
                name: String::from("After-Hours"),
                start: 21 * 3600,
                end: 25 * 3600, // Wraps to 01:00 next day
                days: [false, true, true, true, true, true, false],
                tz_offset: -5 * 3600,
            }),
            holidays: Vec::new(),
            early_closes: Vec::new(),
        }
    }

    /// Check if a date is a holiday
    pub fn is_holiday(&self, timestamp_secs: i64) -> bool {
        let day = (timestamp_secs / 86400) as i32;
        self.holidays.binary_search(&day).is_ok()
    }

    /// Get current session state
    pub fn session_state(&self, timestamp_secs: i64) -> SessionState {
        if self.is_holiday(timestamp_secs) {
            return SessionState::Closed;
        }

        if self.regular.is_open(timestamp_secs) {
            // Check for early close
            let day = (timestamp_secs / 86400) as i32;
            if let Ok(idx) = self.early_closes.binary_search_by_key(&day, |&(d, _)| d) {
                let (_, close_time) = self.early_closes[idx];
                let secs_today = (timestamp_secs % 86400) as u32;
                if secs_today >= close_time {
                    return SessionState::Closed;
                }
            }
            return SessionState::Open;
        }

        if let Some(ref pm) = self.pre_market {
            if pm.is_open(timestamp_secs) {
                return SessionState::PreMarket;
            }
        }

        if let Some(ref ah) = self.after_hours {
            if ah.is_open(timestamp_secs) {
                return SessionState::AfterHours;
            }
        }

        SessionState::Closed
    }

    /// Add a holiday
    pub fn add_holiday(&mut self, timestamp_secs: i64) {
        let day = (timestamp_secs / 86400) as i32;
        if let Err(idx) = self.holidays.binary_search(&day) {
            self.holidays.insert(idx, day);
        }
    }

    /// Add an early close
    pub fn add_early_close(&mut self, timestamp_secs: i64, close_time_secs: u32) {
        let day = (timestamp_secs / 86400) as i32;
        if let Err(idx) = self.early_closes.binary_search_by_key(&day, |&(d, _)| d) {
            self.early_closes.insert(idx, (day, close_time_secs));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_24_7() {
        let session = TradingSession::always_open();
        assert!(session.is_open(0));
        assert!(session.is_open(1700000000));
    }

    #[test]
    fn test_calendar_state() {
        let cal = MarketCalendar::always_open();
        assert_eq!(cal.session_state(1700000000), SessionState::Open);
    }
}
