//! 表示言語（FR-019）。閉じた集合で、文言の表（`wording`）の列。

/// 6 つの表示言語（決め打ちしない: 足すときは `wording` の列も足す）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// 日本語。
    Japanese,
    /// 英語。
    English,
    /// 簡体字の中国語。
    SimplifiedChinese,
    /// 繁体字の中国語。
    TraditionalChinese,
    /// 韓国語。
    Korean,
    /// スペイン語。
    Spanish,
}

impl Language {
    /// 選択肢の順。
    pub const ALL: [Self; 6] = [
        Self::Japanese,
        Self::English,
        Self::SimplifiedChinese,
        Self::TraditionalChinese,
        Self::Korean,
        Self::Spanish,
    ];

    /// 設定に書く語（BCP 47 の小文字）。
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Japanese => "ja",
            Self::English => "en",
            Self::SimplifiedChinese => "zh-hans",
            Self::TraditionalChinese => "zh-hant",
            Self::Korean => "ko",
            Self::Spanish => "es",
        }
    }

    /// その言語での自分の名前。選択肢に出す（どの言語で見ていても読める）。
    #[must_use]
    pub const fn native_name(self) -> &'static str {
        match self {
            Self::Japanese => "日本語",
            Self::English => "English",
            Self::SimplifiedChinese => "简体中文",
            Self::TraditionalChinese => "繁體中文",
            Self::Korean => "한국어",
            Self::Spanish => "Español",
        }
    }

    /// その言語の字形があるかを確かめる字（X11 のフォントの家族を選ぶ・ADR 0007 / 0011）。
    #[must_use]
    pub const fn probe(self) -> &'static str {
        match self {
            Self::Japanese => "日",
            Self::English => "A",
            Self::SimplifiedChinese => "简",
            Self::TraditionalChinese => "繁",
            Self::Korean => "한",
            Self::Spanish => "ñ",
        }
    }

    /// 設定の語から。知らない語は `None`。
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|language| language.tag() == tag)
    }

    /// OS のロケール（`ja_JP.UTF-8`・`zh-Hant-TW`・`zh_CN`・`es-ES`・`C` など）から。分からなければ `None`。
    #[must_use]
    pub fn from_locale(locale: &str) -> Option<Self> {
        let lowered: alloc::string::String = locale
            .chars()
            .take_while(|c| *c != '.' && *c != '@')
            .map(|c| {
                if c == '_' {
                    '-'
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect();
        let mut parts = lowered.split('-');
        let language = parts.next()?;
        let rest: alloc::vec::Vec<&str> = parts.collect();
        match language {
            "ja" => Some(Self::Japanese),
            "en" => Some(Self::English),
            "ko" => Some(Self::Korean),
            "es" => Some(Self::Spanish),
            "zh" => Some(Self::chinese_of(&rest)),
            _ => None,
        }
    }

    /// 中国語の変種。`hant` か台湾・香港・マカオなら繁体字、それ以外は簡体字。
    fn chinese_of(rest: &[&str]) -> Self {
        let traditional = rest
            .iter()
            .any(|part| matches!(*part, "hant" | "tw" | "hk" | "mo"));
        if traditional {
            Self::TraditionalChinese
        } else {
            Self::SimplifiedChinese
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_round_trip_and_locales_resolve() {
        for language in Language::ALL {
            assert_eq!(Language::from_tag(language.tag()), Some(language));
            assert!(!language.native_name().is_empty());
            assert!(!language.probe().is_empty());
        }
        assert_eq!(Language::from_tag("JA"), None);
    }

    #[test]
    fn locales_resolve_to_a_language_or_none() {
        assert_eq!(
            Language::from_locale("ja_JP.UTF-8"),
            Some(Language::Japanese)
        );
        assert_eq!(
            Language::from_locale("en_US.UTF-8"),
            Some(Language::English)
        );
        assert_eq!(
            Language::from_locale("zh_CN"),
            Some(Language::SimplifiedChinese)
        );
        assert_eq!(
            Language::from_locale("zh-Hant-TW"),
            Some(Language::TraditionalChinese)
        );
        assert_eq!(
            Language::from_locale("zh_HK.UTF-8"),
            Some(Language::TraditionalChinese)
        );
        assert_eq!(Language::from_locale("ko-KR"), Some(Language::Korean));
        assert_eq!(Language::from_locale("es_MX@euro"), Some(Language::Spanish));
        assert_eq!(Language::from_locale("C"), None);
        assert_eq!(Language::from_locale(""), None);
    }
}
