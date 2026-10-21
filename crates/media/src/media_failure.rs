//! 画像を復号できなかった理由。閉じた集合（RS-002）。

/// 知らない形・壊れている・大きさが扱えない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaFailure {
    /// PNG でも JPEG でもない。
    UnknownFormat,
    /// 復号の途中で壊れていた。
    Broken,
    /// 幅か高さが 0、または画素の数が扱える範囲を超える。
    BadSize,
}
