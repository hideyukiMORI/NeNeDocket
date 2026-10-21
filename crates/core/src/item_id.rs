//! 課題の ID。作ったアカウントと、そのアカウントの中の連番。調整なしで一意（ADR 0004）。

use crate::account_id::AccountId;

/// 作ったアカウントと連番。表示の番号は併合した台帳の中での作られた順で、これとは別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemId {
    creator: AccountId,
    sequence: u32,
}

impl ItemId {
    /// 作ったアカウントと連番から。
    #[must_use]
    pub const fn new(creator: AccountId, sequence: u32) -> Self {
        Self { creator, sequence }
    }

    /// 作ったアカウント。
    #[must_use]
    pub const fn creator(self) -> AccountId {
        self.creator
    }

    /// 連番。
    #[must_use]
    pub const fn sequence(self) -> u32 {
        self.sequence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_creators_or_sequences_are_different_items() {
        let a = AccountId::new(1);
        let b = AccountId::new(2);
        assert_ne!(ItemId::new(a, 1), ItemId::new(b, 1));
        assert_ne!(ItemId::new(a, 1), ItemId::new(a, 2));
        assert_eq!(ItemId::new(a, 1).creator(), a);
        assert_eq!(ItemId::new(a, 7).sequence(), 7);
    }
}
