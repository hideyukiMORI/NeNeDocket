//! 選んだ課題の履歴の段（FR-006）。表の下に、いつ・誰が・何を。自分のスクロールを持つ。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::language::Language;
use nene_docket_core::paint::Paint;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::response_phase::ResponsePhase;
use nene_docket_core::scroll_bar;
use nene_docket_core::scroll_state::ScrollState;
use nene_docket_core::table_layout::TableLayout;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::clock_display::ClockDisplay;
use crate::history_line::HistoryLine;

/// 履歴のスクロールバーの ID。画面の他の部品（1〜10・12〜19・100〜・200〜・300〜・1000〜・3000〜）と重ならない。
const HISTORY_SCROLL: WidgetId = WidgetId::new(11);
/// 履歴の行の ID。見えている順の番号を足す。押すとその値を選び直す（FR-008）。
const HISTORY_ROW: WidgetId = WidgetId::new(4_000);

/// 1x の寸法。
const ROW_PX: u16 = 24;
const WHEN_WIDTH_PX: u16 = 130;
const WHO_WIDTH_PX: u16 = 56;
const SCROLL_WIDTH_PX: u16 = 12;
const PADDING_PX: u16 = 8;
/// ホイール 1 刻みで送る行数。
const WHEEL_ROWS: i32 = 3;

/// 出すかどうかと、スクロールの位置と、倍率。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryPane {
    shown: bool,
    scroll: ScrollState,
    scale: UiScale,
}

impl HistoryPane {
    /// 隠れている。
    #[must_use]
    pub const fn hidden(scale: UiScale) -> Self {
        Self {
            shown: false,
            scroll: ScrollState::top(0, 0),
            scale,
        }
    }

    /// 出す・隠すを切り替える。
    pub const fn toggle(&mut self) {
        self.shown = !self.shown;
    }

    /// 出しているか。
    #[must_use]
    pub const fn is_shown(&self) -> bool {
        self.shown
    }

    /// スクロールの位置。
    #[must_use]
    pub const fn scroll(&self) -> ScrollState {
        self.scroll
    }

    /// 段の高さ（物理画素）。全体の 2/5 で、ヘッダ ＋ 3 行以上。
    #[must_use]
    pub fn height_in(&self, total: u16) -> u16 {
        let row = self.px(ROW_PX);
        (total / 5).saturating_mul(2).max(row.saturating_mul(4))
    }

    /// 段を問い合わせて描く。ホイールはポインタがこの段の上にあるときだけ受ける。文言は `language` で。
    /// 押した行があればその添字を返す（選び直し・FR-008）。
    pub fn probe(
        &mut self,
        interaction: &mut Interaction,
        (area, lines, language, display): (PixelRect, &[HistoryLine], Language, ClockDisplay),
        commands: &mut Vec<DrawCommand>,
    ) -> Option<usize> {
        let track = PixelRect::new(
            area.right()
                .saturating_sub(self.px(SCROLL_WIDTH_PX))
                .saturating_add(1),
            area.y(),
            self.px(SCROLL_WIDTH_PX),
            area.height(),
        );
        let table = self.table(area, lines.len());
        self.scroll = self
            .scroll
            .resized(table.content_height(), u32::from(table.body().height()));
        self.header(&table, language, commands);
        let clicked = self.rows(interaction, (&table, lines, language, display), commands);
        self.scroll_bar(interaction, (track, area), commands);
        clicked
    }

    /// 点の下の行の添字（ポインタで再現する行・FR-006）。段の外・行の無い所は `None`。
    #[must_use]
    pub fn line_at(&self, area: PixelRect, count: usize, point: PixelPoint) -> Option<usize> {
        self.table(area, count)
            .row_at(self.scroll.offset(), point)
            .and_then(|index| usize::try_from(index).ok())
            .filter(|index| *index < count)
    }

    /// 1x の寸法に倍率を掛ける。
    const fn px(&self, base: u16) -> u16 {
        self.scale.px(base)
    }

    fn table(&self, area: PixelRect, count: usize) -> TableLayout {
        let bounds = PixelRect::new(
            area.x(),
            area.y(),
            area.width().saturating_sub(self.px(SCROLL_WIDTH_PX)),
            area.height(),
        );
        let rows = u32::try_from(count).unwrap_or(u32::MAX);
        TableLayout::new(bounds, self.px(ROW_PX), self.px(ROW_PX), rows).with_columns(alloc::vec![
            ColumnWidth::Fixed(self.px(WHEN_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(WHO_WIDTH_PX)),
            ColumnWidth::Fill,
        ])
    }

    fn header(&self, table: &TableLayout, language: Language, commands: &mut Vec<DrawCommand>) {
        let padding = self.px(PADDING_PX);
        let headings =
            [Word::When, Word::Who, Word::Change].map(|word| wording::text(language, word));
        let header = table.header();
        commands.push(DrawCommand::Stroke {
            rect: PixelRect::new(header.x(), header.y(), header.width(), 1),
            paint: Paint::Border,
        });
        commands.push(DrawCommand::Fill {
            rect: header,
            paint: Paint::Surface,
        });
        for (cell, heading) in table.cells(header).into_iter().zip(headings) {
            commands.push(Self::cell_text(cell, heading, Paint::Muted, padding));
        }
    }

    /// 行を描き、押した行の添字を返す。ポインタの下の行は面の色で示す。
    fn rows(
        &self,
        interaction: &mut Interaction,
        (table, lines, language, display): (&TableLayout, &[HistoryLine], Language, ClockDisplay),
        commands: &mut Vec<DrawCommand>,
    ) -> Option<usize> {
        let padding = self.px(PADDING_PX);
        let mut clicked = None;
        commands.push(DrawCommand::Clip(table.body()));
        for (index, rect) in table.visible_rows(self.scroll.offset()) {
            let Some((position, line)) = usize::try_from(index)
                .ok()
                .and_then(|i| lines.get(i).map(|line| (i, line)))
            else {
                continue;
            };
            let response = interaction.probe(HISTORY_ROW.indexed(index), rect, false);
            if response.clicked() {
                clicked = Some(position);
            }
            if response.phase() != ResponsePhase::Idle {
                commands.push(DrawCommand::Fill {
                    rect,
                    paint: Paint::Surface,
                });
            }
            let paint = if line.is_undo() {
                Paint::Muted
            } else {
                Paint::Ink
            };
            for (cell, text) in table
                .cells(rect)
                .into_iter()
                .zip(line.columns(language, display))
            {
                commands.push(Self::cell_text(cell, &text, paint, padding));
            }
        }
        commands.push(DrawCommand::Unclip);
        clicked
    }

    fn scroll_bar(
        &mut self,
        interaction: &mut Interaction,
        (track, area): (PixelRect, PixelRect),
        commands: &mut Vec<DrawCommand>,
    ) {
        let response = interaction.probe(HISTORY_SCROLL, track, false);
        let pointer = interaction.pointer();
        if let Some(point) = pointer
            .position()
            .filter(|_| response.phase() == ResponsePhase::Held)
        {
            let into = u64::from(point.y().saturating_sub(track.y()));
            let offset = into * u64::from(self.scroll.content()) / u64::from(track.height().max(1));
            self.scroll = self
                .scroll
                .scrolled_to(u32::try_from(offset).unwrap_or(u32::MAX));
        }
        if pointer.position().is_some_and(|point| area.contains(point)) {
            let wheel = i32::from(pointer.wheel())
                .saturating_mul(WHEEL_ROWS)
                .saturating_mul(i32::from(self.px(ROW_PX)));
            self.scroll = self.scroll.scrolled_by(wheel);
        }
        commands.extend(scroll_bar::scroll_bar(track, self.scroll, response));
    }

    fn cell_text(cell: PixelRect, text: &str, paint: Paint, padding: u16) -> DrawCommand {
        DrawCommand::Text {
            rect: PixelRect::new(
                cell.x().saturating_add(padding),
                cell.y(),
                cell.width().saturating_sub(padding.saturating_mul(2)),
                cell.height(),
            ),
            text: String::from(text),
            align: TextAlign::Start,
            paint,
            caret: None,
        }
    }
}
