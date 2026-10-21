//! 添付のファイルの置き場（FR-021・ADR 0016）。台帳フォルダの `attachments/<作ったアカウント>:<連番>/<添付の連番>-<名前>` に写す。
//! 元のファイルは触らない。一時名に書いて rename する。写しは消さない（外すのは `Detach` の追記だけ）。

use std::fs;
use std::path::{Path, PathBuf};

use nene_docket_core::attachment::Attachment;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::sha256::sha256;

use crate::store_failure::StoreFailure;

/// 添付を置くフォルダ。
const DIRECTORY: &str = "attachments";

/// 課題のフォルダ。
fn item_directory(root: &Path, item: ItemId) -> PathBuf {
    root.join(DIRECTORY)
        .join(format!("{}:{}", item.creator().hex(), item.sequence()))
}

/// 写しの場所。
#[must_use]
pub fn path_of(root: &Path, item: ItemId, attachment: &Attachment) -> PathBuf {
    item_directory(root, item).join(attachment.file_name())
}

/// `source` を読んで写し、連番・名前（`source` のファイル名）・大きさ・SHA-256 を返す。
///
/// # Errors
///
/// 元のファイルを読めない（`CannotReadAttachment`）・写しを置けない（`CannotWriteAttachment`）。
pub fn copy_in(
    root: &Path,
    item: ItemId,
    sequence: u32,
    source: &Path,
) -> Result<Attachment, StoreFailure> {
    let name = source
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .ok_or(StoreFailure::CannotReadAttachment)?;
    let bytes = fs::read(source).map_err(|_error| StoreFailure::CannotReadAttachment)?;
    let attachment = Attachment::new(
        sequence,
        name.to_owned(),
        u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        sha256(&bytes),
    );
    let directory = item_directory(root, item);
    let place = || StoreFailure::CannotWriteAttachment;
    fs::create_dir_all(&directory).map_err(|_error| place())?;
    let temporary = directory.join(format!("{}.tmp", attachment.file_name()));
    fs::write(&temporary, &bytes).map_err(|_error| place())?;
    fs::rename(&temporary, path_of(root, item, &attachment)).map_err(|_error| place())?;
    Ok(attachment)
}

/// 写しの中身が記録の SHA-256 と合うか。無い・読めないなら合わない。
#[must_use]
pub fn is_intact(root: &Path, item: ItemId, attachment: &Attachment) -> bool {
    fs::read(path_of(root, item, attachment))
        .is_ok_and(|bytes| sha256(&bytes) == *attachment.sha256())
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;

    use super::*;

    #[test]
    fn copies_hashes_names_and_detects_a_changed_copy() {
        let root = std::env::temp_dir().join(format!(
            "nene-docket-store-{}-attachments",
            std::process::id()
        ));
        let _ignored = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let source = root.join("note.txt");
        fs::write(&source, b"abc").unwrap();
        let item = ItemId::new(AccountId::new(0x11), 2);
        let attachment = copy_in(&root, item, 1, &source).unwrap();
        assert_eq!(attachment.name(), "note.txt");
        assert_eq!(attachment.bytes(), 3);
        assert!(attachment.digest_hex().starts_with("ba7816bf"));
        let copy = path_of(&root, item, &attachment);
        assert!(copy.ends_with("1-note.txt"));
        assert_eq!(fs::read(&copy).unwrap(), b"abc");
        assert!(is_intact(&root, item, &attachment));
        fs::write(&copy, b"abd").unwrap();
        assert!(!is_intact(&root, item, &attachment));
        let _cleaned = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_source_or_a_folder_cannot_be_read() {
        let root = std::env::temp_dir().join(format!(
            "nene-docket-store-{}-attachments-unreadable",
            std::process::id()
        ));
        let _ignored = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let item = ItemId::new(AccountId::new(0x11), 2);
        let missing = copy_in(&root, item, 2, &root.join("missing"));
        assert!(matches!(missing, Err(StoreFailure::CannotReadAttachment)));
        let folder = copy_in(&root, item, 2, Path::new("/"));
        assert!(matches!(folder, Err(StoreFailure::CannotReadAttachment)));
        let _cleaned = fs::remove_dir_all(&root);
    }
}
