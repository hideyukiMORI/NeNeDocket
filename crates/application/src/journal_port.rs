//! ジャーナルのポート（ARC-003）。追記・同期・読み直し。実装は store。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::op::Op;
use nene_docket_core::permissions::Permissions;

use crate::admission_report::AdmissionReport;
use crate::journal_failure::JournalFailure;
use nene_docket_core::admin_command::AdminCommand;

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

    /// 署名と権限の検証の報告（FR-010）。権限を使わない台帳は [`AdmissionReport::open`]。
    fn admission(&self) -> AdmissionReport;

    /// 台帳の `keys/` にあるアカウントの名前（FR-009）。履歴の「誰が」と参加者の段に使う。
    fn names(&self) -> Vec<(AccountId, String)>;

    /// 台帳の持ち主。持ち主の無い古い台帳は `None`。
    fn owner(&self) -> Option<AccountId>;

    /// 権限。有効でなければ `None`。
    fn permissions(&self) -> Option<Permissions>;

    /// 持ち主が権限を変える（FR-010）。書いたら鍵と権限を読み直す。
    ///
    /// # Errors
    ///
    /// 持ち主でない・書けない。
    fn administer(&mut self, commands: &[AdminCommand]) -> Result<(), JournalFailure>;
}
