//! 表示言語の字を描けるフォントの家族を探す（FR-015 / FR-019・ADR 0007 / 0011）。
//!
//! cairo の toy text API は家族名を 1 つしか取らず、無い字形を別のフォントで補わない（Issue #5 の実測で豆腐になった）。
//! 言語ごとの候補を順に試し、その言語の確かめる字（`Language::probe`）を描ける最初の家族を選ぶ。どれにも無ければ `sans-serif`（豆腐を認めて記録する）。

use cairo::{Context, FontSlant, FontWeight, Format, ImageSurface};
use nene_docket_core::language::Language;

use crate::ui_failure::UiFailure;

/// fontconfig の既定。どの言語でも最初に試し、無ければ最後の砦。
const DEFAULT: &str = "sans-serif";

/// 言語ごとの候補（既定の次に試す順）。
const fn candidates(language: Language) -> &'static [&'static str] {
    match language {
        Language::Japanese => &[
            "Noto Sans CJK JP",
            "IPAPGothic",
            "IPAGothic",
            "VL PGothic",
            "TakaoPGothic",
            "WenQuanYi Zen Hei",
        ],
        Language::SimplifiedChinese => &[
            "Noto Sans CJK SC",
            "Noto Sans SC",
            "WenQuanYi Zen Hei",
            "WenQuanYi Micro Hei",
            "Source Han Sans SC",
        ],
        Language::TraditionalChinese => &[
            "Noto Sans CJK TC",
            "Noto Sans TC",
            "WenQuanYi Zen Hei",
            "WenQuanYi Micro Hei",
            "Source Han Sans TC",
        ],
        Language::Korean => &[
            "Noto Sans CJK KR",
            "Noto Sans KR",
            "NanumGothic",
            "UnDotum",
            "Baekmuk Gulim",
            "WenQuanYi Zen Hei",
        ],
        Language::English | Language::Spanish => &["DejaVu Sans", "Liberation Sans"],
    }
}

/// `language` の確かめる字を描ける最初の家族。無ければ `sans-serif`。
///
/// # Errors
///
/// cairo が surface を作れない、または字形の問い合わせに失敗した。
pub fn choose(language: Language) -> Result<&'static str, UiFailure> {
    let surface = ImageSurface::create(Format::Rgb24, 8, 8)?;
    let context = Context::new(&surface)?;
    context.set_font_size(16.0_f64);
    if draws(&context, DEFAULT, language.probe())? {
        return Ok(DEFAULT);
    }
    for family in candidates(language) {
        if draws(&context, family, language.probe())? {
            return Ok(family);
        }
    }
    Ok(DEFAULT)
}

/// その家族で `probe` の字形があるか。字形の番号 0 は `.notdef`（箱）なので無いとみなす。
fn draws(context: &Context, family: &str, probe: &str) -> Result<bool, UiFailure> {
    context.select_font_face(family, FontSlant::Normal, FontWeight::Normal);
    let (glyphs, _clusters) = context
        .scaled_font()
        .text_to_glyphs(0.0_f64, 0.0_f64, probe)?;
    Ok(glyphs.iter().all(|glyph| glyph.index() != 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_language_gets_a_family_from_its_candidates_or_the_default() {
        // このコンテナには IPA ゴシックと文泉驛正黑がある。無い環境では sans-serif に落ちるので、結果は候補のどれか。
        for language in Language::ALL {
            let family = choose(language).unwrap();
            assert!(
                family == DEFAULT || candidates(language).contains(&family),
                "{language:?}: {family}"
            );
        }
    }

    #[test]
    fn a_family_that_lacks_the_glyph_is_rejected() {
        let surface = ImageSurface::create(Format::Rgb24, 8, 8).unwrap();
        let context = Context::new(&surface).unwrap();
        assert!(draws(&context, DEFAULT, "A").unwrap());
        assert!(!draws(&context, "DejaVu Sans", "한").unwrap());
    }
}
