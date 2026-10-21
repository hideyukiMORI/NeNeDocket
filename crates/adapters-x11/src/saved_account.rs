//! このプロセスのアカウント（FR-009）。設定フォルダの `account.v1.txt` を読み、無ければ乱数の ID と `USER` の名前で作る。
//!
//! 置き場は `config_directory`（ADR 0008）。形は application の `account_file`。

use std::env;
use std::fs;
use std::io;
use std::path::Path;

use nene_docket_application::account::Account;
use nene_docket_application::account_file;

use crate::adapter_failure::AdapterFailure;
use crate::config_directory;
use crate::random_id;

/// ファイル名。版は名前にある（ARC-009）。
const FILE_NAME: &str = "account.v1.txt";
/// 名前が環境に無いときの名前。
const DEFAULT_NAME: &str = "user";

/// 保存されたアカウントを読む。無ければ作って保存する。読めない形なら失敗（黙って作り直さない）。
///
/// # Errors
///
/// 設定フォルダが決められない・ファイルが読めない形・読み書きできない・乱数を読めない。
pub fn load_or_create() -> Result<Account, AdapterFailure> {
    let path = config_directory::file_path(FILE_NAME)?;
    match fs::read_to_string(&path) {
        Ok(text) => account_file::parse(&text).ok_or(AdapterFailure::BadAccountFile(path)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => create_at(&path),
        Err(error) => Err(AdapterFailure::Io(error)),
    }
}

fn create_at(path: &Path) -> Result<Account, AdapterFailure> {
    let name = env::var("USER").unwrap_or_else(|_absent| String::from(DEFAULT_NAME));
    let account = Account::new(random_id::fresh()?, name);
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    fs::write(path, account_file::encode(&account))?;
    Ok(account)
}
