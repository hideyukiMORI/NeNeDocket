//! 設定に残す配色の希望（ADR 0012）。`system`（OS の明暗に従う）か、組み込みの名前。表示する配色は観測した明暗と合わせて解決する。

use crate::appearance::Appearance;
use crate::theme::Theme;
use crate::theme_catalog;
use crate::theme_seed::ThemeSeed;

/// `system` の語。
const SYSTEM: &str = "system";

/// OS に従うか、名前で決めるか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeChoice {
    /// OS の明暗に従い、その既定の配色を使う。
    System,
    /// 組み込みの配色（表の中の名前）。
    Named(&'static ThemeSeed),
}

impl ThemeChoice {
    /// 設定の語から。`system` か組み込みの名前。知らない名前は `None`（黙って既定値へ落とさない）。
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        if keyword == SYSTEM {
            return Some(Self::System);
        }
        theme_catalog::named(keyword).map(Self::Named)
    }

    /// 設定に書く語。
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::System => SYSTEM,
            Self::Named(seed) => seed.name(),
        }
    }

    /// 観測した OS の明暗（分からなければライト）と合わせて、表示する配色にする。
    #[must_use]
    pub fn resolve(self, observed: Option<Appearance>) -> Theme {
        let seed = match self {
            Self::System => theme_catalog::default_for(observed.unwrap_or(Appearance::Light)),
            Self::Named(seed) => seed,
        };
        Theme::of(seed)
    }

    /// 表の次の配色（`system` からは先頭）。端で巡る。
    #[must_use]
    pub fn next(self) -> Self {
        let index = match self {
            Self::System => 0,
            Self::Named(seed) => {
                theme_catalog::position_of(seed.name()).map_or(0, |i| i.saturating_add(1))
            }
        };
        Self::Named(theme_catalog::at(index))
    }

    /// 表の前の配色（`system` からは末尾）。端で巡る。
    #[must_use]
    pub fn previous(self) -> Self {
        let index = match self {
            Self::System => theme_catalog::COUNT.saturating_sub(1),
            Self::Named(seed) => theme_catalog::position_of(seed.name()).map_or(0, |i| {
                i.checked_sub(1)
                    .unwrap_or(theme_catalog::COUNT.saturating_sub(1))
            }),
        };
        Self::Named(theme_catalog::at(index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_round_trip_and_system_follows_the_observed_appearance() {
        assert_eq!(
            ThemeChoice::from_keyword("system"),
            Some(ThemeChoice::System)
        );
        let dracula = ThemeChoice::from_keyword("dracula").unwrap();
        assert_eq!(dracula.keyword(), "dracula");
        assert_eq!(
            ThemeChoice::from_keyword("one_dark").map(ThemeChoice::keyword),
            Some("one-dark")
        );
        assert_eq!(ThemeChoice::from_keyword("Dracula"), None);
        assert_eq!(
            ThemeChoice::System.resolve(Some(Appearance::Dark)).name(),
            "ubuntu-aubergine"
        );
        assert_eq!(ThemeChoice::System.resolve(None).name(), "neutral-light");
        assert_eq!(dracula.resolve(Some(Appearance::Light)).name(), "dracula");
    }

    #[test]
    fn next_and_previous_walk_the_table_and_wrap() {
        let first = ThemeChoice::System.next();
        assert_eq!(first.keyword(), "ubuntu-aubergine");
        assert_eq!(
            first.previous().keyword(),
            theme_catalog::at(theme_catalog::COUNT - 1).name()
        );
        assert_eq!(
            ThemeChoice::System.previous().keyword(),
            theme_catalog::at(theme_catalog::COUNT - 1).name()
        );
        assert_eq!(first.next().previous(), first);
        let last = ThemeChoice::Named(theme_catalog::at(theme_catalog::COUNT - 1));
        assert_eq!(last.next(), first);
    }
}
