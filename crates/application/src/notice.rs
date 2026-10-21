//! 利用者に見せるべき降格（FR-014）。黙って既定値へ落とさない。文言は表示言語で（FR-019）。

use alloc::format;
use alloc::string::String;

use nene_docket_core::language::Language;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::journal_failure::JournalFailure;
use crate::settings_failure::SettingsFailure;

/// 画面の下に出す 1 行の元。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Notice {
    /// ジャーナルが書けない・読めない。
    Journal(JournalFailure),
    /// 宛先の無い操作がある（作られていない課題への操作）。
    Orphans(usize),
    /// 読めなかった行がある。
    RejectedLines(usize),
    /// 期限の欄が日付の形（`YYYY-MM-DD`）でない。追記していない。
    NotADate,
    /// 設定を書けない。
    Settings(SettingsFailure),
    /// 配色の欄の名前が組み込みに無い。変えていない。
    UnknownTheme,
    /// 署名か権限が通らず捨てた行がある（FR-010）。
    UnverifiedLines(usize),
    /// 台帳にある自分の公開鍵が手元と違う。
    KeyMismatch,
    /// 自分はこの台帳の編集者でない。追記は読み手が捨てる。
    NotAnEditor,
}

impl Notice {
    /// 文言。
    #[must_use]
    pub fn describe(self, language: Language) -> String {
        let word = |word: Word| wording::text(language, word);
        match self {
            Self::Journal(failure) => String::from(word(failure.word())),
            Self::Orphans(count) => format!("{}: {count}", word(Word::OrphanOperations)),
            Self::RejectedLines(count) => format!("{}: {count}", word(Word::RejectedLines)),
            Self::NotADate => String::from(word(Word::NotADate)),
            Self::Settings(SettingsFailure::Unwritable) => {
                String::from(word(Word::CannotWriteSettings))
            }
            Self::UnknownTheme => String::from(word(Word::UnknownTheme)),
            Self::UnverifiedLines(count) => format!("{}: {count}", word(Word::UnverifiedLines)),
            Self::KeyMismatch => String::from(word(Word::KeyMismatch)),
            Self::NotAnEditor => String::from(word(Word::NotAnEditor)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_notice_has_words_in_every_language() {
        let all = [
            Notice::Journal(JournalFailure::Unwritable),
            Notice::Journal(JournalFailure::Unreadable),
            Notice::Orphans(2),
            Notice::RejectedLines(1),
            Notice::NotADate,
            Notice::Settings(SettingsFailure::Unwritable),
            Notice::UnknownTheme,
            Notice::UnverifiedLines(3),
            Notice::KeyMismatch,
            Notice::NotAnEditor,
        ];
        for language in Language::ALL {
            for notice in all {
                assert!(
                    !notice.describe(language).is_empty(),
                    "{language:?} {notice:?}"
                );
            }
        }
        assert_eq!(
            Notice::Journal(JournalFailure::Unwritable).describe(Language::English),
            "journal: cannot write"
        );
        assert!(
            Notice::Orphans(2)
                .describe(Language::Japanese)
                .ends_with(": 2")
        );
        assert!(
            Notice::RejectedLines(1)
                .describe(Language::English)
                .ends_with(": 1")
        );
        assert!(
            Notice::NotADate
                .describe(Language::Spanish)
                .contains("AAAA-MM-DD")
        );
        assert!(
            Notice::UnverifiedLines(3)
                .describe(Language::Korean)
                .ends_with(": 3")
        );
    }
}
