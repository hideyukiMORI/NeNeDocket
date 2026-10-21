//! アカウントのファイルの形（FR-009）。`version = 1` / `id = <16 進 32 桁>` / `name = <名前>` の 3 行。
//!
//! 形はここ（純関数）、読み書きはアダプタ。版が違えば読めないことを返し、黙って作り直さない（FR-014）。

use alloc::string::String;

use nene_docket_core::account_id::AccountId;

use crate::account::Account;

/// この形の版。
const VERSION: &str = "1";

/// 文字列からアカウントを読む。版が違う・ID が読めない・名前が無いなら `None`。
#[must_use]
pub fn parse(text: &str) -> Option<Account> {
    let mut version = None;
    let mut id = None;
    let mut name = None;
    for line in text.lines() {
        let (key, value) = line.split_once('=')?;
        match key.trim() {
            "version" => version = Some(value.trim()),
            "id" => id = AccountId::from_hex(value.trim()),
            "name" => name = Some(value.trim()),
            _ => return None,
        }
    }
    if version != Some(VERSION) {
        return None;
    }
    Some(Account::new(id?, String::from(name?)))
}

/// アカウントを文字列にする。`parse` の逆。
#[must_use]
pub fn encode(account: &Account) -> String {
    alloc::format!(
        "version = {VERSION}\nid = {}\nname = {}\n",
        account.id().hex(),
        account.name()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_then_parse_round_trips() {
        let account = Account::new(AccountId::new(0x00C0_FFEE), String::from("mori"));
        let text = encode(&account);
        assert!(text.starts_with("version = 1\n"));
        assert_eq!(parse(&text), Some(account));
    }

    #[test]
    fn a_different_version_or_a_broken_id_is_refused() {
        assert_eq!(parse("version = 2\nid = 1\nname = a\n"), None);
        assert_eq!(parse("version = 1\nid = xyz\nname = a\n"), None);
        assert_eq!(parse("version = 1\nid = 1\n"), None);
        assert_eq!(parse("version = 1\nid = 1\nname = a\ncolor = red\n"), None);
        assert_eq!(parse("nonsense"), None);
    }
}
