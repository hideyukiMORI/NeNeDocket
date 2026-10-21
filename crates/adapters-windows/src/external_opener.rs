//! OS の既定のアプリでファイルを開く（FR-022）。`ShellExecuteW` の `open`。プロセスの起動はアダプタだけ（ARC-007）。

use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, PCWSTR, w};

use nene_docket_application::open_failure::OpenFailure;
use nene_docket_application::opener_port::OpenerPort;

/// `ShellExecuteW` の戻りがこれより大きければ成功（Win32 の約束）。
const SUCCESS_ABOVE: usize = 32;

/// 既定のアプリで開く。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExternalOpener;

impl OpenerPort for ExternalOpener {
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の ShellExecuteW を呼ぶ（ADR 0002 の 4）"
    )]
    fn open(&self, path: &str) -> Result<(), OpenFailure> {
        let file = HSTRING::from(path);
        // SAFETY: 文字列は呼び出しの間生きている。戻りは HINSTANCE の形をした数で、32 より大きければ成功。
        let result = unsafe {
            ShellExecuteW(
                None,
                w!("open"),
                PCWSTR(file.as_ptr()),
                PCWSTR::null(),
                PCWSTR::null(),
                SW_SHOWNORMAL,
            )
        };
        if result.0.addr() > SUCCESS_ABOVE {
            Ok(())
        } else {
            Err(OpenFailure::CannotOpen)
        }
    }
}
