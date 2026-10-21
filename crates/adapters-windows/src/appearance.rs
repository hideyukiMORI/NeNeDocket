//! OS の明暗（ADR 0012）。レジストリの `AppsUseLightTheme`（0 ならダーク）を起動時に 1 度だけ読む。

use nene_docket_core::appearance::Appearance;
use windows::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW};
use windows::core::w;

/// OS の明暗。値が無い・読めないなら `None`（`system` はライト）。
#[must_use]
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の RegGetValueW を呼ぶ（ADR 0002 の 4）"
)]
pub fn read() -> Option<Appearance> {
    let mut value = 0_u32;
    let mut size = u32::try_from(size_of::<u32>()).ok()?;
    // SAFETY: 鍵の名前は静的な文字列。value と size はこの関数の局所で、呼び出しの間だけ書かれる。
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some((&raw mut value).cast::<core::ffi::c_void>()),
            Some(&raw mut size),
        )
    };
    let appearance = if value == 0 {
        Appearance::Dark
    } else {
        Appearance::Light
    };
    status.is_ok().then_some(appearance)
}
