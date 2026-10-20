//! 矩形の中での文字の寄せ。

/// 横の寄せ。縦は常に中央。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    /// 左寄せ。
    Start,
    /// 中央。
    Center,
    /// 右寄せ。
    End,
}
