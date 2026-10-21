//! 添付の段（FR-021 / FR-022）。表の下に、選んだ課題の添付（名前・大きさ・SHA-256）を並べる。
//! 上の編集行はパスの欄と「添付する」、行を選んで「外す」。写しの中身が記録と合わない行は印を付ける（FR-014）。
//! 行を選ぶと下に中身を出す: 文は読み取り専用の欄、CSV / TSV は表、画像は伸縮して（ホイールで拡大・ドラッグで移動）、それ以外はパスと「開く」（OS の既定のアプリ）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::attachment::Attachment;
use nene_docket_core::attachment_word::AttachmentWord;
use nene_docket_core::button;
use nene_docket_core::cell_text;
use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::image_fit;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::item::Item;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::language::Language;
use nene_docket_core::layout::Layout;
use nene_docket_core::paint::Paint;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::preedit::Preedit;
use nene_docket_core::response::Response;
use nene_docket_core::response_phase::ResponsePhase;
use nene_docket_core::scroll_bar;
use nene_docket_core::scroll_state::ScrollState;
use nene_docket_core::shared_bitmap::SharedBitmap;
use nene_docket_core::table_layout::TableLayout;
use nene_docket_core::text_area;
use nene_docket_core::text_edit::TextEdit;
use nene_docket_core::text_field;
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::attachment_view::AttachmentView;
use crate::attachments_action::AttachmentsAction;
use crate::attachments_context::AttachmentsContext;

/// パスの欄。
const ATTACH_PATH: WidgetId = WidgetId::new(26);
/// 「外す」。
const DETACH: WidgetId = WidgetId::new(27);
/// 一覧のスクロールバー。
const ATTACHMENTS_SCROLL: WidgetId = WidgetId::new(29);
/// 「添付する」。
const ATTACH: WidgetId = WidgetId::new(30);
/// 中身のスクロールバー。
const VIEW_SCROLL: WidgetId = WidgetId::new(31);
/// 「開く」。
const OPEN: WidgetId = WidgetId::new(32);
/// 画像（ホイールで拡大・ドラッグで移動）。
const IMAGE_VIEW: WidgetId = WidgetId::new(33);
/// 添付の行（添字を足す）。
const ATTACHMENT_ROW: WidgetId = WidgetId::new(8_000);

const ROW_PX: u16 = 24;
const LABEL_WIDTH_PX: u16 = 60;
const BUTTON_WIDTH_PX: u16 = 90;
const SIZE_WIDTH_PX: u16 = 100;
const DIGEST_WIDTH_PX: u16 = 110;
const PADDING_PX: u16 = 8;
const GAP_PX: u16 = 4;
const SCROLL_WIDTH_PX: u16 = 12;
/// 中身の文の 1 行。
const VIEW_LINE_PX: u16 = 20;
/// 表の列の幅を字数から決めるときの 1 字（core はフォントを知らない）。
const CHAR_PX: u16 = 8;
const MIN_COLUMN_PX: u16 = 60;
const MAX_COLUMN_PX: u16 = 320;
/// 中身を出すとき、一覧に残す行の数（見出しを除く）。
const LIST_ROWS_WITH_VIEW: usize = 3;
/// ホイール 1 刻みで動く行。
const WHEEL_ROWS: i32 = 3;
/// ホイール 1 刻みで変わる倍率（%）。
const ZOOM_STEP: u32 = 10;
/// SHA-256 の先頭だけ出す桁数。
const DIGEST_DIGITS: usize = 8;
/// 写しが合わない行の印。
const ALTERED_MARK: &str = "⚠ ";

/// 出すかどうか・パスの欄・選んだ添付の連番・一覧と中身のスクロール・画像の倍率とずれとドラッグ・倍率。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentsPane {
    shown: bool,
    path: TextEdit,
    selected: Option<u32>,
    scroll: ScrollState,
    view_scroll: ScrollState,
    zoom: u32,
    pan: (i32, i32),
    drag: Option<(PixelPoint, (i32, i32))>,
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
            view_scroll: ScrollState::top(0, 0),
            zoom: 100,
            pan: (0_i32, 0_i32),
            drag: None,
            scale,
        }
    }

    /// 画像の倍率（%・収まる大きさが 100）。
    #[must_use]
    pub const fn zoom(&self) -> u32 {
        self.zoom
    }

    /// 画像の中央からのずれ（画素）。
    #[must_use]
    pub const fn pan(&self) -> (i32, i32) {
        self.pan
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

    /// 「開く」の ID（鍵で押す）。
    #[must_use]
    pub const fn open_id() -> WidgetId {
        OPEN
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

    /// 「開く」を鍵で押した。選んでいれば開く求め。
    #[must_use]
    pub fn open_key(&self) -> Option<AttachmentsAction> {
        self.selected.map(AttachmentsAction::Open)
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

    /// 段を問い合わせ、描画命令と求めを返す。
    pub fn probe(
        &mut self,
        interaction: &mut Interaction,
        context: &AttachmentsContext<'_>,
        commands: &mut Vec<DrawCommand>,
    ) -> Vec<AttachmentsAction> {
        let attachments = context.item().map_or(&[][..], Item::attachments);
        if !attachments
            .iter()
            .any(|attachment| Some(attachment.sequence()) == self.selected)
        {
            self.selected = None;
        }
        let mut actions = Vec::new();
        let mut layout = Layout::inside(context.area(), self.px(GAP_PX));
        let row = layout.row(
            self.px(ROW_PX),
            &[
                ColumnWidth::Fixed(self.px(LABEL_WIDTH_PX)),
                ColumnWidth::Fill,
                ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
                ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
            ],
        );
        actions.extend(self.edit_row(interaction, (&row, context.language()), commands));
        let view = self.selected.and(context.view());
        let (list_area, view_area) =
            self.split(layout.remaining(), attachments.len(), view.is_some());
        self.list(interaction, (list_area, attachments, context), commands);
        if let (Some(view_area), Some(view)) = (view_area, view) {
            actions.extend(self.viewer(
                interaction,
                (view_area, view, context.language()),
                commands,
            ));
        }
        actions
    }

    /// 一覧と中身に分ける。中身を出すなら一覧は見出し ＋ 3 行まで、残りが中身（1 行分も無ければ出さない）。
    fn split(
        &self,
        area: PixelRect,
        count: usize,
        with_view: bool,
    ) -> (PixelRect, Option<PixelRect>) {
        if !with_view {
            return (area, None);
        }
        let rows = u16::try_from(count.min(LIST_ROWS_WITH_VIEW).saturating_add(1)).unwrap_or(1);
        let list_height = self.px(ROW_PX).saturating_mul(rows);
        let view_top = list_height.saturating_add(self.px(GAP_PX));
        let view_height = area.height().saturating_sub(view_top);
        if view_height < self.px(ROW_PX) {
            return (area, None);
        }
        (
            PixelRect::new(area.x(), area.y(), area.width(), list_height),
            Some(PixelRect::new(
                area.x(),
                area.y().saturating_add(view_top),
                area.width(),
                view_height,
            )),
        )
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
        commands.extend(cell_text::cell_text(
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

    /// 一覧: 見出し・行・スクロールバー。
    fn list(
        &mut self,
        interaction: &mut Interaction,
        (area, attachments, context): (PixelRect, &[Attachment], &AttachmentsContext<'_>),
        commands: &mut Vec<DrawCommand>,
    ) {
        let track = self.track_of(area);
        let table = self.table(area, attachments.len());
        self.scroll = self
            .scroll
            .resized(table.content_height(), u32::from(table.body().height()));
        self.header(&table, context.language(), commands);
        self.rows(interaction, (&table, attachments, context), commands);
        self.scroll = Self::scrolled(
            interaction,
            (ATTACHMENTS_SCROLL, track, area),
            (self.scroll, self.px(ROW_PX)),
            commands,
        );
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
            commands.extend(cell_text::cell_text(
                cell,
                &heading,
                Paint::Muted,
                self.px(PADDING_PX),
            ));
        }
    }

    /// 見えている行。押すと選ぶ（中身のスクロールは上へ戻る）。写しが合わない行は印と理由を名前に添える。
    /// 先に全部の行を問い合わせて選択を決め、それから描く（同じフレームで前の選択の色が残らない）。
    fn rows(
        &mut self,
        interaction: &mut Interaction,
        (table, attachments, context): (&TableLayout, &[Attachment], &AttachmentsContext<'_>),
        commands: &mut Vec<DrawCommand>,
    ) {
        let mut visible = Vec::new();
        for (index, rect) in table.visible_rows(self.scroll.offset()) {
            let Some(attachment) = usize::try_from(index).ok().and_then(|i| attachments.get(i))
            else {
                continue;
            };
            let response = interaction.probe(ATTACHMENT_ROW.indexed(index), rect, false);
            if response.clicked() {
                self.select(attachment.sequence());
            }
            visible.push((rect, attachment, response.phase()));
        }
        commands.push(DrawCommand::Clip(table.body()));
        for (rect, attachment, phase) in visible {
            if self.selected == Some(attachment.sequence()) {
                commands.push(DrawCommand::Fill {
                    rect,
                    paint: Paint::Selection,
                });
            } else if phase != ResponsePhase::Idle {
                commands.push(DrawCommand::Fill {
                    rect,
                    paint: Paint::Surface,
                });
            }
            let texts = [
                Self::name_of(attachment, context),
                alloc::format!("{}", attachment.bytes()),
                attachment
                    .digest_hex()
                    .chars()
                    .take(DIGEST_DIGITS)
                    .collect(),
            ];
            for (cell, text) in table.cells(rect).into_iter().zip(texts) {
                commands.extend(cell_text::cell_text(
                    cell,
                    &text,
                    Paint::Ink,
                    self.px(PADDING_PX),
                ));
            }
        }
        commands.push(DrawCommand::Unclip);
    }

    /// 行を選ぶ。別の行なら中身のスクロール・倍率・ずれは戻る。
    fn select(&mut self, sequence: u32) {
        if self.selected == Some(sequence) {
            return;
        }
        self.selected = Some(sequence);
        self.view_scroll = ScrollState::top(0, 0);
        self.zoom = 100;
        self.pan = (0_i32, 0_i32);
        self.drag = None;
    }

    /// 名前の列。写しが合わなければ印と理由（印の列は広くないので名前の列に出す）。
    fn name_of(attachment: &Attachment, context: &AttachmentsContext<'_>) -> String {
        if context.altered().contains(&attachment.sequence()) {
            alloc::format!(
                "{ALTERED_MARK}{}: {}",
                attachment.name(),
                wording::text(
                    context.language(),
                    Word::Attachment(AttachmentWord::Altered)
                )
            )
        } else {
            String::from(attachment.name())
        }
    }

    /// 中身。先頭だけならその旨を 1 行。文・表・パスと「開く」。
    fn viewer(
        &mut self,
        interaction: &mut Interaction,
        (area, view, language): (PixelRect, &AttachmentView, Language),
        commands: &mut Vec<DrawCommand>,
    ) -> Vec<AttachmentsAction> {
        let mut area = area;
        if view.is_truncated() {
            let note = PixelRect::new(area.x(), area.y(), area.width(), self.px(ROW_PX));
            commands.extend(cell_text::cell_text(
                note,
                wording::text(language, Word::Attachment(AttachmentWord::Truncated)),
                Paint::Muted,
                self.px(PADDING_PX),
            ));
            area = PixelRect::new(
                area.x(),
                area.y().saturating_add(self.px(ROW_PX)),
                area.width(),
                area.height().saturating_sub(self.px(ROW_PX)),
            );
        }
        match view {
            AttachmentView::Text { edit, .. } => {
                self.view_text(interaction, (area, edit), commands);
            }
            AttachmentView::Table { rows, widths, .. } => {
                self.view_table(interaction, (area, rows, widths), commands);
            }
            AttachmentView::Image(bitmap) => self.view_image(interaction, (area, bitmap), commands),
            AttachmentView::Path(path) => {
                return self.view_path(interaction, (area, path, language), commands);
            }
        }
        Vec::new()
    }

    /// 文: 読み取り専用の欄。行の単位でスクロール。
    fn view_text(
        &mut self,
        interaction: &mut Interaction,
        (area, edit): (PixelRect, &TextEdit),
        commands: &mut Vec<DrawCommand>,
    ) {
        let (line_px, track) = (self.px(VIEW_LINE_PX), self.track_of(area));
        let field = PixelRect::new(
            area.x(),
            area.y(),
            area.width().saturating_sub(self.px(SCROLL_WIDTH_PX)),
            area.height(),
        );
        let visible = u32::try_from(text_area::visible_lines(field, line_px)).unwrap_or(u32::MAX);
        let lines = u32::try_from(edit.line_count()).unwrap_or(u32::MAX);
        self.view_scroll = self.view_scroll.resized(
            lines.saturating_mul(u32::from(line_px)),
            visible.saturating_mul(u32::from(line_px)),
        );
        let first = usize::try_from(self.view_scroll.offset() / u32::from(line_px.max(1)))
            .unwrap_or(usize::MAX);
        commands.extend(text_area::text_area(
            field,
            edit,
            Response::new(ResponsePhase::Idle, false, false),
            (line_px, first),
        ));
        self.view_scroll = Self::scrolled(
            interaction,
            (VIEW_SCROLL, track, area),
            (self.view_scroll, line_px),
            commands,
        );
    }

    /// 表: 最初の行が見出し、列の幅は字数から。
    fn view_table(
        &mut self,
        interaction: &mut Interaction,
        (area, rows, widths): (PixelRect, &[Vec<String>], &[usize]),
        commands: &mut Vec<DrawCommand>,
    ) {
        let track = self.track_of(area);
        let bounds = PixelRect::new(
            area.x(),
            area.y(),
            area.width().saturating_sub(self.px(SCROLL_WIDTH_PX)),
            area.height(),
        );
        let body_rows = u32::try_from(rows.len().saturating_sub(1)).unwrap_or(u32::MAX);
        let columns = widths
            .iter()
            .map(|chars| ColumnWidth::Fixed(self.column_px(*chars)))
            .collect();
        let table = TableLayout::new(bounds, self.px(ROW_PX), self.px(ROW_PX), body_rows)
            .with_columns(columns);
        self.view_scroll = self
            .view_scroll
            .resized(table.content_height(), u32::from(table.body().height()));
        commands.push(DrawCommand::Fill {
            rect: bounds,
            paint: Paint::Surface,
        });
        commands.push(DrawCommand::Stroke {
            rect: bounds,
            paint: Paint::Border,
        });
        commands.push(DrawCommand::Clip(bounds.inset(1)));
        if let Some(heading) = rows.first() {
            self.cells((&table, table.header()), (heading, Paint::Muted), commands);
        }
        for (index, rect) in table.visible_rows(self.view_scroll.offset()) {
            if let Some(row) = usize::try_from(index)
                .ok()
                .and_then(|i| rows.get(i.saturating_add(1)))
            {
                self.cells((&table, rect), (row, Paint::Ink), commands);
            }
        }
        commands.push(DrawCommand::Unclip);
        self.view_scroll = Self::scrolled(
            interaction,
            (VIEW_SCROLL, track, area),
            (self.view_scroll, self.px(ROW_PX)),
            commands,
        );
    }

    fn cells(
        &self,
        (table, rect): (&TableLayout, PixelRect),
        (row, paint): (&[String], Paint),
        commands: &mut Vec<DrawCommand>,
    ) {
        for (cell, text) in table.cells(rect).into_iter().zip(row) {
            commands.extend(cell_text::cell_text(cell, text, paint, self.px(PADDING_PX)));
        }
    }

    /// 字数から列の幅（60〜320 px）。
    fn column_px(&self, chars: usize) -> u16 {
        let text = u16::try_from(chars)
            .unwrap_or(u16::MAX)
            .saturating_mul(CHAR_PX)
            .saturating_add(PADDING_PX.saturating_mul(2));
        self.px(text.clamp(MIN_COLUMN_PX, MAX_COLUMN_PX))
    }

    /// 画像: 収まるように縮めて中央に。ホイールで倍率、ドラッグでずれ。
    fn view_image(
        &mut self,
        interaction: &mut Interaction,
        (area, bitmap): (PixelRect, &SharedBitmap),
        commands: &mut Vec<DrawCommand>,
    ) {
        let response = interaction.probe(IMAGE_VIEW, area, false);
        let pointer = interaction.pointer();
        let position = pointer.position();
        if position.is_some_and(|point| area.contains(point)) {
            self.zoom = Self::zoomed(self.zoom, pointer.wheel());
        }
        // 離した（clicked）フレームも Held なので、そこでドラッグを終える。
        self.drag_image(
            response.phase() == ResponsePhase::Held && !response.clicked(),
            position,
        );
        commands.push(DrawCommand::Fill {
            rect: area,
            paint: Paint::Surface,
        });
        commands.push(DrawCommand::Stroke {
            rect: area,
            paint: Paint::Border,
        });
        let pixels = bitmap.bitmap();
        if let Some((dest, source)) = image_fit::place(
            area.inset(1),
            (pixels.width(), pixels.height()),
            self.zoom,
            self.pan,
        ) {
            commands.push(DrawCommand::Image {
                dest,
                source,
                bitmap: bitmap.clone(),
            });
        }
    }

    /// ホイール 1 刻みで 10%。上（負）で拡大、下で縮小。
    fn zoomed(zoom: u32, notches: i8) -> u32 {
        let step = u32::from(notches.unsigned_abs()).saturating_mul(ZOOM_STEP);
        let next = if notches < 0 {
            zoom.saturating_add(step)
        } else {
            zoom.saturating_sub(step)
        };
        next.clamp(image_fit::MIN_ZOOM, image_fit::MAX_ZOOM)
    }

    /// 押したまま動かした分だけずらす。離したら終わり。
    fn drag_image(&mut self, held: bool, position: Option<PixelPoint>) {
        let Some(point) = position.filter(|_| held) else {
            self.drag = None;
            return;
        };
        match self.drag {
            None => self.drag = Some((point, self.pan)),
            Some((start, (pan_x, pan_y))) => {
                self.pan = (
                    pan_x.saturating_add(i32::from(point.x()) - i32::from(start.x())),
                    pan_y.saturating_add(i32::from(point.y()) - i32::from(start.y())),
                );
            }
        }
    }

    /// それ以外: パスと「開く」。
    fn view_path(
        &self,
        interaction: &mut Interaction,
        (area, path, language): (PixelRect, &str, Language),
        commands: &mut Vec<DrawCommand>,
    ) -> Vec<AttachmentsAction> {
        let mut layout = Layout::inside(area, 0);
        let row = layout.row(
            self.px(ROW_PX),
            &[
                ColumnWidth::Fill,
                ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
            ],
        );
        let (Some(text_rect), Some(button_rect)) = (row.first(), row.get(1)) else {
            return Vec::new();
        };
        commands.extend(cell_text::cell_text(
            *text_rect,
            path,
            Paint::Ink,
            self.px(PADDING_PX),
        ));
        let open = interaction.probe(OPEN, *button_rect, true);
        commands.extend(button::button(
            *button_rect,
            wording::text(language, Word::Attachment(AttachmentWord::Open)),
            open,
        ));
        if open.clicked() {
            return self.open_key().into_iter().collect();
        }
        Vec::new()
    }

    /// つまみのドラッグとホイール（ポインタが `area` の上にあるときだけ）。動かした後の状態を返す。
    fn scrolled(
        interaction: &mut Interaction,
        (id, track, area): (WidgetId, PixelRect, PixelRect),
        (state, step_px): (ScrollState, u16),
        commands: &mut Vec<DrawCommand>,
    ) -> ScrollState {
        let mut state = state;
        let response = interaction.probe(id, track, false);
        let pointer = interaction.pointer();
        if let Some(point) = pointer
            .position()
            .filter(|_| response.phase() == ResponsePhase::Held)
        {
            let into = u64::from(point.y().saturating_sub(track.y()));
            let offset = into * u64::from(state.content()) / u64::from(track.height().max(1));
            state = state.scrolled_to(u32::try_from(offset).unwrap_or(u32::MAX));
        }
        if pointer.position().is_some_and(|point| area.contains(point)) {
            let wheel = i32::from(pointer.wheel())
                .saturating_mul(WHEEL_ROWS)
                .saturating_mul(i32::from(step_px));
            state = state.scrolled_by(wheel);
        }
        commands.extend(scroll_bar::scroll_bar(track, state, response));
        state
    }

    const fn px(&self, base: u16) -> u16 {
        self.scale.px(base)
    }

    /// 右端のスクロールバーの溝。
    fn track_of(&self, area: PixelRect) -> PixelRect {
        PixelRect::new(
            area.right()
                .saturating_sub(self.px(SCROLL_WIDTH_PX))
                .saturating_add(1),
            area.y(),
            self.px(SCROLL_WIDTH_PX),
            area.height(),
        )
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
}
