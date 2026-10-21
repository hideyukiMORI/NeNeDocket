//! 課題 1 件の履歴の 1 行（FR-006）。いつ・誰が・何を・何から何へ。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::field_value::FieldValue;
use nene_docket_core::item::Item;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::merge;
use nene_docket_core::op::Op;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::wall_clock::WallClock;

/// 「誰が」に出すアカウントの ID の桁数。名前はジャーナルに無い。
const WHO_DIGITS: usize = 4;

/// 壁時計・アカウント・Undo の追記か・変更の文言。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryLine {
    when: WallClock,
    who: AccountId,
    undo: bool,
    change: String,
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
                let change = Self::change_of(&current, op.kind());
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

    fn change_of(before: &Item, kind: &OpKind) -> String {
        match kind {
            OpKind::Create(_) => String::from("created"),
            OpKind::Delete(_) => String::from("deleted"),
            OpKind::Restore(_) => String::from("restored"),
            OpKind::Set { value, .. } => {
                let field = value.field();
                let old = Self::value_text(&before.value_of(field));
                let new = Self::value_text(value);
                format!("{} : {old} -> {new}", field.keyword())
            }
        }
    }

    fn value_text(value: &FieldValue) -> String {
        match value {
            FieldValue::Title(text) | FieldValue::Body(text) | FieldValue::Assignee(text) => {
                text.clone()
            }
            FieldValue::State(state) => String::from(state.label()),
            FieldValue::Due(due) => due
                .map(nene_docket_core::date::Date::text)
                .unwrap_or_default(),
            FieldValue::Labels(labels) => labels.join(", "),
            FieldValue::Rank(rank) => format!("{rank}"),
        }
    }

    /// 表の 3 列: いつ（UTC）・誰が（ID の先頭）・何を。Undo の追記は `undo: ` を頭に付ける。
    #[must_use]
    pub fn columns(&self) -> [String; 3] {
        let who: String = self.who.hex().chars().take(WHO_DIGITS).collect();
        let change = if self.undo {
            format!("undo: {}", self.change)
        } else {
            self.change.clone()
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

    #[test]
    fn lines_show_from_and_to_newest_first_and_mark_undo() {
        let item = ItemId::new(A, 1);
        let other = ItemId::new(A, 2);
        let title = |text: &str| FieldValue::Title(String::from(text));
        let ops = [
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
        ];
        let lines = HistoryLine::lines(&ops, item);
        let columns: Vec<[String; 3]> = lines.iter().map(HistoryLine::columns).collect();
        assert_eq!(columns.len(), 6);
        assert_eq!(columns[0], ["1970-01-01 00:06", "0000", "deleted"]);
        assert_eq!(columns[1][2], "undo: title : b -> a");
        assert!(lines[1].is_undo());
        assert_eq!(columns[2][2], "title : a -> b");
        assert_eq!(
            columns[3],
            ["1970-01-01 00:03", "0000", "state : Open -> Done"]
        );
        assert_eq!(columns[5][2], "created");
        assert_eq!(lines[5].when(), WallClock::new(60));
    }

    #[test]
    fn every_value_kind_has_a_text() {
        let item = ItemId::new(A, 1);
        let values = [
            FieldValue::Body(String::from("body")),
            FieldValue::Assignee(String::from("rina")),
            FieldValue::Due(nene_docket_core::date::Date::new(2026, 1, 2)),
            FieldValue::Due(None),
            FieldValue::Labels(alloc::vec![String::from("x"), String::from("y")]),
            FieldValue::Rank(-3),
        ];
        let ops: Vec<Op> = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                op(
                    u64::try_from(index).unwrap() + 1,
                    A,
                    OpKind::Set { item, value },
                )
            })
            .collect();
        let texts: Vec<String> = HistoryLine::lines(&ops, item)
            .iter()
            .map(|line| line.columns()[2].clone())
            .collect();
        assert_eq!(texts[5], "body :  -> body");
        assert_eq!(texts[4], "assignee :  -> rina");
        assert_eq!(texts[3], "due :  -> 2026-01-02");
        assert_eq!(texts[2], "due : 2026-01-02 -> ");
        assert_eq!(texts[1], "labels :  -> x, y");
        assert_eq!(texts[0], "rank : 0 -> -3");
    }
}
