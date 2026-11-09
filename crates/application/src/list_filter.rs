//! 表の絞り込みの札（Issue #124・docs/design/2026-11-09-toolbar-and-settings.md 第 1 節）。1 つだけ選ぶ。閉じた集合（RS-002）。
//! 状態の選択箱と「完了も表示」のチェックはこれに吸収した（同じことを 3 通りでできる状態をなくす）。

use nene_docket_core::date::Date;
use nene_docket_core::item::Item;
use nene_docket_core::item_state::ItemState;
use nene_docket_core::toolbar_word::ToolbarWord;
use nene_docket_core::word::Word;

/// 表に出す課題の条件。既定は「未完了」。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ListFilter {
    /// 閉じていない（完了・中止でない）。
    Open,
    /// 閉じていなくて、期限が今日より前。
    Overdue,
    /// 閉じていなくて、担当が空。
    Unassigned,
    /// 確認中。
    Review,
    /// 全部（閉じた課題も灰色で出す）。
    All,
}

impl ListFilter {
    /// 札の左からの順。
    pub const ALL: [Self; 5] = [
        Self::Open,
        Self::Overdue,
        Self::Unassigned,
        Self::Review,
        Self::All,
    ];

    /// 課題がこの条件に合うか。`today` はこの機械の地方時の日付（期限超過だけが使う）。
    #[must_use]
    pub fn matches(self, item: &Item, today: Option<Date>) -> bool {
        match self {
            Self::Open => !item.state().is_closed(),
            Self::Overdue => Self::is_overdue(item, today),
            Self::Unassigned => !item.state().is_closed() && item.assignee().trim().is_empty(),
            Self::Review => item.state() == ItemState::Review,
            Self::All => true,
        }
    }

    /// 期限を過ぎていて、まだ閉じていない。今日が期限なら過ぎていない（Issue #107 の約束）。
    #[must_use]
    pub fn is_overdue(item: &Item, today: Option<Date>) -> bool {
        !item.state().is_closed() && today.zip(item.due()).is_some_and(|(now, due)| due < now)
    }

    /// 札の語。
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Open => Word::Toolbar(ToolbarWord::FilterOpen),
            Self::Overdue => Word::Toolbar(ToolbarWord::FilterOverdue),
            Self::Unassigned => Word::Toolbar(ToolbarWord::FilterUnassigned),
            Self::Review => Word::Toolbar(ToolbarWord::FilterReview),
            Self::All => Word::All,
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::field_value::FieldValue;
    use nene_docket_core::item_id::ItemId;

    use super::*;

    fn item(state: ItemState, assignee: &str, due: Option<Date>) -> Item {
        Item::created(ItemId::new(AccountId::new(1), 1))
            .with(FieldValue::State(state))
            .with(FieldValue::Assignee(String::from(assignee)))
            .with(FieldValue::Due(due))
    }

    fn today() -> Option<Date> {
        Date::new(2026, 11, 9)
    }

    #[test]
    fn open_and_overdue_leave_out_closed_items_and_today_is_not_overdue() {
        let late = item(ItemState::InProgress, "mori", Date::new(2026, 11, 1));
        let due_today = item(ItemState::Open, "", today());
        let done_late = item(ItemState::Done, "", Date::new(2026, 10, 1));
        assert!(ListFilter::Open.matches(&late, today()));
        assert!(!ListFilter::Open.matches(&done_late, today()));
        assert!(ListFilter::Overdue.matches(&late, today()));
        assert!(!ListFilter::Overdue.matches(&due_today, today()));
        assert!(!ListFilter::Overdue.matches(&done_late, today()));
    }

    #[test]
    fn unassigned_review_and_all_take_their_items() {
        let late = item(ItemState::InProgress, "mori", Date::new(2026, 11, 1));
        let due_today = item(ItemState::Open, "", today());
        let review = item(ItemState::Review, "mori", None);
        let done_late = item(ItemState::Done, "", Date::new(2026, 10, 1));
        assert!(ListFilter::Unassigned.matches(&due_today, today()));
        assert!(!ListFilter::Unassigned.matches(&done_late, today()));
        assert!(ListFilter::Review.matches(&review, today()));
        assert!(!ListFilter::Review.matches(&late, today()));
        assert!(ListFilter::All.matches(&done_late, today()));
        assert_eq!(ListFilter::All.word(), Word::All);
    }
}
