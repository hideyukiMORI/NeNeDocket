//! 本文の 3 色（背景・前景・強調）と明暗から、台帳の 9 つの色を整数の補間だけで導く（ADR 0012。nene-nib ADR 0017 の `derive_ui` と同じ形）。
//!
//! sRGB の値のまま線形に混ぜる（ガンマは扱わない）。百分率の表は明暗ごとに 1 つ。決定的で、浮動小数も乱数も使わない。

use crate::appearance::Appearance;
use crate::contrast;
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

/// 台帳の 9 色。縁・選びの面・灰色の字は下限（ADR 0025）を満たすまで百分率を 1 ずつ動かす。
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
    // 部品の縁: 縁の量から前景へ寄せ、地と面の両方に 3:1。
    let control = first_fit(
        (weights.border..=100).map(|percent| mix(background, foreground, percent)),
        |colour| {
            contrast::at_least(colour, background, contrast::NON_TEXT)
                && contrast::at_least(colour, surface, contrast::NON_TEXT)
        },
    )
    .unwrap_or(foreground);
    // 選びの面と灰色の字は一緒に決める: 選びの面を強調の量から地へ戻しながら、灰色の字が前景から離れたまま
    // 地・面・選びの面に 4.5:1 を取れる最初の組（選びの面の上の前景も 4.5:1）。取れなければ灰色の字は前景。
    let (selection, muted) = (0..=weights.selection)
        .rev()
        .map(|percent| mix(background, accent, percent))
        .filter(|selection| contrast::at_least(foreground, *selection, contrast::TEXT))
        .find_map(|selection| {
            muted_on(
                [background, surface, selection],
                (foreground, background),
                weights.muted,
            )
            .map(|muted| (selection, muted))
        })
        .unwrap_or_else(|| {
            let selection = first_fit(
                (0..=weights.selection)
                    .rev()
                    .map(|percent| mix(background, accent, percent)),
                |colour| contrast::at_least(foreground, colour, contrast::TEXT),
            )
            .unwrap_or(background);
            (selection, foreground)
        });
    Palette::new([
        background,
        surface,
        mix(background, foreground, weights.border),
        control,
        foreground,
        muted,
        accent,
        on_accent(accent),
        selection,
    ])
}

/// 灰色の字: 前景から地へ `most` から 1 ずつ離す量を減らし、`unders` の全部に 4.5:1 を取れる最初の色。
/// 前景そのもの（0）は灰色にならないので候補にしない。
fn muted_on(unders: [Rgb; 3], (foreground, background): (Rgb, Rgb), most: u8) -> Option<Rgb> {
    first_fit(
        (1..=most)
            .rev()
            .map(|percent| mix(foreground, background, percent)),
        |colour| {
            unders
                .into_iter()
                .all(|under| contrast::at_least(colour, under, contrast::TEXT))
        },
    )
}

/// 候補を順に見て、初めて `fits` を満たす色。
fn first_fit<I: Iterator<Item = Rgb>, F: Fn(Rgb) -> bool>(candidates: I, fits: F) -> Option<Rgb> {
    let mut candidates = candidates;
    candidates.find(|colour| fits(*colour))
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
