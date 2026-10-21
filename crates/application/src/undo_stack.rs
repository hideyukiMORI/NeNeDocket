//! Undo / Redo の束（FR-005）。自分の操作だけ。ジャーナルから再構成できる（ADR 0004 の 4）。
//!
//! 自分の操作は 3 種類: 素の操作、素の操作を打ち消す Undo（`undoes` が素の操作）、Undo を打ち消す Redo（`undoes` が Undo）。
//! 順に `record` すれば、どの素の操作が生きていて、どれが戻されているかが決まる。

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use nene_docket_core::op::Op;
use nene_docket_core::op_id::OpId;
use nene_docket_core::op_kind::OpKind;

use crate::undo_state::UndoState;

/// 戻せる素の操作の上限。古いものから忘れる（スナップショットに残す量を抑える・ADR 0015）。
const LIMIT: usize = 1_000;

/// 生きている素の操作の列と、戻された（素の操作, それを戻した Undo）の列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoStack {
    undo_ops: BTreeMap<OpId, OpId>,
    done: Vec<OpId>,
    undone: Vec<(OpId, OpId)>,
}

impl UndoStack {
    /// 何も無い。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            undo_ops: BTreeMap::new(),
            done: Vec::new(),
            undone: Vec::new(),
        }
    }

    /// 自分の操作を 1 つ（ジャーナルの順に）。権限の記録は戻せないので積まない。
    pub fn record(&mut self, op: &Op) {
        if matches!(op.kind(), OpKind::Admin(_)) {
            return;
        }
        self.record_id(op.id(), op.undoes());
    }

    /// 操作の ID と、打ち消す相手だけから。
    pub fn record_id(&mut self, id: OpId, undoes: Option<OpId>) {
        match undoes {
            None => {
                self.done.push(id);
                self.undone.clear();
            }
            Some(target) => self.record_reversal(id, target),
        }
        if self.done.len() > LIMIT {
            let forgotten = self.done.remove(0);
            self.undo_ops.retain(|_, plain| *plain != forgotten);
        }
    }

    /// 中身（スナップショットへ）。
    #[must_use]
    pub fn state(&self) -> UndoState {
        UndoState::new(
            self.done.clone(),
            self.undone.clone(),
            self.undo_ops
                .iter()
                .map(|(undo, plain)| (*undo, *plain))
                .collect(),
        )
    }

    /// 中身から（スナップショットから）。
    #[must_use]
    pub fn from_state(state: &UndoState) -> Self {
        Self {
            undo_ops: state.undo_ops().iter().copied().collect(),
            done: state.done().to_vec(),
            undone: state.undone().to_vec(),
        }
    }

    /// 束にある全部の操作の ID（素の操作と Undo）。自分の操作の課題を覚えておく相手。
    #[must_use]
    pub fn mentioned(&self) -> Vec<OpId> {
        let mut ids: Vec<OpId> = self.done.clone();
        for (plain, undo) in &self.undone {
            ids.push(*plain);
            ids.push(*undo);
        }
        for (undo, plain) in &self.undo_ops {
            ids.push(*undo);
            ids.push(*plain);
        }
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    fn record_reversal(&mut self, id: OpId, target: OpId) {
        if let Some(index) = self.done.iter().rposition(|&done| done == target) {
            // 素の操作を戻した。
            let plain = self.done.remove(index);
            self.undone.push((plain, id));
            self.undo_ops.insert(id, plain);
        } else if let Some(&plain) = self.undo_ops.get(&target) {
            // Undo を打ち消した（やり直し）。素の操作が生き返る。
            self.undone.retain(|(_, undo)| *undo != target);
            self.done.push(plain);
        } else {
            // 知らない操作を打ち消した（他人のか、読めなかった分）。素の操作として扱う。
            self.done.push(id);
            self.undone.clear();
        }
    }

    /// 次に戻す素の操作。
    #[must_use]
    pub fn next_undo(&self) -> Option<OpId> {
        self.done.last().copied()
    }

    /// 次にやり直すときに打ち消す Undo。
    #[must_use]
    pub fn next_redo(&self) -> Option<OpId> {
        self.undone.last().map(|(_, undo)| *undo)
    }

    /// 生きている素の操作の数。
    #[must_use]
    pub fn done_count(&self) -> usize {
        self.done.len()
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;

    const ME: AccountId = AccountId::new(1);

    fn id(clock: u64) -> OpId {
        OpId::new(LogicalClock::new(clock), ME)
    }

    fn plain(clock: u64) -> Op {
        Op::new(
            id(clock),
            WallClock::new(0),
            OpKind::Create(ItemId::new(ME, 1)),
        )
    }

    fn reversal(clock: u64, target: u64) -> Op {
        plain(clock).undoing(id(target))
    }

    #[test]
    fn admin_records_are_not_undoable_so_they_are_not_recorded() {
        let mut stack = UndoStack::empty();
        stack.record(&Op::new(
            id(1),
            WallClock::new(0),
            OpKind::Admin(nene_docket_core::admin_command::AdminCommand::Permissions(
                true,
            )),
        ));
        assert_eq!(stack.next_undo(), None);
        assert_eq!(stack.done_count(), 0);
    }

    #[test]
    fn undo_then_redo_then_a_new_op_clears_the_redo_side() {
        let mut stack = UndoStack::empty();
        stack.record(&plain(1));
        stack.record(&plain(2));
        assert_eq!(stack.next_undo(), Some(id(2)));
        assert_eq!(stack.next_redo(), None);
        stack.record(&reversal(3, 2));
        assert_eq!(stack.next_undo(), Some(id(1)));
        assert_eq!(stack.next_redo(), Some(id(3)));
        stack.record(&reversal(4, 3));
        assert_eq!(stack.next_undo(), Some(id(2)));
        assert_eq!(stack.next_redo(), None);
        stack.record(&reversal(5, 2));
        stack.record(&plain(6));
        assert_eq!(stack.next_redo(), None);
        assert_eq!(stack.next_undo(), Some(id(6)));
        assert_eq!(stack.done_count(), 2);
    }

    #[test]
    fn rebuilding_from_the_journal_order_gives_the_same_stack() {
        let ops = [
            plain(1),
            plain(2),
            reversal(3, 2),
            reversal(4, 3),
            reversal(5, 2),
        ];
        let mut stack = UndoStack::empty();
        for op in &ops {
            stack.record(op);
        }
        assert_eq!(stack.next_undo(), Some(id(1)));
        assert_eq!(stack.next_redo(), Some(id(5)));
    }

    #[test]
    fn a_reversal_of_an_unknown_op_counts_as_a_plain_op() {
        let mut stack = UndoStack::empty();
        stack.record(&reversal(7, 99));
        assert_eq!(stack.next_undo(), Some(id(7)));
        assert_eq!(stack.next_redo(), None);
    }
}
