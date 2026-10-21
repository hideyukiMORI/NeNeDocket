//! このプロセスのアカウント（FR-009）。アダプタが起動時に作るか読み、合成ルートが値として渡す。

use alloc::string::String;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::public_key::PublicKey;
use nene_docket_core::signing_key::SigningKey;

/// ID と名前と、署名する鍵（FR-010・ADR 0010）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    id: AccountId,
    name: String,
    key: SigningKey,
}

impl Account {
    /// ID と名前と鍵から。
    #[must_use]
    pub const fn new(id: AccountId, name: String, key: SigningKey) -> Self {
        Self { id, name, key }
    }

    /// ID。
    #[must_use]
    pub const fn id(&self) -> AccountId {
        self.id
    }

    /// 名前。表の担当と、台帳の `keys/` に使う。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 署名する鍵。
    #[must_use]
    pub const fn key(&self) -> &SigningKey {
        &self.key
    }

    /// 公開鍵。台帳の `keys/` に置く。
    #[must_use]
    pub fn public(&self) -> PublicKey {
        self.key.public()
    }
}
