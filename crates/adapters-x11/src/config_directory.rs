//! 設定フォルダ。`$XDG_CONFIG_HOME/nene-docket/`、無ければ `$HOME/.config/nene-docket/`。アカウントと設定のファイルの置き場（ADR 0008）。

use std::env;
use std::path::PathBuf;

use crate::adapter_failure::AdapterFailure;

/// 設定フォルダの下の名前。
const DIRECTORY: &str = "nene-docket";

/// `name` のファイルの道。フォルダはまだ作らない。
pub(crate) fn file_path(name: &str) -> Result<PathBuf, AdapterFailure> {
    let base = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or(AdapterFailure::NoConfigDirectory)?;
    Ok(base.join(DIRECTORY).join(name))
}
