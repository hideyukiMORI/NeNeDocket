//! 設定ファイルの形（FR-012）。版 5: `version = 5`・`refresh_seconds = <秒>`・`language = system|<tag>`・`theme = system|<name>`・
//! `columns = <番号>,<状態>,<担当>,<期限>,<ラベル>`（論理画素）・`order = <列>,…`（6 列の並び）・`pane_px = auto|<高さ>`（表の下の段）・
//! `recent = <フォルダ>`（新しいものが先、1 行に 1 つ）。
//!
//! 版 4（`pane_px` が無い）・版 3（`order` も無い）・版 2（`columns` も無い）・版 1（`language` と `theme` も無い）も読める。
//! 無い鍵は既定で補い、書くのは版 5 だけ（ARC-009 の移行）。
//! 前の版にその版に無い鍵があれば知らない鍵として拒む。
//! 形はここ（純関数）、読み書きはアダプタ。知らない版・知らない鍵・読めない値は `None` で、黙って既定値へ落とさない（FR-014）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::language_choice::LanguageChoice;
use nene_docket_core::theme_choice::ThemeChoice;

use crate::column_order::ColumnOrder;
use crate::column_widths::ColumnWidths;
use crate::pane_height::PaneHeight;
use crate::settings::Settings;

/// いま書く版。
const VERSION: &str = "5";

/// 読み取り中の値。
struct Reading {
    /// 版の番号（5・4・3・2・1）。
    version: Option<u8>,
    refresh_seconds: Option<u64>,
    language: Option<LanguageChoice>,
    theme: Option<ThemeChoice>,
    columns: Option<ColumnWidths>,
    order: Option<ColumnOrder>,
    pane: Option<PaneHeight>,
    recent: Vec<String>,
}

/// 文字列から設定を読む（版 5・4・3・2・1）。
#[must_use]
pub fn parse(text: &str) -> Option<Settings> {
    let mut reading = Reading {
        version: None,
        refresh_seconds: None,
        language: None,
        theme: None,
        columns: None,
        order: None,
        pane: None,
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
                self.version = Some(match value {
                    VERSION => 5,
                    "4" => 4,
                    "3" => 3,
                    "2" => 2,
                    "1" => 1,
                    _ => return None,
                });
            }
            "refresh_seconds" => self.refresh_seconds = Some(value.parse::<u64>().ok()?),
            "language" => self.language = Some(LanguageChoice::from_keyword(value)?),
            "theme" => self.theme = Some(ThemeChoice::from_keyword(value)?),
            "columns" => self.columns = Some(ColumnWidths::parse(value)?),
            "order" => self.order = Some(ColumnOrder::parse(value)?),
            "pane_px" => self.pane = Some(PaneHeight::parse(value)?),
            "recent" if value.is_empty() => return None,
            "recent" => self.recent.push(String::from(value)),
            _ => return None,
        }
        Some(())
    }

    /// 版ごとの必須の鍵が揃っていれば設定に。前の版にその版に無い鍵があれば知らない鍵として拒む。
    fn into_settings(self) -> Option<Settings> {
        let version = self.version?;
        let choices = match version {
            1 => {
                if self.language.is_some() || self.theme.is_some() {
                    return None;
                }
                (LanguageChoice::System, ThemeChoice::System)
            }
            _ => (self.language?, self.theme?),
        };
        let columns = since(version, 3, self.columns, ColumnWidths::initial())?;
        let order = since(version, 4, self.order, ColumnOrder::initial())?;
        let pane = since(version, 5, self.pane, PaneHeight::Auto)?;
        Some(
            Settings::new(self.recent, self.refresh_seconds?, choices)
                .with_columns(columns)
                .with_order(order)
                .with_pane(pane),
        )
    }
}

/// 版 `from` から入った鍵。その版からは必須、前の版では無いのが正しく（あれば知らない鍵）、無ければ `default`。
fn since<T>(version: u8, from: u8, value: Option<T>, default: T) -> Option<T> {
    if version >= from {
        value
    } else if value.is_some() {
        None
    } else {
        Some(default)
    }
}

/// 設定を文字列にする（版 5）。`parse` の逆。
#[must_use]
pub fn encode(settings: &Settings) -> String {
    let mut text = alloc::format!(
        "version = {VERSION}\nrefresh_seconds = {}\nlanguage = {}\ntheme = {}\ncolumns = {}\norder = {}\npane_px = {}\n",
        settings.refresh_seconds(),
        settings.language().keyword(),
        settings.theme().keyword(),
        settings.columns().text(),
        settings.order().text(),
        settings.pane().text()
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
        let settings = settings.with_pane(PaneHeight::Fixed(200));
        let text = encode(&settings);
        assert_eq!(
            text,
            "version = 5\nrefresh_seconds = 5\nlanguage = ja\ntheme = dracula\ncolumns = 72,110,90,90,120\norder = number,title,state,assignee,due,labels\npane_px = 200\nrecent = /srv/team\nrecent = ./docket\n"
        );
        assert_eq!(parse(&text), Some(settings));
        assert_eq!(
            parse(
                "version = 4\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 72,110,90,90,120\norder = number,title,state,assignee,due,labels\n"
            ),
            Some(Settings::initial())
        );
        assert_eq!(
            parse(
                "version = 4\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 72,110,90,90,120\norder = number,title,state,assignee,due,labels\npane_px = auto\n"
            ),
            None
        );
        assert_eq!(
            parse(
                "version = 3\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 72,110,90,90,120\n"
            ),
            Some(Settings::initial())
        );
        assert_eq!(
            parse("version = 3\nrefresh_seconds = 3\nlanguage = system\ntheme = system\n"),
            None
        );
    }

    #[test]
    fn version_two_is_read_with_default_columns_but_may_not_carry_them() {
        assert_eq!(
            parse(
                "version = 2\nrefresh_seconds = 3\nlanguage = system\ntheme = system\nrecent = /a\n"
            ),
            Some(Settings::initial().opened("/a"))
        );
        assert_eq!(
            parse(
                "version = 2\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 72,110,90,90,120\n"
            ),
            None
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
            "version = 6\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 72,110,90,90,120\norder = number,title,state,assignee,due,labels\npane_px = auto\n",
            "version = 5\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 72,110,90,90,120\norder = number,title,state,assignee,due,labels\n",
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
            "version = 3\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 1,2\n",
            "version = 4\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 72,110,90,90,120\norder = title\n",
            "version = 5\nrefresh_seconds = 3\nlanguage = system\ntheme = system\ncolumns = 72,110,90,90,120\norder = number,title,state,assignee,due,labels\npane_px = 10\n",
            "version = 2\nrefresh_seconds = 3\nlanguage = system\ntheme = system\nrecent =\n",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }
}
