//! 壁時計。人に見せる時刻で、順序には使わない（ADR 0004）。

use alloc::format;
use alloc::string::String;

/// UNIX 時刻の秒。書いた機械の時計なので、ずれていればずれたまま出る。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WallClock {
    seconds: i64,
}

impl WallClock {
    /// 秒から作る。
    #[must_use]
    pub const fn new(seconds: i64) -> Self {
        Self { seconds }
    }

    /// 秒。
    #[must_use]
    pub const fn seconds(self) -> i64 {
        self.seconds
    }

    /// `YYYY-MM-DD HH:MM`（UTC）。
    #[must_use]
    pub fn text(self) -> String {
        self.text_at(0)
    }

    /// `YYYY-MM-DD HH:MM`。UTC との差（分・東が正）を足した地方時で出す（時差はアダプタが知る・Issue #78）。
    #[must_use]
    pub fn text_at(self, offset_minutes: i32) -> String {
        let shifted = self
            .seconds
            .saturating_add(i64::from(offset_minutes).saturating_mul(60));
        let days = shifted.div_euclid(SECONDS_PER_DAY);
        let second_of_day = shifted.rem_euclid(SECONDS_PER_DAY);
        let (year, month, day) = civil_from_days(days);
        let hour = second_of_day / 3_600;
        let minute = (second_of_day % 3_600) / 60;
        format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}")
    }
}

const SECONDS_PER_DAY: i64 = 86_400;

/// 1970-01-01 からの日数を（年・月・日）にする。Howard Hinnant の `civil_from_days`。
fn civil_from_days(days: i64) -> (i64, i64, i64) {
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
    fn text_is_utc_civil_time() {
        assert_eq!(WallClock::new(0).text(), "1970-01-01 00:00");
        assert_eq!(WallClock::new(1_700_000_000).text(), "2023-11-14 22:13");
        assert_eq!(WallClock::new(951_782_400).text(), "2000-02-29 00:00");
        assert_eq!(WallClock::new(-1).text(), "1969-12-31 23:59");
    }

    #[test]
    fn the_offset_shifts_the_shown_time_across_the_day_boundary() {
        let late = WallClock::new(1_700_000_000);
        assert_eq!(late.text(), "2023-11-14 22:13");
        assert_eq!(late.text_at(540), "2023-11-15 07:13");
        assert_eq!(late.text_at(-330), "2023-11-14 16:43");
        assert_eq!(late.text_at(0), late.text());
    }
}
