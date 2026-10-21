//! OS の既定のアプリで開けなかった（FR-022）。閉じた集合（ARC-010）。

use nene_docket_core::attachment_word::AttachmentWord;
use nene_docket_core::word::Word;

/// 起動できなかった。利用者に示す（FR-014）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenFailure {
    /// 起動の命令が失敗した（アプリが無い・パスが無い）。
    CannotOpen,
}

impl OpenFailure {
    /// 利用者に見せる語。
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::CannotOpen => Word::Attachment(AttachmentWord::CannotOpen),
        }
    }
}
