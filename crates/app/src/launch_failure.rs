//! 起動できなかった理由。各区画の失敗を 1 つに畳み、`main` が 1 行で示す（RS-014 / ARC-006）。
//! 画面とアダプタの失敗の型は OS ごとに違う（ADR 0002）。

use core::fmt;

#[cfg(windows)]
use nene_docket_adapters_windows::adapter_failure::AdapterFailure;
#[cfg(not(windows))]
use nene_docket_adapters_x11::adapter_failure::AdapterFailure;
use nene_docket_store::store_failure::StoreFailure;
#[cfg(windows)]
use nene_docket_ui_win32::win32_failure::Win32Failure as UiFailure;
#[cfg(not(windows))]
use nene_docket_ui_x11::ui_failure::UiFailure;

/// 使い方の誤りと、各区画の失敗。
#[derive(Debug)]
pub(crate) enum LaunchFailure {
    /// 引数が使い方に合わない。
    Usage,
    /// アダプタ（アカウント・設定・刻み）。
    Adapter(AdapterFailure),
    /// 台帳のフォルダとジャーナル。
    Store(StoreFailure),
    /// 画面。
    Ui(UiFailure),
}

impl fmt::Display for LaunchFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(
                "usage: nene-docket [FOLDER] [--permissions on|off] [--allow ACCOUNT] [--disallow ACCOUNT] [--group NAME ACCOUNT] [--ungroup NAME ACCOUNT] [--succeed NEW OLD] [--owner ACCOUNT]",
            ),
            Self::Adapter(failure) => fmt::Display::fmt(failure, formatter),
            Self::Store(failure) => fmt::Display::fmt(failure, formatter),
            Self::Ui(failure) => fmt::Display::fmt(failure, formatter),
        }
    }
}

impl From<AdapterFailure> for LaunchFailure {
    fn from(failure: AdapterFailure) -> Self {
        Self::Adapter(failure)
    }
}

impl From<StoreFailure> for LaunchFailure {
    fn from(failure: StoreFailure) -> Self {
        Self::Store(failure)
    }
}

impl From<UiFailure> for LaunchFailure {
    fn from(failure: UiFailure) -> Self {
        Self::Ui(failure)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_and_wrapped_failures_have_words() {
        assert!(
            LaunchFailure::Usage
                .to_string()
                .starts_with("usage: nene-docket [FOLDER]")
        );
        let wrapped: LaunchFailure = StoreFailure::BadDocketFile.into();
        assert!(wrapped.to_string().contains("docket.v1.txt"));
        let adapter: LaunchFailure = AdapterFailure::NoConfigDirectory.into();
        assert!(!adapter.to_string().is_empty());
    }

    #[cfg(not(windows))]
    #[test]
    fn the_x11_ui_failure_is_wrapped() {
        let ui: LaunchFailure = UiFailure::NoScreen.into();
        assert!(ui.to_string().contains("screen"));
    }

    #[cfg(windows)]
    #[test]
    fn the_win32_ui_failure_is_wrapped() {
        let ui: LaunchFailure = UiFailure::GeometryOutOfRange.into();
        assert!(ui.to_string().contains("Windows"));
    }
}
