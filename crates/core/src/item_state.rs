//! 課題の状態（FR-001）。既定の 3 つ。

/// 進み具合。閉じた集合（RS-002）。台帳ごとの定義は「決め打ちしない」のまま。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemState {
    /// 未着手。
    Open,
    /// 着手中。
    InProgress,
    /// 完了。
    Done,
}

impl ItemState {
    /// 表示の順。
    pub const ALL: [Self; 3] = [Self::Open, Self::InProgress, Self::Done];

    /// 表に出す文字。
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::InProgress => "In progress",
            Self::Done => "Done",
        }
    }

    /// ジャーナルに書く語。読み書きで同じ。
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Done => "done",
        }
    }

    /// ジャーナルの語から。知らない語は `None`。
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|state| state.keyword() == keyword)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_round_trip_and_labels_are_distinct() {
        for state in ItemState::ALL {
            assert_eq!(ItemState::from_keyword(state.keyword()), Some(state));
        }
        assert_eq!(ItemState::from_keyword("closed"), None);
        assert_eq!(ItemState::InProgress.label(), "In progress");
        assert_ne!(ItemState::Open.label(), ItemState::Done.label());
    }
}
