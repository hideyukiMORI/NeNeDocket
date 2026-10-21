//! アカウントの ID（FR-009）。

use alloc::string::String;
use core::fmt::Write as _;

/// 128 ビットの番号。アダプタが乱数で作り、以後は変わらない。16 進 32 桁で読み書きする。
///
/// 衝突しないことは乱数の幅に頼る。なりすましを防ぐのは署名（FR-010・別の ADR）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountId {
    number: u128,
}

impl AccountId {
    /// 16 進の桁数。
    pub const HEX_LEN: usize = 32;

    /// 番号から作る。
    #[must_use]
    pub const fn new(number: u128) -> Self {
        Self { number }
    }

    /// 16 進 32 桁から読む。桁数が違う・16 進でないなら `None`。
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        if text.len() != Self::HEX_LEN || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        u128::from_str_radix(text, 16).ok().map(Self::new)
    }

    /// 16 進 32 桁（小文字）。
    #[must_use]
    pub fn hex(self) -> String {
        let mut text = String::with_capacity(Self::HEX_LEN);
        // 幅 32 のゼロ埋め。`write!` は String へは失敗しない。
        let _written: core::fmt::Result = write!(text, "{:032x}", self.number);
        text
    }

    /// 番号。
    #[must_use]
    pub const fn number(self) -> u128 {
        self.number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_with_zero_padding() {
        let id = AccountId::new(0xdead_beef);
        assert_eq!(id.hex(), "000000000000000000000000deadbeef");
        assert_eq!(AccountId::from_hex(&id.hex()), Some(id));
        assert_eq!(AccountId::from_hex("DEADBEEF"), None);
        assert_eq!(AccountId::from_hex(&"g".repeat(32)), None);
        assert_eq!(AccountId::new(u128::MAX).hex().len(), 32);
        assert!(AccountId::new(1) < AccountId::new(2));
        assert_eq!(id.number(), 0xdead_beef);
    }
}
