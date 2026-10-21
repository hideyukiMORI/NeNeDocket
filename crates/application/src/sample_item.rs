//! 見本の課題。ジャーナルと併合（ADR 0004）が入るまでの、表に出す行。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::item_state::ItemState;

/// 番号・題名・状態・担当。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleItem {
    number: u32,
    title: String,
    state: ItemState,
    assignee: String,
}

impl SampleItem {
    /// 値から作る。
    #[must_use]
    pub const fn new(number: u32, title: String, state: ItemState, assignee: String) -> Self {
        Self {
            number,
            title,
            state,
            assignee,
        }
    }

    /// `count` 件の見本。番号は 1 から。状態と担当は決定的に巡る。
    #[must_use]
    pub fn generate(count: u32) -> Vec<Self> {
        let assignees = ["mori", "rina", "ayane"];
        let titles = [
            "設定ファイルの版 2",
            "表の列幅をドラッグで変える",
            "ジャーナルの fsync 間隔",
            "Windows の窓を出す",
            "日本語入力（XIM）を実測する",
        ];
        (1..=count)
            .map(|number| {
                let index = usize::try_from(number).unwrap_or(0);
                let state = ItemState::ALL
                    .get(index % ItemState::ALL.len())
                    .copied()
                    .unwrap_or(ItemState::Open);
                let title = titles.get(index % titles.len()).copied().unwrap_or("");
                let assignee = assignees
                    .get(index % assignees.len())
                    .copied()
                    .unwrap_or("");
                Self::new(
                    number,
                    format!("{title} #{number}"),
                    state,
                    String::from(assignee),
                )
            })
            .collect()
    }

    /// 番号。
    #[must_use]
    pub const fn number(&self) -> u32 {
        self.number
    }

    /// 題名。
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 状態。
    #[must_use]
    pub const fn state(&self) -> ItemState {
        self.state
    }

    /// 担当。
    #[must_use]
    pub fn assignee(&self) -> &str {
        &self.assignee
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_items_are_numbered_from_one_and_cycle_states() {
        let items = SampleItem::generate(7);
        assert_eq!(items.len(), 7);
        assert_eq!(items.first().map(SampleItem::number), Some(1));
        assert_eq!(
            items.first().map(SampleItem::state),
            Some(ItemState::InProgress)
        );
        assert_eq!(items.get(2).map(SampleItem::state), Some(ItemState::Open));
        assert!(
            items
                .get(3)
                .is_some_and(|item| item.title().ends_with("#4"))
        );
        assert_eq!(items.get(3).map(SampleItem::assignee), Some("rina"));
        assert!(SampleItem::generate(0).is_empty());
    }
}
