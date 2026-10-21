//! 途中で止まった置き換えの残骸（`<名前>.tmp`）を、次に開いたときに捨てる（Issue #81）。
//! 捨てるのは自分のアカウントの分だけ（スナップショット・自分の鍵）と、持ち主なら権限と台帳のファイルの分（書くのは持ち主だけ）。
//! 添付の `.tmp` は誰のものか名前から分からず、時刻も使わない（ARC-004）ので捨てない。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nene_docket_application::key_record::KeyRecord;
use nene_docket_core::account_id::AccountId;

use crate::docket_file;
use crate::key_file;
use crate::permissions_store;
use crate::replace;
use crate::snapshot_file;
use crate::store_failure::StoreFailure;

/// 自分の分の残骸を消し、消した数を返す。無ければ何もしない。
///
/// # Errors
///
/// 消せない（無い以外の失敗）。
pub(crate) fn discard(root: &Path, account: AccountId, owner: bool) -> Result<usize, StoreFailure> {
    let mut removed = 0_usize;
    for path in candidates(root, account, owner) {
        match fs::remove_file(replace::temporary_name(&path)) {
            Ok(()) => removed = removed.saturating_add(1),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(StoreFailure::Io(error)),
        }
    }
    Ok(removed)
}

/// 残骸がありうる置き換え先（`.tmp` を付ける前の名前）。
fn candidates(root: &Path, account: AccountId, owner: bool) -> Vec<PathBuf> {
    let (state, index) = snapshot_file::paths(root, account);
    let mut paths = vec![
        state,
        index,
        key_file::directory(root).join(KeyRecord::file_name(account)),
    ];
    if owner {
        paths.push(root.join(permissions_store::FILE_NAME));
        paths.push(root.join(docket_file::FILE_NAME));
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_my_temporaries_and_the_owners_files_are_removed() {
        let root = std::env::temp_dir().join(format!(
            "nene-docket-store-{}-leftovers",
            std::process::id()
        ));
        let _ignored = fs::remove_dir_all(&root);
        let me = AccountId::new(0x11);
        let other = AccountId::new(0x22);
        let mine = candidates(&root, me, true);
        let theirs = candidates(&root, other, false);
        for path in mine.iter().chain(&theirs) {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(replace::temporary_name(path), b"half").unwrap();
        }
        assert_eq!(discard(&root, me, false).unwrap(), 3);
        assert_eq!(discard(&root, me, true).unwrap(), 2);
        assert_eq!(discard(&root, me, true).unwrap(), 0);
        for path in &theirs {
            assert!(replace::temporary_name(path).exists());
        }
        let _cleaned = fs::remove_dir_all(&root);
    }
}
