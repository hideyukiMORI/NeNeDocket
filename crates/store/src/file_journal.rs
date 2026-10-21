//! application の `JournalPort` の実装。自分のジャーナルへの追記・同期と、他人のジャーナルの読み直しを 1 つに束ねる。
//! 権限を有効にした台帳では版 2 に署名して書き、読む行（自分の分も）は検証器に通す（ADR 0010）。
//! 自分のスナップショット（状態・読み位置・課題ごとの索引）を読んで起動を速くし、履歴はジャーナルから索引で引く（ADR 0015）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::thread::{self, JoinHandle};

use nene_docket_application::account::Account;
use nene_docket_application::admission_report::AdmissionReport;
use nene_docket_application::attachment_text::AttachmentText;
use nene_docket_application::journal_failure::JournalFailure;
use nene_docket_application::journal_port::JournalPort;
use nene_docket_application::key_record::KeyRecord;
use nene_docket_application::ledger_snapshot::LedgerSnapshot;
use nene_docket_core::account_id::AccountId;
use nene_docket_core::admin_command::AdminCommand;
use nene_docket_core::attachment::Attachment;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::op::Op;
use nene_docket_core::permissions::Permissions;
use nene_docket_core::shared_bitmap::SharedBitmap;

use crate::administration;
use crate::attachment_store;
use crate::journal_file;
use crate::journal_folder::JournalFolder;
use crate::journal_version::JournalVersion;
use crate::key_file;
use crate::leftovers;
use crate::other_journals::OtherJournals;
use crate::own_journal::{self, OwnJournal};
use crate::permissions_store;
use crate::resumed_snapshot::{Resumed, journal_number, remember};
use crate::scan::Scan;
use crate::scanner::Scanner;
use crate::snapshot_file::{self, Index, StoredSnapshot};
use crate::store_failure::StoreFailure;
use crate::verifier::{self, Verifier};

/// ジャーナルの名（アカウントと版）。索引はこの列の番号で指す。
type JournalName = (AccountId, JournalVersion);

/// 新しい操作を末尾から読むときに、1 ファイルあたり読むバイト数の上限の目安（1 行 200 バイト強）。
const TAIL_BYTES_PER_OP: u64 = 256;

/// 台帳のフォルダ・自分のジャーナル・他人の読み位置・起動時に読んだ自分の操作（最初の `refresh` で渡す）・鍵と検証器・
/// 起動時に読めたスナップショット・課題ごとの索引（ジャーナルの番号と位置）。
#[derive(Debug)]
pub struct FileJournal {
    folder: JournalFolder,
    account: Account,
    own: OwnJournal,
    others: OtherJournals,
    pending: Vec<Op>,
    own_rejected: usize,
    own_unverified: usize,
    keys: Vec<KeyRecord>,
    verifier: Option<Verifier>,
    key_mismatch: bool,
    resumed: Option<LedgerSnapshot>,
    resumed_from_snapshot: bool,
    journals: Vec<JournalName>,
    index: Index,
    /// 他人のジャーナルを読むスレッド（Issue #75）。
    scanner: Scanner,
    /// 書いている途中のスナップショット（別のスレッド・終わりは次の checkpoint か drop で待つ）。
    checkpoint_writer: Option<JoinHandle<Result<(), StoreFailure>>>,
    /// 最初の読み直しは待つ（起動の画面に他人の課題が揃うように）。以後は待たない。
    settle_first: bool,
    /// 追記に失敗した操作（空き容量ゼロ・切れた共有）。次の追記・同期で先に書き直す。残っている間はスナップショットを書かない（Issue #81）。
    unwritten: Vec<Op>,
}

impl FileJournal {
    /// フォルダを開く（無ければ `name` で作る）。自分の鍵を `keys/` に置き、権限が有効なら検証器を読み、
    /// 自分のスナップショットがあれば（権限が変わっておらず、ジャーナルが縮んでいなければ）戻し、
    /// 自分のジャーナルを読む（壊れた末尾は捨てて数え、検証の通らない行も数える）。
    ///
    /// # Errors
    ///
    /// フォルダを作れない・読めない・版が違う・ジャーナルを開けない・鍵や権限のファイルが読めない。
    pub fn open(root: &Path, name: &str, account: &Account) -> Result<Self, StoreFailure> {
        let (folder, key_mismatch) = Self::prepare_folder(root, name, account)?;
        let enabled = folder.docket().permissions_on();
        let (keys, verifier) =
            verifier::load_if_enabled(root, folder.docket().owner().filter(|_| enabled))?;
        let signing = verifier.as_ref().map(|_| account.key().clone());
        let own_version = if signing.is_some() {
            JournalVersion::Two
        } else {
            JournalVersion::One
        };
        let mark = permissions_mark(root)?;
        let stored = Self::usable_snapshot(&folder, account.id(), mark.as_deref())?;
        let resumed = Resumed::from_stored(stored.as_ref(), account.id());
        let (own, read) = OwnJournal::open_from(
            &folder.journal_directory(),
            account.id(),
            signing,
            resumed.offset_of((account.id(), own_version)),
        )?;
        let (mut own_rejected, mut own_unverified) = resumed.own_counts();
        own_rejected = own_rejected.saturating_add(read.rejected().len());
        let (mut journals, other_offsets, mut index) = resumed.into_parts();
        let others = OtherJournals::from_offsets(account.id(), other_offsets);
        let own_journal = journal_number(&mut journals, (account.id(), own_version));
        let (pending, unverified) = admit_own(
            read.into_located_entries(),
            (verifier.as_ref(), account.id()),
            (&mut index, own_journal),
        );
        own_unverified = own_unverified.saturating_add(unverified);
        if verifier.is_some() && stored.is_none() {
            own_unverified =
                own_unverified.saturating_add(Self::count_unsigned_own(&folder, account.id())?);
        }
        let scanner = Scanner::start_for(&folder, &others, (&keys, verifier.as_ref()));
        Ok(Self {
            folder,
            account: account.clone(),
            own,
            others,
            pending,
            own_rejected,
            own_unverified,
            keys,
            verifier,
            key_mismatch,
            resumed_from_snapshot: stored.is_some(),
            resumed: stored.map(StoredSnapshot::into_ledger),
            journals,
            index,
            scanner,
            checkpoint_writer: None,
            settle_first: true,
            unwritten: Vec::new(),
        })
    }

    /// 自分のスナップショットが使えれば読む。無い・壊れている（捨てる）・権限が変わった・ジャーナルが縮んだ・知らないジャーナル名なら `None`。
    /// フォルダを開く（無ければ作る）。途中で止まった置き換えの残骸を捨て（Issue #81）、自分の鍵を置き、
    /// 引き継がれた台帳なら署名し直してから開き直す（ADR 0013）。鍵が台帳のものと違えばその印も返す。
    fn prepare_folder(
        root: &Path,
        name: &str,
        account: &Account,
    ) -> Result<(JournalFolder, bool), StoreFailure> {
        let mut folder = JournalFolder::open_or_create(root, name, account.id())?;
        let owner = folder.docket().owner() == Some(account.id());
        let _discarded = leftovers::discard(root, account.id(), owner)?;
        let key_mismatch = key_file::ensure_own(root, account)?;
        if !key_mismatch && administration::adopt(root, account)? {
            folder = JournalFolder::open_or_create(root, name, account.id())?;
        }
        Ok((folder, key_mismatch))
    }

    fn usable_snapshot(
        folder: &JournalFolder,
        account: AccountId,
        permissions: Option<&str>,
    ) -> Result<Option<StoredSnapshot>, StoreFailure> {
        let stored = match snapshot_file::read(folder.root(), account) {
            Ok(stored) => stored,
            Err(StoreFailure::BadSnapshot) => None,
            Err(failure) => return Err(failure),
        };
        let Some(stored) = stored else {
            return Ok(None);
        };
        if stored.permissions() != permissions {
            return Ok(None);
        }
        let directory = folder.journal_directory();
        for ((other, version), offset) in stored.journals() {
            let path = directory.join(own_journal::file_name(*other, *version));
            let length = match std::fs::metadata(&path) {
                Ok(metadata) => metadata.len(),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
                Err(error) => return Err(StoreFailure::Io(error)),
            };
            if *offset > length {
                return Ok(None);
            }
        }
        Ok(Some(stored))
    }

    /// 権限を有効にした台帳に残る、自分の版 1 のジャーナルの行数。読まないが、捨てたものとして数える（FR-014）。
    fn count_unsigned_own(
        folder: &JournalFolder,
        account: AccountId,
    ) -> Result<usize, StoreFailure> {
        let path = folder
            .journal_directory()
            .join(own_journal::file_name(account, JournalVersion::One));
        if !path.exists() {
            return Ok(0);
        }
        Ok(journal_file::read_since(&path, 0)?.entries().len())
    }

    /// 台帳のフォルダ。
    #[must_use]
    pub const fn folder(&self) -> &JournalFolder {
        &self.folder
    }

    /// 権限が有効か。
    #[must_use]
    pub const fn is_signed(&self) -> bool {
        self.verifier.is_some()
    }

    /// 起動時にスナップショットから戻したか。
    #[must_use]
    pub const fn is_resumed(&self) -> bool {
        self.resumed_from_snapshot
    }

    /// スレッドの結果を取り込む（読み位置・壊れた行・鍵と検証器・索引）。失敗があれば真。
    fn absorb_scan(&mut self, scan: Scan, ops: &mut Vec<Op>) -> bool {
        let ((fresh, failure), (others, keys, verifier)) = scan.into_parts();
        self.others = others;
        self.keys = keys;
        self.verifier = verifier;
        for (name, offset, op) in fresh {
            let number = journal_number(&mut self.journals, name);
            remember(&mut self.index, &op, number, offset);
            ops.push(op);
        }
        failure.is_some()
    }

    /// 読みを頼み、その結果を待ってから返す（テストと `--attach` のように今の状態が要るとき）。
    ///
    /// # Errors
    ///
    /// 他人のジャーナルを読めない。
    pub fn refresh_settled(&mut self) -> Result<Vec<Op>, JournalFailure> {
        self.refresh_with(Scanner::wait)
    }

    /// 書いている途中のスナップショットがあれば待つ（テストと、確実に書き終えたいとき）。
    ///
    /// # Errors
    ///
    /// 書けなかった。
    pub fn finish_checkpoint(&mut self) -> Result<(), StoreFailure> {
        match self.checkpoint_writer.take() {
            Some(writer) => writer
                .join()
                .map_err(|_panicked| StoreFailure::BadSnapshot)?,
            None => Ok(()),
        }
    }

    /// 索引にある操作の数。
    #[must_use]
    pub fn indexed_ops(&self) -> usize {
        self.index.values().map(Vec::len).sum()
    }

    /// 権限を書き、`docket.v1.txt` と検証器を読み直す。有効・無効が変われば自分のジャーナルを版を変えて開き直す。
    /// 既に取り込んだ他人の行は読み直さない（開き直したときから完全に効く・ADR 0010 の 4）。
    fn administer_and_reload(&mut self, commands: &[AdminCommand]) -> Result<(), StoreFailure> {
        let root = self.folder.root().to_path_buf();
        administration::apply(&root, &self.account, commands)?;
        self.folder = JournalFolder::open_or_create(&root, self.folder.name(), self.account.id())?;
        let enabled = self.folder.docket().permissions_on();
        let (keys, verifier) =
            verifier::load_if_enabled(&root, self.folder.docket().owner().filter(|_| enabled))?;
        self.keys = keys;
        if verifier.is_some() != self.verifier.is_some() {
            let signing = verifier.as_ref().map(|_| self.account.key().clone());
            let (own, _already_delivered) =
                OwnJournal::open(&self.folder.journal_directory(), self.account.id(), signing)?;
            self.own = own;
        }
        self.verifier = verifier;
        self.scanner = Scanner::start_for(
            &self.folder,
            &self.others,
            (&self.keys, self.verifier.as_ref()),
        );
        Ok(())
    }

    fn journal_path(&self, number: u8) -> Option<PathBuf> {
        let (account, version) = self.journals.get(usize::from(number))?;
        Some(
            self.folder
                .journal_directory()
                .join(own_journal::file_name(*account, *version)),
        )
    }

    fn read_ops_of(&self, item: ItemId) -> Result<Vec<Op>, StoreFailure> {
        let mut ops = Vec::new();
        for packed in self.index.get(&item).map_or(&[][..], Vec::as_slice) {
            let (number, offset) = snapshot_file::unpack(*packed);
            let path = self.journal_path(number).ok_or(StoreFailure::BadIndex)?;
            let (op, _signature) = journal_file::read_line_at(&path, offset)?;
            ops.push(op);
        }
        ops.sort_by_key(Op::id);
        Ok(ops)
    }

    fn read_recent(&self, limit: usize) -> Result<Vec<Op>, StoreFailure> {
        let bytes = u64::try_from(limit)
            .unwrap_or(u64::MAX)
            .saturating_mul(TAIL_BYTES_PER_OP);
        let mut ops = Vec::new();
        for (account, _version, path) in self.folder.journals()? {
            ops.extend(self.admitted(account, journal_file::read_tail(&path, bytes)?));
        }
        ops.sort_by_key(Op::id);
        let skip = ops.len().saturating_sub(limit);
        Ok(ops.into_iter().skip(skip).collect())
    }

    fn read_all(&self) -> Result<Vec<Op>, StoreFailure> {
        let mut ops = Vec::new();
        for (account, _version, path) in self.folder.journals()? {
            ops.extend(self.admitted(account, journal_file::read_since(&path, 0)?));
        }
        ops.sort_by_key(Op::id);
        Ok(ops)
    }

    /// 読んだ行のうち、検証に通るもの（通らないものは黙って落とす: 数えるのは読み直しのときだけ）。
    fn admitted(&self, account: AccountId, read: crate::journal_read::JournalRead) -> Vec<Op> {
        read.into_entries()
            .into_iter()
            .filter_map(|entry| verifier::admit(self.verifier.as_ref(), account, entry).ok())
            .collect()
    }

    /// 自分のジャーナルに 1 行追記し、索引に入れる。失敗したら 1 度だけ開き直して試す（ハンドルが無効になったとき）。
    fn write_own(&mut self, op: &Op) -> Result<(), StoreFailure> {
        let own_name = (self.account.id(), self.own.version());
        let number = journal_number(&mut self.journals, own_name);
        let offset = match self.own.append(op) {
            Ok(offset) => offset,
            Err(_first) => {
                self.own.reopen()?;
                self.own.append(op)?
            }
        };
        remember(&mut self.index, op, number, offset);
        Ok(())
    }

    /// 書けていない操作を古い順に書き直す。途中で失敗すればそこで止める（順は保つ）。
    fn flush_unwritten(&mut self) -> Result<(), StoreFailure> {
        while let Some(op) = self.unwritten.first().cloned() {
            self.write_own(&op)?;
            self.unwritten.remove(0);
        }
        Ok(())
    }

    /// まだジャーナルに書けていない操作の数（テストから見る。画面は追記の失敗を示すだけ）。
    #[must_use]
    pub fn unwritten(&self) -> usize {
        self.unwritten.len()
    }

    /// テストだけ: 自分のジャーナル（失敗を仕込む）。
    #[cfg(test)]
    pub(crate) const fn own_mut(&mut self) -> &mut OwnJournal {
        &mut self.own
    }

    fn write_checkpoint(&mut self, snapshot: &LedgerSnapshot) -> Result<(), StoreFailure> {
        if !self.unwritten.is_empty() {
            return Err(StoreFailure::Unwritten(self.unwritten.len()));
        }
        let own_name = (self.account.id(), self.own.version());
        journal_number(&mut self.journals, own_name);
        let mut offsets: BTreeMap<JournalName, u64> = self.others.offsets().clone();
        offsets.insert(own_name, self.own.length()?);
        let journals: Vec<(JournalName, u64)> = self
            .journals
            .iter()
            .map(|name| (*name, offsets.get(name).copied().unwrap_or(0)))
            .collect();
        let stored = StoredSnapshot::new(
            snapshot.clone(),
            journals,
            (
                permissions_mark(self.folder.root())?,
                self.own_rejected,
                self.own_unverified,
            ),
            self.index.clone(),
        );
        // 🔴 書くのは別のスレッド（300 万操作で 0.5〜0.9 秒・フレームを止めない・Issue #75）。前の書きが終わっていなければ待つ。
        self.finish_checkpoint()?;
        let (root, account) = (self.folder.root().to_path_buf(), self.account.id());
        self.checkpoint_writer = Some(thread::spawn(move || {
            snapshot_file::write(&root, account, &stored)
        }));
        Ok(())
    }

    /// 届いていた分を受け取り、次の読みを頼み、50 ms だけ待つ。
    /// ローカルの台帳なら 50 ms の中で届くので同じ刻みで画面に出る。遅い共有では待たずに次の刻みへ回す（Issue #75）。
    fn refresh_now(&mut self) -> Result<Vec<Op>, JournalFailure> {
        self.refresh_with(Scanner::wait_briefly)
    }

    /// 届いていた分を受け取り、次の読みを頼み、`wait` の分だけ待って受け取る。
    fn refresh_with<F: Fn(&Scanner) -> Option<Scan>>(
        &mut self,
        wait: F,
    ) -> Result<Vec<Op>, JournalFailure> {
        let mut ops = core::mem::take(&mut self.pending);
        let mut failed = false;
        for scan in self.scanner.take() {
            failed |= self.absorb_scan(scan, &mut ops);
        }
        self.scanner.request();
        if let Some(scan) = wait(&self.scanner) {
            failed |= self.absorb_scan(scan, &mut ops);
        }
        if failed {
            return Err(JournalFailure::Unreadable);
        }
        Ok(ops)
    }
}

impl Drop for FileJournal {
    /// 窓を閉じた後に書いているスナップショットを待つ（プロセスの終わりで切れないため）。
    fn drop(&mut self) {
        let _finished = self.finish_checkpoint();
    }
}

/// 起動時に読んだ自分の行を検証器に通し、索引に入れる。通った操作と、通らなかった数。
fn admit_own(
    entries: Vec<(u64, crate::journal_read::Entry)>,
    (verifier, account): (Option<&Verifier>, AccountId),
    (index, own_journal): (&mut Index, u8),
) -> (Vec<Op>, usize) {
    let mut pending = Vec::new();
    let mut unverified = 0_usize;
    for (position, entry) in entries {
        match verifier::admit(verifier, account, entry) {
            Ok(op) => {
                remember(index, &op, own_journal, position);
                pending.push(op);
            }
            Err(_failure) => unverified = unverified.saturating_add(1),
        }
    }
    (pending, unverified)
}

/// `permissions.v1.txt` の署名（無ければ `None`）。スナップショットが権限の変更をまたがないための印。
fn permissions_mark(root: &Path) -> Result<Option<String>, StoreFailure> {
    Ok(permissions_store::read(root)?.map(|reading| reading.signature().hex()))
}

impl JournalPort for FileJournal {
    /// 先に書けていない操作を書き直してから追記する。失敗した操作は覚えて次に回し、失敗を示す（Issue #81）。
    fn append(&mut self, op: &Op) -> Result<(), JournalFailure> {
        let written = self.flush_unwritten().and_then(|()| self.write_own(op));
        if written.is_err() {
            self.unwritten.push(op.clone());
            return Err(JournalFailure::Unwritable);
        }
        Ok(())
    }

    fn sync(&mut self) -> Result<(), JournalFailure> {
        self.flush_unwritten()
            .and_then(|()| self.own.sync())
            .map_err(|_failure| JournalFailure::Unwritable)
    }

    /// 起動時に読んだ自分の分と、スレッドから届いていた他人の分を返し、次の読みを頼む（待たない・Issue #75）。
    /// 最初の 1 回だけは結果を待つ（起動の画面に他人の課題が揃うように）。
    fn refresh(&mut self) -> Result<Vec<Op>, JournalFailure> {
        if core::mem::take(&mut self.settle_first) {
            return self.refresh_settled();
        }
        self.refresh_now()
    }

    fn snapshot(&mut self) -> Option<LedgerSnapshot> {
        self.resumed.take()
    }
    fn ops_of(&self, item: ItemId) -> Result<Vec<Op>, JournalFailure> {
        self.read_ops_of(item)
            .map_err(|_failure| JournalFailure::Unreadable)
    }

    fn recent_ops(&self, limit: usize) -> Result<Vec<Op>, JournalFailure> {
        self.read_recent(limit)
            .map_err(|_failure| JournalFailure::Unreadable)
    }

    fn all_ops(&self) -> Result<Vec<Op>, JournalFailure> {
        self.read_all()
            .map_err(|_failure| JournalFailure::Unreadable)
    }

    fn checkpoint(&mut self, snapshot: &LedgerSnapshot) -> Result<(), JournalFailure> {
        self.write_checkpoint(snapshot)
            .map_err(|_failure| JournalFailure::Unwritable)
    }

    fn rejected_lines(&self) -> usize {
        self.own_rejected
            .saturating_add(self.others.rejected().len())
    }

    fn admission(&self) -> AdmissionReport {
        match &self.verifier {
            None => AdmissionReport::open(),
            Some(verifier) => AdmissionReport::new(
                self.own_unverified
                    .saturating_add(self.others.unverified().len()),
                self.key_mismatch,
                verifier.permissions().may_edit(self.account.id()),
            ),
        }
    }

    fn names(&self) -> Vec<(AccountId, String)> {
        let name_of = |id: AccountId| {
            self.keys
                .iter()
                .find(|record| record.id() == id)
                .map(|record| record.name().to_owned())
        };
        self.keys
            .iter()
            .map(|record| {
                let successor = self.verifier.as_ref().map_or(record.id(), |verifier| {
                    verifier.permissions().resolved(record.id())
                });
                let name = name_of(successor).unwrap_or_else(|| record.name().to_owned());
                (record.id(), name)
            })
            .collect()
    }

    fn owner(&self) -> Option<AccountId> {
        self.folder.docket().owner()
    }

    fn permissions(&self) -> Option<Permissions> {
        self.verifier
            .as_ref()
            .map(|verifier| verifier.permissions().clone())
    }

    fn administer(&mut self, commands: &[AdminCommand]) -> Result<(), JournalFailure> {
        self.administer_and_reload(commands)
            .map_err(|_failure| JournalFailure::PermissionsUnwritable)
    }

    fn copy_attachment(
        &mut self,
        item: ItemId,
        sequence: u32,
        source: &str,
    ) -> Result<Attachment, JournalFailure> {
        attachment_store::copy_in(self.folder.root(), item, sequence, Path::new(source)).map_err(
            |failure| match failure {
                StoreFailure::CannotReadAttachment => JournalFailure::AttachmentUnreadable,
                StoreFailure::CannotDecodeAttachment
                | StoreFailure::Unwritten(_)
                | StoreFailure::Io(_)
                | StoreFailure::UnknownDocketVersion(_)
                | StoreFailure::BadDocketFile
                | StoreFailure::BadJournalName(_)
                | StoreFailure::BadKeyFile(_)
                | StoreFailure::BadPermissionsFile
                | StoreFailure::UnverifiedPermissions
                | StoreFailure::NoDocket
                | StoreFailure::NotTheOwner
                | StoreFailure::KeyMismatch
                | StoreFailure::BadIndex
                | StoreFailure::BadSnapshot
                | StoreFailure::CannotWriteAttachment => JournalFailure::AttachmentUnwritable,
            },
        )
    }

    fn attachment_is_intact(&self, item: ItemId, attachment: &Attachment) -> bool {
        attachment_store::is_intact(self.folder.root(), item, attachment)
    }

    fn attachment_path(&self, item: ItemId, attachment: &Attachment) -> String {
        attachment_store::path_of(self.folder.root(), item, attachment)
            .to_string_lossy()
            .into_owned()
    }

    fn read_attachment(
        &self,
        item: ItemId,
        attachment: &Attachment,
    ) -> Result<AttachmentText, JournalFailure> {
        attachment_store::read_text(self.folder.root(), item, attachment)
            .map(|(text, truncated)| AttachmentText::new(text, truncated))
            .map_err(|_failure| JournalFailure::AttachmentUnreadable)
    }

    /// ID は記録の SHA-256 の先頭 8 バイト（同じ中身なら同じ ID）。
    fn read_attachment_image(
        &self,
        item: ItemId,
        attachment: &Attachment,
    ) -> Result<SharedBitmap, JournalFailure> {
        let bitmap = attachment_store::read_image(self.folder.root(), item, attachment).map_err(
            |failure| match failure {
                StoreFailure::CannotDecodeAttachment => JournalFailure::AttachmentUndecodable,
                StoreFailure::CannotReadAttachment
                | StoreFailure::CannotWriteAttachment
                | StoreFailure::Unwritten(_)
                | StoreFailure::Io(_)
                | StoreFailure::UnknownDocketVersion(_)
                | StoreFailure::BadDocketFile
                | StoreFailure::BadJournalName(_)
                | StoreFailure::BadKeyFile(_)
                | StoreFailure::BadPermissionsFile
                | StoreFailure::UnverifiedPermissions
                | StoreFailure::NoDocket
                | StoreFailure::NotTheOwner
                | StoreFailure::KeyMismatch
                | StoreFailure::BadIndex
                | StoreFailure::BadSnapshot => JournalFailure::AttachmentUnreadable,
            },
        )?;
        let id = u64::from_be_bytes(
            attachment
                .sha256()
                .get(..8)
                .and_then(|head| <[u8; 8]>::try_from(head).ok())
                .unwrap_or([0_u8; 8]),
        );
        Ok(SharedBitmap::new(id, bitmap))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::signing_key::SigningKey;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;
    use crate::administration;
    use crate::permissions_store;
    use nene_docket_application::key_record::KeyRecord;
    use nene_docket_application::ledger_opening;
    use nene_docket_core::admin_command::AdminCommand;

    const ME: AccountId = AccountId::new(0x11);
    const OTHER: AccountId = AccountId::new(0x22);

    fn account(id: AccountId, seed: u8) -> Account {
        Account::new(id, format!("u{seed}"), SigningKey::from_bytes(&[seed; 32]))
    }

    fn create(clock: u64, account: AccountId) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), account),
            WallClock::new(0),
            OpKind::Create(ItemId::new(account, 1)),
        )
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("nene-docket-file-journal-{name}"));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        root
    }

    #[test]
    fn own_ops_come_back_on_the_first_refresh_and_others_on_later_ones() {
        let root = temp("refresh");
        let me = account(ME, 1);
        let mut first = FileJournal::open(&root, "Team", &me).unwrap();
        assert!(first.refresh_settled().unwrap().is_empty());
        assert!(!first.is_signed());
        assert_eq!(first.admission(), AdmissionReport::open());
        assert_eq!(first.names(), [(ME, String::from("u1"))]);
        first.append(&create(1, ME)).unwrap();
        first.sync().unwrap();
        drop(first);

        let mut again = FileJournal::open(&root, "Team", &me).unwrap();
        assert_eq!(again.folder().name(), "Team");
        assert_eq!(again.refresh_settled().unwrap(), vec![create(1, ME)]);
        let theirs = again
            .folder()
            .journal_directory()
            .join(own_journal::file_name(OTHER, JournalVersion::One));
        let mut line = crate::journal_record::encode(&create(2, OTHER));
        line.push('\n');
        fs::write(&theirs, line).unwrap();
        assert_eq!(again.refresh_settled().unwrap(), vec![create(2, OTHER)]);
        assert!(again.refresh_settled().unwrap().is_empty());
        assert_eq!(again.rejected_lines(), 0);
        fs::write(&theirs, "garbage\n").unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn broken_lines_are_counted_and_a_missing_folder_is_unreadable() {
        let root = temp("rejected");
        let mut journal = FileJournal::open(&root, "Team", &account(ME, 1)).unwrap();
        let theirs = journal
            .folder()
            .journal_directory()
            .join(own_journal::file_name(OTHER, JournalVersion::One));
        fs::write(&theirs, "v1\tnot a record\n").unwrap();
        assert!(journal.refresh_settled().unwrap().is_empty());
        assert_eq!(journal.rejected_lines(), 1);
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(journal.refresh_settled(), Err(JournalFailure::Unreadable));
        assert_eq!(journal.append(&create(1, ME)), Ok(()));
    }

    /// 権限の無いうちに持ち主が 1 行書き、権限を有効にして編集者を 1 人加える。
    #[test]
    fn a_failed_append_is_kept_in_order_and_written_before_the_next_and_blocks_checkpoints() {
        let root = temp("unwritten");
        let me = account(ME, 1);
        let mut mine = FileJournal::open(&root, "Team", &me).unwrap();
        let (ledger, _) = ledger_opening::open(&mut mine, me.clone());
        // 空き容量ゼロの真似: 追記が 4 回失敗する（1 回目 ＋ 開き直しの再試行で 2 回ずつ）。
        mine.own_mut().fail_next(4);
        assert_eq!(mine.append(&create(1, ME)), Err(JournalFailure::Unwritable));
        assert_eq!(mine.unwritten(), 1);
        assert_eq!(
            mine.checkpoint(&ledger.snapshot()),
            Err(JournalFailure::Unwritable)
        );
        assert_eq!(mine.append(&create(2, ME)), Err(JournalFailure::Unwritable));
        assert_eq!(mine.unwritten(), 2);
        // 書けるようになれば、同期で古い順に書き直す。
        mine.sync().unwrap();
        assert_eq!(mine.unwritten(), 0);
        mine.append(&create(3, ME)).unwrap();
        mine.sync().unwrap();
        mine.checkpoint(&ledger.snapshot()).unwrap();
        mine.finish_checkpoint().unwrap();
        let path = mine
            .folder()
            .journal_directory()
            .join(own_journal::file_name(ME, JournalVersion::One));
        let read = journal_file::read_since(&path, 0).unwrap();
        assert_eq!(read.ops(), [create(1, ME), create(2, ME), create(3, ME)]);
        assert!(read.rejected().is_empty());
        let _cleaned = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn temporaries_of_my_account_and_the_owners_files_are_discarded_on_open() {
        let root = temp("leftovers");
        let me = account(ME, 1);
        drop(FileJournal::open(&root, "Team", &me).unwrap());
        let mine = [
            root.join("snapshot")
                .join(format!("{}.v1.txt.tmp", ME.hex())),
            root.join("snapshot")
                .join(format!("{}.v1.idx.tmp", ME.hex())),
            root.join("docket.v1.txt.tmp"),
            root.join("permissions.v1.txt.tmp"),
            key_file::directory(&root).join(format!("{}.tmp", KeyRecord::file_name(ME))),
        ];
        let theirs = root
            .join("snapshot")
            .join(format!("{}.v1.txt.tmp", OTHER.hex()));
        let attachment = root.join("attachments").join("x").join("1-a.png.tmp");
        std::fs::create_dir_all(root.join("snapshot")).unwrap();
        std::fs::create_dir_all(attachment.parent().unwrap()).unwrap();
        for path in mine.iter().chain([&theirs, &attachment]) {
            std::fs::write(path, b"half").unwrap();
        }
        drop(FileJournal::open(&root, "Team", &me).unwrap());
        for path in &mine {
            assert!(!path.exists(), "{}", path.display());
        }
        assert!(theirs.exists());
        assert!(attachment.exists());
        let _cleaned = std::fs::remove_dir_all(&root);
    }

    fn docket_with_permissions(root: &Path, owner: &Account, editor: AccountId) {
        let mut before = FileJournal::open(root, "Team", owner).unwrap();
        before.append(&create(1, ME)).unwrap();
        before.sync().unwrap();
        drop(before);
        administration::apply(
            root,
            owner,
            &[AdminCommand::Permissions(true), AdminCommand::Allow(editor)],
        )
        .unwrap();
    }

    #[test]
    fn with_permissions_on_lines_are_signed_and_verified_and_names_come_from_keys() {
        let root = temp("signed");
        let owner = account(ME, 1);
        let editor = account(OTHER, 2);
        docket_with_permissions(&root, &owner, OTHER);
        let mut mine = FileJournal::open(&root, "Team", &owner).unwrap();
        assert!(mine.is_signed());
        assert!(mine.refresh_settled().unwrap().is_empty());
        // 版 1 の自分の行は読まず、捨てたものとして数える。
        assert_eq!(mine.admission().unverified_lines(), 1);
        assert!(mine.admission().may_edit());
        mine.append(&create(2, ME)).unwrap();
        mine.sync().unwrap();
        let mut theirs = FileJournal::open(&root, "Team", &editor).unwrap();
        assert_eq!(theirs.refresh_settled().unwrap(), vec![create(2, ME)]);
        theirs.append(&create(3, OTHER)).unwrap();
        theirs.sync().unwrap();
        assert_eq!(mine.refresh_settled().unwrap(), vec![create(3, OTHER)]);
        assert_eq!(
            mine.names(),
            [(ME, String::from("u1")), (OTHER, String::from("u2"))]
        );
        // 引き継ぎ（ADR 0013 の 3）: 古い ID は新しい名前で出る。鍵の無い ID へ辿れば自分の名前のまま。
        let newer = AccountId::new(0x44);
        administration::apply(&root, &owner, &[AdminCommand::Succeed(newer, OTHER)]).unwrap();
        mine.refresh_settled().unwrap();
        assert_eq!(
            mine.names(),
            [(ME, String::from("u1")), (OTHER, String::from("u2"))]
        );
        FileJournal::open(&root, "Team", &account(newer, 4)).unwrap();
        mine.refresh_settled().unwrap();
        assert_eq!(
            mine.names(),
            [
                (ME, String::from("u1")),
                (OTHER, String::from("u4")),
                (newer, String::from("u4"))
            ]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    /// 持ち主が台帳を作り、1 件書いてスナップショットを書く。
    fn checkpointed(root: &Path, owner: &Account) -> FileJournal {
        let mut mine = FileJournal::open(root, "Team", owner).unwrap();
        let (mut ledger, failure) =
            nene_docket_application::ledger_opening::open(&mut mine, owner.clone());
        assert_eq!(failure, None);
        let op = ledger.compose(WallClock::new(0), OpKind::Create(ItemId::new(ME, 1)));
        mine.append(&op).unwrap();
        mine.sync().unwrap();
        mine.checkpoint(&ledger.snapshot()).unwrap();
        mine.finish_checkpoint().unwrap();
        mine
    }

    #[test]
    fn a_checkpoint_is_resumed_on_the_next_open_and_only_new_lines_are_read() {
        let root = temp("resume");
        let owner = account(ME, 1);
        let other = account(OTHER, 2);
        drop(FileJournal::open(&root, "Team", &owner).unwrap());
        let mut theirs = FileJournal::open(&root, "Team", &other).unwrap();
        theirs.append(&create(1, OTHER)).unwrap();
        theirs.sync().unwrap();
        let mine = checkpointed(&root, &owner);
        assert!(!mine.is_resumed());
        assert_eq!(mine.indexed_ops(), 2);
        // 他人が 1 行足してから開き直す: スナップショットから戻り、伸びた分だけ届く。
        theirs.append(&create(5, OTHER)).unwrap();
        theirs.sync().unwrap();
        let mut again = FileJournal::open(&root, "Team", &owner).unwrap();
        assert!(again.is_resumed());
        assert_eq!(again.snapshot().map(|s| s.docket().len()), Some(2));
        assert_eq!(again.refresh_settled().unwrap(), vec![create(5, OTHER)]);
        assert_eq!(again.indexed_ops(), 3);
        // 索引から課題の操作を引ける（同じ課題を 2 度作った 2 行）。新しい順の読みと全部の読みも。
        assert_eq!(
            again.ops_of(ItemId::new(OTHER, 1)).unwrap(),
            vec![create(1, OTHER), create(5, OTHER)]
        );
        assert_eq!(again.recent_ops(2).unwrap().len(), 2);
        assert_eq!(again.all_ops().unwrap().len(), 3);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_checkpoint_is_dropped_when_the_permissions_change_or_a_journal_shrinks() {
        let root = temp("resume-dropped");
        let owner = account(ME, 1);
        drop(checkpointed(&root, &owner));
        assert!(
            FileJournal::open(&root, "Team", &owner)
                .unwrap()
                .is_resumed()
        );
        // 権限を変えると（署名が変わる）スナップショットは捨てられ、全部読む。
        administration::apply(&root, &owner, &[AdminCommand::Permissions(true)]).unwrap();
        assert!(
            !FileJournal::open(&root, "Team", &owner)
                .unwrap()
                .is_resumed()
        );
        // 署名付きで書き直した後にジャーナルが縮めば（手で消した）、やはり捨てる。
        let signed = checkpointed(&root, &owner);
        assert!(signed.is_signed());
        drop(signed);
        assert!(
            FileJournal::open(&root, "Team", &owner)
                .unwrap()
                .is_resumed()
        );
        let path = root
            .join("journal")
            .join(own_journal::file_name(ME, JournalVersion::Two));
        fs::write(&path, "").unwrap();
        assert!(
            !FileJournal::open(&root, "Team", &owner)
                .unwrap()
                .is_resumed()
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn attachments_are_copied_checked_and_located_through_the_port() {
        let root = temp("attachments");
        let owner = account(ME, 1);
        let mut mine = FileJournal::open(&root, "Team", &owner).unwrap();
        let source = root.join("memo.txt");
        fs::write(&source, b"hello").unwrap();
        let item = ItemId::new(ME, 1);
        let attachment = mine
            .copy_attachment(item, 1, source.to_str().unwrap())
            .unwrap();
        assert_eq!(attachment.name(), "memo.txt");
        assert_eq!(attachment.bytes(), 5);
        assert!(mine.attachment_is_intact(item, &attachment));
        let path = mine.attachment_path(item, &attachment);
        assert!(path.ends_with("1-memo.txt"), "{path}");
        fs::write(&path, b"HELLO").unwrap();
        assert!(!mine.attachment_is_intact(item, &attachment));
        assert_eq!(
            mine.copy_attachment(item, 2, root.join("missing").to_str().unwrap()),
            Err(JournalFailure::AttachmentUnreadable)
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_handed_over_docket_is_signed_by_the_next_owner_when_opened() {
        let root = temp("handover");
        let owner = account(ME, 1);
        let next = account(OTHER, 2);
        docket_with_permissions(&root, &owner, OTHER);
        FileJournal::open(&root, "Team", &next).unwrap();
        administration::apply(&root, &owner, &[AdminCommand::Owner(OTHER)]).unwrap();
        assert!(
            permissions_store::read(&root)
                .unwrap()
                .unwrap()
                .handover()
                .is_some()
        );
        // 前の持ち主も開けて、編集者として書ける。
        let previous = FileJournal::open(&root, "Team", &owner).unwrap();
        assert!(previous.admission().may_edit());
        assert_eq!(previous.owner(), Some(OTHER));
        let mut reopened = FileJournal::open(&root, "Team", &next).unwrap();
        assert_eq!(
            permissions_store::read(&root).unwrap().unwrap().handover(),
            None
        );
        assert!(reopened.permissions().is_some_and(|p| p.owner() == OTHER));
        reopened
            .administer(&[AdminCommand::Allow(AccountId::new(0x55))])
            .unwrap();
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_owner_administers_from_the_port_and_others_may_not() {
        let root = temp("administer");
        let owner = account(ME, 1);
        let mut mine = FileJournal::open(&root, "Team", &owner).unwrap();
        assert_eq!(mine.owner(), Some(ME));
        assert_eq!(mine.permissions(), None);
        mine.administer(&[AdminCommand::Permissions(true), AdminCommand::Allow(OTHER)])
            .unwrap();
        assert!(mine.is_signed());
        assert!(mine.permissions().is_some_and(|p| p.may_edit(OTHER)));
        mine.append(&create(1, ME)).unwrap();
        mine.sync().unwrap();
        assert!(
            root.join("journal")
                .join(own_journal::file_name(ME, JournalVersion::Two))
                .exists()
        );
        let mut theirs = FileJournal::open(&root, "Team", &account(OTHER, 2)).unwrap();
        assert_eq!(
            theirs.administer(&[AdminCommand::Permissions(false)]),
            Err(JournalFailure::PermissionsUnwritable)
        );
        assert_eq!(theirs.refresh_settled().unwrap(), vec![create(1, ME)]);
        mine.administer(&[AdminCommand::Permissions(false)])
            .unwrap();
        assert!(!mine.is_signed());
        assert_eq!(mine.permissions(), None);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn strangers_forgeries_and_twin_keys_are_dropped_counted_or_flagged() {
        let root = temp("unauthorized");
        let owner = account(ME, 1);
        let editor = account(OTHER, 2);
        let stranger = account(AccountId::new(0x33), 3);
        docket_with_permissions(&root, &owner, OTHER);
        let mut theirs = FileJournal::open(&root, "Team", &editor).unwrap();
        theirs.append(&create(3, OTHER)).unwrap();
        theirs.sync().unwrap();
        // 編集者でない人の行は捨てられ、本人には「編集者でない」と出る。
        let mut strangers = FileJournal::open(&root, "Team", &stranger).unwrap();
        assert!(!strangers.admission().may_edit());
        strangers.append(&create(4, stranger.id())).unwrap();
        strangers.sync().unwrap();
        let mut mine = FileJournal::open(&root, "Team", &owner).unwrap();
        assert_eq!(mine.refresh_settled().unwrap(), vec![create(3, OTHER)]);
        assert_eq!(mine.admission().unverified_lines(), 2);
        // 持ち主が編集者に加えれば、次に開いたときから通る。
        administration::apply(&root, &owner, &[AdminCommand::Allow(stranger.id())]).unwrap();
        let mut reopened = FileJournal::open(&root, "Team", &owner).unwrap();
        let all = reopened.refresh_settled().unwrap();
        assert!(all.contains(&create(4, stranger.id())));
        assert_eq!(all.len(), 2);
        // 改竄: 編集者の行の値を書き換え、検査和だけ合わせ直す。
        let path = reopened
            .folder()
            .journal_directory()
            .join(own_journal::file_name(OTHER, JournalVersion::Two));
        let text = fs::read_to_string(&path).unwrap();
        let (body, _) = text.trim_end().rsplit_once('\t').unwrap();
        let forged = body.replacen("\t3\t", "\t5\t", 1);
        fs::write(
            &path,
            format!(
                "{text}{forged}\t{:08x}\n",
                crate::crc32::crc32(forged.as_bytes())
            ),
        )
        .unwrap();
        assert!(reopened.refresh_settled().unwrap().is_empty());
        assert_eq!(reopened.admission().unverified_lines(), 2);
        // 同じ ID の別の鍵。
        let twin = FileJournal::open(&root, "Team", &account(OTHER, 9)).unwrap();
        assert!(twin.admission().key_mismatch());
        fs::remove_dir_all(&root).unwrap();
    }
}
