//! 持ち主が台帳の権限を変える命令（FR-010・ADR 0010）。合成ルートが引数から、画面が参加者の段から作り、store が当てる。

use alloc::string::String;

use nene_docket_core::account_id::AccountId;

/// 1 つの変更。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdminCommand {
    /// 権限を有効（真）か無効（偽）にする。
    Permissions(bool),
    /// 編集者に加える。
    Allow(AccountId),
    /// 編集者から外す（グループには残る）。
    Disallow(AccountId),
    /// グループに入れる（無ければ作る）。
    Group(String, AccountId),
    /// グループから外す（空になれば消える）。
    Ungroup(String, AccountId),
}
