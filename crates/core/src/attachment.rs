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

    /// 台帳フォルダの中のファイル名 `<連番>-<名前>`。
    #[must_use]
    pub fn file_name(&self) -> String {
        alloc::format!("{}-{}", self.sequence, self.name)
    }
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
}
