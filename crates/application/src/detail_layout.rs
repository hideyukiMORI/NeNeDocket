//! 詳細のペインの中の配置（Issue #122・ADR 0022）。上から: 番号と「削除」・題名・状態の札 6 つ（3 つずつ 2 段）・
//! 担当・期限・ラベル（左に項目名）・区画の切り替え・区画の中身。純関数で、寸法は呼び手が倍率を掛けて渡す。

use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::layout::Layout;
use nene_docket_core::pixel_rect::PixelRect;

/// 状態の札の数。
pub const CHIPS: usize = 6;
/// 項目名付きの欄の数（担当・期限・ラベル）。
pub const FIELDS: usize = 3;
/// 区画の切り替えの札の数。
pub const TABS: usize = 3;

/// ペインの部品の矩形。課題を選んでいなければ区画（台帳全体の履歴）だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetailLayout {
    pane: PixelRect,
    number: PixelRect,
    delete: PixelRect,
    title: PixelRect,
    chips: [PixelRect; CHIPS],
    names: [PixelRect; FIELDS],
    fields: [PixelRect; FIELDS],
    tabs: [PixelRect; TABS],
    section: PixelRect,
}

impl DetailLayout {
    /// 課題を選んでいるときの配置。`row` は 1 段の高さ、`gap` は隙間、`button` は「削除」の幅、`name` は項目名の幅。
    #[must_use]
    pub fn of_item(pane: PixelRect, row: u16, gap: u16, (button, name): (u16, u16)) -> Self {
        let mut layout = Layout::inside(pane, gap);
        let head = layout.row(row, &[ColumnWidth::Fill, ColumnWidth::Fixed(button)]);
        let title = layout.row(row, &[ColumnWidth::Fill]);
        let thirds = [ColumnWidth::Fill, ColumnWidth::Fill, ColumnWidth::Fill];
        let first = layout.row(row, &thirds);
        let second = layout.row(row, &thirds);
        let labelled = [ColumnWidth::Fixed(name), ColumnWidth::Fill];
        let rows: [alloc::vec::Vec<PixelRect>; FIELDS] =
            core::array::from_fn(|_| layout.row(row, &labelled));
        let tabs = layout.row(row, &thirds);
        Self {
            pane,
            number: at(&head, 0),
            delete: at(&head, 1),
            title: at(&title, 0),
            chips: core::array::from_fn(|index| {
                if index < 3 {
                    at(&first, index)
                } else {
                    at(&second, index - 3)
                }
            }),
            names: core::array::from_fn(|index| {
                rows.get(index).map_or(EMPTY, |cells| at(cells, 0))
            }),
            fields: core::array::from_fn(|index| {
                rows.get(index).map_or(EMPTY, |cells| at(cells, 1))
            }),
            tabs: core::array::from_fn(|index| at(&tabs, index)),
            section: layout.remaining(),
        }
    }

    /// 課題を選んでいないときの配置。区画（台帳全体の履歴）がペイン全体。
    #[must_use]
    pub const fn of_docket(pane: PixelRect) -> Self {
        Self {
            pane,
            number: EMPTY,
            delete: EMPTY,
            title: EMPTY,
            chips: [EMPTY; CHIPS],
            names: [EMPTY; FIELDS],
            fields: [EMPTY; FIELDS],
            tabs: [EMPTY; TABS],
            section: pane,
        }
    }

    /// ペイン全体。
    #[must_use]
    pub const fn pane(&self) -> PixelRect {
        self.pane
    }

    /// 番号。
    #[must_use]
    pub const fn number(&self) -> PixelRect {
        self.number
    }

    /// 「削除」。
    #[must_use]
    pub const fn delete(&self) -> PixelRect {
        self.delete
    }

    /// 題名の欄。
    #[must_use]
    pub const fn title(&self) -> PixelRect {
        self.title
    }

    /// 状態の札（`ItemState::ALL` の順）。
    #[must_use]
    pub const fn chips(&self) -> [PixelRect; CHIPS] {
        self.chips
    }

    /// 項目名（担当・期限・ラベル）。
    #[must_use]
    pub const fn names(&self) -> [PixelRect; FIELDS] {
        self.names
    }

    /// 欄（担当・期限・ラベル）。
    #[must_use]
    pub const fn fields(&self) -> [PixelRect; FIELDS] {
        self.fields
    }

    /// 区画の切り替えの札（`DetailSection::ALL` の順）。
    #[must_use]
    pub const fn tabs(&self) -> [PixelRect; TABS] {
        self.tabs
    }

    /// 区画の中身。
    #[must_use]
    pub const fn section(&self) -> PixelRect {
        self.section
    }
}

/// 空の矩形。
const EMPTY: PixelRect = PixelRect::new(0, 0, 0, 0);

fn at(rects: &[PixelRect], index: usize) -> PixelRect {
    rects.get(index).copied().unwrap_or(EMPTY)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PANE: PixelRect = PixelRect::new(600, 44, 380, 400);

    #[test]
    fn the_head_title_and_chips_stack_from_the_top() {
        let layout = DetailLayout::of_item(PANE, 28, 8, (70, 72));
        assert_eq!(layout.number(), PixelRect::new(600, 44, 302, 28));
        assert_eq!(layout.delete(), PixelRect::new(910, 44, 70, 28));
        assert_eq!(layout.title(), PixelRect::new(600, 80, 380, 28));
        assert_eq!(layout.chips()[0].y(), 116);
        assert_eq!(layout.chips()[3].y(), 152);
        assert_eq!(layout.chips()[3].x(), layout.chips()[0].x());
    }

    #[test]
    fn the_named_fields_tabs_and_section_take_the_rest() {
        let layout = DetailLayout::of_item(PANE, 28, 8, (70, 72));
        assert_eq!(layout.names()[0], PixelRect::new(600, 188, 72, 28));
        assert_eq!(layout.fields()[2].y(), 260);
        assert_eq!(layout.tabs()[0].y(), 296);
        assert_eq!(layout.section(), PixelRect::new(600, 332, 380, 112));
        let docket = DetailLayout::of_docket(PANE);
        assert_eq!(docket.section(), PANE);
        assert!(docket.title().is_empty());
    }
}
