//! 解決した配色: 名前・明暗・台帳の 8 色（ADR 0012）。表示フレームが持ち、画面はこれで描く。

use crate::appearance::Appearance;
use crate::palette::Palette;
use crate::theme_derivation;
use crate::theme_seed::ThemeSeed;

/// 名前と明暗と色の表。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    name: &'static str,
    appearance: Appearance,
    palette: Palette,
}

impl Theme {
    /// 組み込みの配色の元から導く。
    #[must_use]
    pub fn of(seed: &ThemeSeed) -> Self {
        Self {
            name: seed.name(),
            appearance: seed.appearance(),
            palette: theme_derivation::derive(
                (seed.background(), seed.foreground(), seed.accent()),
                seed.appearance(),
            ),
        }
    }

    /// 名前。
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// 明暗。
    #[must_use]
    pub const fn appearance(&self) -> Appearance {
        self.appearance
    }

    /// 色の表。
    #[must_use]
    pub const fn palette(&self) -> Palette {
        self.palette
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::Paint;
    use crate::rgb::Rgb;
    use crate::theme_catalog;

    /// WCAG 2 の相対輝度（テストだけ浮動小数を使う）。
    fn luminance(rgb: Rgb) -> f64 {
        let linear = |channel: u8| {
            let unit = f64::from(channel) / 255.0_f64;
            if unit <= 0.040_45_f64 {
                unit / 12.92_f64
            } else {
                ((unit + 0.055_f64) / 1.055_f64).powf(2.4_f64)
            }
        };
        0.2126_f64 * linear(rgb.red())
            + 0.7152_f64 * linear(rgb.green())
            + 0.0722_f64 * linear(rgb.blue())
    }

    fn contrast(a: Rgb, b: Rgb) -> f64 {
        let (lighter, darker) = (
            luminance(a).max(luminance(b)),
            luminance(a).min(luminance(b)),
        );
        (lighter + 0.05_f64) / (darker + 0.05_f64)
    }

    #[test]
    fn every_builtin_theme_keeps_ink_readable_on_the_background_and_the_surface() {
        for seed in &theme_catalog::SEEDS {
            let theme = Theme::of(seed);
            let palette = theme.palette();
            let on_background = contrast(
                palette.colour(Paint::Ink),
                palette.colour(Paint::Background),
            );
            let on_surface = contrast(palette.colour(Paint::Ink), palette.colour(Paint::Surface));
            assert!(on_background >= 4.5_f64, "{}: {on_background}", seed.name());
            assert!(on_surface >= 4.0_f64, "{}: {on_surface}", seed.name());
            assert_eq!(theme.name(), seed.name());
            assert_eq!(theme.appearance(), seed.appearance());
        }
    }

    /// ADR 0025 の下限: 部品の縁は地と面に 3:1、選びの面の上の前景は 4.5:1、灰色の字は地・面・選びの面に 4.5:1
    /// （前景そのものが届かない配色では前景と同じ）。整数の計算とは別に、浮動小数の式で確かめる。
    #[test]
    fn every_builtin_theme_meets_the_contrast_floors() {
        for seed in &theme_catalog::SEEDS {
            let palette = Theme::of(seed).palette();
            let colour = |paint| palette.colour(paint);
            let name = seed.name();
            for under in [Paint::Background, Paint::Surface] {
                let ratio = contrast(colour(Paint::Control), colour(under));
                assert!(ratio >= 3.0_f64, "{name}: control on {under:?} {ratio}");
            }
            let ink = contrast(colour(Paint::Ink), colour(Paint::Selection));
            assert!(ink >= 4.5_f64, "{name}: ink on selection {ink}");
            let muted_fits = [Paint::Background, Paint::Surface, Paint::Selection]
                .into_iter()
                .all(|under| contrast(colour(Paint::Muted), colour(under)) >= 4.5_f64);
            assert!(
                muted_fits || colour(Paint::Muted) == colour(Paint::Ink),
                "{name}: muted"
            );
        }
    }
}
