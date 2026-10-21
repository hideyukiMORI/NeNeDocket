//! 利用者に見せるべき降格（FR-014）。黙って既定値へ落とさない。

use alloc::format;
use alloc::string::String;

use crate::journal_failure::JournalFailure;

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
}

impl Notice {
    /// 文言。
    #[must_use]
    pub fn describe(self) -> String {
        match self {
            Self::Journal(failure) => String::from(failure.describe()),
            Self::Orphans(count) => format!("{count} operations target unknown items"),
            Self::RejectedLines(count) => format!("{count} journal lines could not be read"),
            Self::NotADate => String::from("due: not a date (YYYY-MM-DD), not saved"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_notice_has_words() {
        assert_eq!(
            Notice::Journal(JournalFailure::Unwritable).describe(),
            "journal: cannot write"
        );
        assert!(Notice::Orphans(2).describe().starts_with("2 "));
        assert!(Notice::RejectedLines(1).describe().contains("1 journal"));
        assert!(Notice::NotADate.describe().contains("YYYY-MM-DD"));
    }
}
