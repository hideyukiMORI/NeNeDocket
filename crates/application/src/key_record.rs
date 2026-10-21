//! 台帳の `keys/<account 16 進>.v1.pub` の形（FR-010・ADR 0010）。`version = 1` / `id` / `name` / `public = <16 進 64 桁>`。
//!
//! 台帳に参加したとき自分の分を置く（無いときだけ）。読み手は署名の検証と、履歴の「誰が」の名前に使う（FR-009）。
//! 形はここ（純関数）、読み書きは store。

use alloc::string::String;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::public_key::PublicKey;

use crate::account::Account;

/// この形の版。
const VERSION: &str = "1";

/// ID と名前と公開鍵。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyRecord {
    id: AccountId,
    name: String,
    public: PublicKey,
}

impl KeyRecord {
    /// 自分のアカウントの分。
    #[must_use]
    pub fn of(account: &Account) -> Self {
        Self {
            id: account.id(),
            name: String::from(account.name()),
            public: account.public(),
        }
    }

    /// ID。
    #[must_use]
    pub const fn id(&self) -> AccountId {
        self.id
    }

    /// 名前。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 公開鍵。
    #[must_use]
    pub const fn public(&self) -> PublicKey {
        self.public
    }

    /// ファイル名。`<account 16 進>.v1.pub`。
    #[must_use]
    pub fn file_name(account: AccountId) -> String {
        alloc::format!("{}.v{VERSION}.pub", account.hex())
    }

    /// ファイル名からアカウントを読む。形が違えば `None`。
    #[must_use]
    pub fn account_of(file_name: &str) -> Option<AccountId> {
        AccountId::from_hex(file_name.strip_suffix(".v1.pub")?)
    }

    /// 文字列から読む。版が違う・鍵が読めない・欄が足りない・知らない鍵があるなら `None`。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut version = None;
        let mut id = None;
        let mut name = None;
        let mut public = None;
        for line in text.lines() {
            let (key, value) = line.split_once('=')?;
            match key.trim() {
                "version" => version = Some(value.trim()),
                "id" => id = AccountId::from_hex(value.trim()),
                "name" => name = Some(value.trim()),
                "public" => public = PublicKey::from_hex(value.trim()),
                _ => return None,
            }
        }
        if version != Some(VERSION) {
            return None;
        }
        Some(Self {
            id: id?,
            name: String::from(name?),
            public: public?,
        })
    }

    /// 文字列にする。`parse` の逆。
    #[must_use]
    pub fn encode(&self) -> String {
        alloc::format!(
            "version = {VERSION}\nid = {}\nname = {}\npublic = {}\n",
            self.id.hex(),
            self.name,
            self.public.hex()
        )
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::signing_key::SigningKey;

    use super::*;

    fn account() -> Account {
        Account::new(
            AccountId::new(0xABC),
            String::from("mori"),
            SigningKey::from_bytes(&[2_u8; 32]),
        )
    }

    #[test]
    fn encode_then_parse_round_trips_and_the_file_name_carries_the_account() {
        let record = KeyRecord::of(&account());
        assert_eq!(record.id(), AccountId::new(0xABC));
        assert_eq!(record.name(), "mori");
        assert_eq!(record.public(), account().public());
        let text = record.encode();
        assert!(text.starts_with("version = 1\n"));
        assert_eq!(KeyRecord::parse(&text), Some(record));
        let file_name = KeyRecord::file_name(AccountId::new(0xABC));
        assert!(file_name.ends_with(".v1.pub"));
        assert_eq!(
            KeyRecord::account_of(&file_name),
            Some(AccountId::new(0xABC))
        );
        assert_eq!(KeyRecord::account_of("x.v1.pub"), None);
        assert_eq!(KeyRecord::account_of("notes.txt"), None);
    }

    #[test]
    fn a_different_version_a_missing_field_or_an_unknown_key_is_refused() {
        let good = KeyRecord::of(&account()).encode();
        assert_eq!(
            KeyRecord::parse(&good.replacen("version = 1", "version = 2", 1)),
            None
        );
        assert_eq!(
            KeyRecord::parse(&good.replacen("name = mori\n", "", 1)),
            None
        );
        assert_eq!(
            KeyRecord::parse(&good.replacen("public = ", "public = 0", 1)),
            None
        );
        assert_eq!(KeyRecord::parse(&alloc::format!("{good}extra = 1\n")), None);
        assert_eq!(KeyRecord::parse("nonsense"), None);
    }
}
