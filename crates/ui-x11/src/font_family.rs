//! 日本語を描けるフォントの家族を、起動時に 1 度だけ探す（FR-015）。
//!
//! cairo の toy text API は家族名を 1 つしか取らず、無い字形を別のフォントで補わない（Issue #5 の実測で豆腐になった）。
//! `sans-serif` に「日」が無ければ、よくある日本語の家族を順に試す。どれにも無ければ `sans-serif` のまま（豆腐を認めて記録する）。

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};

use crate::ui_failure::UiFailure;

/// 試す順。先頭は fontconfig の既定。
const CANDIDATES: [&str; 7] = [
    "sans-serif",
    "Noto Sans CJK JP",
    "IPAPGothic",
    "IPAGothic",
    "VL PGothic",
    "TakaoPGothic",
    "WenQuanYi Zen Hei",
];

/// 字形の有無を見る字。
const PROBE: &str = "日";

/// 「日」を描ける最初の家族。無ければ `sans-serif`。
///
/// # Errors
///
/// cairo が surface を作れない、または字形の問い合わせに失敗した。
pub fn choose() -> Result<&'static str, UiFailure> {
    let surface = ImageSurface::create(Format::Rgb24, 8, 8)?;
    let context = Context::new(&surface)?;
    context.set_font_size(16.0_f64);
    for family in CANDIDATES {
        if draws(&context, family)? {
            return Ok(family);
        }
    }
    Ok(CANDIDATES.first().copied().unwrap_or("sans-serif"))
}

/// その家族で「日」の字形があるか。字形の番号 0 は `.notdef`（箱）なので無いとみなす。
fn draws(context: &Context, family: &str) -> Result<bool, UiFailure> {
    context.select_font_face(family, FontSlant::Normal, FontWeight::Normal);
    let (glyphs, _clusters) = context
        .scaled_font()
        .text_to_glyphs(0.0_f64, 0.0_f64, PROBE)?;
    Ok(glyphs.iter().all(|glyph| glyph.index() != 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chooses_a_family_that_draws_japanese_when_one_is_installed() {
        // このコンテナには IPA ゴシックがある。無い環境では sans-serif に落ちるので、結果は候補のどれか。
        let family = choose().unwrap();
        assert!(CANDIDATES.contains(&family));
        let surface = ImageSurface::create(Format::Rgb24, 8, 8).unwrap();
        let context = Context::new(&surface).unwrap();
        context.set_font_size(16.0_f64);
        assert!(draws(&context, family).unwrap() || family == "sans-serif");
    }

    #[test]
    fn a_family_without_the_glyph_is_told_apart_from_one_with_it() {
        let surface = ImageSurface::create(Format::Rgb24, 8, 8).unwrap();
        let context = Context::new(&surface).unwrap();
        context.set_font_size(16.0_f64);
        // DejaVu Sans（このコンテナの sans-serif の既定）には「日」が無い。
        assert!(!draws(&context, "DejaVu Sans").unwrap());
        assert!(draws(&context, "IPAGothic").unwrap());
    }
}
