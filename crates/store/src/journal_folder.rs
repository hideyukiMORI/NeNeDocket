//! 台帳のフォルダ。`docket.v1.txt` と `journal/` の置き場（ADR 0004 の 1）。

use std::fs;
use std::path::{Path, PathBuf};

use nene_docket_core::account_id::AccountId;

use crate::docket_file::DocketFile;
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
    /// フォルダを開く。`docket.v1.txt` が無ければ `name` で作る。在れば版を確かめる。
    ///
    /// # Errors
    ///
    /// 作れない・読めない・版が違う。
    pub fn open_or_create(root: &Path, name: &str) -> Result<Self, StoreFailure> {
        let docket = if let Some(existing) = DocketFile::read_from(root)? {
            existing
        } else {
            let created = DocketFile::named(name);
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

    /// ジャーナルを置くフォルダ。
    #[must_use]
    pub fn journal_directory(&self) -> PathBuf {
        self.root.join(JOURNAL_DIRECTORY)
    }

    /// 在るジャーナルを、アカウントの順に列挙する。名前が形に合わないファイルは失敗として返す（黙って飛ばさない）。
    ///
    /// # Errors
    ///
    /// 読めない・ジャーナルでない名前のファイルがある。
    pub fn journals(&self) -> Result<Vec<(AccountId, PathBuf)>, StoreFailure> {
        let mut found = Vec::new();
        for entry in fs::read_dir(self.journal_directory())? {
            let path = entry?.path();
            let name = path
                .file_name()
                .and_then(|raw| raw.to_str())
                .unwrap_or("")
                .to_owned();
            let account =
                own_journal::account_of(&name).ok_or(StoreFailure::BadJournalName(name))?;
            found.push((account, path));
        }
        found.sort_by_key(|(account, _)| *account);
        Ok(found)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("nene-docket-store-{}-{name}", std::process::id()));
        let _ignored = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn creates_the_docket_file_once_and_lists_journals_by_account() {
        let root = scratch("folder");
        let folder = JournalFolder::open_or_create(&root, "team").unwrap();
        assert_eq!(folder.name(), "team");
        assert_eq!(folder.root(), root);
        assert!(folder.journals().unwrap().is_empty());
        let reopened = JournalFolder::open_or_create(&root, "other name").unwrap();
        assert_eq!(reopened.name(), "team");
        let b = AccountId::new(2);
        let a = AccountId::new(1);
        for account in [b, a] {
            fs::write(
                folder
                    .journal_directory()
                    .join(own_journal::file_name(account)),
                "",
            )
            .unwrap();
        }
        let listed: Vec<AccountId> = folder
            .journals()
            .unwrap()
            .into_iter()
            .map(|(id, _)| id)
            .collect();
        assert_eq!(listed, [a, b]);
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
            JournalFolder::open_or_create(&root, "x"),
            Err(StoreFailure::UnknownDocketVersion(v)) if v == "7"
        ));
        let _ignored = fs::remove_dir_all(&root);
    }
}
