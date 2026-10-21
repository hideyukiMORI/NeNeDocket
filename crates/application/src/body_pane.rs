//! 本文の段（FR-001）。表の下に、選んだ課題の本文を複数行の欄で出す。出すかどうかと、欄の編集状態と、スクロール。

use alloc::vec::Vec;

use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::response::Response;
use nene_docket_core::response_phase::ResponsePhase;
use nene_docket_core::scroll_bar;
use nene_docket_core::scroll_state::ScrollState;
use nene_docket_core::text_area;
use nene_docket_core::text_edit::TextEdit;
use nene_docket_core::widget_id::WidgetId;

/// 本文の欄の ID。画面の他の部品（1〜15・17〜19・21・100〜・200〜・300〜・1000〜・3000〜・4000〜）と重ならない。
const BODY: WidgetId = WidgetId::new(16);
/// 本文のスクロールバーの ID。
const BODY_SCROLL: WidgetId = WidgetId::new(20);
/// ホイール 1 刻みで送る行数。
const WHEEL_LINES: i32 = 3;

/// 出すかどうかと、欄と、スクロール。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyPane {
    shown: bool,
    edit: TextEdit,
    scroll: ScrollState,
}

impl BodyPane {
    /// 隠れていて、欄は空。
    #[must_use]
    pub fn hidden() -> Self {
        Self {
            shown: false,
            edit: TextEdit::empty(),
            scroll: ScrollState::top(0, 0),
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

    /// 欄の ID。フォーカスの判定に使う。
    #[must_use]
    pub const fn id() -> WidgetId {
        BODY
    }

    /// 欄。
    #[must_use]
    pub const fn edit(&self) -> &TextEdit {
        &self.edit
    }

    /// 欄（書く）。
    pub const fn edit_mut(&mut self) -> &mut TextEdit {
        &mut self.edit
    }

    /// スクロールの位置。
    #[must_use]
    pub const fn scroll(&self) -> ScrollState {
        self.scroll
    }

    /// 段を問い合わせて描く。`line_px` は 1 行の高さ、`scroll_width` はスクロールバーの幅（物理画素）。
    /// フォーカスがあればカーソルの行が見えるように送り、ホイールはポインタが段の上にあるときだけ受ける。
    /// `preview` があれば欄の代わりにその文字列を先頭から見せる（過去の時点の再現・FR-006）。
    pub fn probe(
        &mut self,
        interaction: &mut Interaction,
        (area, line_px, scroll_width, preview): (PixelRect, u16, u16, Option<&str>),
        commands: &mut Vec<DrawCommand>,
    ) {
        let field = PixelRect::new(
            area.x(),
            area.y(),
            area.width().saturating_sub(scroll_width),
            area.height(),
        );
        let track = PixelRect::new(
            area.right().saturating_sub(scroll_width).saturating_add(1),
            area.y(),
            scroll_width,
            area.height(),
        );
        let visible = text_area::visible_lines(field, line_px);
        let lines = u32::try_from(self.edit.line_count()).unwrap_or(u32::MAX);
        let viewport = u32::try_from(visible).unwrap_or(u32::MAX);
        self.scroll = self.scroll.resized(
            lines.saturating_mul(u32::from(line_px)),
            viewport.saturating_mul(u32::from(line_px)),
        );
        let response = interaction.probe(BODY, field, true);
        if response.focused() {
            self.follow_caret(visible, line_px);
        }
        let track_response = interaction.probe(BODY_SCROLL, track, false);
        self.take_pointer(interaction, (track, area, line_px), track_response.phase());
        let first = usize::try_from(self.scroll.offset() / u32::from(line_px.max(1))).unwrap_or(0);
        match preview {
            Some(text) => commands.extend(text_area::text_area(
                field,
                &TextEdit::showing(text),
                Response::new(ResponsePhase::Idle, false, false),
                (line_px, 0),
            )),
            None => commands.extend(text_area::text_area(
                field,
                &self.edit,
                response,
                (line_px, first),
            )),
        }
        commands.extend(scroll_bar::scroll_bar(track, self.scroll, track_response));
    }

    /// カーソルの行が見えるまで送る。
    fn follow_caret(&mut self, visible: usize, line_px: u16) {
        let (caret_line, _) = self.edit.position_of(self.edit.caret().cursor());
        let first = usize::try_from(self.scroll.offset() / u32::from(line_px.max(1))).unwrap_or(0);
        let wanted = if caret_line < first {
            caret_line
        } else if caret_line >= first.saturating_add(visible) {
            caret_line.saturating_add(1).saturating_sub(visible)
        } else {
            return;
        };
        let offset = u32::try_from(wanted)
            .unwrap_or(u32::MAX)
            .saturating_mul(u32::from(line_px));
        self.scroll = self.scroll.scrolled_to(offset);
    }

    /// つまみのドラッグとホイール。
    fn take_pointer(
        &mut self,
        interaction: &Interaction,
        (track, area, line_px): (PixelRect, PixelRect, u16),
        phase: ResponsePhase,
    ) {
        let pointer = interaction.pointer();
        if let Some(point) = pointer.position().filter(|_| phase == ResponsePhase::Held) {
            let into = u64::from(point.y().saturating_sub(track.y()));
            let offset = into * u64::from(self.scroll.content()) / u64::from(track.height().max(1));
            self.scroll = self
                .scroll
                .scrolled_to(u32::try_from(offset).unwrap_or(u32::MAX));
        }
        if pointer.position().is_some_and(|point| area.contains(point)) {
            let wheel = i32::from(pointer.wheel())
                .saturating_mul(WHEEL_LINES)
                .saturating_mul(i32::from(line_px));
            self.scroll = self.scroll.scrolled_by(wheel);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use nene_docket_core::button_phase::ButtonPhase;
    use nene_docket_core::key_intent::KeyIntent;
    use nene_docket_core::pixel_point::PixelPoint;
    use nene_docket_core::pointer_input::PointerInput;

    use super::*;

    const AREA: PixelRect = PixelRect::new(0, 100, 300, 72);

    fn texts(commands: &[DrawCommand]) -> Vec<String> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.clone()),
                DrawCommand::Fill { .. }
                | DrawCommand::Stroke { .. }
                | DrawCommand::Clip(_)
                | DrawCommand::Unclip => None,
            })
            .collect()
    }

    fn probe(pane: &mut BodyPane, pointer: PointerInput) -> Vec<DrawCommand> {
        let mut interaction = Interaction::idle();
        interaction.begin(pointer);
        let mut commands = Vec::new();
        pane.probe(&mut interaction, (AREA, 20, 12, None), &mut commands);
        interaction.end();
        commands
    }

    #[test]
    fn toggles_and_draws_the_edit_in_the_area() {
        let mut pane = BodyPane::hidden();
        assert!(!pane.is_shown());
        pane.toggle();
        assert!(pane.is_shown());
        *pane.edit_mut() = TextEdit::showing("a\nb");
        assert_eq!(pane.edit().line_count(), 2);
        let commands = probe(&mut pane, PointerInput::absent());
        assert_eq!(texts(&commands), ["a", "b"]);
        assert_eq!(BodyPane::id(), WidgetId::new(16));
        assert_eq!(pane.scroll().max_offset(), 0);
    }

    #[test]
    fn the_wheel_scrolls_a_long_body_and_the_focus_follows_the_caret() {
        let mut pane = BodyPane::hidden();
        // 72 - 12 = 60 の高さに 20 の行は 3 つ。6 行ある。
        *pane.edit_mut() = TextEdit::showing("1\n2\n3\n4\n5\n6");
        let top = probe(&mut pane, PointerInput::absent());
        assert_eq!(texts(&top), ["1", "2", "3"]);
        // ホイール 1 刻み = 3 行。
        let over = PointerInput::new(Some(PixelPoint::new(10, 120)), ButtonPhase::Idle, 1);
        let scrolled = probe(&mut pane, over);
        assert_eq!(texts(&scrolled), ["4", "5", "6"]);
        assert_eq!(pane.scroll().offset(), 60);
        // 段の外のホイールは効かない。
        let outside = PointerInput::new(Some(PixelPoint::new(10, 10)), ButtonPhase::Idle, -1);
        probe(&mut pane, outside);
        assert_eq!(pane.scroll().offset(), 60);
        // フォーカスを取ると（押下）、カーソルは末尾（6 行目）なので位置はそのまま。Up を 4 回で 2 行目 → 上へ送られる。
        let press = PointerInput::new(Some(PixelPoint::new(10, 120)), ButtonPhase::Pressed, 0);
        probe(&mut pane, press);
        for _ in 0_u8..4 {
            let edit = pane.edit_mut();
            *edit = edit.applied_in_area(KeyIntent::Up);
        }
        let mut interaction = Interaction::idle();
        interaction.begin(press);
        let mut commands = Vec::new();
        pane.probe(&mut interaction, (AREA, 20, 12, None), &mut commands);
        interaction.end();
        assert_eq!(texts(&commands), ["2", "3", "4"]);
        assert_eq!(pane.scroll().offset(), 20);
        // 再現の文字列は欄の代わりに先頭から。
        let mut again = Interaction::idle();
        again.begin(press);
        let mut shown = Vec::new();
        pane.probe(&mut again, (AREA, 20, 12, Some("x\ny")), &mut shown);
        again.end();
        assert_eq!(texts(&shown), ["x", "y"]);
    }
}
