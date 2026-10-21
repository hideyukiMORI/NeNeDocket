//! 設定のポートの失敗。閉じた集合（ARC-010）。

/// 書けない。利用者に示す（FR-014）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsFailure {
    /// 設定ファイルを書けなかった。
    Unwritable,
}
