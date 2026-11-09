//! 経過時間を測るポート（ARC-007・Issue #111）。性能の計測（`--bench`）にだけ使い、台帳にも順序にも使わない。実装はアダプタの単調時計。

use core::time::Duration;

/// 作ったときからの経過。壁時計と違い、戻らない。
pub trait StopwatchPort {
    /// 作ったときからの経過。
    fn elapsed(&self) -> Duration;
}
