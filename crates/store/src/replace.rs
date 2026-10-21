//! ファイルを丸ごと置き換える: 一時名（`<名前>.tmp`）に書いて `rename`。読み手は途中の状態を見ない（ADR 0004 の 9・Issue #77）。
//! 書きと rename は Windows の一時的なロックに再試行する（`retry`・Issue #79）。

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::retry;

/// `path` の中身を `bytes` にする。一時名に書き、rename で置く。
///
/// # Errors
///
/// 書けない・rename できない（再試行の後）。
pub(crate) fn replace_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = temporary_name(path);
    retry::retried(|| fs::write(&temporary, bytes))?;
    retry::retried(|| fs::rename(&temporary, path))
}

/// `<path>.tmp`。
pub(crate) fn temporary_name(path: &Path) -> PathBuf {
    let mut name: OsString = path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_the_whole_file_and_leaves_no_temporary() {
        let directory =
            std::env::temp_dir().join(format!("nene-docket-store-{}-replace", std::process::id()));
        let _ignored = fs::remove_dir_all(&directory);
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join("docket.v1.txt");
        replace_file(&path, b"first\n").unwrap();
        replace_file(&path, b"second\n").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second\n");
        assert!(!temporary_name(&path).exists());
        assert_eq!(
            temporary_name(Path::new("a/b.txt")),
            PathBuf::from("a/b.txt.tmp")
        );
        assert!(replace_file(&directory.join("missing/x.txt"), b"x").is_err());
        let _cleaned = fs::remove_dir_all(&directory);
    }
}
