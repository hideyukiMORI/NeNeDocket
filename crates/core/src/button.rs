//! ボタンの描画命令。

use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::response_phase::ResponsePhase;
use crate::text_align::TextAlign;

/// ボタン。押している間は強調の面、フォーカスがあれば強調の縁。
#[must_use]
pub fn button(rect: PixelRect, label: &str, response: Response) -> Vec<DrawCommand> {
    let (face, ink) = match response.phase() {
        ResponsePhase::Held => (Paint::Accent, Paint::OnAccent),
        ResponsePhase::Hovered => (Paint::Selection, Paint::Ink),
        ResponsePhase::Idle => (Paint::Surface, Paint::Ink),
    };
    let border = if response.focused() {
        Paint::Accent
    } else {
        Paint::Border
    };
    alloc::vec![
        DrawCommand::Fill { rect, paint: face },
        DrawCommand::Stroke {
            rect,
            paint: border
        },
        DrawCommand::Text {
            rect,
            text: label.to_owned(),
            align: TextAlign::Center,
            paint: ink,
            caret: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: PixelRect = PixelRect::new(0, 0, 80, 24);

    #[test]
    fn idle_button_is_a_surface_with_a_border_and_a_centered_label() {
        let commands = button(RECT, "OK", Response::new(ResponsePhase::Idle, false, false));
        assert_eq!(commands.len(), 3);
        assert_eq!(
            commands.first(),
            Some(&DrawCommand::Fill {
                rect: RECT,
                paint: Paint::Surface
            })
        );
        assert!(matches!(
            commands.last(),
            Some(DrawCommand::Text { text, align: TextAlign::Center, paint: Paint::Ink, caret: None, .. }) if text == "OK"
        ));
    }

    #[test]
    fn held_and_focused_buttons_use_the_accent() {
        let held = button(RECT, "OK", Response::new(ResponsePhase::Held, false, true));
        assert_eq!(
            held.first(),
            Some(&DrawCommand::Fill {
                rect: RECT,
                paint: Paint::Accent
            })
        );
        assert_eq!(
            held.get(1),
            Some(&DrawCommand::Stroke {
                rect: RECT,
                paint: Paint::Accent
            })
        );
        let hovered = button(
            RECT,
            "OK",
            Response::new(ResponsePhase::Hovered, false, false),
        );
        assert_eq!(
            hovered.first(),
            Some(&DrawCommand::Fill {
                rect: RECT,
                paint: Paint::Selection
            })
        );
    }
}
