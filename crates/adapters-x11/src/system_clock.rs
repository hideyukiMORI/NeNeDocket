//! 壁時計のポートの実装。`SystemTime` を Unix 秒にする。地方時との差は起動時に `date +%z` で 1 度だけ読む（Issue #78）。

use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use nene_docket_application::clock_port::ClockPort;
use nene_docket_core::wall_clock::WallClock;

/// OS の時計と、起動時に読んだ地方時との差（分）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemClock {
    offset_minutes: i32,
}

impl SystemClock {
    /// 起動時に時差を読んで作る。`date` が無い・形が読めないなら 0（UTC で出す）。
    /// プロセスの間は固定（夏時間の切り替えをまたいでも、次の起動まで変えない）。
    #[must_use]
    pub fn observed() -> Self {
        let offset_minutes = Command::new("date")
            .arg("+%z")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| offset_of(&String::from_utf8_lossy(&output.stdout)))
            .unwrap_or(0_i32);
        Self { offset_minutes }
    }
}

/// `+0900` / `-0530`（`date +%z` の形）を分にする。形が違えば `None`。
#[must_use]
pub fn offset_of(text: &str) -> Option<i32> {
    let text = text.trim();
    let (sign, digits) = match text.split_at_checked(1)? {
        ("+", digits) => (1_i32, digits),
        ("-", digits) => (-1_i32, digits),
        _ => return None,
    };
    if digits.len() != 4 || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let (hours, minutes) = digits.split_at_checked(2)?;
    let hours: i32 = hours.parse().ok()?;
    let minutes: i32 = minutes.parse().ok()?;
    if hours > 14_i32 || minutes > 59_i32 {
        return None;
    }
    Some(sign * (hours * 60 + minutes))
}

impl ClockPort for SystemClock {
    /// いまの Unix 秒。1970 年より前なら 0。
    fn now(&self) -> WallClock {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u64, |elapsed| elapsed.as_secs());
        WallClock::new(i64::try_from(seconds).unwrap_or(i64::MAX))
    }

    fn offset_minutes(&self) -> i32 {
        self.offset_minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_after_2026_and_the_offset_is_a_real_zone() {
        let clock = SystemClock::observed();
        let year_2026 = 1_767_225_600_i64;
        assert!(clock.now().seconds() > year_2026);
        assert!((-14_i32 * 60_i32..=14_i32 * 60_i32).contains(&clock.offset_minutes()));
    }

    #[test]
    fn date_offsets_are_read_and_wrong_shapes_are_refused() {
        assert_eq!(offset_of("+0900\n"), Some(540_i32));
        assert_eq!(offset_of("-0530"), Some(-330_i32));
        assert_eq!(offset_of("+0000"), Some(0_i32));
        assert_eq!(offset_of("0900"), None);
        assert_eq!(offset_of("+09:00"), None);
        assert_eq!(offset_of("+1500"), None);
        assert_eq!(offset_of(""), None);
    }
}
