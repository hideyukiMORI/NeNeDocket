//! 論理時計（Lamport）。操作の全順序を決める（ADR 0004 の 7）。

/// 自分の操作は、これまでに見た最大の時計より 1 大きい値を持つ。壁時計は順序に使わない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LogicalClock {
    tick: u64,
}

impl LogicalClock {
    /// 値から作る。
    #[must_use]
    pub const fn new(tick: u64) -> Self {
        Self { tick }
    }

    /// まだ何も見ていない。
    #[must_use]
    pub const fn zero() -> Self {
        Self::new(0)
    }

    /// 見た時計のうち大きいほう。
    #[must_use]
    pub const fn seen(self, other: Self) -> Self {
        if other.tick > self.tick { other } else { self }
    }

    /// 次の操作に付ける値（1 大きい。上限で止まる）。
    #[must_use]
    pub const fn next(self) -> Self {
        Self::new(self.tick.saturating_add(1))
    }

    /// 値。
    #[must_use]
    pub const fn tick(self) -> u64 {
        self.tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_is_one_past_the_largest_seen() {
        let mine = LogicalClock::new(3).seen(LogicalClock::new(7));
        assert_eq!(mine.next(), LogicalClock::new(8));
        assert_eq!(
            LogicalClock::new(9).seen(LogicalClock::new(7)),
            LogicalClock::new(9)
        );
        assert_eq!(LogicalClock::zero().next().tick(), 1);
        assert_eq!(LogicalClock::new(u64::MAX).next().tick(), u64::MAX);
    }
}
