// utils/time.rs

use near_sdk::{env, Timestamp};

pub struct Time;

impl Time {
    // Constants for time calculations (in nanoseconds)
    pub const BLOCK_DURATION: u64 = 300_000_000_000;      // 5 minutes
    pub const PRIORITY_DURATION: u64 = 120_000_000_000;   // 2 minutes
    pub const PUBLIC_DURATION: u64 = 180_000_000_000;     // 3 minutes
    pub const ONE_DAY: u64 = 86_400_000_000_000;         // 24 hours
    
    pub fn assert_valid_time_range(start: Timestamp, end: Timestamp) -> bool {
        assert!(start < end, "Invalid time range: start must be before end");
        assert!(
            end > env::block_timestamp(),
            "End time must be in the future"
        );
        true
    }

    pub fn is_within_range(current: Timestamp, start: Timestamp, end: Timestamp) -> bool {
        current >= start && current <= end
    }

    pub fn get_block_end_time(start_time: Timestamp) -> Timestamp {
        start_time + Self::BLOCK_DURATION
    }

    pub fn get_priority_end_time(block_end: Timestamp) -> Timestamp {
        block_end + Self::PRIORITY_DURATION
    }

    pub fn get_public_end_time(block_end: Timestamp) -> Timestamp {
        block_end + Self::PUBLIC_DURATION
    }

    pub fn is_same_day(time1: Timestamp, time2: Timestamp) -> bool {
        time1 / Self::ONE_DAY == time2 / Self::ONE_DAY
    }

    pub fn get_days_between(time1: Timestamp, time2: Timestamp) -> u64 {
        let diff = if time2 > time1 { time2 - time1 } else { time1 - time2 };
        diff / Self::ONE_DAY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_ranges() {
        let current = 1_000_000;
        let start = current - 1_000;
        let end = current + 1_000;
        
        assert!(Time::is_within_range(current, start, end));
        assert!(Time::assert_valid_time_range(start, end));
    }

    #[test]
    fn test_block_timings() {
        let start_time = 1_000_000;
        let block_end = Time::get_block_end_time(start_time);
        let priority_end = Time::get_priority_end_time(block_end);
        let public_end = Time::get_public_end_time(block_end);
        
        assert_eq!(block_end - start_time, Time::BLOCK_DURATION);
        assert_eq!(priority_end - block_end, Time::PRIORITY_DURATION);
        assert_eq!(public_end - block_end, Time::PUBLIC_DURATION);
    }

    #[test]
    fn test_day_calculations() {
        let time1 = Time::ONE_DAY * 2;
        let time2 = Time::ONE_DAY * 3;
        
        assert_eq!(Time::get_days_between(time1, time2), 1);
        assert!(!Time::is_same_day(time1, time2));
    }
}
