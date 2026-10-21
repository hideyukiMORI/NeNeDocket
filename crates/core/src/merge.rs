//! 併合。全アカウントのジャーナルの操作から、いまの台帳を作る純関数（ADR 0004 の 6）。
//!
//! 入力の順に依らず、同じ操作の集合からは同じ台帳が出る。時刻も乱数も読まない（ARC-007）。
//! 同じ操作が 2 度来ても（読み直しの重なり）1 度だけ数える。権限の記録（`Admin`）は台帳を変えない（ADR 0014）。

use alloc::vec::Vec;

use crate::attachment::Attachment;
use crate::docket::Docket;
use crate::field::Field;
use crate::field_value::FieldValue;
use crate::item::Item;
use crate::item_id::ItemId;
use crate::op::Op;
use crate::op_id::OpId;
use crate::op_kind::OpKind;

/// 操作を（論理時計・アカウント）の全順序に並べ、重複を落とし、順に適用する。
#[must_use]
pub fn merge<'a, I>(ops: I) -> Docket
where
    I: IntoIterator<Item = &'a Op>,
{
    ordered(ops)
        .into_iter()
        .fold(Docket::empty(), Docket::applied)
}

/// 課題 1 件に触れた操作を全順序で。負けた操作も残る（FR-006）。
#[must_use]
pub fn history<'a, I>(ops: I, item: ItemId) -> Vec<&'a Op>
where
    I: IntoIterator<Item = &'a Op>,
{
    ordered(ops)
        .into_iter()
        .filter(|op| op.kind().item() == Some(item))
        .collect()
}

/// 全部の操作を全順序で（台帳全体の時系列・監査・ADR 0014）。重複は落とす。
#[must_use]
pub fn chronicle<'a, I>(ops: I) -> Vec<&'a Op>
where
    I: IntoIterator<Item = &'a Op>,
{
    ordered(ops)
}

/// `target` を打ち消す操作の種類。`Create` は `Delete`、`Delete` は `Restore`、`Restore` は `Delete`、
/// `Set` はその直前に同じ項目が持っていた値を置く `Set`。`target` が無いか権限の記録なら `None`（ADR 0004 の 4）。
#[must_use]
pub fn inverse_of<'a, I>(ops: I, target: OpId) -> Option<OpKind>
where
    I: IntoIterator<Item = &'a Op>,
{
    let all = ordered(ops);
    let found = all.iter().find(|op| op.id() == target)?;
    Some(match found.kind() {
        OpKind::Create(item) | OpKind::Restore(item) => OpKind::Delete(*item),
        OpKind::Delete(item) => OpKind::Restore(*item),
        OpKind::Set { item, value } => OpKind::Set {
            item: *item,
            value: value_before(&all, target, *item, value.field()),
        },
        OpKind::Attach { item, attachment } => OpKind::Detach {
            item: *item,
            sequence: attachment.sequence(),
        },
        OpKind::Detach { item, sequence } => OpKind::Attach {
            item: *item,
            attachment: attachment_before(&all, target, *item, *sequence)?,
        },
        OpKind::Admin(_) => return None,
    })
}

/// 全順序で `target` より前に、同じ課題の同じ連番で最後に添付したもの。無ければ `None`（戻せない）。
fn attachment_before(
    ordered: &[&Op],
    target: OpId,
    item: ItemId,
    sequence: u32,
) -> Option<Attachment> {
    ordered
        .iter()
        .take_while(|op| op.id() < target)
        .filter_map(|op| match op.kind() {
            OpKind::Attach {
                item: attached,
                attachment,
            } if *attached == item && attachment.sequence() == sequence => Some(attachment.clone()),
            OpKind::Attach { .. }
            | OpKind::Detach { .. }
            | OpKind::Set { .. }
            | OpKind::Create(_)
            | OpKind::Delete(_)
            | OpKind::Restore(_)
            | OpKind::Admin(_) => None,
        })
        .last()
}

/// 全順序で `target` より前に、同じ課題の同じ項目へ最後に置かれた値。無ければ作った直後の値。
fn value_before(ordered: &[&Op], target: OpId, item: ItemId, field: Field) -> FieldValue {
    ordered
        .iter()
        .take_while(|op| op.id() < target)
        .filter_map(|op| match op.kind() {
            OpKind::Set {
                item: set_item,
                value,
            } if *set_item == item && value.field() == field => Some(value.clone()),
            OpKind::Set { .. }
            | OpKind::Create(_)
            | OpKind::Delete(_)
            | OpKind::Restore(_)
            | OpKind::Admin(_)
            | OpKind::Attach { .. }
            | OpKind::Detach { .. } => None,
        })
        .last()
        .unwrap_or_else(|| Item::created(item).value_of(field))
}

/// 全順序に並べ、同じ ID の操作は最初の 1 つだけ残す。
fn ordered<'a, I>(ops: I) -> Vec<&'a Op>
where
    I: IntoIterator<Item = &'a Op>,
{
    let mut sorted: Vec<&'a Op> = ops.into_iter().collect();
    sorted.sort_by_key(|op| op.id());
    let mut last: Option<OpId> = None;
    sorted.retain(|op| {
        let keep = last != Some(op.id());
        last = Some(op.id());
        keep
    });
    sorted
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::*;
    use crate::account_id::AccountId;
    use crate::field_value::FieldValue;
    use crate::item_state::ItemState;
    use crate::logical_clock::LogicalClock;
    use crate::op_kind::OpKind;
    use crate::wall_clock::WallClock;

    const A: AccountId = AccountId::new(1);
    const B: AccountId = AccountId::new(2);

    fn op(clock: u64, account: AccountId, kind: OpKind) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), account),
            WallClock::new(0),
            kind,
        )
    }

    fn title(text: &str) -> FieldValue {
        FieldValue::Title(String::from(text))
    }

    /// A が課題を作って題を置き、B が同時に題と状態を変え、A が消して戻す。
    fn two_accounts() -> Vec<Op> {
        let item = ItemId::new(A, 1);
        alloc::vec![
            op(1, A, OpKind::Create(item)),
            op(
                2,
                A,
                OpKind::Set {
                    item,
                    value: title("first")
                }
            ),
            op(
                3,
                B,
                OpKind::Set {
                    item,
                    value: title("from b")
                }
            ),
            op(
                3,
                A,
                OpKind::Set {
                    item,
                    value: title("from a")
                }
            ),
            op(
                4,
                B,
                OpKind::Set {
                    item,
                    value: FieldValue::State(ItemState::Done)
                }
            ),
            op(5, A, OpKind::Delete(item)),
            op(6, A, OpKind::Restore(item)),
        ]
    }

    #[test]
    fn admin_records_change_nothing_are_not_invertible_and_the_chronicle_keeps_every_op_once() {
        let item = ItemId::new(A, 1);
        let record = op(
            2,
            B,
            OpKind::Admin(crate::admin_command::AdminCommand::Permissions(true)),
        );
        let ops = [
            op(1, A, OpKind::Create(item)),
            record.clone(),
            record.clone(),
        ];
        let docket = merge(&ops);
        assert_eq!(docket.len(), 1);
        assert_eq!(docket.orphans(), 0);
        assert_eq!(merge(ops.iter().rev()), docket);
        assert_eq!(inverse_of(&ops, record.id()), None);
        assert!(
            history(&ops, item)
                .iter()
                .all(|op| op.kind().item() == Some(item))
        );
        let all = chronicle(ops.iter().rev());
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].id(), record.id());
    }

    #[test]
    fn attach_and_detach_invert_each_other_and_a_detach_without_an_attach_is_not_invertible() {
        let item = ItemId::new(A, 1);
        let attachment = Attachment::new(1, String::from("a.txt"), 3, [9_u8; 32]);
        let ops = [
            op(1, A, OpKind::Create(item)),
            op(
                2,
                A,
                OpKind::Attach {
                    item,
                    attachment: attachment.clone(),
                },
            ),
            op(3, A, OpKind::Detach { item, sequence: 1 }),
            op(4, B, OpKind::Detach { item, sequence: 7 }),
        ];
        let docket = merge(&ops);
        assert!(docket.item(item).unwrap().attachments().is_empty());
        assert_eq!(docket.item(item).unwrap().next_attachment(), 2);
        assert_eq!(
            inverse_of(&ops, ops[1].id()),
            Some(OpKind::Detach { item, sequence: 1 })
        );
        assert_eq!(
            inverse_of(&ops, ops[2].id()),
            Some(OpKind::Attach { item, attachment })
        );
        assert_eq!(inverse_of(&ops, ops[3].id()), None);
        assert_eq!(merge(ops.iter().rev()), docket);
    }

    #[test]
    fn the_same_ops_in_any_order_give_the_same_docket() {
        let ops = two_accounts();
        let expected = merge(&ops);
        let mut reversed = ops.clone();
        reversed.reverse();
        assert_eq!(merge(&reversed), expected);
        let mut rotated = ops.clone();
        rotated.rotate_left(3);
        assert_eq!(merge(&rotated), expected);
        let (a_ops, b_ops): (Vec<Op>, Vec<Op>) =
            ops.iter().cloned().partition(|op| op.id().account() == A);
        let by_account: Vec<Op> = b_ops.iter().chain(a_ops.iter()).cloned().collect();
        assert_eq!(merge(&by_account), expected);
        let scattered: Vec<&Op> = [5_usize, 0, 6, 2, 4, 1, 3]
            .iter()
            .filter_map(|&i| ops.get(i))
            .collect();
        assert_eq!(merge(scattered), expected);
    }

    #[test]
    fn wall_clock_skew_between_machines_does_not_change_the_docket() {
        let ops = two_accounts();
        let expected = merge(&ops);
        // B の時計が 1 時間進んでいる（壁時計は表示にだけ・ADR 0004）。
        let skewed: Vec<Op> = ops
            .iter()
            .map(|op| {
                let shift = if op.id().account() == A {
                    0_i64
                } else {
                    3_600_i64
                };
                let moved = Op::new(
                    op.id(),
                    WallClock::new(op.wall().seconds() + shift),
                    op.kind().clone(),
                );
                match op.undoes() {
                    Some(target) => moved.undoing(target),
                    None => moved,
                }
            })
            .collect();
        assert_eq!(merge(&skewed), expected);
        let mut reversed = skewed.clone();
        reversed.reverse();
        assert_eq!(merge(&reversed), expected);
    }

    #[test]
    fn the_last_op_in_the_total_order_wins_and_losers_stay_in_the_history() {
        let ops = two_accounts();
        let item = ItemId::new(A, 1);
        let docket = merge(&ops);
        let found = docket.item(item).unwrap();
        // 時計 3 は A と B が同時。アカウントの番号が大きい B が後なので B が勝つ。
        assert_eq!(found.title(), "from b");
        assert_eq!(found.state(), ItemState::Done);
        assert!(!found.is_deleted());
        let history = history(&ops, item);
        assert_eq!(history.len(), 7);
        assert_eq!(history.get(2).map(|op| op.id().account()), Some(A));
        assert_eq!(history.get(3).map(|op| op.id().account()), Some(B));
        assert_eq!(docket.live_items().len(), 1);
        assert_eq!(docket.len(), 1);
    }

    #[test]
    fn duplicates_are_counted_once_and_orphans_are_counted_not_dropped() {
        let ops = two_accounts();
        let twice: Vec<&Op> = ops.iter().chain(ops.iter()).collect();
        assert_eq!(merge(twice), merge(&ops));
        let stray = ItemId::new(B, 9);
        let with_orphans = alloc::vec![
            op(
                1,
                B,
                OpKind::Set {
                    item: stray,
                    value: title("nobody")
                }
            ),
            op(2, B, OpKind::Delete(stray)),
            op(3, B, OpKind::Create(stray)),
            op(4, B, OpKind::Restore(stray)),
        ];
        let docket = merge(&with_orphans);
        assert_eq!(docket.orphans(), 2);
        assert_eq!(docket.item(stray).map(Item::title), Some(""));
        assert!(!docket.is_empty());
    }

    #[test]
    fn a_second_create_of_the_same_item_keeps_the_first_and_deleted_items_leave_the_live_list() {
        let item = ItemId::new(A, 1);
        let ops = alloc::vec![
            op(1, A, OpKind::Create(item)),
            op(
                2,
                A,
                OpKind::Set {
                    item,
                    value: title("kept")
                }
            ),
            op(3, B, OpKind::Create(item)),
            op(4, A, OpKind::Delete(item)),
        ];
        let docket = merge(&ops);
        assert_eq!(docket.item(item).map(Item::title), Some("kept"));
        assert!(docket.live_items().is_empty());
        assert_eq!(docket.items_in_creation_order().len(), 1);
        assert_eq!(Docket::empty().len(), 0);
    }

    #[test]
    fn inverse_restores_the_previous_value_and_flips_create_delete_restore() {
        let ops = two_accounts();
        let item = ItemId::new(A, 1);
        let id = |clock: u64, account: AccountId| OpId::new(LogicalClock::new(clock), account);
        assert_eq!(inverse_of(&ops, id(1, A)), Some(OpKind::Delete(item)));
        assert_eq!(inverse_of(&ops, id(5, A)), Some(OpKind::Restore(item)));
        assert_eq!(inverse_of(&ops, id(6, A)), Some(OpKind::Delete(item)));
        assert_eq!(
            inverse_of(&ops, id(3, B)),
            Some(OpKind::Set {
                item,
                value: title("from a")
            })
        );
        assert_eq!(
            inverse_of(&ops, id(2, A)),
            Some(OpKind::Set {
                item,
                value: title("")
            })
        );
        assert_eq!(
            inverse_of(&ops, id(4, B)),
            Some(OpKind::Set {
                item,
                value: FieldValue::State(ItemState::Open)
            })
        );
        assert_eq!(inverse_of(&ops, id(9, B)), None);
    }
}
