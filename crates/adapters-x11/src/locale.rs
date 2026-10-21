//! OS の言語（ADR 0011）。`LC_ALL` → `LC_MESSAGES` → `LANG` の順に最初の空でない値を読み、core の純関数で言語にする。起動時に 1 度だけ。

use std::env;

use nene_docket_core::language::Language;

/// 見る環境変数。POSIX の優先順。
const VARIABLES: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];

/// OS の言語。分からなければ `None`（`system` は英語になる）。
#[must_use]
pub fn read() -> Option<Language> {
    VARIABLES
        .into_iter()
        .filter_map(|name| env::var(name).ok())
        .find(|value| !value.is_empty())
        .and_then(|value| Language::from_locale(&value))
}
