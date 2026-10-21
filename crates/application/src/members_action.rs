//! 参加者の段で利用者が求めたこと。閉じた集合（RS-002）。

use nene_docket_core::account_id::AccountId;

/// 段の押下の意味。持ち主だけが求められる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MembersAction {
    /// その人を編集者にする・編集者から外す。
    ToggleEditor(AccountId),
    /// 権限を有効にする・無効にする。
    TogglePermissions,
}
