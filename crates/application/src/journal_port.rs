//! ジャーナルのポート（ARC-003）。追記・同期・読み直し。実装は store。

use alloc::vec::Vec;

use nene_docket_core::op::Op;

use crate::journal_failure::JournalFailure;

/// 自分のジャーナルに追記し、同期し、他人の伸びた分を読む。
pub trait JournalPort {
    /// 自分のジャーナルに 1 操作を追記する。
    ///
    /// # Errors
    ///
    /// 書けない。
    fn append(&mut self, op: &Op) -> Result<(), JournalFailure>;

    /// 追記を記憶装置に確実に書く。
    ///
    /// # Errors
    ///
    /// 同期できない。
    fn sync(&mut self) -> Result<(), JournalFailure>;

    /// まだ渡していない操作。最初の呼び出しは起動時に読んだ自分の分を含み、以後は他人のジャーナルの伸びた分。無ければ空。
    ///
    /// # Errors
    ///
    /// 読めない。
    fn refresh(&mut self) -> Result<Vec<Op>, JournalFailure>;

    /// これまでに見た壊れた行の数。利用者に示す（FR-014）。
    fn rejected_lines(&self) -> usize;
}
