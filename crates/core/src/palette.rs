//! 台帳の 8 つの色の役割に、配色の色を当てた表。X11 と Windows の画面が同じ表を読む（ARC-001・ADR 0012）。

use crate::paint::Paint;
use crate::rgb::Rgb;

/// `Paint` ごとの色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    background: Rgb,
    surface: Rgb,
    border: Rgb,
    ink: Rgb,
    muted: Rgb,
    accent: Rgb,
    on_accent: Rgb,
    selection: Rgb,
}

impl Palette {
    /// 8 色から。並びは `Paint` の順。
    #[must_use]
    pub const fn new(colours: [Rgb; 8]) -> Self {
        let [
            background,
            surface,
            border,
            ink,
            muted,
            accent,
            on_accent,
            selection,
        ] = colours;
        Self {
            background,
            surface,
            border,
            ink,
            muted,
            accent,
            on_accent,
            selection,
        }
    }

    /// 色の意味から色へ。
    #[must_use]
    pub const fn colour(&self, paint: Paint) -> Rgb {
        match paint {
            Paint::Background => self.background,
            Paint::Surface => self.surface,
            Paint::Border => self.border,
            Paint::Ink => self.ink,
            Paint::Muted => self.muted,
            Paint::Accent => self.accent,
            Paint::OnAccent => self.on_accent,
            Paint::Selection => self.selection,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_meaning_reads_its_own_colour() {
        let colours = [
            Rgb::new(0, 0, 0),
            Rgb::new(1, 1, 1),
            Rgb::new(2, 2, 2),
            Rgb::new(3, 3, 3),
            Rgb::new(4, 4, 4),
            Rgb::new(5, 5, 5),
            Rgb::new(6, 6, 6),
            Rgb::new(7, 7, 7),
        ];
        let palette = Palette::new(colours);
        let paints = [
            Paint::Background,
            Paint::Surface,
            Paint::Border,
            Paint::Ink,
            Paint::Muted,
            Paint::Accent,
            Paint::OnAccent,
            Paint::Selection,
        ];
        for (paint, colour) in paints.into_iter().zip(colours) {
            assert_eq!(palette.colour(paint), colour);
        }
    }
}
