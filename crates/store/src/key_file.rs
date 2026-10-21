//! 台帳の `keys/`。アカウントごとの公開鍵と名前（FR-009 / FR-010・ADR 0010）。形は application の `key_record`。
//!
//! 自分の分は台帳を開くときに置く（無いときだけ）。既にあって鍵が違えば「食い違い」として返し、上書きしない（FR-014）。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nene_docket_application::account::Account;
use nene_docket_application::key_record::KeyRecord;

use crate::replace;
use crate::store_failure::StoreFailure;

/// 鍵を置く下のフォルダ。
const KEYS_DIRECTORY: &str = "keys";

/// `keys/` の場所。
#[must_use]
pub fn directory(root: &Path) -> PathBuf {
    root.join(KEYS_DIRECTORY)
}

/// 自分の鍵を置く。既にあって同じ鍵なら何もしない。鍵が違えば `Ok(true)`（食い違い）。
///
/// # Errors
///
/// 読めない・書けない・既にある分が読めない形。
pub fn ensure_own(root: &Path, account: &Account) -> Result<bool, StoreFailure> {
    let directory = directory(root);
    let path = directory.join(KeyRecord::file_name(account.id()));
    match fs::read_to_string(&path) {
        Ok(text) => {
            let existing = KeyRecord::parse(&text)
                .ok_or_else(|| StoreFailure::BadKeyFile(file_name_of(&path)))?;
            Ok(existing.public() != account.public())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            fs::create_dir_all(&directory)?;
            replace::replace_file(&path, KeyRecord::of(account).encode().as_bytes())?;
            Ok(false)
        }
        Err(error) => Err(StoreFailure::Io(error)),
    }
}

/// `keys/` の全部を、アカウントの順に読む。フォルダが無ければ空。名前か中身が形に合わないファイルは失敗（黙って飛ばさない）。
///
/// # Errors
///
/// 読めない・鍵のファイルでない名前・読めない形・名前の ID と中の ID が違う。
pub fn read_all(root: &Path) -> Result<Vec<KeyRecord>, StoreFailure> {
    let entries = match fs::read_dir(directory(root)) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(StoreFailure::Io(error)),
    };
    let mut records = Vec::new();
    for entry in entries {
        let path = entry?.path();
        let name = file_name_of(&path);
        let account =
            KeyRecord::account_of(&name).ok_or_else(|| StoreFailure::BadKeyFile(name.clone()))?;
        let record = KeyRecord::parse(&fs::read_to_string(&path)?)
            .filter(|record| record.id() == account)
            .ok_or(StoreFailure::BadKeyFile(name))?;
        records.push(record);
    }
    records.sort_by_key(KeyRecord::id);
    Ok(records)
}

fn file_name_of(path: &Path) -> String {
    path.file_name()
        .and_then(|raw| raw.to_str())
        .unwrap_or("")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::signing_key::SigningKey;

    use super::*;

    fn account(id: u128, seed: u8) -> Account {
        Account::new(
            AccountId::new(id),
            format!("user{id}"),
            SigningKey::from_bytes(&[seed; 32]),
        )
    }

    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("nene-docket-store-{}-{name}", std::process::id()));
        let _ignored = fs::remove_dir_all(&directory);
        directory
    }

    #[test]
    fn own_key_is_written_once_read_back_and_a_different_key_is_a_mismatch() {
        let root = scratch("keys");
        assert!(read_all(&root).unwrap().is_empty());
        let me = account(2, 1);
        let other = account(1, 2);
        assert!(!ensure_own(&root, &me).unwrap());
        assert!(!ensure_own(&root, &me).unwrap());
        assert!(!ensure_own(&root, &other).unwrap());
        let records = read_all(&root).unwrap();
        assert_eq!(records, [KeyRecord::of(&other), KeyRecord::of(&me)]);
        assert!(ensure_own(&root, &account(2, 9)).unwrap());
        assert_eq!(read_all(&root).unwrap().len(), 2);
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_stray_file_a_broken_record_or_a_renamed_record_is_reported() {
        let root = scratch("bad-keys");
        let me = account(2, 1);
        ensure_own(&root, &me).unwrap();
        let stray = directory(&root).join("notes.txt");
        fs::write(&stray, "").unwrap();
        assert!(
            matches!(read_all(&root), Err(StoreFailure::BadKeyFile(name)) if name == "notes.txt")
        );
        fs::remove_file(&stray).unwrap();
        let renamed = directory(&root).join(KeyRecord::file_name(AccountId::new(3)));
        fs::write(&renamed, KeyRecord::of(&me).encode()).unwrap();
        assert!(matches!(read_all(&root), Err(StoreFailure::BadKeyFile(_))));
        fs::write(&renamed, "garbage").unwrap();
        assert!(matches!(read_all(&root), Err(StoreFailure::BadKeyFile(_))));
        fs::write(
            directory(&root).join(KeyRecord::file_name(me.id())),
            "garbage",
        )
        .unwrap();
        assert!(matches!(
            ensure_own(&root, &me),
            Err(StoreFailure::BadKeyFile(_))
        ));
        let _ignored = fs::remove_dir_all(&root);
    }
}
