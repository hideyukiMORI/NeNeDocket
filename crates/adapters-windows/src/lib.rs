//! `NeNe Docket` のアダプタ（Windows）。application のポートを OS の資源で実装する（ARC-007 / CNF-012）。
//!
//! 壁時計（`SystemTime`）・アカウントの保存（`%APPDATA%\nene-docket\` と `BCryptGenRandom` の ID）・設定の読み書き・
//! デスクトップの dpi（`GetDpiForSystem`）。読み直しの刻みは Windows では画面のタイマー（ADR 0009）。
//! `unsafe` を書ける 2 クレートの 1 つで、Win32 を呼ぶ関数ごとに `#[expect(unsafe_code)]` と SAFETY の注釈を置く。
//! Win32 に触れるモジュールは Windows でだけ組み立て、残り（標準ライブラリだけ）は Linux でもテストする。

pub mod adapter_failure;
mod config_directory;
#[cfg(windows)]
pub mod desktop_dpi;
#[cfg(windows)]
mod random_id;
#[cfg(windows)]
pub mod saved_account;
pub mod saved_settings;
pub mod system_clock;
