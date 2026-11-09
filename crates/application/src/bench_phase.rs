//! 計測のフレームの段（Issue #111）。

/// 計測の筋書きの中で、そのフレームが何を測るか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchPhase {
    /// 暖機。数えない（初回の字の家族・資源の作成を外す）。
    Warmup,
    /// 何もしないフレーム。
    Idle,
    /// 検索の欄を押して離す。数えない。
    Focus,
    /// 検索の欄に 1 字足す・消すフレーム（表の一覧を作り直す）。
    Search,
}
