//! アダプタの失敗。閉じた集合（ARC-010）。合成ルートが 1 行で示す。

use core::fmt;
use std::io;
use std::path::PathBuf;

/// 起動時に起こりうる失敗。
#[derive(Debug)]
pub enum AdapterFailure {
    /// ファイルを読めない・書けない。
    Io(io::Error),
    /// 設定フォルダが決められない（`APPDATA` が無い）。
    NoConfigDirectory,
    /// アカウントのファイルが読めない形（版が違う・壊れている）。黙って作り直さない（FR-014）。
    BadAccountFile(PathBuf),
    /// 設定ファイルが読めない形（版が違う・知らない鍵・壊れている）。黙って既定値へ落とさない（FR-014）。
    BadSettingsFile(PathBuf),
    /// 乱数を読めない（`BCryptGenRandom`）。
    #[cfg(windows)]
    Random(windows::core::Error),
}

impl fmt::Display for AdapterFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "file error: {error}"),
            Self::NoConfigDirectory => formatter.write_str("APPDATA is not set"),
            Self::BadAccountFile(path) => {
                write!(
                    formatter,
                    "account file is not readable: {}",
                    path.display()
                )
            }
            Self::BadSettingsFile(path) => {
                write!(
                    formatter,
                    "settings file is not readable: {}",
                    path.display()
                )
            }
            #[cfg(windows)]
            Self::Random(error) => write!(formatter, "cannot read random bytes: {error}"),
        }
    }
}

impl From<io::Error> for AdapterFailure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
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
                .contains("APPDATA")
        );
        assert!(
            AdapterFailure::BadAccountFile(PathBuf::from("C:\\x\\account.v1.txt"))
                .to_string()
                .ends_with("account.v1.txt")
        );
        assert!(
            AdapterFailure::BadSettingsFile(PathBuf::from("C:\\x\\settings.v1.txt"))
                .to_string()
                .contains("settings")
        );
    }
}
