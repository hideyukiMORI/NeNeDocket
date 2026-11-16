//! 道具の行と 0 件の案内が使う件数（Issue #124 / #126 / #145）。1 フレームに 1 度だけ数える。

/// 札ごとの件数・タブの中の全件・台帳の全件・タブごとの未完了の件数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListCounts {
    by_filter: [usize; 5],
    in_view: usize,
    total: usize,
    open_by_tab: [usize; 2],
}

impl ListCounts {
    /// 数えた値から。`by_filter` は `ListFilter::ALL` の順（タブの中で検索だけを当てた数）、`open_by_tab` は
    /// `ViewTab::ALL` の順（検索は当てない・閉じていない課題）。
    #[must_use]
    pub const fn new(
        by_filter: [usize; 5],
        (in_view, total): (usize, usize),
        open_by_tab: [usize; 2],
    ) -> Self {
        Self {
            by_filter,
            in_view,
            total,
            open_by_tab,
        }
    }

    /// `ListFilter::ALL` の `index` 番目の札の件数。
    #[must_use]
    pub fn filter(&self, index: usize) -> usize {
        self.by_filter.get(index).copied().unwrap_or(0)
    }

    /// タブの中で検索だけを当てた件数（札「すべて」の件数）。
    #[must_use]
    pub fn searched(&self) -> usize {
        self.filter(4)
    }

    /// タブの中の全件（検索も札も当てない）。
    #[must_use]
    pub const fn in_view(&self) -> usize {
        self.in_view
    }

    /// 台帳の全件。
    #[must_use]
    pub const fn total(&self) -> usize {
        self.total
    }

    /// `ViewTab::ALL` の `index` 番目のタブの未完了の件数。
    #[must_use]
    pub fn open_in_tab(&self, index: usize) -> usize {
        self.open_by_tab.get(index).copied().unwrap_or(0)
    }
}
