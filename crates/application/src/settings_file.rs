//! 設定ファイルの形（FR-012）。版 2: `version = 2`・`refresh_seconds = <秒>`・`language = system|<tag>`・`theme = system|<name>`・`recent = <フォルダ>`（新しいものが先、1 行に 1 つ）。
//!
//! 版 1（`language` と `theme` が無い）も読める。版 1 を読んだら言語と配色は `system` で補い、書くのは版 2 だけ（ARC-009 の移行）。
//! 形はここ（純関数）、読み書きはアダプタ。知らない版・知らない鍵・読めない値は `None` で、黙って既定値へ落とさない（FR-014）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::language_choice::LanguageChoice;
use nene_docket_core::theme_choice::ThemeChoice;

use crate::settings::Settings;

/// いま書く版。
const VERSION: &str = "2";
/// 読めるだけの前の版。
const PREVIOUS_VERSION: &str = "1";

/// 読み取り中の値。
struct Reading {
    /// 版 2 なら真、版 1 なら偽。
    current_version: Option<bool>,
    refresh_seconds: Option<u64>,
    language: Option<LanguageChoice>,
    theme: Option<ThemeChoice>,
    recent: Vec<String>,
}

/// 文字列から設定を読む（版 2 か版 1）。
#[must_use]
pub fn parse(text: &str) -> Option<Settings> {
    let mut reading = Reading {
        current_version: None,
        refresh_seconds: None,
        language: None,
        theme: None,
        recent: Vec::new(),
    };
    for line in text.lines() {
        let (key, value) = line.split_once('=')?;
        reading.take(key.trim(), value.trim())?;
    }
    reading.into_settings()
}

impl Reading {
    /// 1 行を取り込む。知らない鍵・読めない値は `None`。
    fn take(&mut self, key: &str, value: &str) -> Option<()> {
        match key {
            "version" => {
                self.current_version = Some(match value {
                    VERSION => true,
                    PREVIOUS_VERSION => false,
                    _ => return None,
                });
            }
            "refresh_seconds" => self.refresh_seconds = Some(value.parse::<u64>().ok()?),
            "language" => self.language = Some(LanguageChoice::from_keyword(value)?),
            "theme" => self.theme = Some(ThemeChoice::from_keyword(value)?),
            "recent" if value.is_empty() => return None,
            "recent" => self.recent.push(String::from(value)),
            _ => return None,
        }
        Some(())
    }

    /// 版ごとの必須の鍵が揃っていれば設定に。版 1 に言語や配色があれば知らない鍵として拒む。
    fn into_settings(self) -> Option<Settings> {
        let choices = if self.current_version? {
            (self.language?, self.theme?)
        } else if self.language.is_some() || self.theme.is_some() {
            return None;
        } else {
            (LanguageChoice::System, ThemeChoice::System)
        };
        Some(Settings::new(self.recent, self.refresh_seconds?, choices))
    }
}

/// 設定を文字列にする（版 2）。`parse` の逆。
#[must_use]
pub fn encode(settings: &Settings) -> String {
    let mut text = alloc::format!(
        "version = {VERSION}\nrefresh_seconds = {}\nlanguage = {}\ntheme = {}\n",
        settings.refresh_seconds(),
        settings.language().keyword(),
        settings.theme().keyword()
    );
    for folder in settings.recent() {
        text.push_str("recent = ");
        text.push_str(folder);
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use nene_docket_core::language::Language;

    use super::*;

    #[test]
    fn encode_then_parse_round_trips_in_order() {
        let settings = Settings::new(
            alloc::vec![String::from("/srv/team"), String::from("./docket")],
            5,
            (
                LanguageChoice::Fixed(Language::Japanese),
                ThemeChoice::from_keyword("dracula").unwrap(),
            ),
        );
        let text = encode(&settings);
        assert_eq!(
            text,
            "version = 2\nrefresh_seconds = 5\nlanguage = ja\ntheme = dracula\nrecent = /srv/team\nrecent = ./docket\n"
        );
        assert_eq!(parse(&text), Some(settings));
        assert_eq!(
            parse("version = 2\nrefresh_seconds = 3\nlanguage = system\ntheme = system\n"),
            Some(Settings::initial())
        );
    }

    #[test]
    fn version_one_is_read_with_system_choices_but_may_not_carry_them() {
        assert_eq!(
            parse("version = 1\nrefresh_seconds = 3\nrecent = /a\n"),
            Some(Settings::initial().opened("/a"))
        );
        assert_eq!(
            parse("version = 1\nrefresh_seconds = 3\nlanguage = ja\n"),
            None
        );
        assert_eq!(
            parse("version = 1\nrefresh_seconds = 3\ntheme = system\n"),
            None
        );
    }

    #[test]
    fn a_different_version_a_missing_key_or_an_unknown_key_is_refused() {
        for text in [
            "version = 3\nrefresh_seconds = 3\nlanguage = system\ntheme = system\n",
            "version = 2\nrefresh_seconds = 3\ntheme = system\n",
            "version = 2\nrefresh_seconds = 3\nlanguage = system\n",
            "version = 2\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolor = red\n",
            "refresh_seconds = 3\n",
            "garbage",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }

    #[test]
    fn a_bad_value_is_refused() {
        for text in [
            "version = 2\nrefresh_seconds = soon\nlanguage = system\ntheme = system\n",
            "version = 2\nrefresh_seconds = 3\nlanguage = fr\ntheme = system\n",
            "version = 2\nrefresh_seconds = 3\nlanguage = system\ntheme = no-such\n",
            "version = 2\nrefresh_seconds = 3\nlanguage = system\ntheme = system\nrecent =\n",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
