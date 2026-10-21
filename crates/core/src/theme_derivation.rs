//! 本文の 3 色（背景・前景・強調）と明暗から、台帳の 8 つの色を整数の補間だけで導く（ADR 0012。nene-nib ADR 0017 の `derive_ui` と同じ形）。
//!
//! sRGB の値のまま線形に混ぜる（ガンマは扱わない）。百分率の表は明暗ごとに 1 つ。決定的で、浮動小数も乱数も使わない。

use crate::appearance::Appearance;
use crate::palette::Palette;
use crate::rgb::Rgb;

/// 明暗ごとの百分率。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Weights {
    /// 面: 背景 → 前景へ（ダーク）／背景 → 白へ（ライト）。
    surface: u8,
    /// 縁: 背景 → 前景へ。
    border: u8,
    /// 控えめな字: 前景 → 背景へ。
    muted: u8,
    /// 選択の背景: 背景 → 強調へ。
    selection: u8,
}

const DARK: Weights = Weights {
    surface: 6,
    border: 22,
    muted: 30,
    selection: 28,
};

const LIGHT: Weights = Weights {
    surface: 100,
    border: 15,
    muted: 30,
    selection: 22,
};

const WHITE: Rgb = Rgb::new(0xFF, 0xFF, 0xFF);
const BLACK: Rgb = Rgb::new(0x00, 0x00, 0x00);
/// 強調の明るさがこれ以上なら、その上の字は黒。
const BRIGHT_ACCENT: u32 = 160;

/// 台帳の 8 色。
#[must_use]
pub fn derive(
    (background, foreground, accent): (Rgb, Rgb, Rgb),
    appearance: Appearance,
) -> Palette {
    let weights = match appearance {
        Appearance::Dark => DARK,
        Appearance::Light => LIGHT,
    };
    let surface = match appearance {
        Appearance::Dark => mix(background, foreground, weights.surface),
        Appearance::Light => mix(background, WHITE, weights.surface),
    };
    Palette::new([
        background,
        surface,
        mix(background, foreground, weights.border),
        foreground,
        mix(foreground, background, weights.muted),
        accent,
        on_accent(accent),
        mix(background, accent, weights.selection),
    ])
}

/// `from` から `to` へ `percent` だけ寄せる（0 で `from`、100 で `to`）。
fn mix(from: Rgb, to: Rgb, percent: u8) -> Rgb {
    Rgb::new(
        channel(from.red(), to.red(), percent),
        channel(from.green(), to.green(), percent),
        channel(from.blue(), to.blue(), percent),
    )
}

fn channel(from: u8, to: u8, percent: u8) -> u8 {
    let percent = u32::from(percent.min(100));
    let mixed = (u32::from(from) * (100 - percent) + u32::from(to) * percent + 50) / 100;
    u8::try_from(mixed).unwrap_or(u8::MAX)
}

/// 強調の上に置く字。明るさ `(299R + 587G + 114B) / 1000` が 160 以上なら黒、未満なら白。
fn on_accent(accent: Rgb) -> Rgb {
    let brightness = (299 * u32::from(accent.red())
        + 587 * u32::from(accent.green())
        + 114 * u32::from(accent.blue()))
        / 1000;
    if brightness >= BRIGHT_ACCENT {
        BLACK
    } else {
        WHITE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::Paint;

    #[test]
    fn mix_is_linear_and_clamped() {
        let black = Rgb::new(0, 0, 0);
        assert_eq!(mix(black, WHITE, 0), black);
        assert_eq!(mix(black, WHITE, 100), WHITE);
        assert_eq!(mix(black, WHITE, 50), Rgb::new(128, 128, 128));
        assert_eq!(mix(black, WHITE, 200), WHITE);
    }

    #[test]
    fn dark_and_light_derive_their_surfaces_and_readable_accent_text() {
        let dark = derive(
            (
                Rgb::new(0x30, 0x0A, 0x24),
                Rgb::new(0xEE, 0xEE, 0xEC),
                Rgb::new(0xE9, 0x54, 0x20),
            ),
            Appearance::Dark,
        );
        assert_eq!(dark.colour(Paint::Background), Rgb::new(0x30, 0x0A, 0x24));
        assert_eq!(dark.colour(Paint::Ink), Rgb::new(0xEE, 0xEE, 0xEC));
        assert_eq!(dark.colour(Paint::Surface), Rgb::new(0x3B, 0x18, 0x30));
        assert_eq!(dark.colour(Paint::OnAccent), WHITE);
        let light = derive(
            (
                Rgb::new(0xF5, 0xF5, 0xF5),
                Rgb::new(0x21, 0x21, 0x21),
                Rgb::new(0xFF, 0xD7, 0x00),
            ),
            Appearance::Light,
        );
        assert_eq!(light.colour(Paint::Surface), WHITE);
        assert_eq!(light.colour(Paint::OnAccent), BLACK);
        assert_ne!(
            light.colour(Paint::Selection),
            light.colour(Paint::Background)
        );
    }
}
