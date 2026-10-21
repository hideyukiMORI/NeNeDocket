//! 課題の項目の名前。

/// 閉じた集合（RS-002）。ジャーナルの語と表の列に使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Field {
    /// 題名。
    Title,
    /// 本文。
    Body,
    /// 状態。
    State,
    /// 担当。
    Assignee,
    /// 期限。
    Due,
    /// ラベル。
    Labels,
    /// 並び順。
    Rank,
}

impl Field {
    /// 全部。
    pub const ALL: [Self; 7] = [
        Self::Title,
        Self::Body,
        Self::State,
        Self::Assignee,
        Self::Due,
        Self::Labels,
        Self::Rank,
    ];

    /// ジャーナルに書く語。
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Body => "body",
            Self::State => "state",
            Self::Assignee => "assignee",
            Self::Due => "due",
            Self::Labels => "labels",
            Self::Rank => "rank",
        }
    }

    /// ジャーナルの語から。知らない語は `None`。
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|field| field.keyword() == keyword)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_round_trip() {
        for field in Field::ALL {
            assert_eq!(Field::from_keyword(field.keyword()), Some(field));
        }
        assert_eq!(Field::from_keyword("owner"), None);
    }
}
