//! 札の描画命令（状態の札・区画の切り替え・Issue #122）。押すと選ぶボタンで、選んでいる札は強調の縁と選びの面。

use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::response_phase::ResponsePhase;
use crate::text_align::TextAlign;

/// 札。選んでいれば選びの面と強調の縁、押している間は強調の面、ポインタが乗れば選びの面。
#[must_use]
pub fn chip(rect: PixelRect, label: &str, chosen: bool, response: Response) -> Vec<DrawCommand> {
    let (face, ink) = match (response.phase(), chosen) {
        (ResponsePhase::Held, _) => (Paint::Accent, Paint::OnAccent),
        (ResponsePhase::Hovered, _) | (ResponsePhase::Idle, true) => (Paint::Selection, Paint::Ink),
        (ResponsePhase::Idle, false) => (Paint::Surface, Paint::Muted),
    };
    let border = if chosen || response.focused() {
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
    const IDLE: Response = Response::new(ResponsePhase::Idle, false, false);

    #[test]
    fn a_chosen_chip_has_the_selection_face_and_the_accent_border() {
        let chosen = chip(RECT, "完了", true, IDLE);
        assert_eq!(
            chosen.first(),
            Some(&DrawCommand::Fill {
                rect: RECT,
                paint: Paint::Selection
            })
        );
        assert_eq!(
            chosen.get(1),
            Some(&DrawCommand::Stroke {
                rect: RECT,
                paint: Paint::Accent
            })
        );
        let other = chip(RECT, "保留", false, IDLE);
        assert!(matches!(
            other.last(),
            Some(DrawCommand::Text {
                paint: Paint::Muted,
                ..
            })
        ));
        let held = chip(
            RECT,
            "保留",
            false,
            Response::new(ResponsePhase::Held, false, false),
        );
        assert_eq!(
            held.first(),
            Some(&DrawCommand::Fill {
                rect: RECT,
                paint: Paint::Accent
            })
        );
    }
}
