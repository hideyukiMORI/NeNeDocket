//! 台帳の状態の所有者（ARC-004）。見た操作の列・併合した台帳・論理時計・Undo の束。
//!
//! 正本はジャーナルで、ここは派生値（ADR 0004 の 6）。自分の操作は [`Ledger::compose`] が時計を進めて作り、
//! 呼び手がジャーナルへ追記する。他人の操作は [`Ledger::absorb`] で取り込み、台帳を併合し直す。

use alloc::vec::Vec;

use nene_docket_core::docket::Docket;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::logical_clock::LogicalClock;
use nene_docket_core::merge;
use nene_docket_core::op::Op;
use nene_docket_core::op_id::OpId;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::wall_clock::WallClock;

use crate::account::Account;
use crate::undo_stack::UndoStack;

/// 自分のアカウント・見た操作・併合した台帳・時計・Undo の束・次に作る課題の番号・
/// 台帳が変わった回数（画面が派生値のキャッシュを捨てる合図・FR-017）・全順序で最後に当てた操作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ledger {
    account: Account,
    ops: Vec<Op>,
    docket: Docket,
    clock: LogicalClock,
    undo: UndoStack,
    next_sequence: u32,
    generation: u64,
    last_applied: Option<OpId>,
}

impl Ledger {
    /// ジャーナルから読んだ操作（自分と他人、順不同）から始める。
    #[must_use]
    pub fn open(account: Account, ops: Vec<Op>) -> Self {
        let mut own: Vec<&Op> = ops
            .iter()
            .filter(|op| op.id().account() == account.id())
            .collect();
        own.sort_by_key(|op| op.id());
        let mut undo = UndoStack::empty();
        for op in &own {
            undo.record(op);
        }
        let clock = ops.iter().fold(LogicalClock::zero(), |clock, op| {
            clock.seen(op.id().clock())
        });
        let next_sequence = ops
            .iter()
            .filter_map(|op| op.kind().item())
            .filter(|item| item.creator() == account.id())
            .map(ItemId::sequence)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let last_applied = ops.iter().map(Op::id).max();
        Self {
            docket: merge::merge(&ops),
            account,
            ops,
            clock,
            undo,
            next_sequence,
            generation: 0,
            last_applied,
        }
    }

    /// 台帳が変わった回数。変わるたびに 1 増える（派生値のキャッシュの鍵）。
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// 他人の操作を取り込む。空なら何もしない。全部が全順序で最後に当てた操作より後なら、併合し直さずに順に当てる
    /// （読み直しの普段の形・FR-017）。そうでなければ併合し直す（同じ集合からは同じ台帳・ADR 0004 の 6）。
    pub fn absorb(&mut self, fresh: Vec<Op>) {
        if fresh.is_empty() {
            return;
        }
        let mut fresh = fresh;
        fresh.sort_by_key(Op::id);
        let appendable = self
            .last_applied
            .is_none_or(|last| fresh.first().is_some_and(|first| first.id() > last));
        for op in &fresh {
            self.clock = self.clock.seen(op.id().clock());
        }
        if appendable {
            for op in &fresh {
                self.docket = core::mem::replace(&mut self.docket, Docket::empty()).applied(op);
            }
            self.last_applied = fresh.last().map(Op::id);
            self.ops.extend(fresh);
        } else {
            self.ops.extend(fresh);
            self.docket = merge::merge(&self.ops);
            self.last_applied = self.ops.iter().map(Op::id).max();
        }
        self.generation = self.generation.saturating_add(1);
    }

    /// 自分の操作を 1 つ作り、台帳に当て、Undo の束に積む。呼び手がジャーナルへ追記する。
    pub fn compose(&mut self, now: WallClock, kind: OpKind) -> Op {
        let op = Op::new(self.mint(), now, kind);
        self.push_own(op.clone());
        op
    }

    /// 直前の自分の操作を戻す操作。戻すものが無ければ `None`。
    pub fn compose_undo(&mut self, now: WallClock) -> Option<Op> {
        let target = self.undo.next_undo()?;
        self.compose_reversal(now, target)
    }

    /// 戻した操作をやり直す操作。やり直すものが無ければ `None`。
    pub fn compose_redo(&mut self, now: WallClock) -> Option<Op> {
        let target = self.undo.next_redo()?;
        self.compose_reversal(now, target)
    }

    fn compose_reversal(&mut self, now: WallClock, target: OpId) -> Option<Op> {
        let kind = merge::inverse_of(&self.ops, target)?;
        let op = Op::new(self.mint(), now, kind).undoing(target);
        self.push_own(op.clone());
        Some(op)
    }

    /// 新しい課題の ID。自分が作った課題の番号は飛ばさず増える。
    pub fn fresh_item_id(&mut self) -> ItemId {
        let id = ItemId::new(self.account.id(), self.next_sequence);
        self.next_sequence = self.next_sequence.saturating_add(1);
        id
    }

    /// 見た最大の時計より 1 大きい ID。
    fn mint(&mut self) -> OpId {
        self.clock = self.clock.next();
        OpId::new(self.clock, self.account.id())
    }

    /// 自分の新しい操作は全順序の末尾なので、併合し直さずに当てる。
    fn push_own(&mut self, op: Op) {
        self.docket = core::mem::replace(&mut self.docket, Docket::empty()).applied(&op);
        self.undo.record(&op);
        self.last_applied = Some(op.id());
        self.ops.push(op);
        self.generation = self.generation.saturating_add(1);
    }

    /// 併合した台帳。
    #[must_use]
    pub const fn docket(&self) -> &Docket {
        &self.docket
    }

    /// 自分のアカウント。
    #[must_use]
    pub const fn account(&self) -> &Account {
        &self.account
    }

    /// 見た操作（届いた順）。
    #[must_use]
    pub fn ops(&self) -> &[Op] {
        &self.ops
    }

    /// 戻せる自分の操作があるか。
    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.undo.next_undo().is_some()
    }

    /// やり直せる操作があるか。
    #[must_use]
    pub fn can_redo(&self) -> bool {
        self.undo.next_redo().is_some()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::field_value::FieldValue;
    use nene_docket_core::item::Item;
    use nene_docket_core::item_state::ItemState;
    use nene_docket_core::signing_key::SigningKey;

    use super::*;

    const ME: AccountId = AccountId::new(1);
    const OTHER: AccountId = AccountId::new(2);
    /// ID が小さいので、同じ時計では自分より前に並ぶ。
    const EARLIER: AccountId = AccountId::new(0);

    fn me() -> Account {
        Account::new(ME, String::from("me"), SigningKey::from_bytes(&[1_u8; 32]))
    }

    fn ledger() -> Ledger {
        Ledger::open(me(), Vec::new())
    }

    fn set_title(ledger: &mut Ledger, item: ItemId, title: &str) -> Op {
        ledger.compose(
            WallClock::new(0),
            OpKind::Set {
                item,
                value: FieldValue::Title(String::from(title)),
            },
        )
    }

    #[test]
    fn absorbing_later_ops_applies_them_in_place_and_earlier_ops_remerge_to_the_same_docket() {
        let account = me();
        let item = ItemId::new(OTHER, 1);
        let later = |clock: u64, kind: OpKind| {
            Op::new(
                OpId::new(LogicalClock::new(clock), OTHER),
                WallClock::new(0),
                kind,
            )
        };
        let mut ledger = Ledger::open(account.clone(), alloc::vec![later(1, OpKind::Create(item))]);
        assert_eq!(ledger.generation(), 0);
        // 後の操作: 併合し直さずに当てる（世代が進み、台帳は同じ）。
        ledger.absorb(alloc::vec![later(
            3,
            OpKind::Set {
                item,
                value: FieldValue::Title(String::from("b")),
            },
        )]);
        assert_eq!(ledger.generation(), 1);
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("b"));
        // 前の操作（時計 2）が後から届く: 全順序では 3 の前なので題名は b のまま。併合し直した台帳と同じ。
        ledger.absorb(alloc::vec![later(
            2,
            OpKind::Set {
                item,
                value: FieldValue::Title(String::from("a")),
            },
        )]);
        assert_eq!(ledger.generation(), 2);
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("b"));
        assert_eq!(ledger.docket(), &merge::merge(ledger.ops()));
        assert_eq!(ledger.ops().len(), 3);
        ledger.absorb(Vec::new());
        assert_eq!(ledger.generation(), 2);
        ledger.compose(WallClock::new(0), OpKind::Delete(item));
        assert_eq!(ledger.generation(), 3);
    }

    #[test]
    fn composing_applies_locally_and_undo_redo_walk_the_own_ops() {
        let mut ledger = ledger();
        let item = ledger.fresh_item_id();
        assert_eq!(item, ItemId::new(ME, 1));
        ledger.compose(WallClock::new(0), OpKind::Create(item));
        set_title(&mut ledger, item, "a");
        set_title(&mut ledger, item, "b");
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("b"));
        let undo = ledger.compose_undo(WallClock::new(1));
        assert!(undo.is_some_and(|op| op.undoes().is_some()));
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("a"));
        assert!(ledger.can_redo());
        assert!(ledger.compose_redo(WallClock::new(2)).is_some());
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("b"));
        assert!(!ledger.can_redo());
        assert_eq!(ledger.ops().len(), 5);
    }

    #[test]
    fn undo_of_a_create_deletes_and_nothing_to_undo_gives_none() {
        let mut ledger = ledger();
        assert_eq!(ledger.compose_undo(WallClock::new(0)), None);
        assert_eq!(ledger.compose_redo(WallClock::new(0)), None);
        let item = ledger.fresh_item_id();
        ledger.compose(WallClock::new(0), OpKind::Create(item));
        ledger.compose_undo(WallClock::new(0));
        assert!(ledger.docket().live_items().is_empty());
        assert!(!ledger.can_undo());
    }

    #[test]
    fn absorbing_others_merges_and_advances_the_clock() {
        let mut ledger = ledger();
        let theirs = ItemId::new(OTHER, 1);
        let create = Op::new(
            OpId::new(LogicalClock::new(10), OTHER),
            WallClock::new(0),
            OpKind::Create(theirs),
        );
        ledger.absorb(alloc::vec![create.clone()]);
        ledger.absorb(Vec::new());
        assert_eq!(ledger.docket().len(), 1);
        let mine = set_title(&mut ledger, theirs, "shared");
        assert_eq!(mine.id().clock(), LogicalClock::new(11));
        let late = Op::new(
            OpId::new(LogicalClock::new(11), EARLIER),
            WallClock::new(0),
            OpKind::Set {
                item: theirs,
                value: FieldValue::State(ItemState::Done),
            },
        );
        ledger.absorb(alloc::vec![late]);
        let item = ledger.docket().item(theirs);
        assert_eq!(item.map(Item::title), Some("shared"));
        assert_eq!(item.map(Item::state), Some(ItemState::Done));
        assert_eq!(ledger.ops().len(), 3);
    }

    #[test]
    fn undo_skips_others_ops_and_restores_every_kind_of_field() {
        let mut ledger = ledger();
        let item = ledger.fresh_item_id();
        ledger.compose(WallClock::new(0), OpKind::Create(item));
        ledger.compose(
            WallClock::new(0),
            OpKind::Set {
                item,
                value: FieldValue::Labels(alloc::vec![String::from("a")]),
            },
        );
        ledger.compose(
            WallClock::new(0),
            OpKind::Set {
                item,
                value: FieldValue::Body(String::from("two\nlines")),
            },
        );
        // 他人が間に題名を変える。自分の Undo はそれを飛ばす。
        ledger.absorb(alloc::vec![Op::new(
            OpId::new(LogicalClock::new(10), OTHER),
            WallClock::new(0),
            OpKind::Set {
                item,
                value: FieldValue::Title(String::from("theirs")),
            },
        )]);
        assert!(ledger.compose_undo(WallClock::new(0)).is_some());
        let after_one = ledger.docket().item(item).cloned().unwrap();
        assert_eq!(after_one.body(), "");
        assert_eq!(after_one.title(), "theirs");
        assert!(ledger.compose_undo(WallClock::new(0)).is_some());
        let after_two = ledger.docket().item(item).cloned().unwrap();
        assert!(after_two.labels().is_empty());
        assert_eq!(after_two.title(), "theirs");
    }

    #[test]
    fn redo_revives_the_op_and_a_fresh_op_clears_redo() {
        let mut ledger = ledger();
        let item = ledger.fresh_item_id();
        ledger.compose(WallClock::new(0), OpKind::Create(item));
        ledger.compose(
            WallClock::new(0),
            OpKind::Set {
                item,
                value: FieldValue::Labels(alloc::vec![String::from("a")]),
            },
        );
        assert!(ledger.compose_undo(WallClock::new(0)).is_some());
        assert!(ledger.compose_redo(WallClock::new(0)).is_some());
        assert_eq!(
            ledger
                .docket()
                .item(item)
                .map(|found| found.labels().to_vec()),
            Some(alloc::vec![String::from("a")])
        );
        // 新しい操作をすると Redo は消える。
        set_title(&mut ledger, item, "fresh");
        assert!(!ledger.can_redo());
        assert!(ledger.can_undo());
    }

    #[test]
    fn reopening_from_the_ops_restores_clock_sequence_and_undo() {
        let mut first = ledger();
        let item = first.fresh_item_id();
        first.compose(WallClock::new(0), OpKind::Create(item));
        set_title(&mut first, item, "x");
        first.compose_undo(WallClock::new(0));
        let mut again = Ledger::open(me(), first.ops().to_vec());
        assert_eq!(again.fresh_item_id(), ItemId::new(ME, 2));
        assert!(again.can_redo());
        assert_eq!(again.account().name(), "me");
        let redo = again.compose_redo(WallClock::new(0));
        assert_eq!(redo.map(|op| op.id().clock()), Some(LogicalClock::new(4)));
    }
}
