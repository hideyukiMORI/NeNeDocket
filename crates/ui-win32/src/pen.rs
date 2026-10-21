//! Direct2D / DirectWrite で `DocketFrame` の描画命令を窓に描く（ADR 0002 / 0009）。X11 版の `canvas.rs` と同じ順・同じ意味。
//!
//! 1 DIP = 1 物理画素（レンダーターゲットの dpi を 96 に固定）。文字の幅はここで測る。
//! core はカーソルを文字の添字で渡し、ここが DirectWrite で画素の位置に直す。

use std::cell::RefCell;
use std::collections::BTreeMap;

use nene_docket_application::docket_frame::DocketFrame;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::language::Language;
use nene_docket_core::paint::Paint;
use nene_docket_core::palette::Palette;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::shared_bitmap::SharedBitmap;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::text_caret::TextCaret;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct2D::Common::{
    D2D_RECT_F, D2D_SIZE_U, D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_ALPHA_MODE_UNKNOWN, D2D1_COLOR_F,
    D2D1_PIXEL_FORMAT,
};
use windows::Win32::Graphics::Direct2D::{
    D2D1_ANTIALIAS_MODE_ALIASED, D2D1_BITMAP_INTERPOLATION_MODE_LINEAR, D2D1_BITMAP_PROPERTIES,
    D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1_FEATURE_LEVEL_DEFAULT,
    D2D1_HWND_RENDER_TARGET_PROPERTIES, D2D1_PRESENT_OPTIONS_NONE, D2D1_RENDER_TARGET_PROPERTIES,
    D2D1_RENDER_TARGET_TYPE_DEFAULT, D2D1_RENDER_TARGET_USAGE_NONE,
    D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE, D2D1CreateFactory, ID2D1Bitmap, ID2D1Brush, ID2D1Factory,
    ID2D1HwndRenderTarget, ID2D1SolidColorBrush,
};
use windows::Win32::Graphics::DirectWrite::{
    DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL,
    DWRITE_FONT_WEIGHT_NORMAL, DWRITE_HIT_TEST_METRICS, DWRITE_LINE_METRICS, DWRITE_TEXT_METRICS,
    DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, IDWriteTextLayout,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_UNKNOWN};
use windows::core::w;
use windows_numerics::Vector2;

use crate::text_line::TextLine;
use crate::win32_failure::Win32Failure;

/// 字の家族。Windows の UI の字。日本語・中国語・韓国語は DirectWrite の fallback に任せる（ADR 0002 の 6）。地域名は表示言語から（ADR 0011）。
const FONT_FAMILY: windows::core::PCWSTR = w!("Segoe UI");

/// DirectWrite に渡す地域名。fallback の字体（日本・中国・台湾・韓国の漢字の字形）がこれで決まる。
const fn locale_name(language: Language) -> windows::core::PCWSTR {
    match language {
        Language::Japanese => w!("ja-jp"),
        Language::English => w!("en-us"),
        Language::SimplifiedChinese => w!("zh-cn"),
        Language::TraditionalChinese => w!("zh-tw"),
        Language::Korean => w!("ko-kr"),
        Language::Spanish => w!("es-es"),
    }
}
/// 字の配置の最大の幅と高さ（1 行）。
const LAYOUT_LIMIT: f32 = 100_000.0;

/// 覚えておく bitmap の数の上限。超えたら全部忘れる。
const BITMAP_LIMIT: usize = 8;

/// Direct2D の工場・窓のレンダーターゲット・DirectWrite の工場。
#[derive(Debug)]
pub struct Pen {
    target: ID2D1HwndRenderTarget,
    words: IDWriteFactory,
    /// 添付の画像の bitmap の覚え（ID が鍵・FR-022）。描画は `&self` なので中で可変。
    bitmaps: RefCell<BTreeMap<u64, ID2D1Bitmap>>,
}

impl Pen {
    /// 窓に結んだレンダーターゲットを作る。
    ///
    /// # Errors
    ///
    /// Direct2D か DirectWrite を作れなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D / DirectWrite の工場と窓のレンダーターゲットを作る（ADR 0002 の 4）"
    )]
    pub fn new(window: HWND, bounds: PixelRect) -> Result<Self, Win32Failure> {
        // SAFETY: 単一スレッドの工場。既定の生成の引数だけを渡す。
        let factory: ID2D1Factory =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None) }?;
        let properties = D2D1_RENDER_TARGET_PROPERTIES {
            r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_UNKNOWN,
                alphaMode: D2D1_ALPHA_MODE_UNKNOWN,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            usage: D2D1_RENDER_TARGET_USAGE_NONE,
            minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
        };
        let window_properties = D2D1_HWND_RENDER_TARGET_PROPERTIES {
            hwnd: window,
            pixelSize: size_u(bounds),
            presentOptions: D2D1_PRESENT_OPTIONS_NONE,
        };
        // SAFETY: 2 つの properties はこの関数の局所で、呼び出しの間だけ読まれる。窓はこのアプリのもの。
        let target = unsafe {
            factory.CreateHwndRenderTarget(&raw const properties, &raw const window_properties)
        }?;
        // SAFETY: 共有の DirectWrite の工場。
        let words: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED) }?;
        Ok(Self {
            target,
            words,
            bitmaps: RefCell::new(BTreeMap::new()),
        })
    }

    /// 窓の大きさが変わった。
    ///
    /// # Errors
    ///
    /// レンダーターゲットの大きさを変えられなかった。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の Resize を呼ぶ（ADR 0002 の 4）"
    )]
    pub fn resize(&self, bounds: PixelRect) -> Result<(), Win32Failure> {
        let size = size_u(bounds);
        // SAFETY: size はこの関数の局所で、呼び出しの間だけ読まれる。
        unsafe { self.target.Resize(&raw const size) }?;
        Ok(())
    }

    /// フレームを描き切って窓に出す。
    ///
    /// # Errors
    ///
    /// 描画に失敗した（レンダーターゲットを作り直すべき失敗も含む）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の BeginDraw / EndDraw を呼ぶ（ADR 0002 の 4）"
    )]
    pub fn paint(&self, frame: &DocketFrame) -> Result<(), Win32Failure> {
        let format = self.format(f32::from(frame.font_px()), frame.language())?;
        let palette = frame.theme().palette();
        // SAFETY: 描画は BeginDraw と EndDraw の間だけ。矩形は画素に揃え、字は灰色の AA。
        unsafe {
            self.target.BeginDraw();
            self.target.SetAntialiasMode(D2D1_ANTIALIAS_MODE_ALIASED);
            self.target
                .SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
        }
        let mut drawn = Ok(());
        let mut clips = 0_u32;
        for command in frame.commands() {
            drawn = self.draw((&format, &palette), command, &mut clips);
            if drawn.is_err() {
                break;
            }
        }
        for _ in 0..clips {
            self.unclip();
        }
        // SAFETY: 上の BeginDraw と対になる。
        unsafe { self.target.EndDraw(None, None) }?;
        drawn
    }

    /// 1 つの命令を描く。`clips` は開いている切り抜きの数（失敗しても閉じるため）。
    fn draw(
        &self,
        (format, palette): (&IDWriteTextFormat, &Palette),
        command: &DrawCommand,
        clips: &mut u32,
    ) -> Result<(), Win32Failure> {
        match command {
            DrawCommand::Fill { rect, paint } => {
                self.fill(*rect, &self.brush(palette, *paint)?);
                Ok(())
            }
            DrawCommand::Stroke { rect, paint } => {
                self.stroke(*rect, &self.brush(palette, *paint)?);
                Ok(())
            }
            DrawCommand::Text {
                rect,
                text,
                align,
                paint,
                caret,
            } => self.text((format, palette), (*rect, *align, *paint), (text, *caret)),
            DrawCommand::Image {
                dest,
                source,
                bitmap,
            } => self.image((*dest, *source), bitmap),
            DrawCommand::Clip(rect) => {
                self.clip(*rect);
                *clips = clips.saturating_add(1);
                Ok(())
            }
            DrawCommand::Unclip => {
                if *clips > 0 {
                    self.unclip();
                    *clips = clips.saturating_sub(1);
                }
                Ok(())
            }
        }
    }

    /// 画素の列の `source` を `dest` に伸縮して描く。bitmap は ID で覚える（上限を超えたら全部忘れる）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の CreateBitmap と DrawBitmap を呼ぶ（ADR 0002 の 4）"
    )]
    fn image(
        &self,
        (dest, source): (PixelRect, PixelRect),
        bitmap: &SharedBitmap,
    ) -> Result<(), Win32Failure> {
        if dest.is_empty() || source.is_empty() {
            return Ok(());
        }
        let mut bitmaps = self.bitmaps.borrow_mut();
        if !bitmaps.contains_key(&bitmap.id()) {
            if bitmaps.len() >= BITMAP_LIMIT {
                bitmaps.clear();
            }
            let pixels = bitmap.bitmap();
            let bgra = pixels.premultiplied_bgra();
            let size = D2D_SIZE_U {
                width: pixels.width(),
                height: pixels.height(),
            };
            let properties = D2D1_BITMAP_PROPERTIES {
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
            };
            // SAFETY: bgra は幅 × 高さ × 4 バイトで、呼び出しの間生きている。pitch は 1 行のバイト数。
            let made = unsafe {
                self.target.CreateBitmap(
                    size,
                    Some(bgra.as_ptr().cast()),
                    pixels.width().saturating_mul(4),
                    &raw const properties,
                )
            }?;
            bitmaps.insert(bitmap.id(), made);
        }
        let Some(made) = bitmaps.get(&bitmap.id()) else {
            return Ok(());
        };
        let (dest_f, source_f) = (rect_f(dest), rect_f(source));
        // SAFETY: 描画は paint の中で呼ばれる。2 つの矩形はこの関数の局所。
        unsafe {
            self.target.DrawBitmap(
                made,
                Some(&raw const dest_f),
                1.0,
                D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                Some(&raw const source_f),
            );
        }
        Ok(())
    }

    /// 1 色の筆。色はフレームの配色（ARC-001）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の CreateSolidColorBrush を呼ぶ（ADR 0002 の 4）"
    )]
    fn brush(&self, palette: &Palette, paint: Paint) -> Result<ID2D1Brush, Win32Failure> {
        let rgb = palette.colour(paint);
        let colour = D2D1_COLOR_F {
            r: f32::from(rgb.red()) / 255.0,
            g: f32::from(rgb.green()) / 255.0,
            b: f32::from(rgb.blue()) / 255.0,
            a: 1.0,
        };
        // SAFETY: colour はこの関数の局所で、呼び出しの間だけ読まれる。
        let solid: ID2D1SolidColorBrush =
            unsafe { self.target.CreateSolidColorBrush(&raw const colour, None) }?;
        Ok(solid.into())
    }

    /// 整数座標の矩形を塗る。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の FillRectangle を呼ぶ（ADR 0002 の 4）"
    )]
    fn fill(&self, rect: PixelRect, brush: &ID2D1Brush) {
        let area = rect_f(rect);
        // SAFETY: 描画は paint の中で呼ばれる。area はこの関数の局所。
        unsafe {
            self.target.FillRectangle(&raw const area, brush);
        }
    }

    /// 矩形の縁を 1 画素で描く。画素の中心に線を置くので半画素ずらす（X11 版と同じ）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の DrawRectangle を呼ぶ（ADR 0002 の 4）"
    )]
    fn stroke(&self, rect: PixelRect, brush: &ID2D1Brush) {
        if rect.is_empty() {
            return;
        }
        if rect.width() == 1 || rect.height() == 1 {
            self.fill(rect, brush);
            return;
        }
        let area = D2D_RECT_F {
            left: f32::from(rect.x()) + 0.5,
            top: f32::from(rect.y()) + 0.5,
            right: f32::from(rect.right()) + 0.5,
            bottom: f32::from(rect.bottom()) + 0.5,
        };
        // SAFETY: 描画は paint の中で呼ばれる。area はこの関数の局所。
        unsafe {
            self.target.DrawRectangle(&raw const area, brush, 1.0, None);
        }
    }

    /// 矩形で切り抜く（`Unclip` まで）。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の PushAxisAlignedClip を呼ぶ（ADR 0002 の 4）"
    )]
    fn clip(&self, rect: PixelRect) {
        let area = rect_f(rect);
        // SAFETY: 描画は paint の中で呼ばれる。area はこの関数の局所。PopAxisAlignedClip と対になる。
        unsafe {
            self.target
                .PushAxisAlignedClip(&raw const area, D2D1_ANTIALIAS_MODE_ALIASED);
        }
    }

    /// 切り抜きを戻す。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の PopAxisAlignedClip を呼ぶ（ADR 0002 の 4）"
    )]
    fn unclip(&self) {
        // SAFETY: 上の PushAxisAlignedClip と対になる。
        unsafe {
            self.target.PopAxisAlignedClip();
        }
    }

    /// 矩形の中に 1 行。縦は中央、横は寄せに従う。カーソルがあれば選択を塗り、文字を描き、カーソルの線を置く。
    #[expect(
        unsafe_code,
        reason = "RS-010: Direct2D の DrawTextLayout を呼ぶ（ADR 0002 の 4）"
    )]
    fn text(
        &self,
        (format, palette): (&IDWriteTextFormat, &Palette),
        (rect, align, paint): (PixelRect, TextAlign, Paint),
        (text, caret): (&str, Option<TextCaret>),
    ) -> Result<(), Win32Failure> {
        let line = self.measure(format, text)?;
        let left = match align {
            TextAlign::Start => f32::from(rect.x()),
            TextAlign::Center => {
                f32::from(rect.x()) + (f32::from(rect.width()) - line.width()) / 2.0
            }
            TextAlign::End => f32::from(rect.x()) + f32::from(rect.width()) - line.width(),
        };
        let top = f32::from(rect.y()) + (f32::from(rect.height()) - line.height()) / 2.0;
        let top = (top + line.ascent()).floor() - line.ascent();
        if let Some(caret) = caret.filter(|caret| caret.has_selection()) {
            let start = left + prefix_advance(line.layout(), text, caret.start())?;
            let end = left + prefix_advance(line.layout(), text, caret.end())?;
            let selection = D2D_RECT_F {
                left: start,
                top,
                right: end,
                bottom: top + line.height(),
            };
            // SAFETY: 描画は paint の中で呼ばれる。selection はこの関数の局所。
            unsafe {
                self.target.FillRectangle(
                    &raw const selection,
                    &self.brush(palette, Paint::Selection)?,
                );
            }
        }
        let brush = self.brush(palette, paint)?;
        let origin = Vector2 { X: left, Y: top };
        // SAFETY: 描画は paint の中で呼ばれる。配置はこの関数の間生きている。
        unsafe {
            self.target
                .DrawTextLayout(origin, line.layout(), &brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
        }
        if let Some(caret) = caret.filter(|caret| caret.shows_bar()) {
            let x = (left + prefix_advance(line.layout(), text, caret.cursor())?).floor();
            let cursor_line = D2D_RECT_F {
                left: x,
                top,
                right: x + 1.0,
                bottom: top + line.height(),
            };
            // SAFETY: 描画は paint の中で呼ばれる。cursor_line はこの関数の局所。
            unsafe {
                self.target.FillRectangle(&raw const cursor_line, &brush);
            }
        }
        Ok(())
    }

    /// 字の形式（家族と大きさ）。
    #[expect(
        unsafe_code,
        reason = "RS-010: DirectWrite の CreateTextFormat を呼ぶ（ADR 0002 の 4）"
    )]
    fn format(&self, size: f32, language: Language) -> Result<IDWriteTextFormat, Win32Failure> {
        // SAFETY: 家族名と地域名は静的な文字列。
        let format = unsafe {
            self.words.CreateTextFormat(
                FONT_FAMILY,
                None,
                DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_FONT_STYLE_NORMAL,
                DWRITE_FONT_STRETCH_NORMAL,
                size,
                locale_name(language),
            )
        }?;
        Ok(format)
    }

    /// 1 行の字の配置と、その幅・高さ・ベースラインまでの高さ。
    #[expect(
        unsafe_code,
        reason = "RS-010: DirectWrite の文字の配置を作り、寸法を読む（ADR 0002 の 4）"
    )]
    fn measure(&self, format: &IDWriteTextFormat, text: &str) -> Result<TextLine, Win32Failure> {
        let wide: Vec<u16> = text.encode_utf16().collect();
        let mut metrics = DWRITE_TEXT_METRICS::default();
        let mut line = [DWRITE_LINE_METRICS::default()];
        let mut lines = 0_u32;
        // SAFETY: wide と出力はこの関数の局所。1 行。
        unsafe {
            let layout = self
                .words
                .CreateTextLayout(&wide, format, LAYOUT_LIMIT, LAYOUT_LIMIT)?;
            layout.GetMetrics(&raw mut metrics)?;
            layout.GetLineMetrics(Some(&mut line), &raw mut lines)?;
            let first = line.first().copied().unwrap_or_default();
            Ok(TextLine::new(
                layout,
                (
                    metrics.widthIncludingTrailingWhitespace,
                    first.height,
                    first.baseline,
                ),
            ))
        }
    }
}

/// 先頭から `chars` 文字分の幅。DirectWrite の位置は UTF-16 の単位なので、文字の添字を直す。
#[expect(
    unsafe_code,
    reason = "RS-010: DirectWrite の HitTestTextPosition を呼ぶ（ADR 0002 の 4）"
)]
fn prefix_advance(
    layout: &IDWriteTextLayout,
    text: &str,
    chars: usize,
) -> Result<f32, Win32Failure> {
    let units: usize = text.chars().take(chars).map(char::len_utf16).sum();
    let position = u32::try_from(units).map_err(|_error| Win32Failure::GeometryOutOfRange)?;
    let mut x = 0.0_f32;
    let mut y = 0.0_f32;
    let mut metrics = DWRITE_HIT_TEST_METRICS::default();
    // SAFETY: 出力はこの関数の局所。position は配置の中の UTF-16 の位置。
    unsafe {
        layout.HitTestTextPosition(position, false, &raw mut x, &raw mut y, &raw mut metrics)?;
    }
    Ok(x)
}

/// 整数座標の矩形を Direct2D の矩形へ。
fn rect_f(rect: PixelRect) -> D2D_RECT_F {
    D2D_RECT_F {
        left: f32::from(rect.x()),
        top: f32::from(rect.y()),
        right: f32::from(rect.x()) + f32::from(rect.width()),
        bottom: f32::from(rect.y()) + f32::from(rect.height()),
    }
}

/// 矩形の大きさを Direct2D の大きさへ。
fn size_u(bounds: PixelRect) -> D2D_SIZE_U {
    D2D_SIZE_U {
        width: u32::from(bounds.width()),
        height: u32::from(bounds.height()),
    }
}
