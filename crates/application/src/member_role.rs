//! 参加者の役（FR-009 / FR-010）。閉じた集合（RS-002）。

use nene_docket_core::word::Word;

/// 台帳に対して何ができるか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberRole {
    /// 持ち主。権限を決める。
    Owner,
    /// 編集者（グループの中も）。
    Editor,
    /// 読むだけ。
    Viewer,
    /// 権限の無い台帳。誰でも書ける。
    Anyone,
}

impl MemberRole {
    /// 表示言語で引く語。
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Owner => Word::Owner,
            Self::Editor => Word::Editor,
            Self::Viewer => Word::Viewer,
            Self::Anyone => Word::Anyone,
        }
    }
}
