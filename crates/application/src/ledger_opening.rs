//! 台帳の状態を開く唯一の経路（ARC-001・ADR 0015）。スナップショットがあれば戻して伸びた分を当て、無ければ全部読んで併合する。
//! 画面も合成ルート（権限の記録・監査）もここを通る。

use alloc::vec::Vec;

use nene_docket_core::op::Op;

use crate::account::Account;
use crate::journal_failure::JournalFailure;
use crate::journal_port::JournalPort;
use crate::ledger::Ledger;

/// ジャーナルから台帳の状態を作る。読めなければ読めた所までの状態と失敗を返す（起動は止めない・FR-014 で示す）。
pub fn open(journal: &mut dyn JournalPort, account: Account) -> (Ledger, Option<JournalFailure>) {
    let snapshot = journal.snapshot();
    let (fresh, failure) = match journal.refresh() {
        Ok(ops) => (ops, None),
        Err(failure) => (Vec::<Op>::new(), Some(failure)),
    };
    match snapshot {
        Some(snapshot) => {
            let mut ledger = Ledger::resume(account, &snapshot);
            let absorbed = ledger.absorb(fresh, journal).err();
            (ledger, failure.or(absorbed))
        }
        None => (Ledger::open(account, &fresh), failure),
    }
}
