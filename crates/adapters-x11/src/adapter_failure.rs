//! アダプタの失敗。閉じた集合（ARC-010）。合成ルートが 1 行で示す。

use core::fmt;
use std::io;
use std::path::PathBuf;

use x11rb::errors::{ConnectError, ConnectionError, ReplyError};

/// 起動時に起こりうる失敗。
#[derive(Debug)]
pub enum AdapterFailure {
    /// ファイルを読めない・書けない・乱数を読めない。
    Io(io::Error),
    /// 設定フォルダが決められない（`XDG_CONFIG_HOME` も `HOME` も無い）。
    NoConfigDirectory,
    /// アカウントのファイルが読めない形（版が違う・壊れている）。黙って作り直さない（FR-014）。
    BadAccountFile(PathBuf),
    /// 刻みの接続が X サーバに繋げなかった。
    Connect(ConnectError),
    /// 刻みの接続が X の要求を送れなかった。
    Connection(ConnectionError),
    /// 刻みの要求が X のエラーで返った。
    Reply(ReplyError),
}

impl fmt::Display for AdapterFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "file error: {error}"),
            Self::NoConfigDirectory => {
                formatter.write_str("neither XDG_CONFIG_HOME nor HOME is set")
            }
            Self::BadAccountFile(path) => {
                write!(
                    formatter,
                    "account file is not readable: {}",
                    path.display()
                )
            }
            Self::Connect(error) => {
                write!(formatter, "cannot connect to the X server: {error}")
            }
            Self::Connection(error) => write!(formatter, "X connection failed: {error}"),
            Self::Reply(error) => write!(formatter, "X request failed: {error}"),
        }
    }
}

impl From<io::Error> for AdapterFailure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<ConnectError> for AdapterFailure {
    fn from(error: ConnectError) -> Self {
        Self::Connect(error)
    }
}

impl From<ConnectionError> for AdapterFailure {
    fn from(error: ConnectionError) -> Self {
        Self::Connection(error)
    }
}

impl From<ReplyError> for AdapterFailure {
    fn from(error: ReplyError) -> Self {
        Self::Reply(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_failure_has_words() {
        let io: AdapterFailure = io::Error::other("boom").into();
        assert!(io.to_string().contains("boom"));
        assert!(
            AdapterFailure::NoConfigDirectory
                .to_string()
                .contains("HOME")
        );
        assert!(
            AdapterFailure::BadAccountFile(PathBuf::from("/x/account.v1.txt"))
                .to_string()
                .ends_with("account.v1.txt")
        );
    }
}
