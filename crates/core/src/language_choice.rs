//! 設定に残す表示言語の希望（ADR 0011）。`system`（OS の言語に従う）か、決まった言語。表示する言語は観測した OS の言語と合わせて解決する。

use crate::language::Language;

/// `system` の語。
const SYSTEM: &str = "system";

/// OS に従うか、決めるか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageChoice {
    /// OS の言語に従う。分からなければ英語。
    System,
    /// この言語。
    Fixed(Language),
}

impl LanguageChoice {
    /// 選択肢の順。先頭が `system`。
    pub const ALL: [Self; 7] = [
        Self::System,
        Self::Fixed(Language::Japanese),
        Self::Fixed(Language::English),
        Self::Fixed(Language::SimplifiedChinese),
        Self::Fixed(Language::TraditionalChinese),
        Self::Fixed(Language::Korean),
        Self::Fixed(Language::Spanish),
    ];

    /// 設定の語から。知らない語は `None`。
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        if keyword == SYSTEM {
            return Some(Self::System);
        }
        Language::from_tag(keyword).map(Self::Fixed)
    }

    /// 設定に書く語。
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::System => SYSTEM,
            Self::Fixed(language) => language.tag(),
        }
    }

    /// 観測した OS の言語と合わせて、表示する言語にする。
    #[must_use]
    pub fn resolve(self, observed: Option<Language>) -> Language {
        match self {
            Self::System => observed.unwrap_or(Language::English),
            Self::Fixed(language) => language,
        }
    }

    /// 選択肢の中の位置。
    #[must_use]
    pub fn position(self) -> usize {
        Self::ALL
            .iter()
            .position(|choice| *choice == self)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_round_trip_and_system_falls_back_to_english() {
        for choice in LanguageChoice::ALL {
            assert_eq!(LanguageChoice::from_keyword(choice.keyword()), Some(choice));
            assert_eq!(LanguageChoice::ALL.get(choice.position()), Some(&choice));
        }
        assert_eq!(LanguageChoice::from_keyword("fr"), None);
        assert_eq!(
            LanguageChoice::System.resolve(Some(Language::Korean)),
            Language::Korean
        );
        assert_eq!(LanguageChoice::System.resolve(None), Language::English);
        assert_eq!(
            LanguageChoice::Fixed(Language::Japanese).resolve(Some(Language::Korean)),
            Language::Japanese
        );
    }
}
