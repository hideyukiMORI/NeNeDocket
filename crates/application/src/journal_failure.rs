//! ジャーナルのポートの失敗。閉じた集合（ARC-010）。store の失敗はアダプタがここへ写す。

/// 書けない・読めない。どちらも利用者に示す（FR-014）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalFailure {
    /// 追記か同期ができなかった。
    Unwritable,
    /// 他人のジャーナルを読めなかった。
    Unreadable,
}

impl JournalFailure {
    /// 利用者に見せる 1 行。
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Unwritable => "journal: cannot write",
            Self::Unreadable => "journal: cannot read others",
        }
    }
}
