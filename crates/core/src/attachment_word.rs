//! 添付に関わる文言の名前（FR-021）。`Word::Attachment` の中身。閉じた集合（RS-002）。

/// 添付の段と履歴・知らせの語。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentWord {
    /// 添付の段の見出し（道具の段の「添付」）。
    Attachments,
    /// 添付の段の欄の名「パス」。
    Path,
    /// 添付の段の「添付する」。
    Attach,
    /// 添付の段の「外す」。
    Detach,
    /// 添付の「大きさ」の列。
    Size,
    /// 履歴: 添付した（名前が続く）。
    Attached,
    /// 履歴: 添付を外した（名前が続く）。
    Detached,
    /// 添付のファイルを読めない。
    CannotRead,
    /// 添付の写しを置けない。
    CannotWrite,
    /// 添付の写しの中身が記録と合わない。
    Altered,
    /// 添付の段の「開く」（OS の既定のアプリで・FR-022）。
    Open,
    /// 大きなファイルの先頭だけを出している。
    Truncated,
    /// OS の既定のアプリで開けなかった。
    CannotOpen,
    /// 画像を読めない（壊れている・知らない形）。
    CannotDecode,
}

impl AttachmentWord {
    /// 全部の語（閉じた集合を 1 か所で列挙する）。
    pub const ALL: [Self; 14] = [
        Self::Attachments,
        Self::Path,
        Self::Attach,
        Self::Detach,
        Self::Size,
        Self::Attached,
        Self::Detached,
        Self::CannotRead,
        Self::CannotWrite,
        Self::Altered,
        Self::Open,
        Self::Truncated,
        Self::CannotOpen,
        Self::CannotDecode,
    ];
}
