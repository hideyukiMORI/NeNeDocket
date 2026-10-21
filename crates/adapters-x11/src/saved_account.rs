//! このプロセスのアカウント（FR-009 / FR-010）。設定フォルダの `account.v2.txt` を読み、無ければ乱数の ID と鍵と `USER` の名前で作る。
//!
//! 版 2 が無く版 1（`account.v1.txt`・鍵が無い）があれば、鍵を作って版 2 に書き直す。版 1 は消さない（ARC-009 の移行）。
//! 置き場は `config_directory`（ADR 0008）。形は application の `account_file`。

use std::env;
use std::fs;
use std::io;
use std::path::Path;

use nene_docket_application::account::Account;
use nene_docket_application::account_file;
use nene_docket_application::account_reading::AccountReading;

use crate::adapter_failure::AdapterFailure;
use crate::config_directory;
use crate::random_id;

/// いま書く版のファイル名。版は名前にある（ARC-009）。
const FILE_NAME: &str = "account.v2.txt";
/// 読めるだけの前の版のファイル名。
const PREVIOUS_FILE_NAME: &str = "account.v1.txt";
/// 名前が環境に無いときの名前。
const DEFAULT_NAME: &str = "user";

/// 保存されたアカウントを読む。無ければ版 1 から移すか、作って保存する。読めない形なら失敗（黙って作り直さない）。
///
/// # Errors
///
/// 設定フォルダが決められない・ファイルが読めない形・読み書きできない・乱数を読めない。
pub fn load_or_create() -> Result<Account, AdapterFailure> {
    let path = config_directory::file_path(FILE_NAME)?;
    match fs::read_to_string(&path) {
        Ok(text) => match account_file::parse(&text) {
            Some(AccountReading::Complete(account)) => Ok(*account),
            Some(AccountReading::WithoutKey(..)) | None => {
                Err(AdapterFailure::BadAccountFile(path))
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            migrate_or_create(&path, &config_directory::file_path(PREVIOUS_FILE_NAME)?)
        }
        Err(error) => Err(AdapterFailure::Io(error)),
    }
}

/// 版 1 があれば鍵を足して版 2 に。無ければ新しく作る。
fn migrate_or_create(path: &Path, previous: &Path) -> Result<Account, AdapterFailure> {
    let (id, name) = match fs::read_to_string(previous) {
        Ok(text) => match account_file::parse(&text) {
            Some(AccountReading::WithoutKey(id, name)) => (id, name),
            Some(AccountReading::Complete(_)) | None => {
                return Err(AdapterFailure::BadAccountFile(previous.to_path_buf()));
            }
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => (
            random_id::fresh()?,
            env::var("USER").unwrap_or_else(|_absent| String::from(DEFAULT_NAME)),
        ),
        Err(error) => return Err(AdapterFailure::Io(error)),
    };
    let account = Account::new(id, name, random_id::fresh_key()?);
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    fs::write(path, account_file::encode(&account))?;
    Ok(account)
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;

    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let directory = std::env::temp_dir().join(format!(
            "nene-docket-adapters-{}-{name}",
            std::process::id()
        ));
        let _ignored = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn version_one_is_migrated_with_a_fresh_key_and_kept() {
        let directory = scratch("migrate");
        let current = directory.join(FILE_NAME);
        let previous = directory.join(PREVIOUS_FILE_NAME);
        let id = AccountId::new(0xABCD);
        fs::write(
            &previous,
            format!("version = 1\nid = {}\nname = mori\n", id.hex()),
        )
        .unwrap();
        let migrated = migrate_or_create(&current, &previous).unwrap();
        assert_eq!(migrated.id(), id);
        assert_eq!(migrated.name(), "mori");
        assert!(previous.exists());
        assert_eq!(
            account_file::parse(&fs::read_to_string(&current).unwrap()),
            Some(AccountReading::Complete(Box::new(migrated)))
        );
        fs::write(&previous, "garbage").unwrap();
        assert!(matches!(
            migrate_or_create(&directory.join("other.v2.txt"), &previous),
            Err(AdapterFailure::BadAccountFile(_))
        ));
        let _ignored = fs::remove_dir_all(&directory);
    }

    #[test]
    fn without_any_file_a_fresh_account_is_created_and_written() {
        let directory = scratch("create");
        let current = directory.join(FILE_NAME);
        let created = migrate_or_create(&current, &directory.join(PREVIOUS_FILE_NAME)).unwrap();
        assert_eq!(
            account_file::parse(&fs::read_to_string(&current).unwrap()),
            Some(AccountReading::Complete(Box::new(created)))
        );
        let _ignored = fs::remove_dir_all(&directory);
    }
}
