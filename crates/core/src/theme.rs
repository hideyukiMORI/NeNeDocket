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
}
