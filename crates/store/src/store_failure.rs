//! 置き場とファイルの失敗。閉じた集合（ARC-010）。黙って既定値へ落とさない（ARC-009）。

use core::fmt;
use std::io;

/// フォルダとジャーナルで起こりうる失敗。合成ルートと画面はこれを 1 行で示す。
#[derive(Debug)]
pub enum StoreFailure {
    /// フォルダを作れない・ファイルを開けない・書けない。
    Io(io::Error),
    /// `docket.v1.txt` の版を知らない（新しい版の台帳）。
    UnknownDocketVersion(String),
    /// `docket.v1.txt` の形が読めない。
    BadDocketFile,
    /// ジャーナルのファイル名からアカウントが読めない。
    BadJournalName(String),
    /// `keys/` のファイルが鍵の記録の形でない。
    BadKeyFile(String),
    /// `permissions.v1.txt` の形が読めない。
    BadPermissionsFile,
    /// `permissions.v1.txt` の署名が持ち主の鍵で通らない（鍵が無い・持ち主が違う・改竄）。
    UnverifiedPermissions,
    /// 台帳のフォルダに `docket.v1.txt` が無い（権限の操作は在る台帳にだけ）。
    NoDocket,
    /// 権限を変えられるのは持ち主だけ。
    NotTheOwner,
    /// 台帳にある自分の公開鍵が手元の鍵と違うので、署名しても誰にも通らない。
    KeyMismatch,
    /// 索引が指す位置に完全な行が無い（ADR 0015）。
    BadIndex,
    /// スナップショットが読めない形（ADR 0015）。捨てて全部読む。
    BadSnapshot,
    /// 添付する元のファイルを読めない（FR-021）。
    CannotReadAttachment,
    /// 添付の写しを置けない（FR-021）。
    CannotWriteAttachment,
    /// 添付の画像を復号できない（FR-022）。
    CannotDecodeAttachment,
    /// 追記に失敗した操作がまだジャーナルに無い（スナップショットを書くと嘘になる・Issue #81）。
    Unwritten(usize),
}

impl fmt::Display for StoreFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "file error: {error}"),
            Self::UnknownDocketVersion(version) => {
                write!(formatter, "unknown docket version `{version}`")
            }
            Self::BadDocketFile => formatter.write_str("docket.v1.txt is not readable"),
            Self::BadJournalName(name) => write!(formatter, "not a journal file name: `{name}`"),
            Self::BadKeyFile(name) => write!(formatter, "not a key record: `keys/{name}`"),
            Self::BadPermissionsFile => formatter.write_str("permissions.v1.txt is not readable"),
            Self::UnverifiedPermissions => {
                formatter.write_str("permissions.v1.txt is not signed by the docket owner")
            }
            Self::NoDocket => formatter.write_str("no docket.v1.txt in the folder"),
            Self::NotTheOwner => {
                formatter.write_str("only the docket owner may change permissions")
            }
            Self::KeyMismatch => {
                formatter.write_str("the public key in keys/ differs from this account's key")
            }
            Self::BadIndex => formatter.write_str("the snapshot index points at no journal line"),
            Self::BadSnapshot => formatter.write_str("the snapshot is not readable"),
            Self::CannotReadAttachment => formatter.write_str("cannot read the file to attach"),
            Self::CannotWriteAttachment => formatter.write_str("cannot store the attachment"),
            Self::CannotDecodeAttachment => formatter.write_str("添付の画像を復号できない"),
            Self::Unwritten(count) => {
                write!(formatter, "{count} operations are not yet in the journal")
            }
        }
    }
}

impl From<io::Error> for StoreFailure {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
