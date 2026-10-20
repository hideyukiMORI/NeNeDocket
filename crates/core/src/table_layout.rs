//! 表の配置。見えている行だけを計算する（FR-002・1 万件でも 1 フレーム）。

use alloc::vec::Vec;

use crate::column_width::{self, ColumnWidth};
use crate::pixel_point::PixelPoint;
use crate::pixel_rect::PixelRect;

/// 外枠・ヘッダの高さ・行の高さ・行の数。列は `with_columns` で持たせる。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableLayout {
    bounds: PixelRect,
    header_height: u16,
    row_height: u16,
    row_count: u32,
    columns: Vec<ColumnWidth>,
}

impl TableLayout {
    /// 外枠・ヘッダの高さ・行の高さ（1 以上に丸める）・行の数から作る。列は無い。
    #[must_use]
    pub const fn new(
        bounds: PixelRect,
        header_height: u16,
        row_height: u16,
        row_count: u32,
    ) -> Self {
        Self {
            bounds,
            header_height,
            row_height: if row_height == 0 { 1 } else { row_height },
            row_count,
            columns: Vec::new(),
        }
    }

    /// 列の幅を持たせた表。
    #[must_use]
    pub fn with_columns(self, columns: Vec<ColumnWidth>) -> Self {
        Self { columns, ..self }
    }

    /// ヘッダの矩形。
    #[must_use]
    pub const fn header(&self) -> PixelRect {
        PixelRect::new(
            self.bounds.x(),
            self.bounds.y(),
            self.bounds.width(),
            self.header_height,
        )
    }

    /// 行が並ぶ領域（ヘッダの下）。
    #[must_use]
    pub const fn body(&self) -> PixelRect {
        PixelRect::new(
            self.bounds.x(),
            self.bounds.y().saturating_add(self.header_height),
            self.bounds.width(),
            self.bounds.height().saturating_sub(self.header_height),
        )
    }

    /// 全行を並べた高さ。スクロールの中身の高さ。
    #[must_use]
    pub fn content_height(&self) -> u32 {
        self.row_count.saturating_mul(u32::from(self.row_height))
    }

    /// 行の数。
    #[must_use]
    pub const fn row_count(&self) -> u32 {
        self.row_count
    }

    /// `offset` だけ送ったとき、領域に掛かる行の番号と矩形。矩形は領域で切らずに返す（切るのは描くとき）。
    #[must_use]
    pub fn visible_rows(&self, offset: u32) -> Vec<(u32, PixelRect)> {
        let body = self.body();
        let height = u32::from(self.row_height);
        let first = offset / height;
        let last = offset
            .saturating_add(u32::from(body.height()))
            .saturating_add(height.saturating_sub(1))
            / height;
        (first..last.min(self.row_count))
            .filter_map(|index| Some((index, self.row_rect(index, offset)?)))
            .collect()
    }

    /// `offset` だけ送ったとき、点の下にある行の番号。ヘッダや行の無い所なら `None`。
    #[must_use]
    pub fn row_at(&self, offset: u32, point: PixelPoint) -> Option<u32> {
        let body = self.body();
        if !body.contains(point) {
            return None;
        }
        let into = u32::from(point.y() - body.y()).saturating_add(offset);
        let index = into / u32::from(self.row_height);
        (index < self.row_count).then_some(index)
    }

    /// 行の矩形を列に割る。
    #[must_use]
    pub fn cells(&self, row: PixelRect) -> Vec<PixelRect> {
        column_width::split(row, &self.columns, 0)
    }

    /// `index` の行の矩形。領域の上下にはみ出していても、`u16` に収まる限り返す。
    fn row_rect(&self, index: u32, offset: u32) -> Option<PixelRect> {
        let body = self.body();
        let top = i64::from(index)
            .checked_mul(i64::from(self.row_height))?
            .checked_sub(i64::from(offset))?
            .checked_add(i64::from(body.y()))?;
        Some(PixelRect::new(
            body.x(),
            u16::try_from(top).ok()?,
            body.width(),
            self.row_height,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> TableLayout {
        TableLayout::new(PixelRect::new(0, 0, 300, 120), 20, 25, 10_000)
    }

    #[test]
    fn header_and_body_split_the_bounds() {
        let table = table();
        assert_eq!(table.header(), PixelRect::new(0, 0, 300, 20));
        assert_eq!(table.body(), PixelRect::new(0, 20, 300, 100));
        assert_eq!(table.content_height(), 250_000);
        assert_eq!(table.row_count(), 10_000);
    }

    #[test]
    fn only_the_rows_touching_the_body_are_laid_out() {
        let table = table();
        let rows = table.visible_rows(0);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows.first(), Some(&(0, PixelRect::new(0, 20, 300, 25))));
        let scrolled = table.visible_rows(40);
        assert_eq!(scrolled.first().map(|row| row.0), Some(1));
        assert_eq!(scrolled.len(), 5);
        assert!(scrolled.iter().all(|row| row.1.height() == 25));
        let end = table.visible_rows(table.content_height());
        assert!(end.is_empty());
    }

    #[test]
    fn a_partially_scrolled_first_row_starts_above_the_body() {
        let table = table();
        let scrolled = table.visible_rows(30);
        assert_eq!(scrolled.first(), Some(&(1, PixelRect::new(0, 15, 300, 25))));
        let far = table.visible_rows(249_990);
        assert_eq!(far.first().map(|row| row.0), Some(9_999));
        assert_eq!(far.len(), 1);
    }

    #[test]
    fn row_at_maps_a_point_through_the_offset() {
        let table = table();
        assert_eq!(table.row_at(0, PixelPoint::new(5, 20)), Some(0));
        assert_eq!(table.row_at(0, PixelPoint::new(5, 45)), Some(1));
        assert_eq!(table.row_at(30, PixelPoint::new(5, 20)), Some(1));
        assert_eq!(table.row_at(0, PixelPoint::new(5, 5)), None);
        assert_eq!(
            TableLayout::new(PixelRect::new(0, 0, 300, 120), 20, 25, 1)
                .row_at(0, PixelPoint::new(5, 60)),
            None
        );
    }

    #[test]
    fn cells_follow_the_columns_and_zero_row_height_is_lifted_to_one() {
        let table = table().with_columns(alloc::vec![ColumnWidth::Fixed(100), ColumnWidth::Fill]);
        let cells = table.cells(PixelRect::new(0, 20, 300, 25));
        assert_eq!(
            cells,
            alloc::vec![
                PixelRect::new(0, 20, 100, 25),
                PixelRect::new(100, 20, 200, 25)
            ]
        );
        assert_eq!(
            TableLayout::new(PixelRect::new(0, 0, 10, 10), 0, 0, 3).content_height(),
            3
        );
    }
}
