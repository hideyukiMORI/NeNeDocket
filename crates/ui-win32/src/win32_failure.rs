//! Windows の画面側で起こりうる失敗。閉じた集合（ARC-010）。

use core::fmt;

/// Win32 / Direct2D / DirectWrite の失敗を 1 つの型に畳む。合成ルートはこれを 1 行で示して終了する（ARC-006）。
#[derive(Debug)]
pub enum Win32Failure {
    /// Win32 か COM の呼び出しが失敗した（HRESULT）。
    Api(windows::core::Error),
    /// 幾何が Win32 の `i32` / `u32` に収まらない。
    GeometryOutOfRange,
}

impl fmt::Display for Win32Failure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api(error) => write!(formatter, "Windows API failed: {error}"),
            Self::GeometryOutOfRange => {
                formatter.write_str("window geometry exceeds the Windows limits")
            }
        }
    }
}

impl From<windows::core::Error> for Win32Failure {
    fn from(error: windows::core::Error) -> Self {
        Self::Api(error)
    }
}
