//! 暦の日付（期限・FR-001）。時刻も時差も持たない。

use alloc::string::String;
use core::fmt::Write as _;

/// 年・月・日。存在する日付だけを作れる（RS-001）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    year: i32,
    month: u8,
    day: u8,
}

impl Date {
    /// 年・月・日から。存在しない日付（13 月・2 月 30 日）は `None`。
    #[must_use]
    pub const fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if month == 0 || month > 12 || day == 0 || day > Self::days_in(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// `YYYY-MM-DD` から。形が違えば `None`。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (year, rest) = text.split_once('-')?;
        let (month, day) = rest.split_once('-')?;
        if year.len() != 4 || month.len() != 2 || day.len() != 2 {
            return None;
        }
        Self::new(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)
    }

    /// `YYYY-MM-DD`。
    #[must_use]
    pub fn text(self) -> String {
        let mut text = String::with_capacity(10);
        let _written: core::fmt::Result =
            write!(text, "{:04}-{:02}-{:02}", self.year, self.month, self.day);
        text
    }

    /// 年。
    #[must_use]
    pub const fn year(self) -> i32 {
        self.year
    }

    /// 月。
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }

    /// 日。
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }

    const fn days_in(year: i32, month: u8) -> u8 {
        let leap = (year % 4_i32 == 0_i32 && year % 100_i32 != 0_i32) || year % 400_i32 == 0_i32;
        match month {
            2 if leap => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_real_dates_exist() {
        assert!(Date::new(2026, 2, 29).is_none());
        assert!(Date::new(2024, 2, 29).is_some());
        assert!(Date::new(1900, 2, 29).is_none());
        assert!(Date::new(2000, 2, 29).is_some());
        assert!(Date::new(2026, 4, 31).is_none());
        assert!(Date::new(2026, 13, 1).is_none());
        assert!(Date::new(2026, 1, 0).is_none());
    }

    #[test]
    fn text_round_trips_and_orders_by_time() {
        let date = Date::new(2026, 10, 21).unwrap();
        assert_eq!(date.text(), "2026-10-21");
        assert_eq!(Date::parse("2026-10-21"), Some(date));
        assert_eq!(Date::parse("2026-1-21"), None);
        assert_eq!(Date::parse("2026/10/21"), None);
        assert_eq!(Date::parse("abcd-10-21"), None);
        assert!(Date::new(2026, 9, 30).unwrap() < date);
        assert_eq!((date.year(), date.month(), date.day()), (2026_i32, 10, 21));
    }
}
