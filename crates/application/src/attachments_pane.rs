//! 添付の段（FR-021）。表の下に、選んだ課題の添付（名前・大きさ・SHA-256）を並べる。
//! 上の編集行はパスの欄と「添付する」、行を選んで「外す」。写しの中身が記録と合わない行は印を付ける（FR-014）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::attachment::Attachment;
use nene_docket_core::attachment_word::AttachmentWord;
use nene_docket_core::button;
use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::item::Item;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::language::Language;
use nene_docket_core::layout::Layout;
use nene_docket_core::paint::Paint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::preedit::Preedit;
use nene_docket_core::response_phase::ResponsePhase;
use nene_docket_core::scroll_bar;
use nene_docket_core::scroll_state::ScrollState;
use nene_docket_core::table_layout::TableLayout;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::text_edit::TextEdit;
use nene_docket_core::text_field;
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::attachments_action::AttachmentsAction;

/// パスの欄の ID。画面の他の部品（1〜25・28・100〜・200〜・300〜・1000〜・3000〜・4000〜・5000〜・6000〜・7000〜）と重ならない。
const ATTACH_PATH: WidgetId = WidgetId::new(26);
/// 「外す」の ID。
const DETACH: WidgetId = WidgetId::new(27);
/// スクロールバーの ID。
const ATTACHMENTS_SCROLL: WidgetId = WidgetId::new(29);
/// 「添付する」の ID。
const ATTACH: WidgetId = WidgetId::new(30);
/// 添付の行の ID。見えている順の番号を足す。
const ATTACHMENT_ROW: WidgetId = WidgetId::new(8_000);

/// 1x の寸法。
const ROW_PX: u16 = 24;
const LABEL_WIDTH_PX: u16 = 60;
const BUTTON_WIDTH_PX: u16 = 90;
const SIZE_WIDTH_PX: u16 = 100;
const DIGEST_WIDTH_PX: u16 = 110;
const PADDING_PX: u16 = 8;
const GAP_PX: u16 = 4;
const SCROLL_WIDTH_PX: u16 = 12;
/// ホイール 1 刻みで送る行数。
const WHEEL_ROWS: i32 = 3;
/// SHA-256 の列に出す桁数。
const DIGEST_DIGITS: usize = 8;
/// 写しが記録と合わない行の印。
const ALTERED_MARK: &str = "⚠ ";

/// 出すかどうか・パスの欄・選んだ添付の連番・スクロール・倍率。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentsPane {
    shown: bool,
    path: TextEdit,
    selected: Option<u32>,
    scroll: ScrollState,
    scale: UiScale,
}

impl AttachmentsPane {
    /// 隠れている。
    #[must_use]
    pub fn hidden(scale: UiScale) -> Self {
        Self {
            shown: false,
            path: TextEdit::empty(),
            selected: None,
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

    /// パスの欄の ID（フォーカスの判定）。
    #[must_use]
    pub const fn path_id() -> WidgetId {
        ATTACH_PATH
    }

    /// 選んだ添付の連番。
    #[must_use]
    pub const fn selected(&self) -> Option<u32> {
        self.selected
    }

    /// パスの欄の文字列。
    #[must_use]
    pub fn path(&self) -> &str {
        self.path.text()
    }

    /// パスの欄の鍵。Enter で添付する（欄は空になる）。
    pub fn path_key(&mut self, key: KeyIntent) -> Option<AttachmentsAction> {
        if key == KeyIntent::Enter {
            return self.take_path();
        }
        self.path = self.path.applied(key);
        None
    }

    /// 確定前の文字列をパスの欄へ。
    pub fn set_preedit(&mut self, preedit: Option<Preedit>) {
        self.path = self.path.with_preedit(preedit);
    }

    /// 欄のパス（前後の空白を落とす）を添付の求めにし、欄を空にする。空なら何もしない。
    fn take_path(&mut self) -> Option<AttachmentsAction> {
        let path = String::from(self.path.text().trim());
        if path.is_empty() {
            return None;
        }
        self.path = TextEdit::empty();
        Some(AttachmentsAction::Attach(path))
    }

    /// 段を問い合わせ、描画命令と求めを返す。`altered` は写しが記録と合わない連番。
    pub fn probe(
        &mut self,
        interaction: &mut Interaction,
        (area, item, altered, language): (PixelRect, Option<&Item>, &[u32], Language),
        commands: &mut Vec<DrawCommand>,
    ) -> Vec<AttachmentsAction> {
        let attachments = item.map_or(&[][..], Item::attachments);
        if !attachments
            .iter()
            .any(|attachment| Some(attachment.sequence()) == self.selected)
        {
            self.selected = None;
        }
        let mut actions = Vec::new();
        let mut layout = Layout::inside(area, self.px(GAP_PX));
        let row = layout.row(
            self.px(ROW_PX),
            &[
                ColumnWidth::Fixed(self.px(LABEL_WIDTH_PX)),
                ColumnWidth::Fill,
                ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
                ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
            ],
        );
        actions.extend(self.edit_row(interaction, (&row, language), commands));
        let table_area = layout.remaining();
        let track = PixelRect::new(
            table_area
                .right()
                .saturating_sub(self.px(SCROLL_WIDTH_PX))
                .saturating_add(1),
            table_area.y(),
            self.px(SCROLL_WIDTH_PX),
            table_area.height(),
        );
        let table = self.table(table_area, attachments.len());
        self.scroll = self
            .scroll
            .resized(table.content_height(), u32::from(table.body().height()));
        self.header(&table, language, commands);
        self.rows(
            interaction,
            (&table, attachments, altered, language),
            commands,
        );
        self.scroll_bar(interaction, (track, table_area), commands);
        actions
    }

    /// 編集行: 「パス」の名・欄・「添付する」・「外す」。
    fn edit_row(
        &mut self,
        interaction: &mut Interaction,
        (row, language): (&[PixelRect], Language),
        commands: &mut Vec<DrawCommand>,
    ) -> Vec<AttachmentsAction> {
        let mut actions = Vec::new();
        let (Some(label), Some(field), Some(attach_rect), Some(detach_rect)) =
            (row.first(), row.get(1), row.get(2), row.get(3))
        else {
            return actions;
        };
        commands.push(Self::cell_text(
            *label,
            wording::text(language, Word::Attachment(AttachmentWord::Path)),
            Paint::Muted,
            self.px(PADDING_PX),
        ));
        let path = interaction.probe(ATTACH_PATH, *field, true);
        commands.extend(text_field::text_field(*field, &self.path, path));
        let attach = interaction.probe(ATTACH, *attach_rect, true);
        if attach.clicked() {
            actions.extend(self.take_path());
        }
        commands.extend(button::button(
            *attach_rect,
            wording::text(language, Word::Attachment(AttachmentWord::Attach)),
            attach,
        ));
        let detach = interaction.probe(DETACH, *detach_rect, true);
        if detach.clicked() {
            actions.extend(self.selected.map(AttachmentsAction::Detach));
        }
        commands.extend(button::button(
            *detach_rect,
            wording::text(language, Word::Attachment(AttachmentWord::Detach)),
            detach,
        ));
        actions
    }

    fn header(&self, table: &TableLayout, language: Language, commands: &mut Vec<DrawCommand>) {
        let header = table.header();
        commands.push(DrawCommand::Stroke {
            rect: PixelRect::new(header.x(), header.y(), header.width(), 1),
            paint: Paint::Border,
        });
        commands.push(DrawCommand::Fill {
            rect: header,
            paint: Paint::Surface,
        });
        let headings = [
            String::from(wording::text(language, Word::Name)),
            String::from(wording::text(
                language,
                Word::Attachment(AttachmentWord::Size),
            )),
            String::from("SHA-256"),
        ];
        for (cell, heading) in table.cells(header).into_iter().zip(headings) {
            commands.push(Self::cell_text(
                cell,
                &heading,
                Paint::Muted,
                self.px(PADDING_PX),
            ));
        }
    }

    /// 見えている行。押すと選ぶ。写しが合わない行は印を付ける。
    fn rows(
        &mut self,
        interaction: &mut Interaction,
        (table, attachments, altered, language): (&TableLayout, &[Attachment], &[u32], Language),
        commands: &mut Vec<DrawCommand>,
    ) {
        commands.push(DrawCommand::Clip(table.body()));
        for (index, rect) in table.visible_rows(self.scroll.offset()) {
            let Some(attachment) = usize::try_from(index).ok().and_then(|i| attachments.get(i))
            else {
                continue;
            };
            let response = interaction.probe(ATTACHMENT_ROW.indexed(index), rect, false);
            if response.clicked() {
                self.selected = Some(attachment.sequence());
            }
            if self.selected == Some(attachment.sequence()) {
                commands.push(DrawCommand::Fill {
                    rect,
                    paint: Paint::Selection,
                });
            } else if response.phase() != ResponsePhase::Idle {
                commands.push(DrawCommand::Fill {
                    rect,
                    paint: Paint::Surface,
                });
            }
            // 写しが合わない行は名前に印と理由を添える（印の列は広くないので名前の列に出す）。
            let name = if altered.contains(&attachment.sequence()) {
                alloc::format!(
                    "{ALTERED_MARK}{}: {}",
                    attachment.name(),
                    wording::text(language, Word::Attachment(AttachmentWord::Altered))
                )
            } else {
                String::from(attachment.name())
            };
            let digest: String = attachment
                .digest_hex()
                .chars()
                .take(DIGEST_DIGITS)
                .collect();
            let texts = [name, alloc::format!("{}", attachment.bytes()), digest];
            for (cell, text) in table.cells(rect).into_iter().zip(texts) {
                commands.push(Self::cell_text(
                    cell,
                    &text,
                    Paint::Ink,
                    self.px(PADDING_PX),
                ));
            }
        }
        commands.push(DrawCommand::Unclip);
    }

    /// つまみのドラッグとホイール（ポインタが段の上にあるときだけ）。
    fn scroll_bar(
        &mut self,
        interaction: &mut Interaction,
        (track, area): (PixelRect, PixelRect),
        commands: &mut Vec<DrawCommand>,
    ) {
        let response = interaction.probe(ATTACHMENTS_SCROLL, track, false);
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
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(SIZE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(DIGEST_WIDTH_PX)),
        ])
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
