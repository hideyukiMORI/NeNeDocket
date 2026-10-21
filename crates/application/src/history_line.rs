//! 課題 1 件の履歴の 1 行（FR-006）。いつ・誰が・何を・何から何へ。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::item::Item;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::language::Language;
use nene_docket_core::merge;
use nene_docket_core::op::Op;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::wall_clock::WallClock;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::history_change::HistoryChange;

/// 「誰が」に出すアカウントの ID の桁数。名前はジャーナルに無い。
const WHO_DIGITS: usize = 4;

/// 壁時計・アカウント・Undo の追記か・変更。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryLine {
    when: WallClock,
    who: AccountId,
    undo: bool,
    change: HistoryChange,
}

impl HistoryLine {
    /// 課題に触れた操作を全順序に並べ、値を追いながら 1 行ずつにする。新しいものが先。
    #[must_use]
    pub fn lines<'a, I>(ops: I, item: ItemId) -> Vec<Self>
    where
        I: IntoIterator<Item = &'a Op>,
    {
        let mut current = Item::created(item);
        let mut lines: Vec<Self> = merge::history(ops, item)
            .into_iter()
            .map(|op| {
                let change = change_of(&current, op.kind());
                current = applied(
                    core::mem::replace(&mut current, Item::created(item)),
                    op.kind(),
                );
                Self {
                    when: op.wall(),
                    who: op.id().account(),
                    undo: op.undoes().is_some(),
                    change,
                }
            })
            .collect();
        lines.reverse();
        lines
    }

    /// 表の 3 列: いつ（UTC）・誰が（ID の先頭）・何を。Undo の追記は表示言語の印を頭に付ける。
    #[must_use]
    pub fn columns(&self, language: Language) -> [String; 3] {
        let who: String = self.who.hex().chars().take(WHO_DIGITS).collect();
        let change = if self.undo {
            format!(
                "{}{}",
                wording::text(language, Word::UndoMark),
                self.change.describe(language)
            )
        } else {
            self.change.describe(language)
        };
        [self.when.text(), who, change]
    }

    /// 壁時計。
    #[must_use]
    pub const fn when(&self) -> WallClock {
        self.when
    }

    /// Undo の追記か。
    #[must_use]
    pub const fn is_undo(&self) -> bool {
        self.undo
    }
}

fn change_of(before: &Item, kind: &OpKind) -> HistoryChange {
    match kind {
        OpKind::Create(_) => HistoryChange::Created,
        OpKind::Delete(_) => HistoryChange::Deleted,
        OpKind::Restore(_) => HistoryChange::Restored,
        OpKind::Set { value, .. } => HistoryChange::Set {
            field: value.field(),
            before: before.value_of(value.field()),
            after: value.clone(),
        },
    }
}

/// 操作の種類を課題に当てる（`Docket::applied` の 1 件版）。
fn applied(item: Item, kind: &OpKind) -> Item {
    match kind {
        OpKind::Create(_) => item,
        OpKind::Set { value, .. } => item.with(value.clone()),
        OpKind::Delete(_) => item.with_deleted(true),
        OpKind::Restore(_) => item.with_deleted(false),
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::field_value::FieldValue;
    use nene_docket_core::item_state::ItemState;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_id::OpId;

    use super::*;

    const A: AccountId = AccountId::new(0xAB00);
    const B: AccountId = AccountId::new(0xCD00);

    fn op(clock: u64, account: AccountId, kind: OpKind) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), account),
            WallClock::new(60 * clock.cast_signed()),
            kind,
        )
    }

    /// A が作って題を置き、B が状態を変え、A が題を変えて戻し、消す。別の課題も 1 つ。
    fn sample_ops(item: ItemId) -> [Op; 7] {
        let other = ItemId::new(A, 2);
        let title = |text: &str| FieldValue::Title(String::from(text));
        [
            op(1, A, OpKind::Create(item)),
            op(
                2,
                A,
                OpKind::Set {
                    item,
                    value: title("a"),
                },
            ),
            op(
                3,
                B,
                OpKind::Set {
                    item,
                    value: FieldValue::State(ItemState::Done),
                },
            ),
            op(
                4,
                A,
                OpKind::Set {
                    item,
                    value: title("b"),
                },
            ),
            op(
                5,
                A,
                OpKind::Set {
                    item,
                    value: title("a"),
                },
            )
            .undoing(OpId::new(LogicalClock::new(4), A)),
            op(6, A, OpKind::Delete(item)),
            op(7, A, OpKind::Create(other)),
        ]
    }

    #[test]
    fn lines_show_from_and_to_newest_first_and_mark_undo() {
        let item = ItemId::new(A, 1);
        let ops = sample_ops(item);
        let lines = HistoryLine::lines(&ops, item);
        let columns: Vec<[String; 3]> = lines
            .iter()
            .map(|line| line.columns(Language::English))
            .collect();
        assert_eq!(columns.len(), 6);
        assert_eq!(columns[0], ["1970-01-01 00:06", "0000", "deleted"]);
        assert_eq!(columns[1][2], "undo: Title : b -> a");
        assert!(lines[1].is_undo());
        assert_eq!(columns[2][2], "Title : a -> b");
        assert_eq!(
            columns[3],
            ["1970-01-01 00:03", "0000", "State : Open -> Done"]
        );
        assert_eq!(columns[5][2], "created");
        assert_eq!(lines[5].when(), WallClock::new(60));
        assert_eq!(
            lines[1].columns(Language::Japanese)[2],
            "取り消し: 題名 : b -> a"
        );
    }
}
