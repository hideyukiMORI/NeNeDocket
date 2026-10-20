//! 描画命令。core が出し、UI が cairo / Direct2D で描く（ADR 0006）。

use alloc::string::String;

use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::text_align::TextAlign;
use crate::text_caret::TextCaret;

/// 1 つの描画命令。列の順に描く。閉じた集合（RS-002）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawCommand {
    /// 矩形を塗る。
    Fill {
        /// 矩形。
        rect: PixelRect,
        /// 色の意味。
        paint: Paint,
    },
    /// 矩形の縁を 1 画素で描く。
    Stroke {
        /// 矩形。
        rect: PixelRect,
        /// 色の意味。
        paint: Paint,
    },
    /// 矩形の中に 1 行の文字を描く。`caret` があればカーソルと選択も描く（位置は UI がフォントで決める）。
    Text {
        /// 矩形。文字はこの中で縦中央。
        rect: PixelRect,
        /// 文字列。
        text: String,
        /// 横の寄せ。
        align: TextAlign,
        /// 色の意味。
        paint: Paint,
        /// カーソルと選択。無ければ描かない。
        caret: Option<TextCaret>,
    },
    /// 以降の描画を矩形で切り抜く。`Unclip` まで。
    Clip(PixelRect),
    /// 直前の `Clip` を解く。
    Unclip,
}
