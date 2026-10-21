//! 描く文字列の中のカーソルと選択。

/// 文字の添字で表したカーソルと選択の起点。UI がフォントで画素の位置に直す（ADR 0006）。
/// 複数行の欄で選択が行をまたぐとき、カーソルの無い行には棒を描かない（`bar`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextCaret {
    cursor: usize,
    anchor: usize,
    bar: bool,
}

impl TextCaret {
    /// カーソルと起点から作る。選択が無ければ同じ値。棒は描く。
    #[must_use]
    pub const fn new(cursor: usize, anchor: usize) -> Self {
        Self {
            cursor,
            anchor,
            bar: true,
        }
    }

    /// 棒を描かないもの（選択だけ）。
    #[must_use]
    pub const fn without_bar(self) -> Self {
        Self { bar: false, ..self }
    }

    /// カーソルの棒を描くか。
    #[must_use]
    pub const fn shows_bar(self) -> bool {
        self.bar
    }

    /// カーソル（文字の添字）。
    #[must_use]
    pub const fn cursor(self) -> usize {
        self.cursor
    }

    /// 選択の起点（文字の添字）。
    #[must_use]
    pub const fn anchor(self) -> usize {
        self.anchor
    }

    /// 選択の始まり（小さいほう）。
    #[must_use]
    pub const fn start(self) -> usize {
        if self.cursor < self.anchor {
            self.cursor
        } else {
            self.anchor
        }
    }

    /// 選択の終わり（大きいほう・含まない）。
    #[must_use]
    pub const fn end(self) -> usize {
        if self.cursor < self.anchor {
            self.anchor
        } else {
            self.cursor
        }
    }

    /// 選択があるか。
    #[must_use]
    pub const fn has_selection(self) -> bool {
        self.cursor != self.anchor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_and_end_are_ordered_whichever_way_the_selection_was_made() {
        let forward = TextCaret::new(5, 2);
        let backward = TextCaret::new(2, 5);
        assert_eq!((forward.start(), forward.end()), (2, 5));
        assert_eq!((backward.start(), backward.end()), (2, 5));
        assert!(forward.has_selection());
        assert!(!TextCaret::new(3, 3).has_selection());
        assert_eq!(forward.cursor(), 5);
        assert_eq!(forward.anchor(), 2);
        assert!(forward.shows_bar());
        assert!(!forward.without_bar().shows_bar());
        assert_eq!(forward.without_bar().end(), 5);
    }
}
