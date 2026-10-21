//! 履歴と `--audit` の時刻を地方時で出すか UTC で出すか（設定・Issue #78）。

/// 地方時（既定）か UTC か。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockChoice {
    /// この機械の地方時（アダプタが時差を渡す）。
    Local,
    /// UTC。
    Utc,
}

impl ClockChoice {
    /// 設定ファイルの語。
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Utc => "utc",
        }
    }

    /// 設定ファイルの語から。知らない語は `None`。
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        match keyword {
            "local" => Some(Self::Local),
            "utc" => Some(Self::Utc),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_round_trip_and_unknown_words_are_refused() {
        for choice in [ClockChoice::Local, ClockChoice::Utc] {
            assert_eq!(ClockChoice::from_keyword(choice.keyword()), Some(choice));
        }
        assert_eq!(ClockChoice::from_keyword("jst"), None);
    }
}
