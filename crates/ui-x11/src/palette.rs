//! 色の意味（`Paint`）を色にする。いまはライトの見えだけ（FR-012 の見えは次の Issue）。

use nene_docket_core::paint::Paint;

/// 0.0〜1.0 の RGB。cairo の `set_source_rgb` にそのまま渡す。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    /// 赤。
    pub red: f64,
    /// 緑。
    pub green: f64,
    /// 青。
    pub blue: f64,
}

/// 8 ビットの 3 色から。
fn rgb(red: u8, green: u8, blue: u8) -> Rgb {
    Rgb {
        red: f64::from(red) / 255.0_f64,
        green: f64::from(green) / 255.0_f64,
        blue: f64::from(blue) / 255.0_f64,
    }
}

/// ライトの見えの色。
#[must_use]
pub fn light(paint: Paint) -> Rgb {
    match paint {
        Paint::Background => rgb(0xF5, 0xF5, 0xF5),
        Paint::Surface | Paint::OnAccent => rgb(0xFF, 0xFF, 0xFF),
        Paint::Border => rgb(0xC0, 0xC0, 0xC0),
        Paint::Ink => rgb(0x21, 0x21, 0x21),
        Paint::Muted => rgb(0x75, 0x75, 0x75),
        Paint::Accent => rgb(0x1E, 0x88, 0xE5),
        Paint::Selection => rgb(0xD6, 0xE9, 0xFA),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_are_scaled_to_unit() {
        let ink = light(Paint::Ink);
        assert!((ink.red - f64::from(0x21_u8) / 255.0_f64).abs() < 1e-9_f64);
        assert!((light(Paint::Accent).blue - f64::from(0xE5_u8) / 255.0_f64).abs() < 1e-9_f64);
        assert_eq!(light(Paint::Surface), light(Paint::OnAccent));
        assert!(light(Paint::Background).red < 1.0_f64);
    }
}
