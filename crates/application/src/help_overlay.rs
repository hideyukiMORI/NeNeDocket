//! F1 の案内（Issue #125・docs/design/2026-11-09-toolbar-and-settings.md 第 5 節）。運用の筋書きの 6 段と鍵の一覧を
//! 窓の真ん中に重ねて出す。案内の外は下の部品に届かない。段ごとの「飛ぶ先」を押すと、その段の番号を返して閉じる。

use alloc::format;
use alloc::vec::Vec;

use nene_docket_core::button;
use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::help_word::HelpWord;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::language::Language;
use nene_docket_core::layout::Layout;
use nene_docket_core::paint::Paint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::toolbar_word::ToolbarWord;
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

/// 案内の下を塞ぐ面。
const BLOCK: WidgetId = WidgetId::new(39);
/// 「閉じる」。
const CLOSE: WidgetId = WidgetId::new(40);
/// 段ごとの飛ぶ先（段の番号を足す）。
const GO: WidgetId = WidgetId::new(700);
/// 1x の寸法。
const WIDTH_PX: u16 = 880;
const MARGIN_PX: u16 = 16;
const ROW_PX: u16 = 28;
const GAP_PX: u16 = 8;
const NUMBER_PX: u16 = 24;
const NAME_PX: u16 = 110;
const GO_PX: u16 = 200;
const CLOSE_PX: u16 = 70;

/// 開いているか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HelpOverlay {
    open: bool,
    scale: UiScale,
}

impl HelpOverlay {
    /// 閉じている。
    #[must_use]
    pub const fn closed(scale: UiScale) -> Self {
        Self { open: false, scale }
    }

    /// 開いているか。
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// F1: 開け閉め。
    pub const fn toggle(&mut self) {
        self.open = !self.open;
    }

    /// 閉じる（Esc・「閉じる」）。
    pub const fn close(&mut self) {
        self.open = false;
    }

    /// 「閉じる」の ID（鍵で押す）。
    #[must_use]
    pub const fn close_id() -> WidgetId {
        CLOSE
    }

    /// 段の飛ぶ先の ID なら、その段の番号。
    #[must_use]
    pub fn step_of(id: WidgetId) -> Option<usize> {
        id.number()
            .checked_sub(GO.number())
            .and_then(|step| usize::try_from(step).ok())
            .filter(|step| *step < HelpWord::STEPS.len())
    }

    const fn px(self, base: u16) -> u16 {
        self.scale.px(base)
    }

    /// 開いていれば問い合わせて描く。飛ぶ先を押せばその段の番号を返して閉じる。
    pub fn probe(
        &mut self,
        interaction: &mut Interaction,
        (bounds, language): (PixelRect, Language),
        commands: &mut Vec<DrawCommand>,
    ) -> Option<usize> {
        if !self.open {
            return None;
        }
        let dialog = self.dialog(bounds);
        let _blocked = interaction.probe(BLOCK, bounds, false);
        commands.push(DrawCommand::Fill {
            rect: dialog,
            paint: Paint::Surface,
        });
        commands.push(DrawCommand::Stroke {
            rect: dialog,
            paint: Paint::Border,
        });
        let word = |word: Word| wording::text(language, word);
        let mut layout = Layout::inside(dialog.inset(self.px(MARGIN_PX)), self.px(GAP_PX));
        let head = layout.row(
            self.px(ROW_PX),
            &[ColumnWidth::Fill, ColumnWidth::Fixed(self.px(CLOSE_PX))],
        );
        if let (Some(title), Some(close)) = (head.first(), head.get(1)) {
            commands.push(text(*title, word(Word::Help(HelpWord::Title)), Paint::Ink));
            let closed = interaction.probe(CLOSE, *close, true);
            if closed.clicked() {
                self.open = false;
            }
            commands.extend(button::button(
                *close,
                word(Word::Toolbar(ToolbarWord::Close)),
                closed,
            ));
        }
        let mut chosen = None;
        for (step, (name, what, go)) in HelpWord::STEPS.into_iter().enumerate() {
            let row = layout.row(self.px(ROW_PX), &self.step_widths());
            let (Some(number), Some(name_rect), Some(what_rect), Some(go_rect)) =
                (row.first(), row.get(1), row.get(2), row.get(3))
            else {
                continue;
            };
            commands.push(text(*number, &format!("{step}"), Paint::Muted));
            commands.push(text(*name_rect, word(Word::Help(name)), Paint::Ink));
            commands.push(text(*what_rect, word(Word::Help(what)), Paint::Ink));
            let id = GO.indexed(u32::try_from(step).unwrap_or(u32::MAX));
            let response = interaction.probe(id, *go_rect, true);
            if response.clicked() {
                chosen = Some(step);
            }
            commands.extend(button::button(*go_rect, word(Word::Help(go)), response));
        }
        if let Some(keys) = layout.row(self.px(ROW_PX), &[ColumnWidth::Fill]).first() {
            commands.push(text(*keys, word(Word::Help(HelpWord::Keys)), Paint::Muted));
        }
        if chosen.is_some() {
            self.open = false;
        }
        chosen
    }

    /// 窓の真ん中の枠（見出し ＋ 6 段 ＋ 鍵の 8 行）。窓が狭ければ窓に合わせる。
    fn dialog(self, bounds: PixelRect) -> PixelRect {
        let margin = self.px(MARGIN_PX);
        let width = self
            .px(WIDTH_PX)
            .min(bounds.width().saturating_sub(margin.saturating_mul(2)));
        let height = margin
            .saturating_mul(2)
            .saturating_add(self.px(ROW_PX).saturating_mul(8))
            .saturating_add(self.px(GAP_PX).saturating_mul(7));
        PixelRect::new(
            bounds
                .x()
                .saturating_add(bounds.width().saturating_sub(width) / 2),
            bounds
                .y()
                .saturating_add(bounds.height().saturating_sub(height) / 2),
            width,
            height,
        )
    }

    fn step_widths(self) -> [ColumnWidth; 4] {
        [
            ColumnWidth::Fixed(self.px(NUMBER_PX)),
            ColumnWidth::Fixed(self.px(NAME_PX)),
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(GO_PX)),
        ]
    }
}

/// 縦は中央、左寄せの 1 行。
fn text(rect: PixelRect, text: &str, paint: Paint) -> DrawCommand {
    DrawCommand::Text {
        rect,
        text: alloc::borrow::ToOwned::to_owned(text),
        align: TextAlign::Start,
        paint,
        caret: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_overlay_sits_in_the_middle_and_fits_a_narrow_window() {
        let overlay = HelpOverlay::closed(UiScale::from_dpi(96));
        let wide = overlay.dialog(PixelRect::new(0, 0, 1360, 600));
        assert_eq!(wide, PixelRect::new(240, 144, 880, 312));
        let narrow = overlay.dialog(PixelRect::new(0, 0, 600, 600));
        assert_eq!(narrow.width(), 568);
        assert_eq!(HelpOverlay::step_of(GO.indexed(5)), Some(5));
        assert_eq!(HelpOverlay::step_of(GO.indexed(6)), None);
        assert!(!overlay.is_open());
    }
}
