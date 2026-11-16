//! 暦の日付（期限・FR-001）。時刻も時差も持たない。

use alloc::string::String;
use core::fmt::Write as _;

use crate::weekday::Weekday;

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

    /// 期限の欄の書き方から（Issue #133）。`YYYY-MM-DD`・`YYYY/M/D`・`M/D`（今日から 1 年の内・過ぎた日なら翌年）・
    /// `+N`（今日から N 日後・N は 0〜3650）。今日が分からなければ `M/D` と `+N` は読まない。
    #[must_use]
    pub fn parse_near(text: &str, today: Option<Self>) -> Option<Self> {
        let text = text.trim();
        if let Some(days) = text.strip_prefix('+') {
            return Self::ahead(days, today?);
        }
        if text.contains('-') {
            return Self::parse(text);
        }
        let parts: alloc::vec::Vec<&str> = text.split('/').collect();
        match parts.as_slice() {
            [year, month, day] if year.len() == 4 => {
                Self::new(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)
            }
            [month, day] if month.len() <= 2 && day.len() <= 2 => {
                Self::month_day(month.parse().ok()?, day.parse().ok()?, today?)
            }
            _ => None,
        }
    }

    /// `+N` の `N`: 今日から N 日後。
    fn ahead(days: &str, today: Self) -> Option<Self> {
        let days: i64 = days.parse().ok()?;
        if (0..=MAX_AHEAD_DAYS).contains(&days) {
            today.plus_days(days)
        } else {
            None
        }
    }

    /// `M/D`: 今年のその日。過ぎていれば翌年。
    fn month_day(month: u8, day: u8, today: Self) -> Option<Self> {
        match Self::new(today.year, month, day) {
            Some(date) if date >= today => Some(date),
            Some(_) | None => Self::new(today.year.checked_add(1)?, month, day),
        }
    }

    /// 1970-01-01 からの日数（前なら負）。Howard Hinnant の `days_from_civil`。
    #[must_use]
    pub fn days(self) -> i64 {
        let year = i64::from(self.year) - i64::from(self.month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year.rem_euclid(400);
        let month = i64::from(self.month);
        let shifted_month = if month > 2 { month - 3 } else { month + 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + i64::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// 1970-01-01 からの日数から。表せない年なら `None`。
    #[must_use]
    pub fn from_days(days: i64) -> Option<Self> {
        let (year, month, day) = civil_from_days(days);
        Self::new(
            i32::try_from(year).ok()?,
            u8::try_from(month).ok()?,
            u8::try_from(day).ok()?,
        )
    }

    /// `days` 日後（負なら前）。
    #[must_use]
    pub fn plus_days(self, days: i64) -> Option<Self> {
        Self::from_days(self.days().checked_add(days)?)
    }

    /// 曜日。
    #[must_use]
    pub fn weekday(self) -> Weekday {
        // 1970-01-01 は木曜（月曜から数えて 3）。
        let index = (self.days() + 3).rem_euclid(7);
        Weekday::ALL
            .get(usize::try_from(index).unwrap_or(0))
            .copied()
            .unwrap_or(Weekday::Monday)
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

/// `+N` の上限（10 年）。
const MAX_AHEAD_DAYS: i64 = 3_650;

/// 1970-01-01 からの日数を（年・月・日）にする。Howard Hinnant の `civil_from_days`。
pub(crate) fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
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

    #[test]
    fn days_round_trip_and_weekdays_follow() {
        let date = Date::new(2026, 11, 16).unwrap();
        assert_eq!(Date::new(1970, 1, 1).unwrap().days(), 0);
        assert_eq!(Date::from_days(date.days()), Some(date));
        assert_eq!(Date::new(1969, 12, 31).unwrap().days(), -1);
        assert_eq!(date.weekday(), Weekday::Monday);
        assert_eq!(Date::new(1970, 1, 1).unwrap().weekday(), Weekday::Thursday);
        assert_eq!(date.plus_days(14), Date::new(2026, 11, 30));
        assert_eq!(
            Date::new(2024, 2, 28).unwrap().plus_days(1),
            Date::new(2024, 2, 29)
        );
    }

    #[test]
    fn the_due_field_reads_month_day_and_days_ahead_near_today() {
        let today = Date::new(2026, 11, 16);
        assert_eq!(Date::parse_near("11/30", today), Date::new(2026, 11, 30));
        assert_eq!(Date::parse_near("11/16", today), today);
        // 過ぎた日は翌年。
        assert_eq!(Date::parse_near("1/5", today), Date::new(2027, 1, 5));
        assert_eq!(Date::parse_near("+3", today), Date::new(2026, 11, 19));
        assert_eq!(Date::parse_near(" +0 ", today), today);
        assert_eq!(Date::parse_near("2026/12/1", today), Date::new(2026, 12, 1));
        assert_eq!(
            Date::parse_near("2026-12-01", today),
            Date::new(2026, 12, 1)
        );
    }

    #[test]
    fn the_due_field_refuses_other_forms_and_needs_today_for_the_near_ones() {
        let today = Date::new(2026, 11, 16);
        let bad = [
            "+", "+-1", "+3651", "2/30", "13/1", "1/2/3", "x", "", "2026-1-2",
        ];
        assert!(
            bad.iter()
                .all(|text| Date::parse_near(text, today).is_none())
        );
        // 今日が分からなければ近い形は読まない。
        assert_eq!(Date::parse_near("11/30", None), None);
        assert_eq!(
            Date::parse_near("2026-11-30", None),
            Date::new(2026, 11, 30)
        );
    }
}
