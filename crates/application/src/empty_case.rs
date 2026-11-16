//! 表が 0 件のとき、どの案内を出すか（Issue #126・docs/design/2026-11-09-toolbar-and-settings.md 第 4 節）。

use crate::list_filter::ListFilter;

/// 0 件の理由。理由ごとに次の一手が違う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmptyCase {
    /// 台帳に課題が 1 つも無い（「＋ 課題を追加」）。
    Docket,
    /// 検索に合う課題が台帳のどこにも無い（「検索を消す」・その字を題名にして追加）。
    Search,
    /// 検索には合うが、絞り込みの札が隠している（「絞り込みを外す」）。
    Filter(ListFilter),
    /// 台帳に課題はあるが、「自分の担当」のタブに 1 つも無い（「全件を見る」・Issue #145）。
    Mine,
}

impl EmptyCase {
    /// 表が 0 件のときの理由。`total` は台帳の全件、`in_view` はタブの中の件数、`searched` はタブの中で
    /// 検索だけを当てた件数。
    #[must_use]
    pub const fn of(
        (total, in_view, searched): (usize, usize, usize),
        searching: bool,
        filter: ListFilter,
    ) -> Self {
        if total == 0 {
            Self::Docket
        } else if in_view == 0 {
            Self::Mine
        } else if searching && searched == 0 {
            Self::Search
        } else {
            Self::Filter(filter)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_docket_comes_first_then_a_search_that_finds_nothing_then_the_filter() {
        assert_eq!(
            EmptyCase::of((0, 0, 0), true, ListFilter::Overdue),
            EmptyCase::Docket
        );
        assert_eq!(
            EmptyCase::of((5, 5, 0), true, ListFilter::Overdue),
            EmptyCase::Search
        );
        // 検索に合う課題はあるが札が隠しているなら、外すのは札。
        assert_eq!(
            EmptyCase::of((5, 5, 2), true, ListFilter::Overdue),
            EmptyCase::Filter(ListFilter::Overdue)
        );
        assert_eq!(
            EmptyCase::of((5, 5, 5), false, ListFilter::Review),
            EmptyCase::Filter(ListFilter::Review)
        );
        // 自分の担当のタブが空なら、検索や札より先にそれを言う。
        assert_eq!(
            EmptyCase::of((5, 0, 0), true, ListFilter::Open),
            EmptyCase::Mine
        );
    }
}
