//! ジャーナルのポート（ARC-003）。追記・同期・読み直し。実装は store。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::admin_command::AdminCommand;
use nene_docket_core::attachment::Attachment;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::op::Op;
use nene_docket_core::permissions::Permissions;
use nene_docket_core::shared_bitmap::SharedBitmap;

use crate::admission_report::AdmissionReport;
use crate::attachment_text::AttachmentText;
use crate::journal_failure::JournalFailure;
use crate::ledger_snapshot::LedgerSnapshot;

/// 自分のジャーナルに追記し、同期し、他人の伸びた分を読む。
pub trait JournalPort {
    /// 自分のジャーナルに 1 操作を追記する。
    ///
    /// # Errors
    ///
    /// 書けない。
    fn append(&mut self, op: &Op) -> Result<(), JournalFailure>;

    /// 追記を記憶装置に確実に書く。書けていない操作があれば先に古い順に書き直す。
    /// 読み直しの刻みごとにも呼ばれる（Issue #99）ので、書くもの・同期するものが無ければ何もしない（I/O をしない）。
    ///
    /// # Errors
    ///
    /// 同期できない。
    fn sync(&mut self) -> Result<(), JournalFailure>;

    /// まだ渡していない操作。最初の呼び出しは起動時に読んだ自分の分（スナップショットがあればそれ以降）を含み、
    /// 以後は他人のジャーナルの伸びた分。無ければ空。
    ///
    /// # Errors
    ///
    /// 読めない。
    fn refresh(&mut self) -> Result<Vec<Op>, JournalFailure>;

    /// 起動時に読めた自分のスナップショット（ADR 0015）。無い・壊れている・古いなら `None`（そのときは `refresh` が全部を返す）。
    /// 1 度だけ渡す（2 度目は `None`。写しを 2 つ持たないため）。
    fn snapshot(&mut self) -> Option<LedgerSnapshot>;

    /// 課題 1 件に触れた操作を全部（履歴・Undo の逆の値・選び直し）。索引からジャーナルを引く。
    ///
    /// # Errors
    ///
    /// 読めない。
    fn ops_of(&self, item: ItemId) -> Result<Vec<Op>, JournalFailure>;

    /// 新しいものから `limit` 件までの操作（台帳全体の時系列・ADR 0014）。
    ///
    /// # Errors
    ///
    /// 読めない。
    fn recent_ops(&self, limit: usize) -> Result<Vec<Op>, JournalFailure>;

    /// 全部の操作（監査の出力）。
    ///
    /// # Errors
    ///
    /// 読めない。
    fn all_ops(&self) -> Result<Vec<Op>, JournalFailure>;

    /// スナップショットを書く（終了時と、一定の間隔・ADR 0015）。
    ///
    /// # Errors
    ///
    /// 書けない。
    fn checkpoint(&mut self, snapshot: &LedgerSnapshot) -> Result<(), JournalFailure>;

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

    /// ファイルを `attachments/` に写し、添付の記録（連番・名前・大きさ・SHA-256）を返す（FR-021）。元は触らない。
    ///
    /// # Errors
    ///
    /// 元を読めない・写しを置けない。
    fn copy_attachment(
        &mut self,
        item: ItemId,
        sequence: u32,
        source: &str,
    ) -> Result<Attachment, JournalFailure>;

    /// 添付の写しの中身が記録の SHA-256 と合うか。無い・読めないなら偽（FR-014）。
    fn attachment_is_intact(&self, item: ItemId, attachment: &Attachment) -> bool;

    /// 添付の写しの場所（表示と、外部のアプリで開くため）。
    fn attachment_path(&self, item: ItemId, attachment: &Attachment) -> String;

    /// 添付の写しを文として読む（FR-022）。文字コードは実装が決め（UTF-8・UTF-16 の BOM・`Shift_JIS`）、大きければ先頭だけ。
    ///
    /// # Errors
    ///
    /// 写しを読めない。
    fn read_attachment(
        &self,
        item: ItemId,
        attachment: &Attachment,
    ) -> Result<AttachmentText, JournalFailure>;

    /// 添付の写しを画像として読む（FR-022）。PNG / JPEG。大きければ縮めた画素。ID は中身から（UI のキャッシュの鍵）。
    ///
    /// # Errors
    ///
    /// 写しを読めない・復号できない。
    fn read_attachment_image(
        &self,
        item: ItemId,
        attachment: &Attachment,
    ) -> Result<SharedBitmap, JournalFailure>;

    /// 持ち主が権限を変える（FR-010）。書いたら鍵と権限を読み直す。
    ///
    /// # Errors
    ///
    /// 持ち主でない・書けない。
    fn administer(&mut self, commands: &[AdminCommand]) -> Result<(), JournalFailure>;
}
