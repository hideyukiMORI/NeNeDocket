//! 再起動をまたいで残す設定（FR-012）。最近開いた台帳・他人のジャーナルを読み直す間隔・表示言語・配色・表の列の幅と順・段の高さ・時刻の見せ方。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::language_choice::LanguageChoice;
use nene_docket_core::theme_choice::ThemeChoice;

use crate::clock_choice::ClockChoice;
use crate::column_order::ColumnOrder;
use crate::column_widths::ColumnWidths;
use crate::pane_height::PaneHeight;

/// 最近の台帳を覚える数。
const RECENT_LIMIT: usize = 10;
/// 読み直しの間隔の既定（秒・ADR 0004 の 8）。
const DEFAULT_REFRESH_SECONDS: u64 = 3;

/// 最近開いた台帳のフォルダ（新しいものが先）・読み直しの間隔（秒）・表示言語の希望・配色の希望・列の幅。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    recent: Vec<String>,
    refresh_seconds: u64,
    language: LanguageChoice,
    theme: ThemeChoice,
    columns: ColumnWidths,
    order: ColumnOrder,
    pane: PaneHeight,
    clock: ClockChoice,
}

impl Settings {
    /// 何も開いたことが無く、間隔は既定、言語と配色は OS に従う。
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            recent: Vec::new(),
            refresh_seconds: DEFAULT_REFRESH_SECONDS,
            language: LanguageChoice::System,
            theme: ThemeChoice::System,
            columns: ColumnWidths::initial(),
            order: ColumnOrder::initial(),
            pane: PaneHeight::Auto,
            clock: ClockChoice::Local,
        }
    }

    /// 値から。最近の台帳は 10 件に切り、間隔は 1 秒以上に丸める。
    #[must_use]
    pub fn new(
        recent: Vec<String>,
        refresh_seconds: u64,
        (language, theme): (LanguageChoice, ThemeChoice),
    ) -> Self {
        let mut settings = Self::initial();
        settings.refresh_seconds = refresh_seconds.max(1);
        settings.language = language;
        settings.theme = theme;
        for folder in recent.into_iter().rev() {
            settings = settings.opened(&folder);
        }
        settings
    }

    /// 台帳を開いた後の設定。先頭に置き、同じフォルダは 1 つにし、10 件に切る。
    #[must_use]
    pub fn opened(mut self, folder: &str) -> Self {
        self.recent.retain(|known| known != folder);
        self.recent.insert(0, String::from(folder));
        self.recent.truncate(RECENT_LIMIT);
        self
    }

    /// 表示言語の希望を変えた設定。
    #[must_use]
    pub fn with_language(self, language: LanguageChoice) -> Self {
        Self { language, ..self }
    }

    /// 配色の希望を変えた設定。
    #[must_use]
    pub fn with_theme(self, theme: ThemeChoice) -> Self {
        Self { theme, ..self }
    }

    /// 列の幅を変えた設定。
    #[must_use]
    pub fn with_columns(self, columns: ColumnWidths) -> Self {
        Self { columns, ..self }
    }

    /// 列の幅。
    #[must_use]
    pub const fn columns(&self) -> ColumnWidths {
        self.columns
    }

    /// 列の順を変えた設定。
    #[must_use]
    pub fn with_order(self, order: ColumnOrder) -> Self {
        Self { order, ..self }
    }

    /// 列の順。
    #[must_use]
    pub const fn order(&self) -> ColumnOrder {
        self.order
    }

    /// 表の下の段の高さを変えた設定。
    #[must_use]
    pub fn with_pane(self, pane: PaneHeight) -> Self {
        Self { pane, ..self }
    }

    /// 表の下の段の高さ。
    #[must_use]
    pub const fn pane(&self) -> PaneHeight {
        self.pane
    }

    /// 時刻の見せ方を変えた設定（Issue #78）。
    #[must_use]
    pub fn with_clock(self, clock: ClockChoice) -> Self {
        Self { clock, ..self }
    }

    /// 時刻の見せ方（地方時か UTC か）。
    #[must_use]
    pub const fn clock(&self) -> ClockChoice {
        self.clock
    }

    /// 最近開いた台帳。新しいものが先。
    #[must_use]
    pub fn recent(&self) -> &[String] {
        &self.recent
    }

    /// いちばん最近の台帳。
    #[must_use]
    pub fn latest(&self) -> Option<&str> {
        self.recent.first().map(String::as_str)
    }

    /// 読み直しの間隔（秒）。
    #[must_use]
    pub const fn refresh_seconds(&self) -> u64 {
        self.refresh_seconds
    }

    /// 表示言語の希望。
    #[must_use]
    pub const fn language(&self) -> LanguageChoice {
        self.language
    }

    /// 配色の希望。
    #[must_use]
    pub const fn theme(&self) -> ThemeChoice {
        self.theme
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::language::Language;

    use super::*;

    #[test]
    fn opened_folders_go_first_without_duplicates_and_are_capped() {
        let mut settings = Settings::initial();
        assert_eq!(settings.latest(), None);
        assert_eq!(settings.refresh_seconds(), 3);
        assert_eq!(settings.language(), LanguageChoice::System);
        assert_eq!(settings.theme(), ThemeChoice::System);
        for number in 0..12_u32 {
            settings = settings.opened(&alloc::format!("/d/{number}"));
        }
        settings = settings.opened("/d/5");
        assert_eq!(settings.latest(), Some("/d/5"));
        assert_eq!(settings.recent().len(), 10);
        assert_eq!(settings.recent().iter().filter(|f| *f == "/d/5").count(), 1);
        assert!(!settings.recent().iter().any(|f| f == "/d/0"));
    }

    #[test]
    fn new_keeps_the_order_floors_the_interval_and_carries_the_choices() {
        let dracula = ThemeChoice::from_keyword("dracula").unwrap();
        let settings = Settings::new(
            alloc::vec![String::from("a"), String::from("b"), String::from("a")],
            0,
            (LanguageChoice::Fixed(Language::Korean), dracula),
        );
        assert_eq!(settings.recent(), ["a", "b"]);
        assert_eq!(settings.refresh_seconds(), 1);
        assert_eq!(settings.language(), LanguageChoice::Fixed(Language::Korean));
        assert_eq!(settings.theme(), dracula);
        assert_eq!(settings.columns(), ColumnWidths::initial());
    }

    #[test]
    fn the_order_rides_along_and_can_be_changed() {
        let settings = Settings::initial();
        assert_eq!(settings.order(), ColumnOrder::initial());
        assert_eq!(
            settings
                .clone()
                .with_order(
                    ColumnOrder::initial().moved_to(crate::table_column::TableColumn::Due, 0)
                )
                .order()
                .columns()[0],
            crate::table_column::TableColumn::Due
        );
        assert_eq!(
            settings
                .clone()
                .with_columns(
                    ColumnWidths::initial().with(crate::table_column::TableColumn::Due, 200)
                )
                .columns()
                .get(crate::table_column::TableColumn::Due),
            Some(200)
        );
        let changed = settings
            .with_language(LanguageChoice::System)
            .with_theme(ThemeChoice::System);
        assert_eq!(changed.language(), LanguageChoice::System);
        assert_eq!(changed.theme(), ThemeChoice::System);
    }
}
