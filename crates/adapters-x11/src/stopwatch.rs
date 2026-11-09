//! 経過時間のポートの実装（Issue #111）。単調時計（`std::time::Instant`）で、壁時計がずれても戻らない。

use core::time::Duration;
use std::time::Instant;

use nene_docket_application::stopwatch_port::StopwatchPort;

/// 作ったときから測る時計。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stopwatch {
    started: Instant,
}

impl Stopwatch {
    /// いまから測る。
    #[must_use]
    pub fn started() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl StopwatchPort for Stopwatch {
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_stopwatch_never_goes_back() {
        let watch = Stopwatch::started();
        let first = watch.elapsed();
        assert!(watch.elapsed() >= first);
    }
}
