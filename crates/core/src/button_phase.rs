//! ポインタの主ボタンの、このフレームでの状態。

/// 主ボタンが、このフレームでどうなったか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonPhase {
    /// 離れている。
    Idle,
    /// このフレームで押された。
    Pressed,
    /// 前から押されたまま。
    Held,
    /// このフレームで離された。
    Released,
}
