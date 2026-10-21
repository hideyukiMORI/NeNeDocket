//! Undo の束の中身（スナップショットに残す形・ADR 0015）。生きている素の操作・戻された対・Undo から素の操作への対応。

use alloc::vec::Vec;

use nene_docket_core::op_id::OpId;

/// `UndoStack` の 3 つの列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoState {
    done: Vec<OpId>,
    undone: Vec<(OpId, OpId)>,
    undo_ops: Vec<(OpId, OpId)>,
}

impl UndoState {
    /// 値から。
    #[must_use]
    pub const fn new(
        done: Vec<OpId>,
        undone: Vec<(OpId, OpId)>,
        undo_ops: Vec<(OpId, OpId)>,
    ) -> Self {
        Self {
            done,
            undone,
            undo_ops,
        }
    }

    /// 生きている素の操作（古い順）。
    #[must_use]
    pub fn done(&self) -> &[OpId] {
        &self.done
    }

    /// 戻された（素の操作, それを戻した Undo）。
    #[must_use]
    pub fn undone(&self) -> &[(OpId, OpId)] {
        &self.undone
    }

    /// Undo から素の操作へ。
    #[must_use]
    pub fn undo_ops(&self) -> &[(OpId, OpId)] {
        &self.undo_ops
    }
}
