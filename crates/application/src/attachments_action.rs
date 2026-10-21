//! 添付の段で利用者が求めたこと（FR-021）。閉じた集合（RS-002）。

use alloc::string::String;

/// 段の押下の意味。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentsAction {
    /// このパスのファイルを選んだ課題に添付する。
    Attach(String),
    /// この連番の添付を外す。
    Detach(u32),
    /// この連番の添付を OS の既定のアプリで開く（FR-022）。
    Open(u32),
}
