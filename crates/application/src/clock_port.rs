//! 壁時計のポート（ARC-007）。操作に添える表示用の時刻。実装はアダプタ。

use nene_docket_core::wall_clock::WallClock;

/// いまの壁時計。順序には使わない（論理時計は application が持つ）。
pub trait ClockPort {
    /// いま。
    fn now(&self) -> WallClock;
}
