//! 課題に添付したファイル 1 つ（FR-021・ADR 0016）。台帳フォルダの `attachments/<課題>/<連番>-<名前>` に写しがあり、
//! ジャーナルには連番・名前・大きさ・SHA-256 が残る。消しても写しは残る（`Detach` の追記だけ）。

use alloc::string::String;

use crate::hex;

/// 連番（課題の中で 1 から）・元のファイル名・大きさ（バイト）・中身の SHA-256。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    sequence: u32,
    name: String,
    bytes: u64,
    sha256: [u8; 32],
}

impl Attachment {
    /// 値から。
    #[must_use]
    pub const fn new(sequence: u32, name: String, bytes: u64, sha256: [u8; 32]) -> Self {
        Self {
            sequence,
            name,
            bytes,
            sha256,
        }
    }

    /// 連番。
    #[must_use]
    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    /// 元のファイル名（フォルダは含まない）。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 大きさ（バイト）。
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }

    /// 中身の SHA-256。
    #[must_use]
    pub const fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }

    /// SHA-256 の 16 進（64 桁）。
    #[must_use]
    pub fn digest_hex(&self) -> String {
        hex::encode(&self.sha256)
    }

    /// 写しのファイル名 `<連番>-<安全な名前>`。名前は Windows / SMB で使えない字（`< > : " / \ | ? *`・制御文字）を `_` にし、
    /// 末尾の `.` と空白を落とし、予約名（`CON` など）には `_` を前置し、120 バイトに収める。記録の `name` は元のまま。
    #[must_use]
    pub fn file_name(&self) -> String {
        alloc::format!("{}-{}", self.sequence, safe_name(self.name()))
    }
}

/// Windows の予約名（拡張子を除いた部分・大文字小文字を問わない）。
const RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// 写しの名前の上限（バイト・UTF-8 の境で切る）。NTFS の 255 バイトと 260 文字の経路に余裕を残す。
const NAME_LIMIT: usize = 120;

/// 台帳フォルダ（SMB / NTFS / ext4 / APFS）のどれでも作れる名前にする。空なら `file`。
#[must_use]
pub fn safe_name(name: &str) -> String {
    let mut safe: String = name
        .chars()
        .map(|character| {
            if matches!(
                character,
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*'
            ) || character.is_control()
            {
                '_'
            } else {
                character
            }
        })
        .collect();
    while safe.len() > NAME_LIMIT {
        safe.pop();
    }
    let mut named = String::from(safe.trim_end_matches(['.', ' ']).trim_start_matches(' '));
    if named.is_empty() {
        return String::from("file");
    }
    let stem = named.split('.').next().unwrap_or("").to_ascii_uppercase();
    if RESERVED.contains(&stem.as_str()) {
        named.insert(0, '_');
    }
    named
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carries_its_parts_and_names_the_copy() {
        let attachment = Attachment::new(3, String::from("a.png"), 10, [7_u8; 32]);
        assert_eq!(attachment.sequence(), 3);
        assert_eq!(attachment.name(), "a.png");
        assert_eq!(attachment.bytes(), 10);
        assert_eq!(attachment.sha256(), &[7_u8; 32]);
        assert_eq!(attachment.digest_hex().len(), 64);
        assert_eq!(attachment.file_name(), "3-a.png");
    }

    #[test]
    fn the_copy_name_is_safe_on_every_file_system_and_the_record_keeps_the_original() {
        let odd = Attachment::new(3, String::from("a:b/c*d?.txt"), 1, [0_u8; 32]);
        assert_eq!(odd.name(), "a:b/c*d?.txt");
        assert_eq!(odd.file_name(), "3-a_b_c_d_.txt");
        assert_eq!(safe_name("CON"), "_CON");
        assert_eq!(safe_name("con.txt"), "_con.txt");
        assert_eq!(safe_name("trailing. "), "trailing");
        assert_eq!(safe_name("   "), "file");
        assert_eq!(safe_name("tab\there"), "tab_here");
        let long = "日".repeat(100);
        let cut = safe_name(&long);
        assert!(cut.len() <= NAME_LIMIT && cut.chars().all(|c| c == '日'));
        assert_eq!(safe_name("normal-name_1.PNG"), "normal-name_1.PNG");
    }
}
