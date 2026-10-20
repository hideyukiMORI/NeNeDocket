//! 段を上から積む配置（microui の `layout_row` の形・ADR 0006）。

use alloc::vec::Vec;

use crate::column_width::{self, ColumnWidth};
use crate::pixel_rect::PixelRect;

/// 外枠の中に、段を上から順に置く。各段は列に割る。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layout {
    bounds: PixelRect,
    gap: u16,
    next_y: u16,
}

impl Layout {
    /// 外枠と、段と段・列と列の隙間から始める。
    #[must_use]
    pub const fn inside(bounds: PixelRect, gap: u16) -> Self {
        Self {
            bounds,
            gap,
            next_y: bounds.y(),
        }
    }

    /// 高さ `height` の段を置き、`widths` の列に割った矩形を左から返す。次の段はその下。
    pub fn row(&mut self, height: u16, widths: &[ColumnWidth]) -> Vec<PixelRect> {
        let band = PixelRect::new(self.bounds.x(), self.next_y, self.bounds.width(), height);
        self.next_y = self.next_y.saturating_add(height).saturating_add(self.gap);
        column_width::split(band, widths, self.gap)
    }

    /// まだ使っていない残りの矩形。使い切っていれば高さ 0。
    #[must_use]
    pub fn remaining(&self) -> PixelRect {
        let bottom = self.bounds.y().saturating_add(self.bounds.height());
        PixelRect::new(
            self.bounds.x(),
            self.next_y,
            self.bounds.width(),
            bottom.saturating_sub(self.next_y),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_stack_downward_with_a_gap() {
        let mut layout = Layout::inside(PixelRect::new(10, 10, 200, 100), 4);
        let first = layout.row(24, &[ColumnWidth::Fill]);
        let second = layout.row(24, &[ColumnWidth::Fixed(50), ColumnWidth::Fill]);
        assert_eq!(first, alloc::vec![PixelRect::new(10, 10, 200, 24)]);
        assert_eq!(
            second,
            alloc::vec![
                PixelRect::new(10, 38, 50, 24),
                PixelRect::new(64, 38, 146, 24)
            ]
        );
        assert_eq!(layout.remaining(), PixelRect::new(10, 66, 200, 44));
    }

    #[test]
    fn remaining_is_empty_once_the_bounds_are_used_up() {
        let mut layout = Layout::inside(PixelRect::new(0, 0, 50, 20), 0);
        layout.row(30, &[ColumnWidth::Fill]);
        assert!(layout.remaining().is_empty());
    }
}
