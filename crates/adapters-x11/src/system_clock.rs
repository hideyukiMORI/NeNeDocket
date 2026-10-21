//! 壁時計のポートの実装。`SystemTime` を Unix 秒にする。

use std::time::{SystemTime, UNIX_EPOCH};

use nene_docket_application::clock_port::ClockPort;
use nene_docket_core::wall_clock::WallClock;

/// OS の時計。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemClock;

impl ClockPort for SystemClock {
    /// いまの Unix 秒。1970 年より前なら 0。
    fn now(&self) -> WallClock {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0_u64, |elapsed| elapsed.as_secs());
        WallClock::new(i64::try_from(seconds).unwrap_or(i64::MAX))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_after_2026() {
        let year_2026 = 1_767_225_600_i64;
        assert!(SystemClock.now().seconds() > year_2026);
    }
}
