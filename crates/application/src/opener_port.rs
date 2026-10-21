//! OS の既定のアプリでファイルを開くポート（FR-022）。実装は adapters（X11 は `xdg-open`、Windows は `ShellExecuteW`・ARC-007）。

use crate::open_failure::OpenFailure;

/// パスを OS に渡して開く。戻りを待たない。
pub trait OpenerPort {
    /// 開く。
    ///
    /// # Errors
    ///
    /// 起動できない。
    fn open(&self, path: &str) -> Result<(), OpenFailure>;
}
