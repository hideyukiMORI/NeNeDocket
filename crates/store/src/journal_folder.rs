//! 台帳のフォルダ。`docket.v1.txt` と `journal/` の置き場（ADR 0004 の 1）。

use std::fs;
use std::path::{Path, PathBuf};

use nene_docket_core::account_id::AccountId;

use crate::docket_file::DocketFile;
use crate::journal_version::JournalVersion;
use crate::own_journal;
use crate::store_failure::StoreFailure;

/// ジャーナルを置く下のフォルダ。
const JOURNAL_DIRECTORY: &str = "journal";

/// 開いた台帳のフォルダ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalFolder {
    root: PathBuf,
    docket: DocketFile,
}

impl JournalFolder {
    /// フォルダを開く。`docket.v1.txt` が無ければ `name` で作り、`creator` を持ち主にする（権限はまだ無効）。在れば版を確かめる。
    ///
    /// # Errors
    ///
    /// 作れない・読めない・版が違う。
    pub fn open_or_create(
        root: &Path,
        name: &str,
        creator: AccountId,
    ) -> Result<Self, StoreFailure> {
        let docket = if let Some(existing) = DocketFile::read_from(root)? {
            existing
        } else {
            let created = DocketFile::named(name).owned_by(creator);
            created.write_to(root)?;
            created
        };
        fs::create_dir_all(root.join(JOURNAL_DIRECTORY))?;
        Ok(Self {
            root: root.to_path_buf(),
            docket,
        })
    }

    /// 台帳のフォルダ。
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 台帳の名前。
    #[must_use]
    pub fn name(&self) -> &str {
        self.docket.name()
    }

    /// `docket.v1.txt` の中身（持ち主・権限の有無）。
    #[must_use]
    pub const fn docket(&self) -> &DocketFile {
        &self.docket
    }

    /// ジャーナルを置くフォルダ。
    #[must_use]
    pub fn journal_directory(&self) -> PathBuf {
        self.root.join(JOURNAL_DIRECTORY)
    }

    /// 在るジャーナルを、アカウントと版の順に列挙する。名前が形に合わないファイルは失敗として返す（黙って飛ばさない）。
    ///
    /// # Errors
    ///
    /// 読めない・ジャーナルでない名前のファイルがある。
    pub fn journals(&self) -> Result<Vec<(AccountId, JournalVersion, PathBuf)>, StoreFailure> {
        let mut found = Vec::new();
        for entry in fs::read_dir(self.journal_directory())? {
            let path = entry?.path();
            let name = path
                .file_name()
                .and_then(|raw| raw.to_str())
                .unwrap_or("")
                .to_owned();
            let (account, version) =
                own_journal::account_of(&name).ok_or(StoreFailure::BadJournalName(name))?;
            found.push((account, version, path));
        }
        found.sort_by_key(|(account, version, _)| (*account, *version));
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ME: AccountId = AccountId::new(9);

    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("nene-docket-store-{}-{name}", std::process::id()));
        let _ignored = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn creates_the_docket_file_once_with_the_creator_as_owner() {
        let root = scratch("folder");
        let folder = JournalFolder::open_or_create(&root, "team", ME).unwrap();
        assert_eq!(folder.name(), "team");
        assert_eq!(folder.root(), root);
        assert_eq!(folder.docket().owner(), Some(ME));
        assert!(!folder.docket().permissions_on());
        assert!(folder.journals().unwrap().is_empty());
        let reopened =
            JournalFolder::open_or_create(&root, "other name", AccountId::new(1)).unwrap();
        assert_eq!(reopened.name(), "team");
        assert_eq!(reopened.docket().owner(), Some(ME));
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn lists_journals_by_account_and_version_and_reports_strays() {
        let root = scratch("listing");
        let folder = JournalFolder::open_or_create(&root, "team", ME).unwrap();
        let b = AccountId::new(2);
        let a = AccountId::new(1);
        for (account, version) in [
            (b, JournalVersion::One),
            (a, JournalVersion::Two),
            (a, JournalVersion::One),
        ] {
            fs::write(
                folder
                    .journal_directory()
                    .join(own_journal::file_name(account, version)),
                "",
            )
            .unwrap();
        }
        let listed: Vec<(AccountId, JournalVersion)> = folder
            .journals()
            .unwrap()
            .into_iter()
            .map(|(id, version, _)| (id, version))
            .collect();
        assert_eq!(
            listed,
            [
                (a, JournalVersion::One),
                (a, JournalVersion::Two),
                (b, JournalVersion::One)
            ]
        );
        fs::write(folder.journal_directory().join("stray.txt"), "").unwrap();
        assert!(
            matches!(folder.journals(), Err(StoreFailure::BadJournalName(name)) if name == "stray.txt")
        );
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_newer_docket_version_is_refused() {
        let root = scratch("newer");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("docket.v1.txt"), "version = 7\nname = x\n").unwrap();
        assert!(matches!(
            JournalFolder::open_or_create(&root, "x", ME),
            Err(StoreFailure::UnknownDocketVersion(v)) if v == "7"
        ));
        let _ignored = fs::remove_dir_all(&root);
    }
}
