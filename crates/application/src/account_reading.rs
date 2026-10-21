//! アカウントのファイルを読んだ結果。版 2 は鍵まで揃い、版 1 は鍵が無い（アダプタが作って版 2 で書き直す）。

use alloc::boxed::Box;
use alloc::string::String;

use nene_docket_core::account_id::AccountId;

use crate::account::Account;

/// 揃っているか、鍵が要るか。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountReading {
    /// 版 2。そのまま使える。鍵の分だけ大きいので箱に入れる。
    Complete(Box<Account>),
    /// 版 1。ID と名前だけ。鍵を足して版 2 にする（ARC-009 の移行）。
    WithoutKey(AccountId, String),
}
