//! 部品の ID。

/// フレームをまたいで同じ部品を指す番号。application が部品ごとに決める。
///
/// 即時モードではフレームごとに部品を問い合わせ直すので、ホバー・押下・フォーカスの持ち主をこれで覚える（ADR 0006）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WidgetId {
    number: u32,
}

impl WidgetId {
    /// 番号から作る。
    #[must_use]
    pub const fn new(number: u32) -> Self {
        Self { number }
    }

    /// 表の行のように、同じ種類の部品を `index` で区別した ID。`base` から数える。
    #[must_use]
    pub const fn indexed(self, index: u32) -> Self {
        Self::new(self.number.saturating_add(index))
    }

    /// 番号。
    #[must_use]
    pub const fn number(self) -> u32 {
        self.number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_ids_are_distinct_and_saturate() {
        let base = WidgetId::new(100);
        assert_eq!(base.indexed(0), base);
        assert_ne!(base.indexed(1), base.indexed(2));
        assert_eq!(base.indexed(1).number(), 101);
        assert_eq!(WidgetId::new(u32::MAX).indexed(5).number(), u32::MAX);
    }
}
