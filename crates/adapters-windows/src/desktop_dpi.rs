//! デスクトップの dpi（FR-016）。UI の倍率の元。プロセスの dpi 対応を宣言した後に読む。

use windows::Win32::UI::HiDpi::GetDpiForSystem;

/// 系統の dpi。0 なら `None`。
#[must_use]
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetDpiForSystem を呼ぶ（ADR 0002 の 4）"
)]
pub fn read() -> Option<u32> {
    // SAFETY: 引数が無く、値を読むだけ。
    let dpi = unsafe { GetDpiForSystem() };
    (dpi > 0).then_some(dpi)
}
