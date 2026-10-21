//! 台帳の状態のスナップショット（ADR 0015）。併合した台帳・論理時計・次の連番・全順序で最後に当てた操作・Undo の束・
//! Undo の束にある自分の操作の課題。store が自分のアカウントのファイルに書き、起動時に戻す。

use alloc::vec::Vec;

use nene_docket_core::docket::Docket;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::logical_clock::LogicalClock;
use nene_docket_core::op_id::OpId;

use crate::undo_state::UndoState;

/// `Ledger` を戻すのに要る全部。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerSnapshot {
    docket: Docket,
    clock: LogicalClock,
    next_sequence: u32,
    last_applied: Option<OpId>,
    undo: UndoState,
    own_items: Vec<(OpId, ItemId)>,
}

impl LedgerSnapshot {
    /// 値から。
    #[must_use]
    pub fn new(
        (docket, clock, next_sequence, last_applied): (Docket, LogicalClock, u32, Option<OpId>),
        undo: UndoState,
        own_items: Vec<(OpId, ItemId)>,
    ) -> Self {
        Self {
            docket,
            clock,
            next_sequence,
            last_applied,
            undo,
            own_items,
        }
    }

    /// 併合した台帳。
    #[must_use]
    pub const fn docket(&self) -> &Docket {
        &self.docket
    }

    /// 論理時計。
    #[must_use]
    pub const fn clock(&self) -> LogicalClock {
        self.clock
    }

    /// 次に作る課題の番号。
    #[must_use]
    pub const fn next_sequence(&self) -> u32 {
        self.next_sequence
    }

    /// 全順序で最後に当てた操作。
    #[must_use]
    pub const fn last_applied(&self) -> Option<OpId> {
        self.last_applied
    }

    /// Undo の束。
    #[must_use]
    pub const fn undo(&self) -> &UndoState {
        &self.undo
    }

    /// Undo の束にある自分の操作と、その課題。
    #[must_use]
    pub fn own_items(&self) -> &[(OpId, ItemId)] {
        &self.own_items
    }
}
