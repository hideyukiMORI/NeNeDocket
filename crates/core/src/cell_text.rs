//! 表のセルの文字の描画命令。

use alloc::borrow::ToOwned;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::text_align::TextAlign;

/// セルの文字。左右に `padding` を空け、その矩形で切り抜く。
/// 描き手は `Text` の矩形の外にも文字を描くので、狭い列で隣の列に重ならないよう `Clip` で囲む。
/// 表の本体の `Clip` の内側に入れ子になる（cairo の save / restore・Direct2D の Push / Pop は入れ子で交わりを取る）。
#[must_use]
pub fn cell_text(cell: PixelRect, text: &str, paint: Paint, padding: u16) -> [DrawCommand; 3] {
    let rect = PixelRect::new(
        cell.x().saturating_add(padding),
        cell.y(),
        cell.width().saturating_sub(padding.saturating_mul(2)),
        cell.height(),
    );
    [
        DrawCommand::Clip(rect),
        DrawCommand::Text {
            rect,
            text: text.to_owned(),
            align: TextAlign::Start,
            paint,
            caret: None,
        },
        DrawCommand::Unclip,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_clipped_to_the_padded_cell() {
        let [clip, text, unclip] = cell_text(
            PixelRect::new(100, 20, 40, 24),
            "a long title",
            Paint::Ink,
            6,
        );
        let inner = PixelRect::new(106, 20, 28, 24);
        assert_eq!(clip, DrawCommand::Clip(inner));
        assert_eq!(
            text,
            DrawCommand::Text {
                rect: inner,
                text: "a long title".to_owned(),
                align: TextAlign::Start,
                paint: Paint::Ink,
                caret: None,
            }
        );
        assert_eq!(unclip, DrawCommand::Unclip);
    }

    #[test]
    fn a_cell_narrower_than_its_padding_clips_everything() {
        let [clip, ..] = cell_text(PixelRect::new(10, 0, 8, 24), "x", Paint::Muted, 6);
        assert_eq!(clip, DrawCommand::Clip(PixelRect::new(16, 0, 0, 24)));
    }
}
