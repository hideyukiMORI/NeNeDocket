//! 添付のファイルの置き場（FR-021・ADR 0016）。台帳フォルダの `attachments/<作ったアカウント>:<連番>/<添付の連番>-<名前>` に写す。
//! 元のファイルは触らない。一時名に書いて rename する。写しは消さない（外すのは `Detach` の追記だけ）。

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use encoding_rs::{Encoding, SHIFT_JIS};
use nene_docket_core::bitmap::Bitmap;
use nene_docket_media::image_decoding;

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

/// 文として読む上限（バイト）。これより大きければ先頭だけ（大容量でも軽い・FR-017）。
pub const TEXT_LIMIT: usize = 256 * 1024;

/// 写しを文として読む（FR-022）。BOM があればそれ（UTF-8・UTF-16）、無ければ UTF-8、通らなければ `Shift_JIS`（Excel の CSV）。
/// 上限を超えれば先頭だけにして、途中で切れた最後の行は落とす。
///
/// # Errors
///
/// 写しを読めない。
pub fn read_text(
    root: &Path,
    item: ItemId,
    attachment: &Attachment,
) -> Result<(String, bool), StoreFailure> {
    let unreadable = |_error| StoreFailure::CannotReadAttachment;
    let file = fs::File::open(path_of(root, item, attachment)).map_err(unreadable)?;
    let mut bytes = Vec::new();
    let limit = u64::try_from(TEXT_LIMIT)
        .unwrap_or(u64::MAX)
        .saturating_add(1);
    file.take(limit)
        .read_to_end(&mut bytes)
        .map_err(unreadable)?;
    let truncated = bytes.len() > TEXT_LIMIT;
    bytes.truncate(TEXT_LIMIT);
    let mut text = decode(&bytes);
    if truncated {
        let kept = text.rfind('\n').map_or(0, |index| index.saturating_add(1));
        text.truncate(kept);
    }
    Ok((text, truncated))
}

/// 文字コードを決めて文にする。読めない並びは U+FFFD。
fn decode(bytes: &[u8]) -> String {
    if let Some((encoding, bom_length)) = Encoding::for_bom(bytes) {
        let body = bytes.get(bom_length..).unwrap_or(&[]);
        return encoding.decode_without_bom_handling(body).0.into_owned();
    }
    match core::str::from_utf8(bytes) {
        Ok(text) => text.to_owned(),
        Err(_error) => SHIFT_JIS.decode_without_bom_handling(bytes).0.into_owned(),
    }
}

/// 画像の長い辺の上限（画素）。超えれば整数の倍率で縮める（ADR 0018）。
pub const MAX_IMAGE_SIDE: u32 = 2048;

/// 写しを画像として読む（FR-022）。PNG / JPEG。
///
/// # Errors
///
/// 写しを読めない（`CannotReadAttachment`）・復号できない（`CannotDecodeAttachment`）。
pub fn read_image(
    root: &Path,
    item: ItemId,
    attachment: &Attachment,
) -> Result<Bitmap, StoreFailure> {
    let bytes = fs::read(path_of(root, item, attachment))
        .map_err(|_error| StoreFailure::CannotReadAttachment)?;
    image_decoding::decode(&bytes, MAX_IMAGE_SIDE)
        .map_err(|_failure| StoreFailure::CannotDecodeAttachment)
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

    /// 文字コードの判定に使う課題。
    const TEXT_ITEM: ItemId = ItemId::new(AccountId::new(0x11), 3);

    /// 写しを置いて、その記録を返す。
    fn placed(root: &Path, name: &str, bytes: &[u8]) -> Attachment {
        let attachment = Attachment::new(1, String::from(name), 0, [0_u8; 32]);
        fs::create_dir_all(item_directory(root, TEXT_ITEM)).unwrap();
        fs::write(path_of(root, TEXT_ITEM, &attachment), bytes).unwrap();
        attachment
    }

    fn read(root: &Path, attachment: &Attachment) -> String {
        read_text(root, TEXT_ITEM, attachment).unwrap().0
    }

    #[test]
    fn text_is_decoded_from_utf8_boms_and_shift_jis() {
        let root = std::env::temp_dir().join(format!(
            "nene-docket-store-{}-attachment-text",
            std::process::id()
        ));
        let _ignored = fs::remove_dir_all(&root);
        let utf8 = placed(&root, "utf8.txt", "日本語\n".as_bytes());
        assert_eq!(
            read_text(&root, TEXT_ITEM, &utf8).unwrap(),
            (String::from("日本語\n"), false)
        );
        let bom = placed(&root, "bom.txt", b"\xEF\xBB\xBFabc");
        assert_eq!(read(&root, &bom), "abc");
        let utf16 = placed(&root, "u16.txt", b"\xFF\xFEa\x00b\x00");
        assert_eq!(read(&root, &utf16), "ab");
        // Shift_JIS の「日本語,1」（Excel の CSV）。
        let sjis = placed(&root, "x.csv", b"\x93\xFA\x96\x7B\x8C\xEA,1\r\n");
        assert_eq!(read(&root, &sjis), "日本語,1\r\n");
        let missing = Attachment::new(9, String::from("none"), 0, [0_u8; 32]);
        assert!(matches!(
            read_text(&root, TEXT_ITEM, &missing),
            Err(StoreFailure::CannotReadAttachment)
        ));
        let _cleaned = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_large_file_is_cut_at_the_limit_and_at_a_line_end() {
        let root = std::env::temp_dir().join(format!(
            "nene-docket-store-{}-attachment-big",
            std::process::id()
        ));
        let _ignored = fs::remove_dir_all(&root);
        let line = "0123456789\n";
        let big = line.repeat(TEXT_LIMIT / line.len() + 2);
        let attachment = placed(&root, "big.log", big.as_bytes());
        let (text, truncated) = read_text(&root, TEXT_ITEM, &attachment).unwrap();
        assert!(truncated);
        assert!(text.len() <= TEXT_LIMIT);
        assert!(text.ends_with('\n'));
        assert_eq!(text.len() % line.len(), 0);
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
