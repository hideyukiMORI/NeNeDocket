//! 計測（`--bench`・Issue #111）の置き場。空のフォルダか、まだ無いフォルダだけを受ける。本物の台帳を計測の課題で汚さない。

use std::fs;
use std::path::Path;

use crate::store_failure::StoreFailure;

/// フォルダが無ければ作り、在って中身があれば拒む。
///
/// # Errors
///
/// フォルダに何かある（[`StoreFailure::NotEmptyFolder`]）・作れない・読めない。
pub fn prepare(root: &Path) -> Result<(), StoreFailure> {
    if root.exists() {
        if fs::read_dir(root)?.next().is_some() {
            return Err(StoreFailure::NotEmptyFolder);
        }
        return Ok(());
    }
    fs::create_dir_all(root)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "nene-docket-bench-folder-{name}-{}",
            std::process::id()
        ));
        let _removed = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn a_missing_folder_is_made_an_empty_one_is_taken_and_a_used_one_is_refused() {
        let missing = scratch("missing");
        prepare(&missing).unwrap();
        assert!(missing.is_dir());
        prepare(&missing).unwrap();
        fs::write(missing.join("docket.v1.txt"), "x").unwrap();
        assert!(matches!(
            prepare(&missing),
            Err(StoreFailure::NotEmptyFolder)
        ));
        fs::remove_dir_all(&missing).unwrap();
    }
}
