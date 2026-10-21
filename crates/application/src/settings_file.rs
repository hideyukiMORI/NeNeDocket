//! 設定ファイルの形（FR-012）。`version = 1`、`refresh_seconds = <秒>`、`recent = <フォルダ>`（新しいものが先、1 行に 1 つ）。
//!
//! 形はここ（純関数）、読み書きはアダプタ。版が違う・知らない鍵・読めない値は `None` で、黙って既定値へ落とさない（FR-014）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::settings::Settings;

/// この形の版。
const VERSION: &str = "1";

/// 文字列から設定を読む。
#[must_use]
pub fn parse(text: &str) -> Option<Settings> {
    let mut version = None;
    let mut refresh_seconds = None;
    let mut recent = Vec::new();
    for line in text.lines() {
        let (key, value) = line.split_once('=')?;
        let value = value.trim();
        match key.trim() {
            "version" => version = Some(value),
            "refresh_seconds" => refresh_seconds = Some(value.parse::<u64>().ok()?),
            "recent" if value.is_empty() => return None,
            "recent" => recent.push(String::from(value)),
            _ => return None,
        }
    }
    if version != Some(VERSION) {
        return None;
    }
    Some(Settings::new(recent, refresh_seconds?))
}

/// 設定を文字列にする。`parse` の逆。
#[must_use]
pub fn encode(settings: &Settings) -> String {
    let mut text = alloc::format!(
        "version = {VERSION}\nrefresh_seconds = {}\n",
        settings.refresh_seconds()
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
    use super::*;

    #[test]
    fn encode_then_parse_round_trips_in_order() {
        let settings = Settings::new(
            alloc::vec![String::from("/srv/team"), String::from("./docket")],
            5,
        );
        let text = encode(&settings);
        assert_eq!(
            text,
            "version = 1\nrefresh_seconds = 5\nrecent = /srv/team\nrecent = ./docket\n"
        );
        assert_eq!(parse(&text), Some(settings));
        assert_eq!(
            parse("version = 1\nrefresh_seconds = 3\n"),
            Some(Settings::initial())
        );
    }

    #[test]
    fn a_different_version_an_unknown_key_or_a_bad_value_is_refused() {
        assert_eq!(parse("version = 2\nrefresh_seconds = 3\n"), None);
        assert_eq!(parse("version = 1\nrefresh_seconds = soon\n"), None);
        assert_eq!(parse("version = 1\n"), None);
        assert_eq!(
            parse("version = 1\nrefresh_seconds = 3\ntheme = dark\n"),
            None
        );
        assert_eq!(parse("version = 1\nrefresh_seconds = 3\nrecent =\n"), None);
        assert_eq!(parse("garbage"), None);
    }
}
