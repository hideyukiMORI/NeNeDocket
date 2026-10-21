//! 課題の番号の表示（表の「#」と履歴）。作ったアカウントの ID の先頭 4 桁と連番を `-` で結ぶ。

use alloc::string::String;

use nene_docket_core::item_id::ItemId;

/// 作ったアカウントの ID を出す桁数。
const CREATOR_DIGITS: usize = 4;

/// `17a6-1` の形。
#[must_use]
pub fn text(id: ItemId) -> String {
    let creator: String = id.creator().hex().chars().take(CREATOR_DIGITS).collect();
    alloc::format!("{creator}-{}", id.sequence())
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;

    use super::*;

    #[test]
    fn the_number_is_four_hex_digits_a_dash_and_the_sequence() {
        assert_eq!(
            text(ItemId::new(AccountId::new(0x17a6 << 112), 1)),
            "17a6-1"
        );
        assert_eq!(text(ItemId::new(AccountId::new(0), 12)), "0000-12");
    }
}
