//! 起動時に一度だけ取り込む環境の事実（ARC-004 の表）。dpi・OS の言語・OS の明暗。アダプタが観測し、合成ルートが渡す。

use nene_docket_core::appearance::Appearance;
use nene_docket_core::language::Language;
use nene_docket_core::ui_scale::UiScale;

/// デスクトップの dpi（X11 は `Xft.dpi`、Windows はモニタの dpi。無ければ 1 倍）・OS の言語・OS の明暗。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    desktop_dpi: Option<u32>,
    language: Option<Language>,
    appearance: Option<Appearance>,
}

impl Environment {
    /// デスクトップの dpi から作る。分からなければ `None`。言語と明暗は分からない。
    #[must_use]
    pub const fn new(desktop_dpi: Option<u32>) -> Self {
        Self {
            desktop_dpi,
            language: None,
            appearance: None,
        }
    }

    /// OS の言語を添える（`system` の解決に使う・ADR 0011）。
    #[must_use]
    pub const fn with_language(self, language: Option<Language>) -> Self {
        Self { language, ..self }
    }

    /// OS の明暗を添える（`system` の解決に使う・ADR 0012）。
    #[must_use]
    pub const fn with_appearance(self, appearance: Option<Appearance>) -> Self {
        Self { appearance, ..self }
    }

    /// UI の倍率（FR-016）。
    #[must_use]
    pub fn scale(self) -> UiScale {
        UiScale::choose(self.desktop_dpi)
    }

    /// デスクトップの dpi。
    #[must_use]
    pub const fn desktop_dpi(self) -> Option<u32> {
        self.desktop_dpi
    }

    /// OS の言語。
    #[must_use]
    pub const fn language(self) -> Option<Language> {
        self.language
    }

    /// OS の明暗。
    #[must_use]
    pub const fn appearance(self) -> Option<Appearance> {
        self.appearance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_follows_the_dpi_and_observations_ride_along() {
        assert_eq!(Environment::new(None).scale(), UiScale::Single);
        assert_eq!(Environment::new(Some(192)).scale(), UiScale::Double);
        assert_eq!(Environment::new(Some(192)).desktop_dpi(), Some(192));
        let observed = Environment::new(None)
            .with_language(Some(Language::Korean))
            .with_appearance(Some(Appearance::Dark));
        assert_eq!(observed.language(), Some(Language::Korean));
        assert_eq!(observed.appearance(), Some(Appearance::Dark));
        assert_eq!(Environment::new(None).language(), None);
    }
}
