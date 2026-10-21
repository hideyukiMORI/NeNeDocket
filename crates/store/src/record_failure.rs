//! レコード 1 行を読めない理由。閉じた集合（ARC-010）。

use core::fmt;

/// 1 行がジャーナルの版 1 の形になっていない理由。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordFailure {
    /// 先頭の版が `v1` でも `v2` でもない。
    UnknownVersion,
    /// 欄の数が違う。
    WrongFieldCount,
    /// 検査和が合わない（書きかけ・壊れている）。
    ChecksumMismatch,
    /// 数（時計・壁時計・連番・並び順）が読めない。
    BadNumber,
    /// アカウントの 16 進が読めない。
    BadAccount,
    /// 種類の語を知らない。
    UnknownKind,
    /// 項目の語を知らない。
    UnknownField,
    /// 値が項目の形になっていない（状態の語・日付・逃がし）。
    BadValue,
    /// 版 2 の署名の欄が 16 進 128 桁でない。
    BadSignature,
}

impl RecordFailure {
    /// 利用者に見せる 1 行。
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::UnknownVersion => "unknown record version",
            Self::WrongFieldCount => "wrong number of fields",
            Self::ChecksumMismatch => "checksum mismatch",
            Self::BadNumber => "unreadable number",
            Self::BadAccount => "unreadable account id",
            Self::UnknownKind => "unknown operation kind",
            Self::UnknownField => "unknown field",
            Self::BadValue => "unreadable value",
            Self::BadSignature => "unreadable signature",
        }
    }
}

impl fmt::Display for RecordFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.describe())
    }
}
