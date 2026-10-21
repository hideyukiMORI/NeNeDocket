//! `NeNe Docket` のアダプタ（Linux）。application のポートを OS の資源で実装する（ARC-007 / CNF-012）。
//!
//! 壁時計（`SystemTime`）・アカウントの保存（設定フォルダのファイルと `/dev/urandom` の ID）・
//! 他人のジャーナルを読み直す刻み（スレッドが X の窓へ `ClientMessage` を送る）。業務の判断は持たない。

pub mod adapter_failure;
mod random_id;
pub mod refresh_ticker;
pub mod saved_account;
pub mod system_clock;
