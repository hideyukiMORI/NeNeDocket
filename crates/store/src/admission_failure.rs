//! 権限を有効にした台帳で、行を捨てる理由（FR-010・ADR 0010）。閉じた集合（ARC-010）。

use core::fmt;

/// 署名か権限が通らない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionFailure {
    /// 版 1 の行（署名が無い）。
    Unsigned,
    /// 行のアカウントがジャーナルのファイルのアカウントと違う。
    ForeignAccount,
    /// `keys/` にそのアカウントの公開鍵が無い。
    UnknownKey,
    /// 署名が公開鍵で通らない（改竄・なりすまし）。
    BadSignature,
    /// そのアカウントは編集者でない。
    NotAnEditor,
}

impl AdmissionFailure {
    /// 利用者に見せる 1 行。
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::Unsigned => "unsigned line in a docket with permissions",
            Self::ForeignAccount => "line claims another account",
            Self::UnknownKey => "no public key for the account",
            Self::BadSignature => "signature does not verify",
            Self::NotAnEditor => "account is not an editor",
        }
    }
}

impl fmt::Display for AdmissionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.describe())
    }
}
