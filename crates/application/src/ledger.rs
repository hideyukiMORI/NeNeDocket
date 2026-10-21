//! 台帳の状態の所有者（ARC-004）。併合した台帳・論理時計・Undo の束。操作の列は持たない（ADR 0015 の 4）。
//!
//! 正本はジャーナルで、ここは派生値（ADR 0004 の 6）。自分の操作は [`Ledger::compose`] が時計を進めて作り、
//! 呼び手がジャーナルへ追記する。他人の操作は [`Ledger::absorb`] で取り込む。履歴と Undo の逆の値はジャーナルから引く。

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use nene_docket_core::docket::Docket;
use nene_docket_core::item::Item;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::logical_clock::LogicalClock;
use nene_docket_core::merge;
use nene_docket_core::op::Op;
use nene_docket_core::op_id::OpId;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::wall_clock::WallClock;

use crate::account::Account;
use crate::journal_failure::JournalFailure;
use crate::journal_port::JournalPort;
use crate::ledger_snapshot::LedgerSnapshot;
use crate::undo_stack::UndoStack;

/// 自分のアカウント・併合した台帳・時計・Undo の束・次に作る課題の番号・
/// 台帳が変わった回数（画面が派生値のキャッシュを捨てる合図・FR-017）・全順序で最後に当てた操作・
/// Undo の束にある自分の操作の課題（逆の値をジャーナルから引くため）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ledger {
    account: Account,
    docket: Docket,
    clock: LogicalClock,
    undo: UndoStack,
    next_sequence: u32,
    generation: u64,
    last_applied: Option<OpId>,
    own_items: BTreeMap<OpId, ItemId>,
}

impl Ledger {
    /// ジャーナルから読んだ操作（自分と他人、順不同）から始める（スナップショットが無いとき）。
    #[must_use]
    pub fn open(account: Account, ops: &[Op]) -> Self {
        let mut own: Vec<&Op> = ops
            .iter()
            .filter(|op| op.id().account() == account.id())
            .collect();
        own.sort_by_key(|op| op.id());
        let mut undo = UndoStack::empty();
        let mut own_items = BTreeMap::new();
        for op in &own {
            undo.record(op);
            if let Some(item) = op.kind().item() {
                own_items.insert(op.id(), item);
            }
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
        let mut ledger = Self {
            docket: merge::merge(ops),
            account,
            clock,
            undo,
            next_sequence,
            generation: 0,
            last_applied,
            own_items,
        };
        ledger.prune_own_items();
        ledger
    }

    /// スナップショットから戻す（ADR 0015 の 3）。伸びた分は呼び手が [`Ledger::absorb`] で当てる。
    #[must_use]
    pub fn resume(account: Account, snapshot: &LedgerSnapshot) -> Self {
        Self {
            account,
            docket: snapshot.docket().clone(),
            clock: snapshot.clock(),
            undo: UndoStack::from_state(snapshot.undo()),
            next_sequence: snapshot.next_sequence(),
            generation: 0,
            last_applied: snapshot.last_applied(),
            own_items: snapshot.own_items().iter().copied().collect(),
        }
    }

    /// いまの状態のスナップショット（store が書く）。
    #[must_use]
    pub fn snapshot(&self) -> LedgerSnapshot {
        LedgerSnapshot::new(
            (
                self.docket.clone(),
                self.clock,
                self.next_sequence,
                self.last_applied,
            ),
            self.undo.state(),
            self.own_items
                .iter()
                .map(|(id, item)| (*id, *item))
                .collect(),
        )
    }

    /// 台帳が変わった回数。変わるたびに 1 増える（派生値のキャッシュの鍵）。
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// ジャーナルから届いた操作を取り込む。空なら何もしない。全部が全順序で最後に当てた操作より後なら順に当て
    /// （読み直しの普段の形・FR-017）、そうでなければ触れた課題をその課題の全操作から作り直す（同じ集合からは同じ台帳・ADR 0004 の 6）。
    /// 自分の操作（前の回にスナップショットの後に書いた分）は Undo の束にも積む。
    ///
    /// # Errors
    ///
    /// 作り直しに要る課題の操作をジャーナルから読めない。
    pub fn absorb(
        &mut self,
        fresh: Vec<Op>,
        journal: &dyn JournalPort,
    ) -> Result<(), JournalFailure> {
        if fresh.is_empty() {
            return Ok(());
        }
        let mut fresh = fresh;
        fresh.sort_by_key(Op::id);
        let appendable = self
            .last_applied
            .is_none_or(|last| fresh.first().is_some_and(|first| first.id() > last));
        for op in &fresh {
            self.clock = self.clock.seen(op.id().clock());
            if op.id().account() == self.account.id() {
                self.note_own(op);
            }
        }
        if appendable {
            for op in &fresh {
                self.docket = core::mem::replace(&mut self.docket, Docket::empty()).applied(op);
            }
        } else {
            let mut touched: Vec<ItemId> = fresh.iter().filter_map(|op| op.kind().item()).collect();
            touched.sort_unstable();
            touched.dedup();
            for item in touched {
                self.rebuild_item(item, journal)?;
            }
        }
        self.last_applied = self.last_applied.max(fresh.last().map(Op::id));
        self.prune_own_items();
        self.generation = self.generation.saturating_add(1);
        Ok(())
    }

    /// 前の回に書いた自分の操作（スナップショットの後の分）を Undo の束に積み、課題と連番を覚える。
    fn note_own(&mut self, op: &Op) {
        self.undo.record(op);
        if let Some(item) = op.kind().item() {
            self.own_items.insert(op.id(), item);
            self.next_sequence = self.next_sequence.max(item.sequence().saturating_add(1));
        }
    }

    /// 課題 1 件を、その全操作（ジャーナルから）だけから作り直して置く。
    fn rebuild_item(
        &mut self,
        item: ItemId,
        journal: &dyn JournalPort,
    ) -> Result<(), JournalFailure> {
        let ops = journal.ops_of(item)?;
        let rebuilt = merge::history(&ops, item)
            .into_iter()
            .fold(Item::created(item), |current, op| {
                current.applied(op.kind())
            });
        self.docket = core::mem::replace(&mut self.docket, Docket::empty()).replaced(rebuilt);
        Ok(())
    }

    /// 自分の操作を 1 つ作り、台帳に当て、Undo の束に積む。呼び手がジャーナルへ追記する。
    pub fn compose(&mut self, now: WallClock, kind: OpKind) -> Op {
        let op = Op::new(self.mint(), now, kind);
        self.push_own(&op);
        op
    }

    /// 直前の自分の操作を戻す操作。戻すものが無ければ `None`。逆の値はその課題の操作をジャーナルから引いて決める。
    ///
    /// # Errors
    ///
    /// 課題の操作を読めない。
    pub fn compose_undo(
        &mut self,
        now: WallClock,
        journal: &dyn JournalPort,
    ) -> Result<Option<Op>, JournalFailure> {
        match self.undo.next_undo() {
            Some(target) => self.compose_reversal(now, target, journal),
            None => Ok(None),
        }
    }

    /// 戻した操作をやり直す操作。やり直すものが無ければ `None`。
    ///
    /// # Errors
    ///
    /// 課題の操作を読めない。
    pub fn compose_redo(
        &mut self,
        now: WallClock,
        journal: &dyn JournalPort,
    ) -> Result<Option<Op>, JournalFailure> {
        match self.undo.next_redo() {
            Some(target) => self.compose_reversal(now, target, journal),
            None => Ok(None),
        }
    }

    fn compose_reversal(
        &mut self,
        now: WallClock,
        target: OpId,
        journal: &dyn JournalPort,
    ) -> Result<Option<Op>, JournalFailure> {
        let Some(item) = self.own_items.get(&target).copied() else {
            return Ok(None);
        };
        let ops = journal.ops_of(item)?;
        let Some(kind) = merge::inverse_of(&ops, target) else {
            return Ok(None);
        };
        let op = Op::new(self.mint(), now, kind).undoing(target);
        self.push_own(&op);
        Ok(Some(op))
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
    fn push_own(&mut self, op: &Op) {
        self.docket = core::mem::replace(&mut self.docket, Docket::empty()).applied(op);
        self.undo.record(op);
        if let Some(item) = op.kind().item() {
            self.own_items.insert(op.id(), item);
        }
        self.last_applied = Some(op.id());
        self.generation = self.generation.saturating_add(1);
        self.prune_own_items();
    }

    /// Undo の束から消えた操作の課題は忘れる（束の上限と同じ量に抑える）。
    fn prune_own_items(&mut self) {
        let mentioned = self.undo.mentioned();
        self.own_items
            .retain(|id, _| mentioned.binary_search(id).is_ok());
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
    use core::cell::RefCell;

    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::admin_command::AdminCommand;
    use nene_docket_core::field_value::FieldValue;
    use nene_docket_core::item::Item;
    use nene_docket_core::item_state::ItemState;
    use nene_docket_core::permissions::Permissions;
    use nene_docket_core::signing_key::SigningKey;

    use super::*;
    use crate::admission_report::AdmissionReport;

    const ME: AccountId = AccountId::new(1);
    const OTHER: AccountId = AccountId::new(2);
    /// ID が小さいので、同じ時計では自分より前に並ぶ。
    const EARLIER: AccountId = AccountId::new(0);

    /// 記憶の上のジャーナル。追記した操作と届いた操作を全部持ち、課題ごとに引ける。
    struct MemoryJournal {
        ops: RefCell<Vec<Op>>,
        fail: bool,
    }

    impl MemoryJournal {
        fn new() -> Self {
            Self {
                ops: RefCell::new(Vec::new()),
                fail: false,
            }
        }

        fn arrive(&self, ops: Vec<Op>) -> Vec<Op> {
            self.ops.borrow_mut().extend(ops.iter().cloned());
            ops
        }
    }

    impl JournalPort for MemoryJournal {
        fn append(&mut self, op: &Op) -> Result<(), JournalFailure> {
            self.ops.borrow_mut().push(op.clone());
            Ok(())
        }

        fn sync(&mut self) -> Result<(), JournalFailure> {
            Ok(())
        }

        fn refresh(&mut self) -> Result<Vec<Op>, JournalFailure> {
            Ok(Vec::new())
        }

        fn snapshot(&mut self) -> Option<LedgerSnapshot> {
            None
        }

        fn ops_of(&self, item: ItemId) -> Result<Vec<Op>, JournalFailure> {
            if self.fail {
                return Err(JournalFailure::Unreadable);
            }
            Ok(self
                .ops
                .borrow()
                .iter()
                .filter(|op| op.kind().item() == Some(item))
                .cloned()
                .collect())
        }

        fn recent_ops(&self, _limit: usize) -> Result<Vec<Op>, JournalFailure> {
            Ok(self.ops.borrow().clone())
        }

        fn all_ops(&self) -> Result<Vec<Op>, JournalFailure> {
            Ok(self.ops.borrow().clone())
        }

        fn checkpoint(&mut self, _snapshot: &LedgerSnapshot) -> Result<(), JournalFailure> {
            Ok(())
        }

        fn rejected_lines(&self) -> usize {
            0
        }

        fn admission(&self) -> AdmissionReport {
            AdmissionReport::open()
        }

        fn names(&self) -> Vec<(AccountId, String)> {
            Vec::new()
        }

        fn owner(&self) -> Option<AccountId> {
            None
        }

        fn permissions(&self) -> Option<Permissions> {
            None
        }

        fn administer(&mut self, _commands: &[AdminCommand]) -> Result<(), JournalFailure> {
            Ok(())
        }
    }

    fn me() -> Account {
        Account::new(ME, String::from("me"), SigningKey::from_bytes(&[1_u8; 32]))
    }

    fn ledger() -> Ledger {
        Ledger::open(me(), &[])
    }

    /// 自分の操作を作り、ジャーナルにも書く（画面の `compose` ＋ `write` と同じ）。
    fn compose(ledger: &mut Ledger, journal: &mut MemoryJournal, kind: OpKind) -> Op {
        let op = ledger.compose(WallClock::new(0), kind);
        journal.append(&op).unwrap();
        op
    }

    fn set_title(
        ledger: &mut Ledger,
        journal: &mut MemoryJournal,
        item: ItemId,
        title: &str,
    ) -> Op {
        compose(
            ledger,
            journal,
            OpKind::Set {
                item,
                value: FieldValue::Title(String::from(title)),
            },
        )
    }

    fn undo(ledger: &mut Ledger, journal: &mut MemoryJournal) -> Option<Op> {
        let op = ledger.compose_undo(WallClock::new(1), journal).unwrap();
        if let Some(op) = &op {
            journal.append(op).unwrap();
        }
        op
    }

    fn redo(ledger: &mut Ledger, journal: &mut MemoryJournal) -> Option<Op> {
        let op = ledger.compose_redo(WallClock::new(2), journal).unwrap();
        if let Some(op) = &op {
            journal.append(op).unwrap();
        }
        op
    }

    /// 題名以外が作った直後の値に戻っている。
    fn assert_blank_but_titled(restored: &Item, title: &str) {
        assert_eq!(restored.title(), title);
        assert_eq!(restored.body(), "");
        assert_eq!(restored.state(), ItemState::Open);
        assert_eq!(restored.assignee(), "");
        assert_eq!(restored.due(), None);
        assert!(restored.labels().is_empty());
        assert_eq!(restored.rank(), 0);
    }

    fn later(clock: u64, account: AccountId, kind: OpKind) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), account),
            WallClock::new(0),
            kind,
        )
    }

    #[test]
    fn absorbing_later_ops_applies_them_in_place_and_earlier_ops_rebuild_the_item() {
        let journal = MemoryJournal::new();
        let item = ItemId::new(OTHER, 1);
        let first = journal.arrive(alloc::vec![later(1, OTHER, OpKind::Create(item))]);
        let mut ledger = Ledger::open(me(), &first);
        assert_eq!(ledger.generation(), 0);
        // 後の操作: 併合し直さずに当てる（世代が進む）。
        let title = |text: &str| FieldValue::Title(String::from(text));
        ledger
            .absorb(
                journal.arrive(alloc::vec![later(
                    3,
                    OTHER,
                    OpKind::Set {
                        item,
                        value: title("b"),
                    },
                )]),
                &journal,
            )
            .unwrap();
        assert_eq!(ledger.generation(), 1);
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("b"));
        // 前の操作（時計 2）が後から届く: 課題をその全操作から作り直すので、題名は b のまま（併合と同じ）。
        ledger
            .absorb(
                journal.arrive(alloc::vec![later(
                    2,
                    OTHER,
                    OpKind::Set {
                        item,
                        value: title("a"),
                    },
                )]),
                &journal,
            )
            .unwrap();
        assert_eq!(ledger.generation(), 2);
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("b"));
        assert_eq!(ledger.docket(), &merge::merge(&journal.all_ops().unwrap()));
        ledger.absorb(Vec::new(), &journal).unwrap();
        assert_eq!(ledger.generation(), 2);
        // 作り直しに要る読みが失敗すれば失敗を返す。
        let broken = MemoryJournal {
            ops: RefCell::new(Vec::new()),
            fail: true,
        };
        assert_eq!(
            ledger.absorb(
                alloc::vec![later(1, EARLIER, OpKind::Delete(item))],
                &broken
            ),
            Err(JournalFailure::Unreadable)
        );
    }

    #[test]
    fn composing_applies_locally_and_undo_redo_walk_the_own_ops() {
        let mut journal = MemoryJournal::new();
        let mut ledger = ledger();
        let item = ledger.fresh_item_id();
        assert_eq!(item, ItemId::new(ME, 1));
        compose(&mut ledger, &mut journal, OpKind::Create(item));
        set_title(&mut ledger, &mut journal, item, "a");
        set_title(&mut ledger, &mut journal, item, "b");
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("b"));
        let undone = undo(&mut ledger, &mut journal);
        assert!(undone.is_some_and(|op| op.undoes().is_some()));
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("a"));
        assert!(ledger.can_redo());
        assert!(redo(&mut ledger, &mut journal).is_some());
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("b"));
        assert!(!ledger.can_redo());
        assert_eq!(journal.all_ops().unwrap().len(), 5);
    }

    #[test]
    fn undo_of_a_create_deletes_and_nothing_to_undo_gives_none() {
        let mut journal = MemoryJournal::new();
        let mut ledger = ledger();
        assert_eq!(undo(&mut ledger, &mut journal), None);
        assert_eq!(redo(&mut ledger, &mut journal), None);
        let item = ledger.fresh_item_id();
        compose(&mut ledger, &mut journal, OpKind::Create(item));
        undo(&mut ledger, &mut journal);
        assert!(ledger.docket().live_items().is_empty());
        assert!(!ledger.can_undo());
    }

    #[test]
    fn absorbing_others_merges_and_advances_the_clock() {
        let mut journal = MemoryJournal::new();
        let mut ledger = ledger();
        let theirs = ItemId::new(OTHER, 1);
        let create = later(10, OTHER, OpKind::Create(theirs));
        ledger
            .absorb(journal.arrive(alloc::vec![create]), &journal)
            .unwrap();
        assert_eq!(ledger.docket().len(), 1);
        let mine = set_title(&mut ledger, &mut journal, theirs, "shared");
        assert_eq!(mine.id().clock(), LogicalClock::new(11));
        let late = later(
            11,
            EARLIER,
            OpKind::Set {
                item: theirs,
                value: FieldValue::State(ItemState::Done),
            },
        );
        ledger
            .absorb(journal.arrive(alloc::vec![late]), &journal)
            .unwrap();
        let item = ledger.docket().item(theirs);
        assert_eq!(item.map(Item::title), Some("shared"));
        assert_eq!(item.map(Item::state), Some(ItemState::Done));
        assert_eq!(journal.all_ops().unwrap().len(), 3);
    }

    #[test]
    fn undo_skips_others_ops_and_restores_every_kind_of_field() {
        let mut journal = MemoryJournal::new();
        let mut ledger = ledger();
        let item = ledger.fresh_item_id();
        compose(&mut ledger, &mut journal, OpKind::Create(item));
        let values = [
            FieldValue::Body(String::from("b")),
            FieldValue::State(ItemState::InProgress),
            FieldValue::Assignee(String::from("a")),
            FieldValue::Due(nene_docket_core::date::Date::new(2026, 1, 2)),
            FieldValue::Labels(alloc::vec![String::from("x")]),
            FieldValue::Rank(3),
        ];
        for value in values.clone() {
            compose(&mut ledger, &mut journal, OpKind::Set { item, value });
        }
        // 他人が間に題名を変えても、自分の Undo は自分の操作だけを戻す。
        let theirs = later(
            100,
            OTHER,
            OpKind::Set {
                item,
                value: FieldValue::Title(String::from("theirs")),
            },
        );
        ledger
            .absorb(journal.arrive(alloc::vec![theirs]), &journal)
            .unwrap();
        for _ in &values {
            undo(&mut ledger, &mut journal);
        }
        assert_blank_but_titled(ledger.docket().item(item).unwrap(), "theirs");
        assert!(ledger.can_undo());
    }

    #[test]
    fn redo_revives_the_op_and_a_fresh_op_clears_redo() {
        let mut journal = MemoryJournal::new();
        let mut ledger = ledger();
        let item = ledger.fresh_item_id();
        compose(&mut ledger, &mut journal, OpKind::Create(item));
        set_title(&mut ledger, &mut journal, item, "a");
        undo(&mut ledger, &mut journal);
        assert!(ledger.can_redo());
        set_title(&mut ledger, &mut journal, item, "c");
        assert!(!ledger.can_redo());
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("c"));
        undo(&mut ledger, &mut journal);
        assert_eq!(ledger.docket().item(item).map(Item::title), Some(""));
        assert!(redo(&mut ledger, &mut journal).is_some());
        assert_eq!(ledger.docket().item(item).map(Item::title), Some("c"));
    }

    #[test]
    fn reopening_from_the_ops_or_a_snapshot_restores_clock_sequence_and_undo() {
        let mut journal = MemoryJournal::new();
        let mut first = ledger();
        let item = first.fresh_item_id();
        compose(&mut first, &mut journal, OpKind::Create(item));
        set_title(&mut first, &mut journal, item, "x");
        undo(&mut first, &mut journal);
        let mut again = Ledger::open(me(), &journal.all_ops().unwrap());
        assert_eq!(again.fresh_item_id(), ItemId::new(ME, 2));
        assert!(again.can_redo());
        assert_eq!(again.account().name(), "me");
        let redone = redo(&mut again, &mut journal);
        assert_eq!(redone.map(|op| op.id().clock()), Some(LogicalClock::new(4)));
    }

    #[test]
    fn resuming_from_a_snapshot_restores_the_docket_the_clock_and_the_undo_stack() {
        let mut journal = MemoryJournal::new();
        let mut first = ledger();
        let item = first.fresh_item_id();
        compose(&mut first, &mut journal, OpKind::Create(item));
        set_title(&mut first, &mut journal, item, "x");
        undo(&mut first, &mut journal);
        let mut again = Ledger::open(me(), &journal.all_ops().unwrap());
        redo(&mut again, &mut journal);
        // スナップショットから戻しても同じ台帳・時計・Undo。
        let snapshot = again.snapshot();
        let mut resumed = Ledger::resume(me(), &snapshot);
        assert_eq!(resumed.docket(), again.docket());
        assert_eq!(resumed.fresh_item_id(), ItemId::new(ME, 2));
        assert!(resumed.can_undo());
        assert_eq!(
            undo(&mut resumed, &mut journal).map(|op| op.id().clock()),
            Some(LogicalClock::new(5))
        );
        assert_eq!(resumed.docket().item(item).map(Item::title), Some(""));
        // 自分の操作が後から届いても（前の回のスナップショットの後の分）Undo の束に積む。
        let mut late = Ledger::resume(me(), &snapshot);
        let own = later(9, ME, OpKind::Delete(item));
        late.absorb(journal.arrive(alloc::vec![own]), &journal)
            .unwrap();
        assert!(late.docket().item(item).is_some_and(Item::is_deleted));
        assert!(undo(&mut late, &mut journal).is_some());
        assert!(late.docket().item(item).is_some_and(|i| !i.is_deleted()));
    }

    #[test]
    fn admin_records_are_not_undoable_and_undo_of_an_unknown_target_gives_none() {
        let mut journal = MemoryJournal::new();
        let mut ledger = ledger();
        compose(
            &mut ledger,
            &mut journal,
            OpKind::Admin(AdminCommand::Permissions(true)),
        );
        assert!(!ledger.can_undo());
        assert_eq!(undo(&mut ledger, &mut journal), None);
    }
}
