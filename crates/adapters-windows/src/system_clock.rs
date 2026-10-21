//! 壁時計のポートの実装。`SystemTime` を Unix 秒にする。地方時との差は起動時に `GetTimeZoneInformation` で 1 度だけ読む（Issue #78）。

use std::time::{SystemTime, UNIX_EPOCH};

use nene_docket_application::clock_port::ClockPort;
use nene_docket_core::wall_clock::WallClock;
use windows::Win32::System::Time::{
    GetTimeZoneInformation, TIME_ZONE_ID_INVALID, TIME_ZONE_INFORMATION,
};

/// `TIME_ZONE_ID_DAYLIGHT`（winbase.h の 2）。windows クレートの Time には無いので値を置く。
const TIME_ZONE_ID_DAYLIGHT: u32 = 2;

/// OS の時計と、起動時に読んだ地方時との差（分）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemClock {
    offset_minutes: i32,
}

impl SystemClock {
    /// 起動時に時差を読んで作る。読めなければ 0（UTC で出す）。プロセスの間は固定。
    #[must_use]
    pub fn observed() -> Self {
        Self {
            offset_minutes: zone_offset().unwrap_or(0_i32),
        }
    }
}

/// `GetTimeZoneInformation`: `Bias` は UTC − 地方時（分・西が正）。夏時間の中なら `DaylightBias` を足す。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetTimeZoneInformation を呼ぶ（ADR 0002 の 4）"
)]
fn zone_offset() -> Option<i32> {
    let mut information = TIME_ZONE_INFORMATION::default();
    // SAFETY: information はこの関数の局所で、呼び出しの間だけ書かれる。
    let kind = unsafe { GetTimeZoneInformation(&raw mut information) };
    if kind == TIME_ZONE_ID_INVALID {
        return None;
    }
    let active = if kind == TIME_ZONE_ID_DAYLIGHT {
        information.DaylightBias
    } else {
        information.StandardBias
    };
    Some(-(information.Bias.saturating_add(active)))
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
