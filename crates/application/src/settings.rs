//! 再起動をまたいで残す設定（FR-012）。最近開いた台帳と、他人のジャーナルを読み直す間隔。

use alloc::string::String;
use alloc::vec::Vec;

/// 最近の台帳を覚える数。
const RECENT_LIMIT: usize = 10;
/// 読み直しの間隔の既定（秒・ADR 0004 の 8）。
const DEFAULT_REFRESH_SECONDS: u64 = 3;

/// 最近開いた台帳のフォルダ（新しいものが先）と、読み直しの間隔（秒）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    recent: Vec<String>,
    refresh_seconds: u64,
}

impl Settings {
    /// 何も開いたことが無く、間隔は既定。
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            recent: Vec::new(),
            refresh_seconds: DEFAULT_REFRESH_SECONDS,
        }
    }

    /// 値から。最近の台帳は 10 件に切り、間隔は 1 秒以上に丸める。
    #[must_use]
    pub fn new(recent: Vec<String>, refresh_seconds: u64) -> Self {
        let mut settings = Self::initial();
        settings.refresh_seconds = refresh_seconds.max(1);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opened_folders_go_first_without_duplicates_and_are_capped() {
        let mut settings = Settings::initial();
        assert_eq!(settings.latest(), None);
        assert_eq!(settings.refresh_seconds(), 3);
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
    fn new_keeps_the_order_and_floors_the_interval() {
        let settings = Settings::new(
            alloc::vec![String::from("a"), String::from("b"), String::from("a")],
            0,
        );
        assert_eq!(settings.recent(), ["a", "b"]);
        assert_eq!(settings.refresh_seconds(), 1);
    }
}
