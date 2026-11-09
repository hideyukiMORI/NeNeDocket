//! 1 フレームの 3 つの区間（Issue #111）。中核の `step`・描画命令を積む・窓へ出す。

use core::time::Duration;

/// `DocketScreen::step` の時間と、UI が描画命令を画素か描画の要求にする時間と、窓へ出して終わるまでの時間。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameLap {
    step: Duration,
    encode: Duration,
    present: Duration,
}

impl FrameLap {
    /// 3 つの区間から。
    #[must_use]
    pub const fn new(step: Duration, encode: Duration, present: Duration) -> Self {
        Self {
            step,
            encode,
            present,
        }
    }

    /// 中核（`step`）。
    #[must_use]
    pub const fn step(&self) -> Duration {
        self.step
    }

    /// 描画命令を積む（X11 は cairo で画素に・Windows は `BeginDraw` から `EndDraw` の前まで）。
    #[must_use]
    pub const fn encode(&self) -> Duration {
        self.encode
    }

    /// 窓へ出す（X11 は `PutImage` と往復 1 回・Windows は `EndDraw`）。
    #[must_use]
    pub const fn present(&self) -> Duration {
        self.present
    }
}
