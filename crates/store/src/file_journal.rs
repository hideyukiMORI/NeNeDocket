//! application の `JournalPort` の実装。自分のジャーナルへの追記・同期と、他人のジャーナルの読み直しを 1 つに束ねる。
//! 権限を有効にした台帳では版 2 に署名して書き、読む行（自分の分も）は検証器に通す（ADR 0010）。

use std::path::Path;

use nene_docket_application::account::Account;
use nene_docket_application::admission_report::AdmissionReport;
use nene_docket_application::journal_failure::JournalFailure;
use nene_docket_application::journal_port::JournalPort;
use nene_docket_application::key_record::KeyRecord;
use nene_docket_core::account_id::AccountId;
use nene_docket_core::admin_command::AdminCommand;
use nene_docket_core::op::Op;
use nene_docket_core::permissions::Permissions;

use crate::administration;
use crate::journal_folder::JournalFolder;
use crate::journal_version::JournalVersion;
use crate::key_file;
use crate::other_journals::OtherJournals;
use crate::own_journal::{self, OwnJournal};
use crate::store_failure::StoreFailure;
use crate::verifier::{self, Verifier};

/// 台帳のフォルダ・自分のジャーナル・他人の読み位置・起動時に読んだ自分の操作（最初の `refresh` で渡す）・鍵と検証器。
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
}

impl FileJournal {
    /// フォルダを開く（無ければ `name` で作る）。自分の鍵を `keys/` に置き、権限が有効なら検証器を読み、
    /// 自分のジャーナルを読む（壊れた末尾は捨てて数え、検証の通らない行も数える）。
    ///
    /// # Errors
    ///
    /// フォルダを作れない・読めない・版が違う・ジャーナルを開けない・鍵や権限のファイルが読めない。
    pub fn open(root: &Path, name: &str, account: &Account) -> Result<Self, StoreFailure> {
        let mut folder = JournalFolder::open_or_create(root, name, account.id())?;
        let key_mismatch = key_file::ensure_own(root, account)?;
        if !key_mismatch && administration::adopt(root, account)? {
            folder = JournalFolder::open_or_create(root, name, account.id())?;
        }
        let enabled = folder.docket().permissions_on();
        let (keys, verifier) =
            verifier::load_if_enabled(root, folder.docket().owner().filter(|_| enabled))?;
        let signing = verifier.as_ref().map(|_| account.key().clone());
        let (own, read) = OwnJournal::open(&folder.journal_directory(), account.id(), signing)?;
        let own_rejected = read.rejected().len();
        let mut own_unverified = 0_usize;
        let mut pending = Vec::new();
        for entry in read.into_entries() {
            match verifier::admit(verifier.as_ref(), account.id(), entry) {
                Ok(op) => pending.push(op),
                Err(_failure) => own_unverified = own_unverified.saturating_add(1),
            }
        }
        if verifier.is_some() {
            own_unverified =
                own_unverified.saturating_add(Self::count_unsigned_own(&folder, account.id())?);
        }
        Ok(Self {
            folder,
            account: account.clone(),
            own,
            others: OtherJournals::fresh(account.id()),
            pending,
            own_rejected,
            own_unverified,
            keys,
            verifier,
            key_mismatch,
        })
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
        Ok(crate::journal_file::read_since(&path, 0)?.entries().len())
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

    /// `keys/` と権限を読み直す（他人が参加した・持ち主が権限を変えた）。
    fn reload_verifier(&mut self) -> Result<(), StoreFailure> {
        let owner = self
            .folder
            .docket()
            .owner()
            .filter(|_| self.verifier.is_some());
        let (keys, verifier) = verifier::load_if_enabled(self.folder.root(), owner)?;
        self.keys = keys;
        self.verifier = verifier;
        Ok(())
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
        Ok(())
    }
}

impl JournalPort for FileJournal {
    fn append(&mut self, op: &Op) -> Result<(), JournalFailure> {
        self.own
            .append(op)
            .map_err(|_failure| JournalFailure::Unwritable)
    }

    fn sync(&mut self) -> Result<(), JournalFailure> {
        self.own
            .sync()
            .map_err(|_failure| JournalFailure::Unwritable)
    }

    fn refresh(&mut self) -> Result<Vec<Op>, JournalFailure> {
        let mut ops = core::mem::take(&mut self.pending);
        self.reload_verifier()
            .map_err(|_failure| JournalFailure::Unreadable)?;
        let fresh = self
            .others
            .refresh(&self.folder, self.verifier.as_ref())
            .map_err(|_failure| JournalFailure::Unreadable)?;
        ops.extend(fresh);
        Ok(ops)
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
        assert!(first.refresh().unwrap().is_empty());
        assert!(!first.is_signed());
        assert_eq!(first.admission(), AdmissionReport::open());
        assert_eq!(first.names(), [(ME, String::from("u1"))]);
        first.append(&create(1, ME)).unwrap();
        first.sync().unwrap();
        drop(first);

        let mut again = FileJournal::open(&root, "Team", &me).unwrap();
        assert_eq!(again.folder().name(), "Team");
        assert_eq!(again.refresh().unwrap(), vec![create(1, ME)]);
        let theirs = again
            .folder()
            .journal_directory()
            .join(own_journal::file_name(OTHER, JournalVersion::One));
        let mut line = crate::journal_record::encode(&create(2, OTHER));
        line.push('\n');
        fs::write(&theirs, line).unwrap();
        assert_eq!(again.refresh().unwrap(), vec![create(2, OTHER)]);
        assert!(again.refresh().unwrap().is_empty());
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
        assert!(journal.refresh().unwrap().is_empty());
        assert_eq!(journal.rejected_lines(), 1);
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(journal.refresh(), Err(JournalFailure::Unreadable));
        assert_eq!(journal.append(&create(1, ME)), Ok(()));
    }

    /// 権限の無いうちに持ち主が 1 行書き、権限を有効にして編集者を 1 人加える。
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
        assert!(mine.refresh().unwrap().is_empty());
        // 版 1 の自分の行は読まず、捨てたものとして数える。
        assert_eq!(mine.admission().unverified_lines(), 1);
        assert!(mine.admission().may_edit());
        mine.append(&create(2, ME)).unwrap();
        mine.sync().unwrap();
        let mut theirs = FileJournal::open(&root, "Team", &editor).unwrap();
        assert_eq!(theirs.refresh().unwrap(), vec![create(2, ME)]);
        theirs.append(&create(3, OTHER)).unwrap();
        theirs.sync().unwrap();
        assert_eq!(mine.refresh().unwrap(), vec![create(3, OTHER)]);
        assert_eq!(
            mine.names(),
            [(ME, String::from("u1")), (OTHER, String::from("u2"))]
        );
        // 引き継ぎ（ADR 0013 の 3）: 古い ID は新しい名前で出る。鍵の無い ID へ辿れば自分の名前のまま。
        let newer = AccountId::new(0x44);
        administration::apply(&root, &owner, &[AdminCommand::Succeed(newer, OTHER)]).unwrap();
        mine.refresh().unwrap();
        assert_eq!(
            mine.names(),
            [(ME, String::from("u1")), (OTHER, String::from("u2"))]
        );
        FileJournal::open(&root, "Team", &account(newer, 4)).unwrap();
        mine.refresh().unwrap();
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
        assert_eq!(theirs.refresh().unwrap(), vec![create(1, ME)]);
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
        assert_eq!(mine.refresh().unwrap(), vec![create(3, OTHER)]);
        assert_eq!(mine.admission().unverified_lines(), 2);
        // 持ち主が編集者に加えれば、次に開いたときから通る。
        administration::apply(&root, &owner, &[AdminCommand::Allow(stranger.id())]).unwrap();
        let mut reopened = FileJournal::open(&root, "Team", &owner).unwrap();
        let all = reopened.refresh().unwrap();
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
        assert!(reopened.refresh().unwrap().is_empty());
        assert_eq!(reopened.admission().unverified_lines(), 2);
        // 同じ ID の別の鍵。
        let twin = FileJournal::open(&root, "Team", &account(OTHER, 9)).unwrap();
        assert!(twin.admission().key_mismatch());
        fs::remove_dir_all(&root).unwrap();
    }
}
