//! 自分のジャーナル。追記と同期（fsync）だけ。書き換えない・縮めない（ADR 0004 の 2・4・5）。
//! 権限を有効にした台帳では版 2（`.v2.log`）に署名して書く（ADR 0010）。

use std::fs::{File, OpenOptions};
use std::io::{Read as _, Seek as _, SeekFrom, Write as _};
use std::path::{Path, PathBuf};

use nene_docket_core::account_id::AccountId;
use nene_docket_core::op::Op;
use nene_docket_core::signing_key::SigningKey;

use crate::journal_file;
use crate::journal_read::JournalRead;
use crate::journal_record;
use crate::journal_version::JournalVersion;
use crate::store_failure::StoreFailure;

/// 開いたファイルと、まだ `sync` していない追記があるか。鍵があれば版 2 で署名する。
#[derive(Debug)]
pub struct OwnJournal {
    path: PathBuf,
    file: File,
    account: AccountId,
    signing: Option<SigningKey>,
    dirty: bool,
}

impl OwnJournal {
    /// `journal/<account>.v1.log`（鍵があれば `.v2.log`）を追記で開く（無ければ作る）。起動時に中身を読み、壊れた末尾は捨てて数える。
    ///
    /// # Errors
    ///
    /// フォルダを作れない・開けない・読めない。
    pub fn open(
        journal_directory: &Path,
        account: AccountId,
        signing: Option<SigningKey>,
    ) -> Result<(Self, JournalRead), StoreFailure> {
        Self::open_from(journal_directory, account, signing, 0)
    }

    /// 同じく開くが、中身は `offset` から読む（スナップショットの後の分だけ・ADR 0015）。
    /// 最後の行が改行で終わっていなければ（前回の追記の途中で落ちた）、先に改行を足して 1 行の壊れに閉じ込める（Issue #77）。
    ///
    /// # Errors
    ///
    /// フォルダを作れない・開けない・読めない。
    pub fn open_from(
        journal_directory: &Path,
        account: AccountId,
        signing: Option<SigningKey>,
        offset: u64,
    ) -> Result<(Self, JournalRead), StoreFailure> {
        std::fs::create_dir_all(journal_directory)?;
        let version = version_for(signing.is_some());
        let path = journal_directory.join(file_name(account, version));
        let mut file = open_append(&path)?;
        if ends_without_newline(&path)? {
            file.write_all(b"\n")?;
            file.sync_data()?;
        }
        let read = journal_file::read_since(&path, offset)?;
        let opened = Self {
            path,
            file,
            account,
            signing,
            dirty: false,
        };
        Ok((opened, read))
    }

    /// 操作を 1 行追記し、その行の先頭のバイト位置を返す（索引に入れる・ADR 0015）。同期はしない（`sync` で）。
    ///
    /// # Errors
    ///
    /// 書けない。
    pub fn append(&mut self, op: &Op) -> Result<u64, StoreFailure> {
        let mut line = match &self.signing {
            Some(key) => journal_record::encode_signed(op, key),
            None => journal_record::encode(op),
        };
        line.push('\n');
        let offset = self.file.metadata()?.len();
        self.file.write_all(line.as_bytes())?;
        self.dirty = true;
        Ok(offset)
    }

    /// いまのファイルの長さ（全部読み終えた位置・スナップショットに残す）。
    ///
    /// # Errors
    ///
    /// 長さを読めない。
    pub fn length(&self) -> Result<u64, StoreFailure> {
        Ok(self.file.metadata()?.len())
    }

    /// 追記を記憶装置に確実に書く。追記が無ければ何もしない。
    ///
    /// # Errors
    ///
    /// 同期に失敗した。
    pub fn sync(&mut self) -> Result<(), StoreFailure> {
        if !self.dirty {
            return Ok(());
        }
        self.file.sync_data()?;
        self.dirty = false;
        Ok(())
    }

    /// まだ同期していない追記があるか。
    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// このジャーナルのアカウント。
    #[must_use]
    pub const fn account(&self) -> AccountId {
        self.account
    }

    /// 書いている版。
    #[must_use]
    pub const fn version(&self) -> JournalVersion {
        version_for(self.signing.is_some())
    }

    /// ファイルの場所。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// 追記で開く（無ければ作る）。Windows では共有モードから `FILE_SHARE_DELETE` を外し、同期ツールやバックアップが
/// 開いている最中に rename / 削除できないようにする（追記が別名のファイルへ消えるのを防ぐ・Issue #79）。読みと書きの共有は残す。
#[cfg(windows)]
fn open_append(path: &Path) -> Result<File, StoreFailure> {
    use std::os::windows::fs::OpenOptionsExt as _;
    /// `FILE_SHARE_READ | FILE_SHARE_WRITE`（`FILE_SHARE_DELETE` の 4 を含まない）。
    const SHARE_READ_WRITE: u32 = 0x0000_0003;
    Ok(OpenOptions::new()
        .append(true)
        .create(true)
        .share_mode(SHARE_READ_WRITE)
        .open(path)?)
}

/// 追記で開く（無ければ作る）。
#[cfg(not(windows))]
fn open_append(path: &Path) -> Result<File, StoreFailure> {
    Ok(OpenOptions::new().append(true).create(true).open(path)?)
}

/// 最後のバイトが改行でないか（空のファイルは偽）。
fn ends_without_newline(path: &Path) -> Result<bool, StoreFailure> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();
    if length == 0 {
        return Ok(false);
    }
    file.seek(SeekFrom::Start(length.saturating_sub(1)))?;
    let mut last = [0_u8; 1];
    file.read_exact(&mut last)?;
    Ok(last != *b"\n")
}

/// 署名するなら版 2。
const fn version_for(signed: bool) -> JournalVersion {
    if signed {
        JournalVersion::Two
    } else {
        JournalVersion::One
    }
}

/// `<account 16 進>.vN.log`。
#[must_use]
pub fn file_name(account: AccountId, version: JournalVersion) -> String {
    format!("{}{}", account.hex(), version.file_suffix())
}

/// ファイル名からアカウントと版を読む。形が違えば `None`。
#[must_use]
pub fn account_of(file_name: &str) -> Option<(AccountId, JournalVersion)> {
    let (stem, version) = JournalVersion::split_file_name(file_name)?;
    Some((AccountId::from_hex(stem)?, version))
}

#[cfg(test)]
mod tests {
    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;

    const ACCOUNT: AccountId = AccountId::new(0x00C0_FFEE);

    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("nene-docket-store-{}-{name}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&directory);
        directory
    }

    fn op(clock: u64) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), ACCOUNT),
            WallClock::new(0),
            OpKind::Create(ItemId::new(ACCOUNT, clock.try_into().unwrap())),
        )
    }

    #[test]
    fn appends_survive_reopening_and_a_torn_tail_is_counted_not_read() {
        let directory = scratch("own");
        let (mut journal, first_read) = OwnJournal::open(&directory, ACCOUNT, None).unwrap();
        assert!(first_read.entries().is_empty());
        assert!(!journal.is_dirty());
        journal.append(&op(1)).unwrap();
        journal.append(&op(2)).unwrap();
        assert!(journal.is_dirty());
        journal.sync().unwrap();
        assert!(!journal.is_dirty());
        assert_eq!(journal.account(), ACCOUNT);
        assert_eq!(journal.version(), JournalVersion::One);
        assert!(
            journal
                .path()
                .ends_with(file_name(ACCOUNT, JournalVersion::One))
        );
        let _ignored = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_torn_tail_is_closed_on_reopen_and_the_next_append_does_not_join_it() {
        let directory = scratch("own-torn");
        let (mut journal, _) = OwnJournal::open(&directory, ACCOUNT, None).unwrap();
        journal.append(&op(1)).unwrap();
        journal.append(&op(2)).unwrap();
        journal.sync().unwrap();
        // 落ちた直後を真似る: 改行の無い断片を足す。
        let mut raw = std::fs::OpenOptions::new()
            .append(true)
            .open(journal.path())
            .unwrap();
        raw.write_all(b"v1\t3\tgarbage").unwrap();
        drop(raw);
        // 開き直すと改行が足され、断片は 1 行の壊れとして数えられる（次の追記と繋がらない・Issue #77）。
        let (mut again, read) = OwnJournal::open(&directory, ACCOUNT, None).unwrap();
        assert_eq!(read.ops(), [op(1), op(2)]);
        assert!(!read.has_partial_tail());
        assert_eq!(read.rejected().len(), 1);
        again.append(&op(3)).unwrap();
        again.sync().unwrap();
        let (_third, after) = OwnJournal::open(&directory, ACCOUNT, None).unwrap();
        assert_eq!(after.ops(), [op(1), op(2), op(3)]);
        assert_eq!(after.rejected().len(), 1);
        let _ignored = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn a_signing_key_selects_the_signed_version_and_signs_each_line() {
        let directory = scratch("own-signed");
        let key = SigningKey::from_bytes(&[3_u8; 32]);
        let (mut journal, _) = OwnJournal::open(&directory, ACCOUNT, Some(key.clone())).unwrap();
        assert_eq!(journal.version(), JournalVersion::Two);
        assert!(
            journal
                .path()
                .ends_with(file_name(ACCOUNT, JournalVersion::Two))
        );
        journal.append(&op(1)).unwrap();
        journal.sync().unwrap();
        let (_again, read) = OwnJournal::open(&directory, ACCOUNT, Some(key.clone())).unwrap();
        let (read_op, signature) = read.entries().first().cloned().unwrap();
        assert_eq!(read_op, op(1));
        assert!(journal_record::is_signed_by(
            &read_op,
            &signature.unwrap(),
            &key.public()
        ));
        let _ignored = std::fs::remove_dir_all(&directory);
    }

    #[test]
    fn file_names_carry_the_account_and_the_version() {
        for version in [JournalVersion::One, JournalVersion::Two] {
            let name = file_name(ACCOUNT, version);
            assert!(name.ends_with(version.file_suffix()));
            assert_eq!(account_of(&name), Some((ACCOUNT, version)));
        }
        assert_eq!(account_of("notes.txt"), None);
        assert_eq!(account_of("zz.v1.log"), None);
    }
}
