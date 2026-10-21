//! 他人のジャーナルの読み直し。アカウントごとに読んだ位置を覚え、伸びた分だけ読む（ADR 0004 の 8）。
//! 権限を有効にした台帳では、読んだ行を検証器に通し、通らない行は捨てて数える（ADR 0010 の 4）。

use std::collections::BTreeMap;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::op::Op;

use crate::admission_failure::AdmissionFailure;
use crate::journal_file;
use crate::journal_folder::JournalFolder;
use crate::journal_version::JournalVersion;
use crate::record_failure::RecordFailure;
use crate::store_failure::StoreFailure;
use crate::verifier::{self, Verifier};

/// 読んだ操作と、それがどのジャーナル（アカウントと版）の何バイト目か。
pub type LocatedOp = ((AccountId, JournalVersion), u64, Op);

/// 自分のアカウントと、他人のジャーナル（アカウントと版）ごとの次に読む位置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtherJournals {
    own: AccountId,
    offsets: BTreeMap<(AccountId, JournalVersion), u64>,
    rejected: Vec<(AccountId, u64, RecordFailure)>,
    unverified: Vec<(AccountId, AdmissionFailure)>,
}

impl OtherJournals {
    /// まだ何も読んでいない。
    #[must_use]
    pub const fn fresh(own: AccountId) -> Self {
        Self {
            own,
            offsets: BTreeMap::new(),
            rejected: Vec::new(),
            unverified: Vec::new(),
        }
    }

    /// スナップショットの読み位置から（ADR 0015）。
    #[must_use]
    pub const fn from_offsets(
        own: AccountId,
        offsets: BTreeMap<(AccountId, JournalVersion), u64>,
    ) -> Self {
        Self {
            own,
            offsets,
            rejected: Vec::new(),
            unverified: Vec::new(),
        }
    }

    /// 読み終えた位置（アカウント・版ごと）。
    #[must_use]
    pub const fn offsets(&self) -> &BTreeMap<(AccountId, JournalVersion), u64> {
        &self.offsets
    }

    /// 他人のジャーナルの伸びた分を読み、新しい操作を返す。書きかけの末尾は次回に回る。
    /// 壊れた行は覚えて `rejected` で、検証（`verifier` があるとき）の通らない行は `unverified` で見せる。
    ///
    /// # Errors
    ///
    /// フォルダを読めない・ジャーナルを開けない。
    pub fn refresh(
        &mut self,
        folder: &JournalFolder,
        verifier: Option<&Verifier>,
    ) -> Result<Vec<Op>, StoreFailure> {
        Ok(self
            .refresh_located(folder, verifier)?
            .into_iter()
            .map(|(_, _, op)| op)
            .collect())
    }

    /// 同じく読むが、各操作がどのジャーナルの何バイト目かも返す（索引に入れる・ADR 0015）。
    ///
    /// # Errors
    ///
    /// フォルダを読めない・ジャーナルを開けない。
    pub fn refresh_located(
        &mut self,
        folder: &JournalFolder,
        verifier: Option<&Verifier>,
    ) -> Result<Vec<LocatedOp>, StoreFailure> {
        let mut fresh = Vec::new();
        for (account, version, path) in folder.journals()? {
            if account == self.own {
                continue;
            }
            let offset = self.offsets.get(&(account, version)).copied().unwrap_or(0);
            let read = journal_file::read_since(&path, offset)?;
            self.offsets.insert((account, version), read.next_offset());
            self.rejected.extend(
                read.rejected()
                    .iter()
                    .map(|(position, failure)| (account, *position, *failure)),
            );
            for (position, entry) in read.into_located_entries() {
                match verifier::admit(verifier, account, entry) {
                    Ok(op) => fresh.push(((account, version), position, op)),
                    Err(failure) => self.unverified.push((account, failure)),
                }
            }
        }
        Ok(fresh)
    }

    /// これまでに見た壊れた行（アカウント・位置・理由）。
    #[must_use]
    pub fn rejected(&self) -> &[(AccountId, u64, RecordFailure)] {
        &self.rejected
    }

    /// これまでに検証で捨てた行（アカウント・理由）。
    #[must_use]
    pub fn unverified(&self) -> &[(AccountId, AdmissionFailure)] {
        &self.unverified
    }

    /// 読んだことのある他人のアカウント。
    #[must_use]
    pub fn seen_accounts(&self) -> Vec<AccountId> {
        let mut seen: Vec<AccountId> = self.offsets.keys().map(|(account, _)| *account).collect();
        seen.dedup();
        seen
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use nene_docket_application::account::Account;
    use nene_docket_application::key_record::KeyRecord;
    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::merge;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::permissions::Permissions;
    use nene_docket_core::signing_key::SigningKey;
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
        let folder = JournalFolder::open_or_create(&root, "t", ME).unwrap();
        let (mut mine, _) = OwnJournal::open(&folder.journal_directory(), ME, None).unwrap();
        let (mut theirs, _) = OwnJournal::open(&folder.journal_directory(), OTHER, None).unwrap();
        mine.append(&op(ME, 1)).unwrap();
        theirs.append(&op(OTHER, 2)).unwrap();
        theirs.sync().unwrap();
        let mut others = OtherJournals::fresh(ME);
        let first = others.refresh(&folder, None).unwrap();
        assert_eq!(first, [op(OTHER, 2)]);
        assert!(others.refresh(&folder, None).unwrap().is_empty());
        theirs.append(&op(OTHER, 3)).unwrap();
        theirs.sync().unwrap();
        let second = others.refresh(&folder, None).unwrap();
        assert_eq!(second, [op(OTHER, 3)]);
        assert_eq!(others.seen_accounts(), [OTHER]);
        assert!(others.unverified().is_empty());
        let all: Vec<Op> = vec![op(ME, 1), op(OTHER, 2), op(OTHER, 3)];
        let from_files: Vec<Op> = first.into_iter().chain(second).chain([op(ME, 1)]).collect();
        assert_eq!(merge::merge(&from_files), merge::merge(&all));
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_torn_tail_waits_and_a_broken_line_is_remembered() {
        let root = scratch("torn");
        let folder = JournalFolder::open_or_create(&root, "t", ME).unwrap();
        let path = folder
            .journal_directory()
            .join(crate::own_journal::file_name(OTHER, JournalVersion::One));
        let good = crate::journal_record::encode(&op(OTHER, 1));
        fs::write(&path, format!("{good}\nbroken line\n{}", &good[..5])).unwrap();
        let mut others = OtherJournals::fresh(ME);
        assert_eq!(others.refresh(&folder, None).unwrap(), [op(OTHER, 1)]);
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
        assert_eq!(others.refresh(&folder, None).unwrap(), [op(OTHER, 2)]);
        assert_eq!(others.rejected().len(), 1);
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn with_a_verifier_unsigned_and_unauthorized_lines_are_dropped_and_counted() {
        let root = scratch("verified");
        let folder = JournalFolder::open_or_create(&root, "t", ME).unwrap();
        let editor = Account::new(
            OTHER,
            String::from("o"),
            SigningKey::from_bytes(&[2_u8; 32]),
        );
        let stranger = Account::new(
            AccountId::new(3),
            String::from("s"),
            SigningKey::from_bytes(&[3_u8; 32]),
        );
        let directory = folder.journal_directory();
        let (mut signed, _) =
            OwnJournal::open(&directory, OTHER, Some(editor.key().clone())).unwrap();
        signed.append(&op(OTHER, 1)).unwrap();
        signed.sync().unwrap();
        let (mut unsigned, _) = OwnJournal::open(&directory, OTHER, None).unwrap();
        unsigned.append(&op(OTHER, 2)).unwrap();
        unsigned.sync().unwrap();
        let (mut strangers, _) =
            OwnJournal::open(&directory, stranger.id(), Some(stranger.key().clone())).unwrap();
        strangers.append(&op(stranger.id(), 3)).unwrap();
        strangers.sync().unwrap();
        let verifier = Verifier::new(
            Permissions::owner_only(ME).allowing(OTHER),
            &[KeyRecord::of(&editor), KeyRecord::of(&stranger)],
        );
        let mut others = OtherJournals::fresh(ME);
        assert_eq!(
            others.refresh(&folder, Some(&verifier)).unwrap(),
            [op(OTHER, 1)]
        );
        assert_eq!(
            others.unverified(),
            [
                (OTHER, AdmissionFailure::Unsigned),
                (stranger.id(), AdmissionFailure::NotAnEditor)
            ]
        );
        // 権限の無い台帳なら全部通る。
        let mut open = OtherJournals::fresh(ME);
        assert_eq!(open.refresh(&folder, None).unwrap().len(), 3);
        let _ignored = fs::remove_dir_all(&root);
    }
}
