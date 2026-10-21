//! 壁時計。人に見せる時刻で、順序には使わない（ADR 0004）。

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
}
