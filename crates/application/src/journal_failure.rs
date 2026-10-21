//! ジャーナルのポートの失敗。閉じた集合（ARC-010）。store の失敗はアダプタがここへ写す。

use nene_docket_core::attachment_word::AttachmentWord;
use nene_docket_core::word::Word;

/// 書けない・読めない。どちらも利用者に示す（FR-014）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalFailure {
    /// 追記か同期ができなかった。
    Unwritable,
    /// 他人のジャーナルを読めなかった。
    Unreadable,
    /// 権限（`permissions.v1.txt` と `docket.v1.txt`）を書けなかった。
    PermissionsUnwritable,
    /// 添付する元のファイルを読めない（FR-021）。
    AttachmentUnreadable,
    /// 添付の写しを置けない（FR-021）。
    AttachmentUnwritable,
    /// 添付の画像を復号できない（FR-022）。
    AttachmentUndecodable,
}

impl JournalFailure {
    /// 利用者に見せる語（表示言語で引く）。
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Unwritable => Word::CannotWriteJournal,
            Self::Unreadable => Word::CannotReadJournals,
            Self::PermissionsUnwritable => Word::CannotWritePermissions,
            Self::AttachmentUnreadable => Word::Attachment(AttachmentWord::CannotRead),
            Self::AttachmentUnwritable => Word::Attachment(AttachmentWord::CannotWrite),
            Self::AttachmentUndecodable => Word::Attachment(AttachmentWord::CannotDecode),
        }
    }
}
