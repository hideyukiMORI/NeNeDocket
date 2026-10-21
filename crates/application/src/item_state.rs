//! 課題の状態（FR-001）。既定の 3 つ。

/// 進み具合。閉じた集合（RS-002）。台帳ごとの定義は「決め打ちしない」のまま。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_are_distinct() {
        assert_eq!(ItemState::ALL.len(), 3);
        assert_eq!(ItemState::InProgress.label(), "In progress");
        assert_ne!(ItemState::Open.label(), ItemState::Done.label());
    }
}
