//! `NeNe Docket` のジャーナル（版 1）の形と置き場（ADR 0004 / ARC-009）。
//!
//! 台帳はフォルダ。`docket.v1.txt` に版と名前、`journal/<account>.v1.log` にアカウントごとの操作の列。
//! 各プロセスは自分のジャーナルにだけ**追記**し、他人のジャーナルは伸びた分だけ**読む**。ロックは使わない。
//! 現在時刻・乱数・環境・ファイルを読める 3 区画の 1 つ（ARC-007 / CNF-012）。直列化の型は core の型と分ける。

mod crc32;
pub mod docket_file;
mod escape;
pub mod file_journal;
pub mod journal_file;
pub mod journal_folder;
pub mod journal_read;
pub mod journal_record;
pub mod other_journals;
pub mod own_journal;
pub mod record_failure;
pub mod store_failure;
