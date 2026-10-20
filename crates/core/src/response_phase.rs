//! 部品とポインタの関係。

/// このフレームで、ポインタが部品に対してどうなっているか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponsePhase {
    /// 何もしていない。
    Idle,
    /// ポインタが上に乗っている。
    Hovered,
    /// この部品の上で押され、まだ離されていない（ポインタが外へ出ても続く）。
    Held,
}
