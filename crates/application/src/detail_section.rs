//! 詳細のペインの区画（Issue #122・ADR 0022）。閉じた集合（RS-002）。

/// 詳細のペインの下で、1 つずつ切り替えて出すもの。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailSection {
    /// 本文（筋書きの 3 段で対応策を書く場所）。
    Body,
    /// 添付。
    Attachments,
    /// 課題の履歴。
    History,
}

impl DetailSection {
    /// 切り替えの札の左からの順。
    pub const ALL: [Self; 3] = [Self::Body, Self::Attachments, Self::History];
}
