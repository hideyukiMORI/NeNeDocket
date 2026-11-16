//! 詳細のペインの中の配置（Issue #122 / #127・ADR 0022）。上から: 番号と状態と「…」・起票と更新・題名・
//! 状態の札（1 段目に 未着手→対応中→確認中→完了、2 段目に 保留・中止）・担当・期限・ラベル（左に項目名）・
//! 区画のタブ・区画の中身。純関数で、寸法は呼び手が倍率を掛けて渡す。

use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::layout::Layout;
use nene_docket_core::pixel_rect::PixelRect;

/// 状態の札の数。
pub const CHIPS: usize = 6;
/// 状態の札の 1 段の列の数（1 段目は進む順の 4 つ、2 段目は脇の 2 つを左から）。
const CHIP_COLUMNS: usize = 4;
/// 状態の札の置き場（`ItemState::ALL` の順に、段と列）。保留と中止は 2 段目へ。
const CHIP_PLACES: [(usize, usize); CHIPS] = [(0, 0), (0, 1), (0, 2), (1, 0), (0, 3), (1, 1)];
/// 状態の札を見えている順に（`ItemState::ALL` の添字）。Tab はこの順で進む（WCAG 2.4.3）。
pub const CHIP_TAB_ORDER: [usize; CHIPS] = [0, 1, 2, 4, 3, 5];
/// 項目名付きの欄の数（担当・期限・ラベル）。
pub const FIELDS: usize = 3;
/// 区画の切り替えの札の数。
pub const TABS: usize = 3;

/// ペインの部品の矩形。課題を選んでいなければ区画（台帳全体の履歴）だけ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DetailLayout {
    pane: PixelRect,
    number: PixelRect,
    more: PixelRect,
    stamps: PixelRect,
    title_name: PixelRect,
    title: PixelRect,
    chips: [PixelRect; CHIPS],
    names: [PixelRect; FIELDS],
    fields: [PixelRect; FIELDS],
    tabs: [PixelRect; TABS],
    section: PixelRect,
}

impl DetailLayout {
    /// 課題を選んでいるときの配置。`row` は 1 段の高さ、`gap` は隙間、`name` は項目名の幅。「…」は 1 段の高さの正方形。
    #[must_use]
    pub fn of_item(pane: PixelRect, row: u16, gap: u16, name: u16) -> Self {
        let mut layout = Layout::inside(pane, gap);
        let head = layout.row(row, &[ColumnWidth::Fill, ColumnWidth::Fixed(row)]);
        let stamps = layout.row(row, &[ColumnWidth::Fill]);
        let labelled = [ColumnWidth::Fixed(name), ColumnWidth::Fill];
        let title = layout.row(row, &labelled);
        let quarters = [ColumnWidth::Fill; CHIP_COLUMNS];
        let chip_rows = [layout.row(row, &quarters), layout.row(row, &quarters)];
        let rows: [alloc::vec::Vec<PixelRect>; FIELDS] =
            core::array::from_fn(|_| layout.row(row, &labelled));
        let tabs = layout.row(row, &[ColumnWidth::Fill; TABS]);
        Self {
            pane,
            number: at(&head, 0),
            more: at(&head, 1),
            stamps: at(&stamps, 0),
            title_name: at(&title, 0),
            title: at(&title, 1),
            chips: CHIP_PLACES
                .map(|(line, column)| chip_rows.get(line).map_or(EMPTY, |cells| at(cells, column))),
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
            more: EMPTY,
            stamps: EMPTY,
            title_name: EMPTY,
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

    /// 番号と状態。
    #[must_use]
    pub const fn number(&self) -> PixelRect {
        self.number
    }

    /// 「…」（削除などの普段は使わない操作の入口）。
    #[must_use]
    pub const fn more(&self) -> PixelRect {
        self.more
    }

    /// 起票と更新の 1 行。
    #[must_use]
    pub const fn stamps(&self) -> PixelRect {
        self.stamps
    }

    /// 題名の項目名。
    #[must_use]
    pub const fn title_name(&self) -> PixelRect {
        self.title_name
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
    fn the_head_stamps_title_and_chips_stack_from_the_top() {
        let layout = DetailLayout::of_item(PANE, 28, 8, 72);
        assert_eq!(layout.number(), PixelRect::new(600, 44, 344, 28));
        assert_eq!(layout.more(), PixelRect::new(952, 44, 28, 28));
        assert_eq!(layout.stamps(), PixelRect::new(600, 80, 380, 28));
        assert_eq!(layout.title_name(), PixelRect::new(600, 116, 72, 28));
        assert_eq!(layout.title(), PixelRect::new(680, 116, 300, 28));
        let chips = layout.chips();
        // 1 段目: 未着手・対応中・確認中・完了。2 段目: 保留・中止。
        assert_eq!(chips.map(PixelRect::y), [152, 152, 152, 188, 152, 188]);
        assert_eq!(chips[3].x(), chips[0].x());
        assert_eq!(chips[5].x(), chips[1].x());
        assert!(chips[4].x() > chips[2].x());
    }

    #[test]
    fn the_named_fields_tabs_and_section_take_the_rest() {
        let layout = DetailLayout::of_item(PANE, 28, 8, 72);
        assert_eq!(layout.names()[0], PixelRect::new(600, 224, 72, 28));
        assert_eq!(layout.fields()[2].y(), 296);
        assert_eq!(layout.tabs()[0].y(), 332);
        assert_eq!(layout.section(), PixelRect::new(600, 368, 380, 76));
        let docket = DetailLayout::of_docket(PANE);
        assert_eq!(docket.section(), PANE);
        assert!(docket.title().is_empty());
    }
}
