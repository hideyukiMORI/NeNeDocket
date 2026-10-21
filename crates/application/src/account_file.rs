//! アカウントのファイルの形（FR-009 / FR-010）。版 2: `version = 2` / `id = <16 進 32 桁>` / `name = <名前>` / `secret = <16 進 64 桁>`。
//!
//! 版 1（`secret` が無い）も読める。読んだらアダプタが鍵を作り、書くのは版 2 だけ（ARC-009 の移行）。
//! 形はここ（純関数）、読み書きはアダプタ。知らない版・知らない鍵・読めない値は `None` で、黙って作り直さない（FR-014）。

use alloc::boxed::Box;
use alloc::string::String;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::signing_key::SigningKey;

use crate::account::Account;
use crate::account_reading::AccountReading;

/// いま書く版。
const VERSION: &str = "2";
/// 読めるだけの前の版。
const PREVIOUS_VERSION: &str = "1";

/// 文字列からアカウントを読む。版が違う・ID が読めない・名前が無い・版 2 なのに鍵が無い（版 1 なのに有る）なら `None`。
#[must_use]
pub fn parse(text: &str) -> Option<AccountReading> {
    let mut version = None;
    let mut id = None;
    let mut name = None;
    let mut secret = None;
    for line in text.lines() {
        let (key, value) = line.split_once('=')?;
        match key.trim() {
            "version" => version = Some(value.trim()),
            "id" => id = AccountId::from_hex(value.trim()),
            "name" => name = Some(value.trim()),
            "secret" => secret = Some(SigningKey::from_hex(value.trim())?),
            _ => return None,
        }
    }
    let (id, name) = (id?, String::from(name?));
    match (version?, secret) {
        (VERSION, Some(key)) => Some(AccountReading::Complete(Box::new(Account::new(
            id, name, key,
        )))),
        (PREVIOUS_VERSION, None) => Some(AccountReading::WithoutKey(id, name)),
        _ => None,
    }
}

/// アカウントを文字列にする（版 2）。`parse` の逆。
#[must_use]
pub fn encode(account: &Account) -> String {
    alloc::format!(
        "version = {VERSION}\nid = {}\nname = {}\nsecret = {}\n",
        account.id().hex(),
        account.name(),
        account.key().hex()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account() -> Account {
        Account::new(
            AccountId::new(0x00C0_FFEE),
            String::from("mori"),
            SigningKey::from_bytes(&[1_u8; 32]),
        )
    }

    #[test]
    fn encode_then_parse_round_trips() {
        let text = encode(&account());
        assert!(text.starts_with("version = 2\n"));
        assert!(text.contains("\nsecret = "));
        assert_eq!(
            parse(&text),
            Some(AccountReading::Complete(Box::new(account())))
        );
    }

    #[test]
    fn version_one_is_read_without_a_key_but_may_not_carry_one() {
        let hex = AccountId::new(7).hex();
        assert_eq!(
            parse(&alloc::format!("version = 1\nid = {hex}\nname = a\n")),
            Some(AccountReading::WithoutKey(
                AccountId::new(7),
                String::from("a")
            ))
        );
        let secret = SigningKey::from_bytes(&[1_u8; 32]).hex();
        assert_eq!(
            parse(&alloc::format!(
                "version = 1\nid = {hex}\nname = a\nsecret = {secret}\n"
            )),
            None
        );
        assert_eq!(
            parse(&alloc::format!("version = 2\nid = {hex}\nname = a\n")),
            None
        );
    }

    #[test]
    fn a_different_version_or_a_broken_value_is_refused() {
        let hex = AccountId::new(7).hex();
        assert_eq!(parse("version = 3\nid = 1\nname = a\n"), None);
        assert_eq!(parse("version = 1\nid = xyz\nname = a\n"), None);
        assert_eq!(parse(&alloc::format!("version = 1\nid = {hex}\n")), None);
        assert_eq!(
            parse(&alloc::format!(
                "version = 1\nid = {hex}\nname = a\ncolor = red\n"
            )),
            None
        );
        assert_eq!(
            parse(&alloc::format!(
                "version = 2\nid = {hex}\nname = a\nsecret = 12\n"
            )),
            None
        );
        assert_eq!(parse("nonsense"), None);
    }
}
