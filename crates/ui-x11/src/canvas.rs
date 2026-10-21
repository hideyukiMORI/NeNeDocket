//! `DocketFrame` の描画命令を cairo の `ImageSurface` に描く。X 無しで画素を検証できる（ADR 0002 / 0006）。
//!
//! 文字の幅はここで測る。core はカーソルを文字の添字で渡し、ここがフォントで画素の位置に直す。

use cairo::{Context, Filter, FontSlant, FontWeight, Format, ImageSurface, Matrix, SurfacePattern};
use nene_docket_application::docket_frame::DocketFrame;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::paint::Paint;
use nene_docket_core::palette::Palette;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::shared_bitmap::SharedBitmap;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::text_caret::TextCaret;

use crate::surface_cache::SurfaceCache;
use crate::ui_failure::UiFailure;

/// 完成した 1 フレームの画素の列。
#[derive(Debug)]
pub struct Canvas {
    surface: ImageSurface,
    width: u16,
    height: u16,
}

impl Canvas {
    /// フレームを描き切る。
    ///
    /// # Errors
    ///
    /// cairo が surface を作れない、または描画に失敗した。
    pub fn render(
        frame: &DocketFrame,
        bounds: PixelRect,
        (format, font_family): (Format, &str),
        cache: &mut SurfaceCache,
    ) -> Result<Self, UiFailure> {
        let surface = ImageSurface::create(
            format,
            i32::from(bounds.width()),
            i32::from(bounds.height()),
        )?;
        {
            let context = Context::new(&surface)?;
            context.select_font_face(font_family, FontSlant::Normal, FontWeight::Normal);
            context.set_font_size(f64::from(frame.font_px()));
            let palette = frame.theme().palette();
            for command in frame.commands() {
                draw(&context, &palette, command, cache)?;
            }
        }
        surface.flush();
        Ok(Self {
            surface,
            width: bounds.width(),
            height: bounds.height(),
        })
    }

    /// 幅。
    #[must_use]
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// 高さ。
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// 1 行のバイト数。
    ///
    /// # Errors
    ///
    /// cairo の stride が負（起こらないが型の上では `i32`）。
    pub fn stride(&self) -> Result<usize, UiFailure> {
        usize::try_from(self.surface.stride()).map_err(|_error| UiFailure::GeometryOutOfRange)
    }

    /// 画素の列をそのまま読む。ネイティブエンディアンの 32 ビット（X の `ZPixmap` と同じ並び）。
    ///
    /// # Errors
    ///
    /// 描画中の `Context` が残っていて借りられない。
    pub fn with_pixels<T, F: FnOnce(&[u8]) -> T>(&mut self, read: F) -> Result<T, UiFailure> {
        let bytes = self.surface.data()?;
        Ok(read(&bytes))
    }

    /// 1 画素の `(blue, green, red, alpha)`。リトルエンディアンの並び。範囲外は `None`。
    ///
    /// # Errors
    ///
    /// 画素を借りられない。
    pub fn pixel(&mut self, x: u16, y: u16) -> Result<Option<[u8; 4]>, UiFailure> {
        if x >= self.width || y >= self.height {
            return Ok(None);
        }
        let stride = self.stride()?;
        let offset = usize::from(y) * stride + usize::from(x) * 4;
        self.with_pixels(|bytes| {
            bytes
                .get(offset..offset.checked_add(4)?)
                .and_then(|slice| <[u8; 4]>::try_from(slice).ok())
        })
    }
}

/// 1 つの命令を描く。
fn draw(
    context: &Context,
    palette: &Palette,
    command: &DrawCommand,
    cache: &mut SurfaceCache,
) -> Result<(), UiFailure> {
    match command {
        DrawCommand::Fill { rect, paint } => {
            set_paint(context, palette, *paint);
            fill_rect(context, *rect)
        }
        DrawCommand::Stroke { rect, paint } => {
            set_paint(context, palette, *paint);
            stroke_rect(context, *rect)
        }
        DrawCommand::Text {
            rect,
            text,
            align,
            paint,
            caret,
        } => draw_text(context, palette, (*rect, *align, *paint), (text, *caret)),
        DrawCommand::Image {
            dest,
            source,
            bitmap,
        } => draw_image(context, (*dest, *source), bitmap, cache),
        DrawCommand::Clip(rect) => {
            context.save()?;
            rectangle(context, *rect);
            context.clip();
            Ok(())
        }
        DrawCommand::Unclip => {
            context.restore()?;
            Ok(())
        }
    }
}

/// 画素の列の `source` を `dest` に伸縮して描く。`dest` で切り抜く。
fn draw_image(
    context: &Context,
    (dest, source): (PixelRect, PixelRect),
    bitmap: &SharedBitmap,
    cache: &mut SurfaceCache,
) -> Result<(), UiFailure> {
    if dest.is_empty() || source.is_empty() {
        return Ok(());
    }
    let surface = cache.surface_for(bitmap)?;
    context.save()?;
    rectangle(context, dest);
    context.clip();
    context.translate(f64::from(dest.x()), f64::from(dest.y()));
    context.scale(
        f64::from(dest.width()) / f64::from(source.width()),
        f64::from(dest.height()) / f64::from(source.height()),
    );
    // 🔴 既定の Good は縮めるときに畳み込みで、2000 × 1500 を 1 フレーム 36 ms かかった。Bilinear は 2 ms（gate-proofs 第 41 節）。
    let pattern = SurfacePattern::create(surface);
    pattern.set_filter(Filter::Bilinear);
    let mut matrix = Matrix::identity();
    matrix.translate(f64::from(source.x()), f64::from(source.y()));
    pattern.set_matrix(matrix);
    context.set_source(&pattern)?;
    context.paint()?;
    context.restore()?;
    Ok(())
}

/// 描く色を意味から決める（フレームの配色・0〜255 を 0〜1 に）。
fn set_paint(context: &Context, palette: &Palette, paint: Paint) {
    let rgb = palette.colour(paint);
    let unit = |channel: u8| f64::from(channel) / 255.0_f64;
    context.set_source_rgb(unit(rgb.red()), unit(rgb.green()), unit(rgb.blue()));
}

fn rectangle(context: &Context, rect: PixelRect) {
    context.rectangle(
        f64::from(rect.x()),
        f64::from(rect.y()),
        f64::from(rect.width()),
        f64::from(rect.height()),
    );
}

/// 整数座標の矩形を塗る。
fn fill_rect(context: &Context, rect: PixelRect) -> Result<(), UiFailure> {
    rectangle(context, rect);
    context.fill()?;
    Ok(())
}

/// 矩形の縁を 1 画素で描く。画素の中心に線を置くので半画素ずらす。
fn stroke_rect(context: &Context, rect: PixelRect) -> Result<(), UiFailure> {
    if rect.is_empty() {
        return Ok(());
    }
    if rect.width() == 1 || rect.height() == 1 {
        return fill_rect(context, rect);
    }
    context.set_line_width(1.0_f64);
    context.rectangle(
        f64::from(rect.x()) + 0.5_f64,
        f64::from(rect.y()) + 0.5_f64,
        f64::from(rect.width()) - 1.0_f64,
        f64::from(rect.height()) - 1.0_f64,
    );
    context.stroke()?;
    Ok(())
}

/// 矩形の中に 1 行。縦は中央、横は寄せに従う。カーソルがあれば選択を塗り、文字を描き、カーソルの線を置く。
fn draw_text(
    context: &Context,
    palette: &Palette,
    (rect, align, paint): (PixelRect, TextAlign, Paint),
    (text, caret): (&str, Option<TextCaret>),
) -> Result<(), UiFailure> {
    let font = context.font_extents()?;
    let advance = context.text_extents(text)?.x_advance();
    let left = match align {
        TextAlign::Start => f64::from(rect.x()),
        TextAlign::Center => f64::from(rect.x()) + (f64::from(rect.width()) - advance) / 2.0_f64,
        TextAlign::End => f64::from(rect.x()) + f64::from(rect.width()) - advance,
    };
    let baseline =
        f64::from(rect.y()) + (f64::from(rect.height()) - font.height()) / 2.0_f64 + font.ascent();
    let top = baseline - font.ascent();
    if let Some(caret) = caret.filter(|caret| caret.has_selection()) {
        let start = left + prefix_advance(context, text, caret.start())?;
        let end = left + prefix_advance(context, text, caret.end())?;
        set_paint(context, palette, Paint::Selection);
        context.rectangle(start, top, end - start, font.height());
        context.fill()?;
    }
    set_paint(context, palette, paint);
    context.move_to(left, baseline);
    context.show_text(text)?;
    if let Some((start, end)) = caret.and_then(TextCaret::underline) {
        draw_underline(context, (left, top + font.height()), text, (start, end))?;
    }
    if let Some(caret) = caret.filter(|caret| caret.shows_bar()) {
        let x = left + prefix_advance(context, text, caret.cursor())?;
        context.rectangle(x.floor(), top, 1.0_f64, font.height());
        context.fill()?;
    }
    Ok(())
}

/// 確定前の文字列の下線（1 画素・いまの色）。`bottom` は文字の下端。
fn draw_underline(
    context: &Context,
    (left, bottom): (f64, f64),
    text: &str,
    (start, end): (usize, usize),
) -> Result<(), UiFailure> {
    let from = left + prefix_advance(context, text, start)?;
    let to = left + prefix_advance(context, text, end)?;
    context.rectangle(from.floor(), bottom.floor() - 1.0_f64, to - from, 1.0_f64);
    context.fill()?;
    Ok(())
}

/// 先頭から `chars` 文字分の幅。
fn prefix_advance(context: &Context, text: &str, chars: usize) -> Result<f64, UiFailure> {
    let prefix: String = text.chars().take(chars).collect();
    Ok(context.text_extents(&prefix)?.x_advance())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_docket_core::language::Language;
    use nene_docket_core::rgb::Rgb;
    use nene_docket_core::theme::Theme;
    use nene_docket_core::theme_choice::ThemeChoice;
    use nene_docket_core::ui_scale::UiScale;

    const BOUNDS: PixelRect = PixelRect::new(0, 0, 100, 40);

    fn theme() -> Theme {
        ThemeChoice::System.resolve(None)
    }

    /// 配色の色を、cairo の画素の並び（青・緑・赤・不透明）で。
    fn pixel_of(paint: Paint) -> [u8; 4] {
        let rgb: Rgb = theme().palette().colour(paint);
        [rgb.blue(), rgb.green(), rgb.red(), 0xff]
    }

    fn render(commands: Vec<DrawCommand>) -> Canvas {
        let frame = DocketFrame::new((UiScale::Single, Language::English, theme()), commands);
        Canvas::render(
            &frame,
            BOUNDS,
            (Format::Rgb24, "sans-serif"),
            &mut SurfaceCache::empty(),
        )
        .unwrap()
    }

    #[test]
    fn fill_and_stroke_land_on_integer_pixels() {
        let mut canvas = render(vec![
            DrawCommand::Fill {
                rect: BOUNDS,
                paint: Paint::Surface,
            },
            DrawCommand::Fill {
                rect: PixelRect::new(10, 10, 20, 10),
                paint: Paint::Ink,
            },
            DrawCommand::Stroke {
                rect: PixelRect::new(40, 10, 20, 10),
                paint: Paint::Accent,
            },
        ]);
        assert_eq!(canvas.pixel(10, 10).unwrap(), Some(pixel_of(Paint::Ink)));
        assert_eq!(canvas.pixel(29, 19).unwrap(), Some(pixel_of(Paint::Ink)));
        assert_eq!(
            canvas.pixel(30, 10).unwrap(),
            Some(pixel_of(Paint::Surface))
        );
        assert_eq!(canvas.pixel(40, 10).unwrap(), Some(pixel_of(Paint::Accent)));
        assert_eq!(canvas.pixel(59, 19).unwrap(), Some(pixel_of(Paint::Accent)));
        assert_eq!(
            canvas.pixel(50, 15).unwrap(),
            Some(pixel_of(Paint::Surface))
        );
        assert_eq!(canvas.pixel(100, 0).unwrap(), None);
        assert_eq!((canvas.width(), canvas.height()), (100, 40));
    }

    #[test]
    fn clip_limits_the_fill_until_unclip() {
        let mut canvas = render(vec![
            DrawCommand::Fill {
                rect: BOUNDS,
                paint: Paint::Surface,
            },
            DrawCommand::Clip(PixelRect::new(0, 0, 50, 40)),
            DrawCommand::Fill {
                rect: BOUNDS,
                paint: Paint::Ink,
            },
            DrawCommand::Unclip,
            DrawCommand::Fill {
                rect: PixelRect::new(90, 0, 10, 40),
                paint: Paint::Accent,
            },
        ]);
        assert_eq!(canvas.pixel(49, 20).unwrap(), Some(pixel_of(Paint::Ink)));
        assert_eq!(
            canvas.pixel(50, 20).unwrap(),
            Some(pixel_of(Paint::Surface))
        );
        assert_eq!(canvas.pixel(95, 20).unwrap(), Some(pixel_of(Paint::Accent)));
    }

    /// その色の画素の数。
    fn count(canvas: &mut Canvas, colour: [u8; 4]) -> u32 {
        let pixels = canvas.with_pixels(<[u8]>::to_vec).unwrap();
        pixels
            .chunks(4)
            .filter(|pixel| pixel.first().copied() == colour.first().copied())
            .filter(|pixel| pixel.get(1).copied() == colour.get(1).copied())
            .filter(|pixel| pixel.get(2).copied() == colour.get(2).copied())
            .count()
            .try_into()
            .unwrap()
    }

    #[test]
    fn text_with_a_caret_draws_the_selection_and_the_bar() {
        let text = DrawCommand::Text {
            rect: PixelRect::new(4, 0, 92, 40),
            text: String::from("abcdef"),
            align: TextAlign::Start,
            paint: Paint::Ink,
            caret: Some(TextCaret::new(3, 1)),
        };
        let mut canvas = render(vec![
            DrawCommand::Fill {
                rect: BOUNDS,
                paint: Paint::Surface,
            },
            text,
        ]);
        let ink = count(&mut canvas, pixel_of(Paint::Ink));
        let selection = count(&mut canvas, pixel_of(Paint::Selection));
        assert!(ink > 20, "{ink}");
        assert!(selection > 20, "{selection}");
    }

    #[test]
    fn an_underline_adds_ink_below_the_composing_characters() {
        let plain = DrawCommand::Text {
            rect: PixelRect::new(4, 0, 92, 40),
            text: String::from("abcdef"),
            align: TextAlign::Start,
            paint: Paint::Ink,
            caret: Some(TextCaret::new(3, 3)),
        };
        let underlined = DrawCommand::Text {
            rect: PixelRect::new(4, 0, 92, 40),
            text: String::from("abcdef"),
            align: TextAlign::Start,
            paint: Paint::Ink,
            caret: Some(TextCaret::new(3, 3).with_underline(1, 4)),
        };
        let background = DrawCommand::Fill {
            rect: BOUNDS,
            paint: Paint::Surface,
        };
        let mut without = render(vec![background.clone(), plain]);
        let mut with = render(vec![background, underlined]);
        let before = count(&mut without, pixel_of(Paint::Ink));
        let after = count(&mut with, pixel_of(Paint::Ink));
        assert!(after > before + 10, "{before} -> {after}");
    }

    #[test]
    fn centered_and_end_aligned_text_stay_inside_the_rect() {
        for align in [TextAlign::Center, TextAlign::End] {
            let mut canvas = render(vec![
                DrawCommand::Fill {
                    rect: BOUNDS,
                    paint: Paint::Surface,
                },
                DrawCommand::Text {
                    rect: PixelRect::new(50, 0, 50, 40),
                    text: String::from("ab"),
                    align,
                    paint: Paint::Ink,
                    caret: None,
                },
            ]);
            let left_half_untouched = (0_u16..50).all(|x| {
                (0_u16..40).all(|y| canvas.pixel(x, y).unwrap() == Some(pixel_of(Paint::Surface)))
            });
            assert!(left_half_untouched, "{align:?}");
        }
    }
}
