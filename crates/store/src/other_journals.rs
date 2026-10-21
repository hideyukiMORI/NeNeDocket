//! 他人のジャーナルの読み直し。アカウントごとに読んだ位置を覚え、伸びた分だけ読む（ADR 0004 の 8）。

use std::collections::BTreeMap;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::op::Op;

use crate::journal_file;
use crate::journal_folder::JournalFolder;
use crate::record_failure::RecordFailure;
use crate::store_failure::StoreFailure;

/// 自分のアカウントと、他人ごとの次に読む位置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherJournals {
    own: AccountId,
    offsets: BTreeMap<AccountId, u64>,
    rejected: Vec<(AccountId, u64, RecordFailure)>,
}

impl OtherJournals {
    /// まだ何も読んでいない。
    #[must_use]
    pub const fn fresh(own: AccountId) -> Self {
        Self {
            own,
            offsets: BTreeMap::new(),
            rejected: Vec::new(),
        }
    }

    /// 他人のジャーナルの伸びた分を読み、新しい操作を返す。書きかけの末尾は次回に回る。壊れた行は覚えて `rejected` で見せる。
    ///
    /// # Errors
    ///
    /// フォルダを読めない・ジャーナルを開けない。
    pub fn refresh(&mut self, folder: &JournalFolder) -> Result<Vec<Op>, StoreFailure> {
        let mut fresh = Vec::new();
        for (account, path) in folder.journals()? {
            if account == self.own {
                continue;
            }
            let offset = self.offsets.get(&account).copied().unwrap_or(0);
            let read = journal_file::read_since(&path, offset)?;
            self.offsets.insert(account, read.next_offset());
            self.rejected.extend(
                read.rejected()
                    .iter()
                    .map(|(position, failure)| (account, *position, *failure)),
            );
            fresh.extend(read.into_ops());
        }
        Ok(fresh)
    }

    /// これまでに見た壊れた行（アカウント・位置・理由）。
    #[must_use]
    pub fn rejected(&self) -> &[(AccountId, u64, RecordFailure)] {
        &self.rejected
    }

    /// 読んだことのある他人のアカウント。
    #[must_use]
    pub fn seen_accounts(&self) -> Vec<AccountId> {
        self.offsets.keys().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::merge;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;
    use crate::own_journal::OwnJournal;

    const ME: AccountId = AccountId::new(1);
    const OTHER: AccountId = AccountId::new(2);

    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("nene-docket-store-{}-{name}", std::process::id()));
        let _ignored = fs::remove_dir_all(&directory);
        directory
    }

    fn op(account: AccountId, clock: u64) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), account),
            WallClock::new(0),
            OpKind::Create(ItemId::new(account, clock.try_into().unwrap())),
        )
    }

    #[test]
    fn only_new_lines_of_other_accounts_come_back_and_the_merge_matches() {
        let root = scratch("others");
        let folder = JournalFolder::open_or_create(&root, "t").unwrap();
        let (mut mine, _) = OwnJournal::open(&folder.journal_directory(), ME).unwrap();
        let (mut theirs, _) = OwnJournal::open(&folder.journal_directory(), OTHER).unwrap();
        mine.append(&op(ME, 1)).unwrap();
        theirs.append(&op(OTHER, 2)).unwrap();
        theirs.sync().unwrap();
        let mut others = OtherJournals::fresh(ME);
        let first = others.refresh(&folder).unwrap();
        assert_eq!(first, [op(OTHER, 2)]);
        assert!(others.refresh(&folder).unwrap().is_empty());
        theirs.append(&op(OTHER, 3)).unwrap();
        theirs.sync().unwrap();
        let second = others.refresh(&folder).unwrap();
        assert_eq!(second, [op(OTHER, 3)]);
        assert_eq!(others.seen_accounts(), [OTHER]);
        let all: Vec<Op> = vec![op(ME, 1), op(OTHER, 2), op(OTHER, 3)];
        let from_files: Vec<Op> = first.into_iter().chain(second).chain([op(ME, 1)]).collect();
        assert_eq!(merge::merge(&from_files), merge::merge(&all));
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_torn_tail_waits_and_a_broken_line_is_remembered() {
        let root = scratch("torn");
        let folder = JournalFolder::open_or_create(&root, "t").unwrap();
        let path = folder
            .journal_directory()
            .join(crate::own_journal::file_name(OTHER));
        let good = crate::journal_record::encode(&op(OTHER, 1));
        fs::write(&path, format!("{good}\nbroken line\n{}", &good[..5])).unwrap();
        let mut others = OtherJournals::fresh(ME);
        assert_eq!(others.refresh(&folder).unwrap(), [op(OTHER, 1)]);
        assert_eq!(others.rejected().len(), 1);
        assert_eq!(others.rejected().first().map(|r| r.0), Some(OTHER));
        // 書きかけが完成した。
        fs::write(
            &path,
            format!(
                "{good}\nbroken line\n{}\n",
                crate::journal_record::encode(&op(OTHER, 2))
            ),
        )
        .unwrap();
        assert_eq!(others.refresh(&folder).unwrap(), [op(OTHER, 2)]);
        assert_eq!(others.rejected().len(), 1);
        let _ignored = fs::remove_dir_all(&root);
    }
}
