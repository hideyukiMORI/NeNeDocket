//! 画面の状態の所有者（ARC-004）。検索欄・状態の絞り込み・完了の表示・表の選択とスクロール。
//!
//! 即時モード（ADR 0006）。`step` が 1 フレームの入力を受け、鍵を前のフレームのフォーカスに配り、
//! 部品を問い合わせ、描画命令を返す。業務の判断（どの行を出すか・何が選ばれたか）はすべてここにあり、UI には無い（ARC-011）。

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use nene_docket_core::button;
use nene_docket_core::checkbox;
use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::layout::Layout;
use nene_docket_core::paint::Paint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::response::Response;
use nene_docket_core::response_phase::ResponsePhase;
use nene_docket_core::scroll_bar;
use nene_docket_core::scroll_state::ScrollState;
use nene_docket_core::select_box;
use nene_docket_core::select_state::SelectState;
use nene_docket_core::table_layout::TableLayout;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::text_edit::TextEdit;
use nene_docket_core::text_field;
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;

use crate::docket_frame::DocketFrame;
use crate::environment::Environment;
use crate::frame_input::FrameInput;
use crate::item_state::ItemState;
use crate::sample_item::SampleItem;

/// 部品の ID。表の行は `ROW` に課題の番号を足す。候補は `OPTION` に番号を足す。
const SEARCH: WidgetId = WidgetId::new(1);
const FILTER: WidgetId = WidgetId::new(2);
const SHOW_DONE: WidgetId = WidgetId::new(3);
const ADD: WidgetId = WidgetId::new(4);
const SCROLL: WidgetId = WidgetId::new(5);
const OPTION: WidgetId = WidgetId::new(100);
const ROW: WidgetId = WidgetId::new(1_000);

/// 1x の寸法（SPECIFICATION 第 2 節・決め打ちしない）。
const MARGIN_PX: u16 = 8;
const ROW_PX: u16 = 28;
const FILTER_WIDTH_PX: u16 = 140;
const SHOW_DONE_WIDTH_PX: u16 = 120;
const ADD_WIDTH_PX: u16 = 80;
const SCROLL_WIDTH_PX: u16 = 12;
const NUMBER_WIDTH_PX: u16 = 56;
const STATE_WIDTH_PX: u16 = 110;
const ASSIGNEE_WIDTH_PX: u16 = 90;
/// ホイール 1 刻みで送る行数。
const WHEEL_ROWS: i32 = 3;

/// 絞り込みの候補。先頭は「すべて」。
const FILTER_OPTIONS: [&str; 4] = ["All", "Open", "In progress", "Done"];

/// 画面の状態。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocketScreen {
    scale: UiScale,
    interaction: Interaction,
    search: TextEdit,
    filter: SelectState,
    show_done: bool,
    scroll: ScrollState,
    selected: Option<u32>,
    items: Vec<SampleItem>,
}

impl DocketScreen {
    /// 環境と見本の課題から始める。
    #[must_use]
    pub fn open(environment: Environment, items: Vec<SampleItem>) -> Self {
        Self {
            scale: environment.scale(),
            interaction: Interaction::idle(),
            search: TextEdit::empty(),
            filter: SelectState::closed(FILTER_OPTIONS.len(), 0),
            show_done: true,
            scroll: ScrollState::top(0, 0),
            selected: None,
            items,
        }
    }

    /// 1 フレーム。鍵 → 部品の問い合わせ → 描画命令。
    pub fn step(&mut self, input: &FrameInput) -> DocketFrame {
        for key in input.keys() {
            self.key(*key);
        }
        if self.interaction.focus() == Some(SEARCH) {
            self.search = self.search.with_preedit(input.preedit().cloned());
        }
        let bounds = input.bounds();
        let mut layout = Layout::inside(bounds.inset(self.px(MARGIN_PX)), self.px(MARGIN_PX));
        let toolbar = layout.row(self.px(ROW_PX), &self.toolbar_widths());
        let remaining = layout.remaining();
        let table = self.table_layout(remaining);
        let track = Self::track_of(remaining, self.px(SCROLL_WIDTH_PX));
        self.scroll = self
            .scroll
            .resized(table.content_height(), u32::from(table.body().height()));

        self.interaction.begin(input.pointer());
        let mut commands = alloc::vec![DrawCommand::Fill {
            rect: bounds,
            paint: Paint::Background,
        }];
        let (filter_rect, filter_response) = self.probe_toolbar(&toolbar, &mut commands);
        self.probe_table(&table, &mut commands);
        self.probe_scroll(track, &mut commands);
        self.probe_options(filter_rect);
        commands.extend(select_box::select_box(
            filter_rect,
            &FILTER_OPTIONS,
            self.filter,
            filter_response,
        ));
        self.interaction.end();
        DocketFrame::new(self.scale, commands)
    }

    /// 検索欄の文字列。
    #[must_use]
    pub fn search_text(&self) -> &str {
        self.search.text()
    }

    /// 選んでいる課題の番号。
    #[must_use]
    pub const fn selected(&self) -> Option<u32> {
        self.selected
    }

    /// 完了を表に出すか。
    #[must_use]
    pub const fn shows_done(&self) -> bool {
        self.show_done
    }

    /// 絞り込みの状態。
    #[must_use]
    pub const fn filter(&self) -> SelectState {
        self.filter
    }

    /// スクロールの位置。
    #[must_use]
    pub const fn scroll(&self) -> ScrollState {
        self.scroll
    }

    /// 課題の数。
    #[must_use]
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// 絞り込んだ後の課題。表に出す順。
    #[must_use]
    pub fn visible_items(&self) -> Vec<&SampleItem> {
        self.visible_indices()
            .into_iter()
            .filter_map(|index| self.items.get(index))
            .collect()
    }

    /// 絞り込んだ後の課題の、`items` の中の添字。表に出す順。
    fn visible_indices(&self) -> Vec<usize> {
        let wanted = match self.filter.chosen() {
            1 => Some(ItemState::Open),
            2 => Some(ItemState::InProgress),
            3 => Some(ItemState::Done),
            _ => None,
        };
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| wanted.is_none_or(|state| item.state() == state))
            .filter(|(_, item)| self.show_done || item.state() != ItemState::Done)
            .filter(|(_, item)| {
                self.search.text().is_empty() || item.title().contains(self.search.text())
            })
            .map(|(index, _)| index)
            .collect()
    }

    /// 1x の寸法に倍率を掛ける。
    const fn px(&self, base: u16) -> u16 {
        self.scale.px(base)
    }

    fn toolbar_widths(&self) -> [ColumnWidth; 4] {
        [
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(FILTER_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(SHOW_DONE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(ADD_WIDTH_PX)),
        ]
    }

    fn table_layout(&self, remaining: PixelRect) -> TableLayout {
        let width = remaining.width().saturating_sub(self.px(SCROLL_WIDTH_PX));
        let bounds = PixelRect::new(remaining.x(), remaining.y(), width, remaining.height());
        let rows = u32::try_from(self.visible_indices().len()).unwrap_or(u32::MAX);
        TableLayout::new(bounds, self.px(ROW_PX), self.px(ROW_PX), rows).with_columns(alloc::vec![
            ColumnWidth::Fixed(self.px(NUMBER_WIDTH_PX)),
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(STATE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(ASSIGNEE_WIDTH_PX)),
        ])
    }

    const fn track_of(remaining: PixelRect, width: u16) -> PixelRect {
        PixelRect::new(
            remaining.right().saturating_sub(width).saturating_add(1),
            remaining.y(),
            width,
            remaining.height(),
        )
    }

    /// 鍵を、前のフレームのフォーカスの持ち主へ。Tab と Esc は持ち主に依らない。
    fn key(&mut self, key: KeyIntent) {
        match key {
            KeyIntent::Tab => self.interaction.focus_next(),
            KeyIntent::BackTab => self.interaction.focus_previous(),
            KeyIntent::Escape if !self.filter.is_open() => self.interaction.blur(),
            KeyIntent::Char(_)
            | KeyIntent::Move { .. }
            | KeyIntent::Backspace
            | KeyIntent::Delete
            | KeyIntent::SelectAll
            | KeyIntent::Enter
            | KeyIntent::Escape
            | KeyIntent::Up
            | KeyIntent::Down => self.key_to_focus(key),
        }
    }

    fn key_to_focus(&mut self, key: KeyIntent) {
        match self.interaction.focus() {
            Some(SEARCH) => self.search = self.search.applied(key),
            Some(FILTER) => self.filter = self.filter.applied(key),
            Some(SHOW_DONE) if matches!(key, KeyIntent::Enter | KeyIntent::Char(' ')) => {
                self.show_done = !self.show_done;
            }
            Some(ADD) if key == KeyIntent::Enter => self.add_item(),
            Some(_) | None => self.key_to_table(key),
        }
    }

    /// フォーカスが無いときの上下はスクロール。
    fn key_to_table(&mut self, key: KeyIntent) {
        let rows = match key {
            KeyIntent::Up => -1_i32,
            KeyIntent::Down => 1_i32,
            KeyIntent::Char(_)
            | KeyIntent::Move { .. }
            | KeyIntent::Backspace
            | KeyIntent::Delete
            | KeyIntent::SelectAll
            | KeyIntent::Enter
            | KeyIntent::Escape
            | KeyIntent::Tab
            | KeyIntent::BackTab => 0_i32,
        };
        self.scroll = self
            .scroll
            .scrolled_by(rows.saturating_mul(i32::from(self.px(ROW_PX))));
    }

    fn add_item(&mut self) {
        let number = self
            .items
            .iter()
            .map(SampleItem::number)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        let title = if self.search.text().is_empty() {
            format!("New issue #{number}")
        } else {
            self.search.text().to_owned()
        };
        self.items.push(SampleItem::new(
            number,
            title,
            ItemState::Open,
            String::from("mori"),
        ));
        self.selected = Some(number);
    }

    /// 検索欄・絞り込み・完了の表示・追加。絞り込みの箱は候補が表の上に乗るので、最後に描く。
    fn probe_toolbar(
        &mut self,
        toolbar: &[PixelRect],
        commands: &mut Vec<DrawCommand>,
    ) -> (PixelRect, Response) {
        let [search_rect, filter_rect, done_rect, add_rect] = Self::four(toolbar);
        let search = self.interaction.probe(SEARCH, search_rect, true);
        commands.extend(text_field::text_field(search_rect, &self.search, search));
        let filter = self.interaction.probe(FILTER, filter_rect, true);
        if filter.clicked() {
            self.filter = self.filter.toggled();
        }
        let done = self.interaction.probe(SHOW_DONE, done_rect, true);
        if done.clicked() {
            self.show_done = !self.show_done;
        }
        commands.extend(checkbox::checkbox(
            done_rect,
            "Show done",
            self.show_done,
            done,
        ));
        let add = self.interaction.probe(ADD, add_rect, true);
        if add.clicked() {
            self.add_item();
        }
        commands.extend(button::button(add_rect, "Add", add));
        (filter_rect, filter)
    }

    /// 4 列の段。足りなければ空の矩形で埋める（起こらないが型の上では `Vec`）。
    fn four(rects: &[PixelRect]) -> [PixelRect; 4] {
        let empty = PixelRect::new(0, 0, 0, 0);
        let at = |index: usize| rects.get(index).copied().unwrap_or(empty);
        [at(0), at(1), at(2), at(3)]
    }

    /// 開いた絞り込みの候補。表より後に問い合わせるので、重なった所では候補が勝つ。候補の列は `select_box` が描く。
    fn probe_options(&mut self, filter_rect: PixelRect) {
        if !self.filter.is_open() {
            return;
        }
        let rects = select_box::option_rects(filter_rect, FILTER_OPTIONS.len());
        for (index, rect) in rects.into_iter().enumerate() {
            let id = OPTION.indexed(u32::try_from(index).unwrap_or(u32::MAX));
            if self.interaction.probe(id, rect, false).clicked() {
                self.filter = self.filter.chosen_at(index);
            }
        }
    }

    /// ヘッダと、見えている行だけ。行はクリックで選ぶ。
    fn probe_table(&mut self, table: &TableLayout, commands: &mut Vec<DrawCommand>) {
        let indices = self.visible_indices();
        let header = table.header();
        commands.push(DrawCommand::Fill {
            rect: header,
            paint: Paint::Surface,
        });
        for (cell, title) in table
            .cells(header)
            .into_iter()
            .zip(["#", "Title", "State", "Assignee"])
        {
            commands.push(Self::cell_text(
                cell,
                title,
                Paint::Muted,
                self.px(MARGIN_PX),
            ));
        }
        commands.push(DrawCommand::Clip(table.body()));
        for (index, rect) in table.visible_rows(self.scroll.offset()) {
            let Some(item) = usize::try_from(index)
                .ok()
                .and_then(|i| indices.get(i))
                .and_then(|&i| self.items.get(i))
                .cloned()
            else {
                continue;
            };
            let response = if self.filter.is_open() {
                Response::new(ResponsePhase::Idle, false, false)
            } else {
                self.interaction
                    .probe(ROW.indexed(item.number()), rect, false)
            };
            if response.clicked() {
                self.selected = Some(item.number());
            }
            self.row_commands((rect, table.cells(rect)), &item, commands);
        }
        commands.push(DrawCommand::Unclip);
    }

    fn row_commands(
        &self,
        (rect, cells): (PixelRect, Vec<PixelRect>),
        item: &SampleItem,
        commands: &mut Vec<DrawCommand>,
    ) {
        if self.selected == Some(item.number()) {
            commands.push(DrawCommand::Fill {
                rect,
                paint: Paint::Selection,
            });
        }
        let texts = [
            item.number().to_string(),
            item.title().to_owned(),
            item.state().label().to_owned(),
            item.assignee().to_owned(),
        ];
        for (cell, text) in cells.into_iter().zip(texts) {
            commands.push(Self::cell_text(cell, &text, Paint::Ink, self.px(MARGIN_PX)));
        }
        commands.push(DrawCommand::Stroke {
            rect: PixelRect::new(rect.x(), rect.bottom(), rect.width(), 1),
            paint: Paint::Border,
        });
    }

    fn cell_text(cell: PixelRect, text: &str, paint: Paint, padding: u16) -> DrawCommand {
        DrawCommand::Text {
            rect: PixelRect::new(
                cell.x().saturating_add(padding),
                cell.y(),
                cell.width().saturating_sub(padding.saturating_mul(2)),
                cell.height(),
            ),
            text: text.to_owned(),
            align: TextAlign::Start,
            paint,
            caret: None,
        }
    }

    /// スクロールバー。掴んで引けば位置が追い、ホイールは 3 行ずつ。
    fn probe_scroll(&mut self, track: PixelRect, commands: &mut Vec<DrawCommand>) {
        let response = self.interaction.probe(SCROLL, track, false);
        let pointer = self.interaction.pointer();
        let dragged = pointer
            .position()
            .filter(|_| response.phase() == ResponsePhase::Held);
        if let Some(point) = dragged {
            let into = u64::from(point.y().saturating_sub(track.y()));
            let offset = into * u64::from(self.scroll.content()) / u64::from(track.height().max(1));
            self.scroll = self
                .scroll
                .scrolled_to(u32::try_from(offset).unwrap_or(u32::MAX));
        }
        let wheel = i32::from(pointer.wheel())
            .saturating_mul(WHEEL_ROWS)
            .saturating_mul(i32::from(self.px(ROW_PX)));
        self.scroll = self.scroll.scrolled_by(wheel);
        commands.extend(scroll_bar::scroll_bar(track, self.scroll, response));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_docket_core::button_phase::ButtonPhase;
    use nene_docket_core::pixel_point::PixelPoint;
    use nene_docket_core::pointer_input::PointerInput;

    const BOUNDS: PixelRect = PixelRect::new(0, 0, 640, 400);

    fn screen(count: u32) -> DocketScreen {
        DocketScreen::open(Environment::new(None), SampleItem::generate(count))
    }

    fn pointer(x: u16, y: u16, phase: ButtonPhase) -> FrameInput {
        FrameInput::new(
            BOUNDS,
            PointerInput::new(Some(PixelPoint::new(x, y)), phase, 0),
        )
    }

    fn click(screen: &mut DocketScreen, x: u16, y: u16) {
        screen.step(&pointer(x, y, ButtonPhase::Pressed));
        screen.step(&pointer(x, y, ButtonPhase::Released));
    }

    fn keys(screen: &mut DocketScreen, keys: &[KeyIntent]) -> DocketFrame {
        screen.step(&FrameInput::new(BOUNDS, PointerInput::absent()).with_keys(keys.to_vec()))
    }

    #[test]
    fn a_frame_starts_with_the_background_and_lists_only_visible_rows() {
        let mut screen = screen(10_000);
        let frame = screen.step(&FrameInput::new(BOUNDS, PointerInput::absent()));
        assert_eq!(
            frame.commands().first(),
            Some(&DrawCommand::Fill {
                rect: BOUNDS,
                paint: Paint::Background
            })
        );
        let rows = frame
            .commands()
            .iter()
            .filter(
                |command| matches!(command, DrawCommand::Stroke { rect, .. } if rect.height() == 1),
            )
            .count();
        assert!(rows < 20, "{rows}");
        assert_eq!(screen.item_count(), 10_000);
        assert_eq!(frame.font_px(), 13);
    }

    #[test]
    fn clicking_a_row_selects_it_and_the_wheel_scrolls() {
        let mut screen = screen(50);
        click(&mut screen, 100, 80);
        assert_eq!(screen.selected(), Some(1));
        click(&mut screen, 100, 110);
        assert_eq!(screen.selected(), Some(2));
        screen.step(&FrameInput::new(
            BOUNDS,
            PointerInput::new(Some(PixelPoint::new(100, 200)), ButtonPhase::Idle, 1),
        ));
        assert_eq!(screen.scroll().offset(), 84);
        click(&mut screen, 100, 80);
        assert_eq!(screen.selected(), Some(4));
    }

    #[test]
    fn typing_goes_to_the_focused_search_field_and_filters_the_table() {
        let mut screen = screen(50);
        click(&mut screen, 100, 20);
        keys(&mut screen, &[KeyIntent::Char('#'), KeyIntent::Char('4')]);
        assert_eq!(screen.search_text(), "#4");
        let shown = screen.visible_items();
        assert!(shown.iter().all(|item| item.title().contains("#4")));
        assert_eq!(shown.len(), 11);
        keys(&mut screen, &[KeyIntent::Escape]);
        keys(&mut screen, &[KeyIntent::Char('x')]);
        assert_eq!(screen.search_text(), "#4");
    }

    #[test]
    fn the_filter_box_opens_and_a_click_on_an_option_chooses_it() {
        let mut screen = screen(30);
        click(&mut screen, 400, 20);
        assert!(screen.filter().is_open());
        click(&mut screen, 400, 130);
        assert!(!screen.filter().is_open());
        assert_eq!(screen.filter().chosen(), 3);
        assert!(
            screen
                .visible_items()
                .iter()
                .all(|item| item.state() == ItemState::Done)
        );
        click(&mut screen, 100, 80);
        assert_eq!(screen.selected(), Some(2));
    }

    #[test]
    fn show_done_and_add_work_by_click_and_by_key() {
        let mut screen = screen(6);
        click(&mut screen, 500, 20);
        assert!(!screen.shows_done());
        assert_eq!(screen.visible_items().len(), 4);
        keys(&mut screen, &[KeyIntent::Char(' ')]);
        assert!(screen.shows_done());
        click(&mut screen, 600, 20);
        assert_eq!(screen.item_count(), 7);
        assert_eq!(screen.selected(), Some(7));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(screen.item_count(), 8);
    }

    #[test]
    fn tab_moves_focus_through_the_toolbar() {
        let mut screen = screen(3);
        screen.step(&FrameInput::new(BOUNDS, PointerInput::absent()));
        keys(&mut screen, &[KeyIntent::Tab]);
        keys(&mut screen, &[KeyIntent::Char('a')]);
        assert_eq!(screen.search_text(), "a");
        keys(&mut screen, &[KeyIntent::Tab, KeyIntent::Down]);
        assert_eq!(screen.filter().chosen(), 1);
        keys(&mut screen, &[KeyIntent::BackTab, KeyIntent::Char('b')]);
        assert_eq!(screen.search_text(), "ab");
        keys(&mut screen, &[KeyIntent::Escape, KeyIntent::Down]);
        assert_eq!(screen.scroll().offset(), 0);
    }

    #[test]
    fn dragging_the_scroll_bar_moves_the_offset() {
        let mut screen = screen(200);
        screen.step(&pointer(630, 200, ButtonPhase::Pressed));
        screen.step(&pointer(630, 300, ButtonPhase::Held));
        assert!(screen.scroll().offset() > 0);
        assert!(screen.scroll().offset() <= screen.scroll().max_offset());
    }

    #[test]
    fn preedit_shows_in_the_focused_search_field() {
        let mut screen = screen(3);
        click(&mut screen, 100, 20);
        let frame = screen.step(
            &FrameInput::new(BOUNDS, PointerInput::absent())
                .with_preedit(Some(nene_docket_core::preedit::Preedit::new("か", 1))),
        );
        assert!(frame.commands().iter().any(
            |command| matches!(command, DrawCommand::Text { text, caret: Some(_), .. } if text == "か")
        ));
    }
}
