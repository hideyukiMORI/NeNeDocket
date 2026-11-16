//! フォーカスの輪（ADR 0025・WCAG 2.4.7 / 2.4.13）。部品の縁の外 1〜2 px に強調の色で描き、
//! 選んでいる札の縁（強調の 1 px）とは形で見分ける。部品の描画命令の最後に足す。

use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;

/// フォーカスがあれば `rect` の外 1〜2 px の輪、無ければ何も無い。
#[must_use]
pub fn ring(rect: PixelRect, response: Response) -> Vec<DrawCommand> {
    if !response.focused() {
        return Vec::new();
    }
    alloc::vec![
        DrawCommand::Stroke {
            rect: rect.outset(1),
            paint: Paint::Accent,
        },
        DrawCommand::Stroke {
            rect: rect.outset(2),
            paint: Paint::Accent,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response_phase::ResponsePhase;

    #[test]
    fn the_ring_sits_outside_the_rect_only_while_focused() {
        let rect = PixelRect::new(10, 10, 80, 24);
        assert!(ring(rect, Response::new(ResponsePhase::Idle, false, false)).is_empty());
        assert_eq!(
            ring(rect, Response::new(ResponsePhase::Idle, false, true)),
            alloc::vec![
                DrawCommand::Stroke {
                    rect: PixelRect::new(9, 9, 82, 26),
                    paint: Paint::Accent
                },
                DrawCommand::Stroke {
                    rect: PixelRect::new(8, 8, 84, 28),
                    paint: Paint::Accent
                },
            ]
        );
        assert_eq!(
            PixelRect::new(0, 1, 4, 4).outset(2),
            PixelRect::new(0, 0, 6, 7)
        );
    }
}
