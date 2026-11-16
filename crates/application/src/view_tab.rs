//! 共有ビューのタブ（Issue #145・運用の筋書きの C）。いまは組み込みの 2 つだけで、作る・名前を付けるのは後で
//! 台帳の操作として足す（ADR 0021 の決定 5）。

use nene_docket_core::item::Item;
use nene_docket_core::toolbar_word::ToolbarWord;
use nene_docket_core::word::Word;

/// どの課題を表に出すか。絞り込みの札と検索は、この中で効く。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTab {
    /// 全部の課題。
    All,
    /// 担当の欄の字が自分のアカウントの名前と同じ課題（施主の決定 2026-11-16・データの形は変えない）。
    Mine,
}

impl ViewTab {
    /// 道具の行の左からの順。
    pub const ALL: [Self; 2] = [Self::All, Self::Mine];

    /// タブの名前。
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::All => Word::Toolbar(ToolbarWord::ViewAll),
            Self::Mine => Word::Toolbar(ToolbarWord::ViewMine),
        }
    }

    /// `item` がこのタブに入るか。`me` は自分のアカウントの名前（前後の空白は見ない）。
    #[must_use]
    pub fn matches(self, item: &Item, me: &str) -> bool {
        match self {
            Self::All => true,
            Self::Mine => {
                let me = me.trim();
                !me.is_empty() && item.assignee().trim() == me
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::field_value::FieldValue;
    use nene_docket_core::item_id::ItemId;

    fn assigned(name: &str) -> Item {
        Item::created(ItemId::new(AccountId::new(1), 1))
            .with(FieldValue::Assignee(alloc::string::String::from(name)))
    }

    #[test]
    fn mine_is_the_items_whose_assignee_is_my_name_and_all_is_everything() {
        assert!(ViewTab::Mine.matches(&assigned("森"), "森"));
        assert!(ViewTab::Mine.matches(&assigned(" 森 "), "森"));
        assert!(!ViewTab::Mine.matches(&assigned("森田"), "森"));
        assert!(!ViewTab::Mine.matches(&assigned(""), ""));
        assert!(ViewTab::All.matches(&assigned(""), "森"));
    }
}
