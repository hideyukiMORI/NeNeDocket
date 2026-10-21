//! 壁時計の見せ方（Issue #78）: 地方時への時差と、いまの時刻。順序には関わらない（ADR 0004）。
//! 届いた操作の壁時計が自分の時計より 5 分以上先なら「時計がずれている」印を頭に付ける。

use alloc::string::String;

use nene_docket_core::wall_clock::WallClock;

use crate::clock_choice::ClockChoice;
use crate::clock_port::ClockPort;

/// これより先の壁時計は、書いた機械の時計がずれているとみなす（秒）。
const SKEW_SECONDS: i64 = 5_i64 * 60;
/// ずれの印。
const SKEW_MARK: &str = "\u{26a0} ";

/// UTC との差（分）と、いま。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockDisplay {
    offset_minutes: i32,
    now: WallClock,
}

impl ClockDisplay {
    /// 時差といまから。
    #[must_use]
    pub const fn new(offset_minutes: i32, now: WallClock) -> Self {
        Self {
            offset_minutes,
            now,
        }
    }

    /// UTC で出す。
    #[must_use]
    pub const fn utc(now: WallClock) -> Self {
        Self::new(0, now)
    }

    /// 設定と時計のポートから。地方時なら時差をアダプタに聞く。
    #[must_use]
    pub fn of(choice: ClockChoice, clock: &dyn ClockPort) -> Self {
        let offset_minutes = match choice {
            ClockChoice::Local => clock.offset_minutes(),
            ClockChoice::Utc => 0_i32,
        };
        Self::new(offset_minutes, clock.now())
    }

    /// 時差（分）。
    #[must_use]
    pub const fn offset_minutes(self) -> i32 {
        self.offset_minutes
    }

    /// 見せる文字列。時差を足し、いまより 5 分以上先なら印を付ける。
    #[must_use]
    pub fn text_of(self, when: WallClock) -> String {
        let shown = when.text_at(self.offset_minutes);
        if when.seconds().saturating_sub(self.now.seconds()) >= SKEW_SECONDS {
            let mut marked = String::from(SKEW_MARK);
            marked.push_str(&shown);
            marked
        } else {
            shown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Tokyo;

    impl ClockPort for Tokyo {
        fn now(&self) -> WallClock {
            WallClock::new(1_700_000_000)
        }

        fn offset_minutes(&self) -> i32 {
            540
        }
    }

    #[test]
    fn local_time_is_shifted_and_a_clock_ahead_by_five_minutes_is_marked() {
        let now = WallClock::new(1_700_000_000);
        let display = ClockDisplay::of(ClockChoice::Local, &Tokyo);
        assert_eq!(display.offset_minutes(), 540_i32);
        assert_eq!(display.text_of(now), "2023-11-15 07:13");
        assert_eq!(
            display.text_of(WallClock::new(1_700_000_000_i64 + 299)),
            "2023-11-15 07:18"
        );
        assert_eq!(
            display.text_of(WallClock::new(1_700_000_000_i64 + 300)),
            "\u{26a0} 2023-11-15 07:18"
        );
        assert_eq!(
            display.text_of(WallClock::new(1_700_000_000_i64 - 86_400)),
            "2023-11-14 07:13"
        );
        let utc = ClockDisplay::of(ClockChoice::Utc, &Tokyo);
        assert_eq!(utc, ClockDisplay::utc(now));
        assert_eq!(utc.text_of(now), "2023-11-14 22:13");
    }
}
