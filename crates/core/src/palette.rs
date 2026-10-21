//! 色の意味（`Paint`）を色にする。いまはライトの見えだけ（FR-012 の見えは別の Issue）。
//! X11 と Windows の画面が同じ表を使うので、見た目の色は 1 か所から出る（ARC-001）。

use crate::paint::Paint;
use crate::rgb::Rgb;

/// ライトの見えの色。
#[must_use]
pub const fn light(paint: Paint) -> Rgb {
    match paint {
        Paint::Background => Rgb::new(0xF5, 0xF5, 0xF5),
        Paint::Surface | Paint::OnAccent => Rgb::new(0xFF, 0xFF, 0xFF),
        Paint::Border => Rgb::new(0xC0, 0xC0, 0xC0),
        Paint::Ink => Rgb::new(0x21, 0x21, 0x21),
        Paint::Muted => Rgb::new(0x75, 0x75, 0x75),
        Paint::Accent => Rgb::new(0x1E, 0x88, 0xE5),
        Paint::Selection => Rgb::new(0xD6, 0xE9, 0xFA),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_meaning_has_a_colour_and_surface_matches_on_accent() {
        assert_eq!(light(Paint::Ink), Rgb::new(0x21, 0x21, 0x21));
        assert_eq!(light(Paint::Surface), light(Paint::OnAccent));
        assert_ne!(light(Paint::Background), light(Paint::Surface));
        assert_eq!(light(Paint::Accent).blue(), 0xE5);
    }
}
