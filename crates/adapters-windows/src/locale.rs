//! OS の言語（ADR 0011）。利用者の既定のロケール名（`GetUserDefaultLocaleName`・`ja-JP` など）を core の純関数で言語にする。起動時に 1 度だけ。

use nene_docket_core::language::Language;
use windows::Win32::Globalization::GetUserDefaultLocaleName;

/// `LOCALE_NAME_MAX_LENGTH`。
const NAME_CAPACITY: usize = 85;

/// OS の言語。分からなければ `None`（`system` は英語になる）。
#[must_use]
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetUserDefaultLocaleName を呼ぶ（ADR 0002 の 4）"
)]
pub fn read() -> Option<Language> {
    let mut name = [0_u16; NAME_CAPACITY];
    // SAFETY: name はこの関数の局所で、呼び出しの間だけ書かれる。長さは LOCALE_NAME_MAX_LENGTH。
    let written = unsafe { GetUserDefaultLocaleName(&mut name) };
    let length = usize::try_from(written).ok()?.checked_sub(1)?;
    let text = String::from_utf16(name.get(..length)?).ok()?;
    Language::from_locale(&text)
}
