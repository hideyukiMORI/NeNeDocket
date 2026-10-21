//! `NeNe Docket` の合成ルート（ARC-006）。
//!
//! ポートに実装を結び、UI を起動する唯一の場所。端末への出力とプロセスの終了コードを
//! 扱ってよいのもここだけである（RS-014）。出してよいのは起動できなかった理由 1 行だけで、
//! ログの手段として使わない。OS ごとの結び方は別のモジュールに置き、ここは選ぶだけ（ADR 0002）。

use std::process::ExitCode;

mod arguments;
mod launch_failure;
#[cfg(windows)]
mod windows_root;
#[cfg(not(windows))]
mod x11_root;

use arguments::Arguments;

#[cfg(windows)]
use windows_root as root;
#[cfg(not(windows))]
use x11_root as root;

/// 製品の版。`Cargo.toml` の `[workspace.package] version` から cargo が渡す唯一の経路。
const VERSION: &str = env!("CARGO_PKG_VERSION");

// 🔴 標準エラーへ書いてよい唯一の場所（RS-014）。この #[expect] が宣言そのものである。
#[expect(
    clippy::print_stderr,
    reason = "RS-014: 端末への出力は合成ルートだけが行う"
)]
fn main() -> ExitCode {
    match Arguments::parse(std::env::args().skip(1)).and_then(|arguments| root::run(&arguments)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("NeNe Docket {VERSION}: {failure}");
            ExitCode::FAILURE
        }
    }
}
