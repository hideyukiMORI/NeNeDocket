//! 操作の ID。論理時計とアカウントの組で、全順序を持つ（ADR 0004 の 6）。

use crate::account_id::AccountId;
use crate::logical_clock::LogicalClock;

/// 同じアカウントは操作ごとに時計を進めるので、組は一意。並びは時計が先、同じ時計ならアカウントの番号。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpId {
    clock: LogicalClock,
    account: AccountId,
}

impl OpId {
    /// 時計とアカウントから作る。
    #[must_use]
    pub const fn new(clock: LogicalClock, account: AccountId) -> Self {
        Self { clock, account }
    }

    /// 論理時計。
    #[must_use]
    pub const fn clock(self) -> LogicalClock {
        self.clock
    }

    /// アカウント。
    #[must_use]
    pub const fn account(self) -> AccountId {
        self.account
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_by_clock_then_account() {
        let a = AccountId::new(1);
        let b = AccountId::new(2);
        assert!(OpId::new(LogicalClock::new(1), b) < OpId::new(LogicalClock::new(2), a));
        assert!(OpId::new(LogicalClock::new(2), a) < OpId::new(LogicalClock::new(2), b));
        assert_eq!(OpId::new(LogicalClock::new(2), a).clock().tick(), 2);
        assert_eq!(OpId::new(LogicalClock::new(2), a).account(), a);
    }
}
