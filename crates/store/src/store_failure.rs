//! 置き場とファイルの失敗。閉じた集合（ARC-010）。黙って既定値へ落とさない（ARC-009）。

use core::fmt;
use std::io;

/// フォルダとジャーナルで起こりうる失敗。合成ルートと画面はこれを 1 行で示す。
#[derive(Debug)]
pub enum StoreFailure {
    /// フォルダを作れない・ファイルを開けない・書けない。
    Io(io::Error),
    /// `docket.v1.txt` の版を知らない（新しい版の台帳）。
    UnknownDocketVersion(String),
    /// `docket.v1.txt` の形が読めない。
    BadDocketFile,
    /// ジャーナルのファイル名からアカウントが読めない。
    BadJournalName(String),
}

impl fmt::Display for StoreFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "file error: {error}"),
            Self::UnknownDocketVersion(version) => {
                write!(formatter, "unknown docket version `{version}`")
            }
            Self::BadDocketFile => formatter.write_str("docket.v1.txt is not readable"),
            Self::BadJournalName(name) => write!(formatter, "not a journal file name: `{name}`"),
        }
    }
}

impl From<io::Error> for StoreFailure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
