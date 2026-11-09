//! 課題の状態（FR-001）。区分は 6 つで固定（ADR 0024・PR #108 の方針 2.1）。名前の付け替えは段階 1。

/// 進み具合。閉じた集合（RS-002）。完了と中止は「閉じた」状態（[`ItemState::is_closed`]）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ItemState {
    /// 未着手。
    Open,
    /// 対応中。
    InProgress,
    /// 確認中（担当が終え、確認者が見る）。
    Review,
    /// 保留（止めている）。
    OnHold,
    /// 完了。
    Done,
    /// 中止（やらないと決めた。削除しない）。
    Cancelled,
}

impl ItemState {
    /// 表示の順。
    pub const ALL: [Self; 6] = [
        Self::Open,
        Self::InProgress,
        Self::Review,
        Self::OnHold,
        Self::Done,
        Self::Cancelled,
    ];

    /// 閉じた状態（完了・中止）。既定の表示から外し、灰色で残し、期限超過を付けない。
    #[must_use]
    pub const fn is_closed(self) -> bool {
        matches!(self, Self::Done | Self::Cancelled)
    }

    /// ジャーナルに書く語。読み書きで同じ。
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Review => "review",
            Self::OnHold => "on_hold",
            Self::Done => "done",
            Self::Cancelled => "cancelled",
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
    fn keywords_round_trip_and_are_distinct() {
        for state in ItemState::ALL {
            assert_eq!(ItemState::from_keyword(state.keyword()), Some(state));
        }
        assert_eq!(ItemState::from_keyword("closed"), None);
        assert_ne!(ItemState::Open.keyword(), ItemState::Done.keyword());
        let closed: alloc::vec::Vec<ItemState> = ItemState::ALL
            .into_iter()
            .filter(|state| state.is_closed())
            .collect();
        assert_eq!(closed, [ItemState::Done, ItemState::Cancelled]);
    }
}
