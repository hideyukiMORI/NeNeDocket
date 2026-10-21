//! このプロセスのアカウント（FR-009）。アダプタが起動時に作るか読み、合成ルートが値として渡す。

use alloc::string::String;

use nene_docket_core::account_id::AccountId;

/// ID と名前。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    id: AccountId,
    name: String,
}

impl Account {
    /// ID と名前から。
    #[must_use]
    pub const fn new(id: AccountId, name: String) -> Self {
        Self { id, name }
    }

    /// ID。
    #[must_use]
    pub const fn id(&self) -> AccountId {
        self.id
    }

    /// 名前。表の担当に使う。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
