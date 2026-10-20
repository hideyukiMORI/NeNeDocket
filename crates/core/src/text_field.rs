//! 入力欄の描画命令。

use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::text_align::TextAlign;
use crate::text_edit::TextEdit;

/// 縁と文字の間。
const PADDING_PX: u16 = 6;

/// 入力欄。フォーカスがあれば強調の縁で、カーソルと選択を描く。確定前の文字列はカーソルの位置に差し込んで描く。
#[must_use]
pub fn text_field(rect: PixelRect, edit: &TextEdit, response: Response) -> Vec<DrawCommand> {
    let (text, caret) = edit.display();
    let border = if response.focused() {
        Paint::Accent
    } else {
        Paint::Border
    };
    alloc::vec![
        DrawCommand::Fill {
            rect,
            paint: Paint::Surface
        },
        DrawCommand::Stroke {
            rect,
            paint: border
        },
        DrawCommand::Clip(rect.inset(1)),
        DrawCommand::Text {
            rect: rect.inset(PADDING_PX),
            text,
            align: TextAlign::Start,
            paint: Paint::Ink,
            caret: response.focused().then_some(caret),
        },
        DrawCommand::Unclip,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preedit::Preedit;
    use crate::response_phase::ResponsePhase;
    use crate::text_caret::TextCaret;

    const RECT: PixelRect = PixelRect::new(0, 0, 100, 24);

    #[test]
    fn unfocused_field_has_no_caret() {
        let commands = text_field(
            RECT,
            &TextEdit::showing("ab"),
            Response::new(ResponsePhase::Idle, false, false),
        );
        assert_eq!(commands.len(), 5);
        assert!(matches!(
            commands.get(3),
            Some(DrawCommand::Text { text, caret: None, .. }) if text == "ab"
        ));
        assert_eq!(
            commands.get(2),
            Some(&DrawCommand::Clip(PixelRect::new(1, 1, 98, 22)))
        );
    }

    #[test]
    fn focused_field_shows_the_caret_inside_the_preedit() {
        let edit = TextEdit::showing("ab").with_preedit(Some(Preedit::new("か", 1)));
        let commands = text_field(RECT, &edit, Response::new(ResponsePhase::Idle, false, true));
        assert_eq!(
            commands.get(1),
            Some(&DrawCommand::Stroke {
                rect: RECT,
                paint: Paint::Accent
            })
        );
        assert!(matches!(
            commands.get(3),
            Some(DrawCommand::Text { text, caret: Some(caret), rect, .. })
                if text == "abか" && *caret == TextCaret::new(3, 3) && *rect == RECT.inset(PADDING_PX)
        ));
    }
}
