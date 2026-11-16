//! 下線のタブの描画命令（詳細のペインの区画の切り替え・Issue #127）。状態の札と形を変え、押す札と見分ける。

use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::focus_ring;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::response_phase::ResponsePhase;
use crate::text_align::TextAlign;

/// 下線の太さ（1x）。呼び手が倍率を掛けて渡す。
pub const UNDERLINE_PX: u16 = 2;

/// タブ。選んでいれば濃い字と強調の下線、ポインタが乗るか押していれば選びの面、フォーカスは強調の縁。
#[must_use]
pub fn tab(
    rect: PixelRect,
    label: &str,
    (chosen, underline): (bool, u16),
    response: Response,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    match response.phase() {
        ResponsePhase::Hovered | ResponsePhase::Held => commands.push(DrawCommand::Fill {
            rect,
            paint: Paint::Selection,
        }),
        ResponsePhase::Idle => {}
    }
    if chosen {
        commands.push(DrawCommand::Fill {
            rect: PixelRect::new(
                rect.x(),
                rect.y()
                    .saturating_add(rect.height().saturating_sub(underline)),
                rect.width(),
                underline,
            ),
            paint: Paint::Accent,
        });
    }
    commands.push(DrawCommand::Text {
        rect,
        text: label.to_owned(),
        align: TextAlign::Center,
        paint: if chosen { Paint::Ink } else { Paint::Muted },
        caret: None,
    });
    commands.extend(focus_ring::ring(rect, response));
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: PixelRect = PixelRect::new(0, 10, 90, 28);
    const IDLE: Response = Response::new(ResponsePhase::Idle, false, false);

    #[test]
    fn a_chosen_tab_is_underlined_in_the_accent_with_ink_and_others_are_muted() {
        let chosen = tab(RECT, "履歴 14", (true, 2), IDLE);
        assert_eq!(
            chosen.first(),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(0, 36, 90, 2),
                paint: Paint::Accent
            })
        );
        assert!(matches!(
            chosen.last(),
            Some(DrawCommand::Text {
                paint: Paint::Ink,
                ..
            })
        ));
        let other = tab(RECT, "添付 2", (false, 2), IDLE);
        assert_eq!(other.len(), 1);
        assert!(matches!(
            other.last(),
            Some(DrawCommand::Text {
                paint: Paint::Muted,
                ..
            })
        ));
    }

    #[test]
    fn hover_fills_the_selection_and_focus_adds_the_outer_ring() {
        let hovered = tab(
            RECT,
            "本文",
            (false, 2),
            Response::new(ResponsePhase::Hovered, false, true),
        );
        assert_eq!(
            hovered.first(),
            Some(&DrawCommand::Fill {
                rect: RECT,
                paint: Paint::Selection
            })
        );
        assert!(matches!(
            hovered.last(),
            Some(DrawCommand::Stroke { paint: Paint::Accent, rect }) if *rect == RECT.outset(2)
        ));
    }
}
