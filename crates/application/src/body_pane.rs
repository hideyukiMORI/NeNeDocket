//! 本文の段（FR-001）。表の下に、選んだ課題の本文を複数行の欄で出す。出すかどうかと、欄の編集状態。

use alloc::vec::Vec;

use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::text_area;
use nene_docket_core::text_edit::TextEdit;
use nene_docket_core::widget_id::WidgetId;

/// 本文の欄の ID。画面の他の部品（1〜15・17・100〜・200〜・300〜・1000〜）と重ならない。
const BODY: WidgetId = WidgetId::new(16);

/// 出すかどうかと、欄。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyPane {
    shown: bool,
    edit: TextEdit,
}

impl BodyPane {
    /// 隠れていて、欄は空。
    #[must_use]
    pub fn hidden() -> Self {
        Self {
            shown: false,
            edit: TextEdit::empty(),
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

    /// 段を問い合わせて描く。`line_px` は 1 行の高さ。
    pub fn probe(
        &self,
        interaction: &mut Interaction,
        (area, line_px): (PixelRect, u16),
        commands: &mut Vec<DrawCommand>,
    ) {
        let response = interaction.probe(BODY, area, true);
        commands.extend(text_area::text_area(area, &self.edit, response, line_px));
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::pointer_input::PointerInput;

    use super::*;

    #[test]
    fn toggles_and_draws_the_edit_in_the_area() {
        let mut pane = BodyPane::hidden();
        assert!(!pane.is_shown());
        pane.toggle();
        assert!(pane.is_shown());
        *pane.edit_mut() = TextEdit::showing("a\nb");
        assert_eq!(pane.edit().line_count(), 2);
        let mut interaction = Interaction::idle();
        interaction.begin(PointerInput::absent());
        let mut commands = Vec::new();
        pane.probe(
            &mut interaction,
            (PixelRect::new(0, 100, 300, 80), 20),
            &mut commands,
        );
        interaction.end();
        assert_eq!(
            commands
                .iter()
                .filter(|command| matches!(command, DrawCommand::Text { .. }))
                .count(),
            2
        );
        assert_eq!(BodyPane::id(), WidgetId::new(16));
    }
}
