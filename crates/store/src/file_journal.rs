//! application の `JournalPort` の実装。自分のジャーナルへの追記・同期と、他人のジャーナルの読み直しを 1 つに束ねる。

use std::path::Path;

use nene_docket_application::journal_failure::JournalFailure;
use nene_docket_application::journal_port::JournalPort;
use nene_docket_core::account_id::AccountId;
use nene_docket_core::op::Op;

use crate::journal_folder::JournalFolder;
use crate::other_journals::OtherJournals;
use crate::own_journal::OwnJournal;
use crate::store_failure::StoreFailure;

/// 台帳のフォルダ・自分のジャーナル・他人の読み位置・起動時に読んだ自分の操作（最初の `refresh` で渡す）。
#[derive(Debug)]
pub struct FileJournal {
    folder: JournalFolder,
    own: OwnJournal,
    others: OtherJournals,
    pending: Vec<Op>,
    own_rejected: usize,
}

impl FileJournal {
    /// フォルダを開く（無ければ `name` で作る）。自分のジャーナルを読み、壊れた末尾は捨てて数える。
    ///
    /// # Errors
    ///
    /// フォルダを作れない・読めない・版が違う・ジャーナルを開けない。
    pub fn open(root: &Path, name: &str, account: AccountId) -> Result<Self, StoreFailure> {
        let folder = JournalFolder::open_or_create(root, name)?;
        let (own, read) = OwnJournal::open(&folder.journal_directory(), account)?;
        let own_rejected = read.rejected().len();
        Ok(Self {
            folder,
            own,
            others: OtherJournals::fresh(account),
            pending: read.into_ops(),
            own_rejected,
        })
    }

    /// 台帳のフォルダ。
    #[must_use]
    pub const fn folder(&self) -> &JournalFolder {
        &self.folder
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
        let fresh = self
            .others
            .refresh(&self.folder)
            .map_err(|_failure| JournalFailure::Unreadable)?;
        ops.extend(fresh);
        Ok(ops)
    }

    fn rejected_lines(&self) -> usize {
        self.own_rejected
            .saturating_add(self.others.rejected().len())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;
    use crate::own_journal;

    const ME: AccountId = AccountId::new(0x11);
    const OTHER: AccountId = AccountId::new(0x22);

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
        let mut first = FileJournal::open(&root, "Team", ME).unwrap();
        assert!(first.refresh().unwrap().is_empty());
        first.append(&create(1, ME)).unwrap();
        first.sync().unwrap();
        drop(first);

        let mut again = FileJournal::open(&root, "Team", ME).unwrap();
        assert_eq!(again.folder().name(), "Team");
        assert_eq!(again.refresh().unwrap(), vec![create(1, ME)]);
        let theirs = again
            .folder()
            .journal_directory()
            .join(own_journal::file_name(OTHER));
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
        let mut journal = FileJournal::open(&root, "Team", ME).unwrap();
        let theirs = journal
            .folder()
            .journal_directory()
            .join(own_journal::file_name(OTHER));
        fs::write(&theirs, "v1\tnot a record\n").unwrap();
        assert!(journal.refresh().unwrap().is_empty());
        assert_eq!(journal.rejected_lines(), 1);
        fs::remove_dir_all(&root).unwrap();
        assert_eq!(journal.refresh(), Err(JournalFailure::Unreadable));
        assert_eq!(journal.append(&create(1, ME)), Ok(()));
    }
}
