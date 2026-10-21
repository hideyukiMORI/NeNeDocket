//! チェックボックスの描画命令。

use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::text_align::TextAlign;
use crate::ui_scale::UiScale;

/// 四角の一辺（1x）。
const BOX_PX: u16 = 16;
/// 四角と文字の隙間（1x）。
const GAP_PX: u16 = 8;
/// 印（塗った四角）の、四角の縁からの引っ込み（1x）。
const MARK_INSET_PX: u16 = 4;

/// 四角と、その右の文字。`checked` なら四角の中を強調で塗る。寸法は `scale` を掛ける（FR-016）。
#[must_use]
pub fn checkbox(
    rect: PixelRect,
    (label, checked): (&str, bool),
    response: Response,
    scale: UiScale,
) -> Vec<DrawCommand> {
    let square = checkbox_square(rect, scale);
    let border = if response.focused() {
        Paint::Accent
    } else {
        Paint::Border
    };
    let mut commands = alloc::vec![
        DrawCommand::Fill {
            rect: square,
            paint: Paint::Surface
        },
        DrawCommand::Stroke {
            rect: square,
            paint: border
        },
    ];
    if checked {
        commands.push(DrawCommand::Fill {
            rect: square.inset(scale.px(MARK_INSET_PX)),
            paint: Paint::Accent,
        });
    }
    let text_left = square
        .x()
        .saturating_add(scale.px(BOX_PX))
        .saturating_add(scale.px(GAP_PX));
    commands.push(DrawCommand::Text {
        rect: PixelRect::new(
            text_left,
            rect.y(),
            rect.right().saturating_sub(text_left).saturating_add(1),
            rect.height(),
        ),
        text: label.to_owned(),
        align: TextAlign::Start,
        paint: Paint::Ink,
        caret: None,
    });
    commands
}

/// 四角の矩形。左端で縦中央。一辺は `scale` を掛けた値。
#[must_use]
pub const fn checkbox_square(rect: PixelRect, scale: UiScale) -> PixelRect {
    let side = scale.px(BOX_PX);
    let top = rect
        .y()
        .saturating_add(rect.height().saturating_sub(side) / 2);
    PixelRect::new(rect.x(), top, side, side)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response_phase::ResponsePhase;

    const RECT: PixelRect = PixelRect::new(10, 10, 120, 24);

    #[test]
    fn square_sits_at_the_left_middle_and_the_label_follows() {
        assert_eq!(
            checkbox_square(RECT, UiScale::Single),
            PixelRect::new(10, 14, 16, 16)
        );
        let commands = checkbox(
            RECT,
            ("Done", false),
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Single,
        );
        assert_eq!(commands.len(), 3);
        assert!(matches!(
            commands.last(),
            Some(DrawCommand::Text { rect, text, .. }) if rect.x() == 34 && rect.width() == 96 && text == "Done"
        ));
    }

    #[test]
    fn checked_box_adds_an_accent_mark_and_focus_colours_the_border() {
        let commands = checkbox(
            RECT,
            ("Done", true),
            Response::new(ResponsePhase::Idle, false, true),
            UiScale::Single,
        );
        assert_eq!(commands.len(), 4);
        assert_eq!(
            commands.get(1),
            Some(&DrawCommand::Stroke {
                rect: PixelRect::new(10, 14, 16, 16),
                paint: Paint::Accent
            })
        );
        assert_eq!(
            commands.get(2),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(14, 18, 8, 8),
                paint: Paint::Accent
            })
        );
    }

    #[test]
    fn double_scale_doubles_the_square_the_gap_and_the_mark() {
        let rect = PixelRect::new(10, 10, 240, 48);
        assert_eq!(
            checkbox_square(rect, UiScale::Double),
            PixelRect::new(10, 18, 32, 32)
        );
        let commands = checkbox(
            rect,
            ("Done", true),
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Double,
        );
        assert_eq!(
            commands.get(2),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(18, 26, 16, 16),
                paint: Paint::Accent
            })
        );
        assert!(matches!(
            commands.last(),
            Some(DrawCommand::Text { rect: label, .. }) if label.x() == 58 && label.width() == 192
        ));
    }
}
