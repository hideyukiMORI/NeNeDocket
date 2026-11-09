//! 課題の番号の表示（表の「#」と履歴）。作ったアカウントの略称と連番を `-` で結ぶ（`森-12`・Issue #107）。
//!
//! 略称は、作ったアカウントの名前（台帳の `keys/`）の先頭 6 文字（空白と `-` を除く）。サーバが無いので台帳を通した連番は
//! 約束できず、作った人ごとの連番を読める形にする（PR #108 の施主の判断 2）。名前を知らなければ ID の先頭 4 桁。
//! 別のアカウントと略称が重なれば、ID の先頭 2 桁を `.` で添える（`mori.eb-12`）。

use alloc::format;
use alloc::string::String;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::item_id::ItemId;

/// 名前を知らないとき、作ったアカウントの ID を出す桁数。
const CREATOR_DIGITS: usize = 4;
/// 略称が重なったときに添える ID の桁数。
const TIE_DIGITS: usize = 2;
/// 略称の最大の文字数。
const SHORT_CHARS: usize = 6;

/// `森-12` の形。
#[must_use]
pub fn text(id: ItemId, names: &[(AccountId, String)]) -> String {
    format!("{}-{}", short_name(id.creator(), names), id.sequence())
}

/// アカウントの略称。
#[must_use]
pub fn short_name(account: AccountId, names: &[(AccountId, String)]) -> String {
    let own = names
        .iter()
        .find(|(known, _)| *known == account)
        .map(|(_, name)| shortened(name))
        .filter(|short| !short.is_empty());
    let Some(short) = own else {
        return account.hex().chars().take(CREATOR_DIGITS).collect();
    };
    let shared = names
        .iter()
        .any(|(other, name)| *other != account && shortened(name) == short);
    if shared {
        let tie: String = account.hex().chars().take(TIE_DIGITS).collect();
        format!("{short}.{tie}")
    } else {
        short
    }
}

/// 空白と `-`（番号の区切り）を除いた先頭 6 文字。
fn shortened(name: &str) -> String {
    name.chars()
        .filter(|character| !character.is_whitespace() && *character != '-')
        .take(SHORT_CHARS)
        .collect()
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use super::*;

    fn account(top: u128) -> AccountId {
        AccountId::new(top << 112)
    }

    #[test]
    fn an_unknown_creator_is_four_hex_digits_a_dash_and_the_sequence() {
        assert_eq!(text(ItemId::new(account(0x17a6), 1), &[]), "17a6-1");
        assert_eq!(text(ItemId::new(AccountId::new(0), 12), &[]), "0000-12");
    }

    #[test]
    fn a_known_creator_is_shown_by_the_first_six_characters_of_the_name() {
        let names = Vec::from([
            (account(0x17a6), String::from("森")),
            (account(0x2b00), String::from("hide yuki-mori")),
            (account(0x3c00), String::from("   ")),
        ]);
        assert_eq!(text(ItemId::new(account(0x17a6), 12), &names), "森-12");
        assert_eq!(text(ItemId::new(account(0x2b00), 3), &names), "hideyu-3");
        assert_eq!(text(ItemId::new(account(0x3c00), 4), &names), "3c00-4");
    }

    #[test]
    fn two_accounts_with_the_same_short_name_add_two_hex_digits() {
        let names = Vec::from([
            (account(0xeb03), String::from("mori")),
            (account(0x7a11), String::from("mori")),
        ]);
        assert_eq!(text(ItemId::new(account(0xeb03), 1), &names), "mori.eb-1");
        assert_eq!(text(ItemId::new(account(0x7a11), 1), &names), "mori.7a-1");
    }
}
