//! `NeNe Docket` のジャーナル（版 1・版 2）の形と置き場（ADR 0004 / ADR 0010 / ARC-009）。
//!
//! 台帳はフォルダ。`docket.v1.txt` に版と名前と持ち主、`journal/<account>.v1.log` にアカウントごとの操作の列。
//! 権限を有効にした台帳では `journal/<account>.v2.log` に署名した行、`keys/` に公開鍵、`permissions.v1.txt` に持ち主が署名した権限。
//! 各プロセスは自分のジャーナルにだけ**追記**し、他人のジャーナルは伸びた分だけ**読む**。ロックは使わない。
//! 現在時刻・乱数・環境・ファイルを読める 3 区画の 1 つ（ARC-007 / CNF-012）。直列化の型は core の型と分ける。

pub mod administration;
pub mod admission_failure;
pub mod attachment_store;
mod crc32;
pub mod docket_file;
mod escape;
pub mod file_journal;
pub mod journal_file;
pub mod journal_folder;
pub mod journal_read;
pub mod journal_record;
pub mod journal_version;
pub mod key_file;
pub mod other_journals;
pub mod own_journal;
pub mod permissions_store;
pub mod record_failure;
mod resumed_snapshot;
pub mod snapshot_file;
mod snapshot_reading;
pub mod store_failure;
pub mod verifier;
