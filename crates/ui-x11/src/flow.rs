//! イベントを処理した後の分岐。

/// 続けるか、閉じるか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flow {
    /// 次のイベントを待つ。
    Continue,
    /// 窓を閉じて戻る。
    Quit,
}
