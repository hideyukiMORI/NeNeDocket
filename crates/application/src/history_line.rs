//! 課題 1 件の履歴の 1 行（FR-006）と、台帳全体の時系列の 1 行（監査・ADR 0014）。いつ・誰が・何を・何から何へ。

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::admin_command::AdminCommand;
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
use crate::item_number;

/// 名前が分からないときに「誰が」に出すアカウントの ID の桁数。
const WHO_DIGITS: usize = 4;

/// 壁時計・誰が（`keys/` の名前、無ければ ID の先頭）・Undo の追記か・変更・その直後の課題（再現・FR-006。
/// 台帳全体の行と権限の記録には無い）・台帳全体の行なら課題の番号の頭書き。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryLine {
    when: WallClock,
    who: String,
    undo: bool,
    change: HistoryChange,
    snapshot: Option<Item>,
    prefix: String,
}

impl HistoryLine {
    /// 課題に触れた操作を全順序に並べ、値を追いながら 1 行ずつにする。新しいものが先。
    /// 「誰が」は `names` にあれば名前、無ければ ID の先頭 4 桁。
    #[must_use]
    pub fn lines<'a, I>(ops: I, item: ItemId, names: &[(AccountId, String)]) -> Vec<Self>
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
                    who: who_of(op.id().account(), names),
                    undo: op.undoes().is_some(),
                    change,
                    snapshot: Some(current.clone()),
                    prefix: String::new(),
                }
            })
            .collect();
        lines.reverse();
        lines
    }

    /// 台帳全体の時系列（監査・ADR 0014）。全部の操作を全順序に並べ、課題ごとに値を追う。新しいものが先。
    /// 課題の操作は番号を頭に付け、権限の記録は相手を `names` の名前で出す。再現の課題は持たない。
    #[must_use]
    pub fn all<'a, I>(ops: I, names: &[(AccountId, String)]) -> Vec<Self>
    where
        I: IntoIterator<Item = &'a Op>,
    {
        let mut items: BTreeMap<ItemId, Item> = BTreeMap::new();
        let mut lines: Vec<Self> = merge::chronicle(ops)
            .into_iter()
            .map(|op| {
                let (change, prefix) = match (op.kind(), op.kind().item()) {
                    (OpKind::Admin(command), _) => (
                        HistoryChange::Administered {
                            command: command.clone(),
                            subject: subject_of(command, names),
                        },
                        String::new(),
                    ),
                    (kind, Some(item)) => {
                        let current = items.remove(&item).unwrap_or_else(|| Item::created(item));
                        let change = change_of(&current, kind);
                        items.insert(item, applied(current, kind));
                        (change, format!("{}: ", item_number::text(item)))
                    }
                    (
                        OpKind::Create(_)
                        | OpKind::Set { .. }
                        | OpKind::Delete(_)
                        | OpKind::Restore(_),
                        None,
                    ) => (HistoryChange::Created, String::new()),
                };
                Self {
                    when: op.wall(),
                    who: who_of(op.id().account(), names),
                    undo: op.undoes().is_some(),
                    change,
                    snapshot: None,
                    prefix,
                }
            })
            .collect();
        lines.reverse();
        lines
    }

    /// 表の 3 列: いつ（UTC）・誰が・何を。台帳全体の行は課題の番号を、Undo の追記は表示言語の印を頭に付ける。
    #[must_use]
    pub fn columns(&self, language: Language) -> [String; 3] {
        let who = self.who.clone();
        let mark = if self.undo {
            wording::text(language, Word::UndoMark)
        } else {
            ""
        };
        let change = format!("{}{mark}{}", self.prefix, self.change.describe(language));
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

    /// 何を（`Set` なら項目と前後の値。選び直しに使う・FR-008）。
    #[must_use]
    pub const fn change(&self) -> &HistoryChange {
        &self.change
    }

    /// この操作の直後の課題（その時点の再現・FR-006）。台帳全体の行には無い。
    #[must_use]
    pub const fn snapshot(&self) -> Option<&Item> {
        self.snapshot.as_ref()
    }
}

/// 権限の命令の相手。名前は `keys/` から、無ければ ID の先頭。
fn subject_of(command: &AdminCommand, names: &[(AccountId, String)]) -> String {
    match command {
        AdminCommand::Permissions(_) => String::new(),
        AdminCommand::Allow(id) | AdminCommand::Disallow(id) | AdminCommand::Owner(id) => {
            who_of(*id, names)
        }
        AdminCommand::Group(name, id) | AdminCommand::Ungroup(name, id) => {
            format!("{name}: {}", who_of(*id, names))
        }
        AdminCommand::Succeed(new, old) => {
            format!("{} ← {}", who_of(*new, names), who_of(*old, names))
        }
    }
}

/// `keys/` にある名前、無ければ ID の先頭。
fn who_of(account: AccountId, names: &[(AccountId, String)]) -> String {
    names.iter().find(|(id, _)| *id == account).map_or_else(
        || account.hex().chars().take(WHO_DIGITS).collect(),
        |(_, name)| name.clone(),
    )
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
        OpKind::Admin(command) => HistoryChange::Administered {
            command: command.clone(),
            subject: String::new(),
        },
    }
}

/// 操作の種類を課題に当てる（`Docket::applied` の 1 件版）。
fn applied(item: Item, kind: &OpKind) -> Item {
    match kind {
        OpKind::Create(_) | OpKind::Admin(_) => item,
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
        let names = [(B, String::from("beth"))];
        let lines = HistoryLine::lines(&ops, item, &names);
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
            ["1970-01-01 00:03", "beth", "State : Open -> Done"]
        );
        assert_eq!(columns[5][2], "created");
    }

    /// 課題の操作の後に、B が権限を 4 回変える。
    fn ops_with_admin(item: ItemId) -> Vec<Op> {
        let mut ops = sample_ops(item).to_vec();
        ops.push(op(8, B, OpKind::Admin(AdminCommand::Allow(A))));
        ops.push(op(
            9,
            B,
            OpKind::Admin(AdminCommand::Group(String::from("qa"), A)),
        ));
        ops.push(op(10, B, OpKind::Admin(AdminCommand::Succeed(B, A))));
        ops.push(op(11, B, OpKind::Admin(AdminCommand::Permissions(false))));
        ops
    }

    fn changes_of(lines: &[HistoryLine]) -> Vec<String> {
        lines
            .iter()
            .map(|line| line.columns(Language::English)[2].clone())
            .collect()
    }

    #[test]
    fn all_lists_every_op_newest_first_with_item_numbers() {
        let item = ItemId::new(A, 1);
        let ops = ops_with_admin(item);
        let lines = HistoryLine::all(&ops, &[]);
        let changes = changes_of(&lines);
        assert_eq!(changes.len(), 11);
        assert_eq!(changes[4], "0000-2: created");
        assert_eq!(changes[5], "0000-1: deleted");
        assert_eq!(changes[6], "0000-1: undo: Title : b -> a");
        assert_eq!(changes[10], "0000-1: created");
        assert!(lines.iter().all(|line| line.snapshot().is_none()));
    }

    #[test]
    fn admin_records_name_their_subjects_from_the_keys() {
        let item = ItemId::new(A, 1);
        let ops = ops_with_admin(item);
        let names = [(A, String::from("ann")), (B, String::from("beth"))];
        let lines = HistoryLine::all(&ops, &names);
        let changes = changes_of(&lines);
        assert_eq!(changes[0], "turned permissions off");
        assert_eq!(changes[1], "succession: beth ← ann");
        assert_eq!(changes[2], "added to a group: qa: ann");
        assert_eq!(changes[3], "added an editor: ann");
        assert_eq!(lines[1].columns(Language::English)[1], "beth");
        assert!(matches!(
            lines[0].change(),
            HistoryChange::Administered { subject, .. } if subject.is_empty()
        ));
    }

    #[test]
    fn lines_carry_the_change_the_time_and_the_language() {
        let item = ItemId::new(A, 1);
        let ops = sample_ops(item);
        let lines = HistoryLine::lines(&ops, item, &[]);
        assert_eq!(lines[5].change(), &HistoryChange::Created);
        assert!(matches!(
            lines[2].change(),
            HistoryChange::Set { after, .. } if *after == FieldValue::Title(String::from("b"))
        ));
        assert_eq!(lines[5].when(), WallClock::new(60));
    }

    #[test]
    fn each_line_carries_the_item_as_it_was_right_after() {
        let item = ItemId::new(A, 1);
        let ops = sample_ops(item);
        let lines = HistoryLine::lines(&ops, item, &[]);
        assert_eq!(lines[5].snapshot().map(Item::title), Some(""));
        assert_eq!(lines[3].snapshot().map(Item::state), Some(ItemState::Done));
        assert_eq!(lines[2].snapshot().map(Item::title), Some("b"));
        assert!(lines[0].snapshot().is_some_and(Item::is_deleted));
        assert_eq!(
            lines[1].columns(Language::Japanese)[2],
            "取り消し: 題名 : b -> a"
        );
    }
}
