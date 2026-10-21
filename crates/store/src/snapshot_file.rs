//! 自分のアカウントのスナップショット（ADR 0015 の 3）。`snapshot/<account>.v1.txt`（状態）と `snapshot/<account>.v1.idx`（課題ごとの索引・固定長の二進）。
//!
//! 状態は `key = value` の行: `version = 1`・`clock = <時計>`・`sequence = <次の連番>`・`last = <時計>:<アカウント>|-`・
//! `permissions = <署名の 16 進>|-`（権限が変わったら捨てる）・`rejected = <n>`・`unverified = <n>`・`orphans = <n>`・
//! `journal = <アカウント> <版の印> <読み終えた位置>`（索引の番号はこの順）・
//! `item = <課題> <消した 0|1> <順位> <状態> <期限|-> <題名> <担当> <ラベル> <本文> <添付の最大の連番> <添付…>`（タブ区切り・逃がし・
//! 添付は `<連番>:<大きさ>:<SHA-256>:<名前>` を `,` で結ぶ）・
//! `undo = done|undone|map …`・`own = <操作> <課題>`・`index = <索引の件数>`。
//! 索引は 1 件 32 バイト: 作ったアカウント（16・ビッグエンディアン）・連番（4）・ジャーナルの番号（1）・詰め物（3）・位置（8）。
//! 派生値なので、消しても正しさは変わらない（全部読み直す）。書くときは一時名に書いて rename する。

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use nene_docket_application::ledger_snapshot::LedgerSnapshot;
use nene_docket_core::account_id::AccountId;
use nene_docket_core::attachment::Attachment;
use nene_docket_core::date::Date;
use nene_docket_core::field_value::FieldValue;
use nene_docket_core::hex;
use nene_docket_core::item::Item;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::item_state::ItemState;
use nene_docket_core::logical_clock::LogicalClock;
use nene_docket_core::op_id::OpId;

use core::fmt::Write as _;

use crate::crc32::crc32;
use crate::escape::{escape, unescape};
use crate::journal_record;
use crate::journal_version::JournalVersion;
use crate::replace;
use crate::snapshot_reading::Reading;
use crate::store_failure::StoreFailure;

/// スナップショットを置くフォルダ。
const DIRECTORY: &str = "snapshot";
/// この形の版。
pub(crate) const VERSION: &str = "1";
/// 無い値。
pub(crate) const NONE: &str = "-";
/// 索引 1 件のバイト数。
const RECORD_BYTES: usize = 32;

/// 課題ごとの索引: ジャーナルの番号と、その中の行の先頭のバイト位置を 1 つの数に詰めたもの（[`pack`] / [`unpack`]）。
pub type Index = BTreeMap<ItemId, Vec<u64>>;

/// 位置に使うビット数（残りの 8 ビットがジャーナルの番号）。
const OFFSET_BITS: u32 = 56;

/// ジャーナルの番号（上位 8 ビット）と位置（下位 56 ビット）を 1 つの数に。
#[must_use]
pub fn pack(journal: u8, offset: u64) -> u64 {
    (u64::from(journal) << OFFSET_BITS) | (offset & ((1_u64 << OFFSET_BITS) - 1))
}

/// [`pack`] の逆。
#[must_use]
pub fn unpack(packed: u64) -> (u8, u64) {
    let journal = u8::try_from(packed >> OFFSET_BITS).unwrap_or(u8::MAX);
    (journal, packed & ((1_u64 << OFFSET_BITS) - 1))
}

/// 状態と、ジャーナルの読み位置と、索引。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredSnapshot {
    ledger: LedgerSnapshot,
    journals: Vec<((AccountId, JournalVersion), u64)>,
    permissions: Option<String>,
    own_rejected: usize,
    own_unverified: usize,
    index: Index,
}

impl StoredSnapshot {
    /// 値から。
    #[must_use]
    pub fn new(
        ledger: LedgerSnapshot,
        journals: Vec<((AccountId, JournalVersion), u64)>,
        (permissions, own_rejected, own_unverified): (Option<String>, usize, usize),
        index: Index,
    ) -> Self {
        Self {
            ledger,
            journals,
            permissions,
            own_rejected,
            own_unverified,
            index,
        }
    }

    /// 台帳の状態。
    #[must_use]
    pub const fn ledger(&self) -> &LedgerSnapshot {
        &self.ledger
    }

    /// ジャーナルと読み終えた位置（索引の番号の順）。
    #[must_use]
    pub fn journals(&self) -> &[((AccountId, JournalVersion), u64)] {
        &self.journals
    }

    /// 書いたときの `permissions.v1.txt` の署名。
    #[must_use]
    pub fn permissions(&self) -> Option<&str> {
        self.permissions.as_deref()
    }

    /// 自分のジャーナルの壊れた行と検証で捨てた行の数（読み位置より前の分）。
    #[must_use]
    pub const fn own_counts(&self) -> (usize, usize) {
        (self.own_rejected, self.own_unverified)
    }

    /// 索引を取り出す。
    #[must_use]
    pub fn into_index(self) -> Index {
        self.index
    }

    /// 台帳の状態を取り出す。
    #[must_use]
    pub fn into_ledger(self) -> LedgerSnapshot {
        self.ledger
    }
}

pub(crate) fn paths(root: &Path, account: AccountId) -> (PathBuf, PathBuf) {
    let directory = root.join(DIRECTORY);
    (
        directory.join(format!("{}.v{VERSION}.txt", account.hex())),
        directory.join(format!("{}.v{VERSION}.idx", account.hex())),
    )
}

/// 読む。無ければ `None`。形が違えば失敗（呼ぶ側は捨てて全部読む）。
///
/// # Errors
///
/// 読めない・形が違う。
pub fn read(root: &Path, account: AccountId) -> Result<Option<StoredSnapshot>, StoreFailure> {
    let (state_path, index_path) = paths(root, account);
    let text = match fs::read_to_string(&state_path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(StoreFailure::Io(error)),
    };
    let bytes = fs::read(&index_path)?;
    parse(&text, &bytes).map(Some)
}

/// 書く（一時名に書いて rename・Windows のロックには再試行・Issue #79）。
///
/// # Errors
///
/// 書けない。
pub fn write(
    root: &Path,
    account: AccountId,
    snapshot: &StoredSnapshot,
) -> Result<(), StoreFailure> {
    let (state_path, index_path) = paths(root, account);
    fs::create_dir_all(root.join(DIRECTORY))?;
    replace::replace_file(&index_path, &encode_index(&snapshot.index))?;
    replace::replace_file(&state_path, encode(snapshot).as_bytes())?;
    Ok(())
}

/// 状態の文字列。
#[must_use]
pub fn encode(snapshot: &StoredSnapshot) -> String {
    let ledger = &snapshot.ledger;
    let mut text = format!(
        "version = {VERSION}\nclock = {}\nsequence = {}\nlast = {}\npermissions = {}\nrejected = {}\nunverified = {}\norphans = {}\n",
        ledger.clock().tick(),
        ledger.next_sequence(),
        ledger
            .last_applied()
            .map_or_else(|| NONE.to_owned(), op_id_text),
        snapshot.permissions.as_deref().unwrap_or(NONE),
        snapshot.own_rejected,
        snapshot.own_unverified,
        ledger.docket().orphans()
    );
    for ((account, version), offset) in &snapshot.journals {
        let _written = writeln!(
            text,
            "journal = {} {} {offset}",
            account.hex(),
            version.tag()
        );
    }
    for item in ledger.docket().items_in_creation_order() {
        text.push_str("item = ");
        text.push_str(&item_text(item));
        text.push('\n');
    }
    let undo = ledger.undo();
    for id in undo.done() {
        let _written = writeln!(text, "undo = done {}", op_id_text(*id));
    }
    for (plain, reversal) in undo.undone() {
        let _written = writeln!(
            text,
            "undo = undone {} {}",
            op_id_text(*plain),
            op_id_text(*reversal)
        );
    }
    for (reversal, plain) in undo.undo_ops() {
        let _written = writeln!(
            text,
            "undo = map {} {}",
            op_id_text(*reversal),
            op_id_text(*plain)
        );
    }
    for (id, item) in ledger.own_items() {
        let _written = writeln!(text, "own = {} {}", op_id_text(*id), item_id_text(*item));
    }
    let count: usize = snapshot.index.values().map(Vec::len).sum();
    let _written = writeln!(text, "index = {count}");
    // 🔴 最後の行は検査和（状態の文・索引のバイト列）。ビット反転も捨てて全部読む側に倒す（Issue #77）。
    let _checksum = writeln!(
        text,
        "crc = {:08x} {:08x}",
        crc32(text.as_bytes()),
        crc32(&encode_index(&snapshot.index))
    );
    text
}

/// 最後の行の検査和を確かめ、それより前の文を返す。無い・合わないなら `None`。
fn checked_body<'a>(text: &'a str, index_bytes: &[u8]) -> Option<&'a str> {
    let last = text.lines().last()?;
    let body = text.get(..text.len().checked_sub(last.len())?.checked_sub(1)?)?;
    let (state_hex, index_hex) = last.strip_prefix("crc = ")?.split_once(' ')?;
    let state_ok = u32::from_str_radix(state_hex, 16).ok()? == crc32(body.as_bytes());
    let index_ok = u32::from_str_radix(index_hex, 16).ok()? == crc32(index_bytes);
    (state_ok && index_ok).then_some(body)
}

fn op_id_text(id: OpId) -> String {
    format!("{}:{}", id.clock().tick(), id.account().hex())
}

pub(crate) fn parse_op_id(text: &str) -> Option<OpId> {
    let (clock, account) = text.split_once(':')?;
    Some(OpId::new(
        LogicalClock::new(clock.parse().ok()?),
        AccountId::from_hex(account)?,
    ))
}

fn item_id_text(item: ItemId) -> String {
    format!("{}:{}", item.creator().hex(), item.sequence())
}

pub(crate) fn parse_item_id(text: &str) -> Option<ItemId> {
    let (creator, sequence) = text.split_once(':')?;
    Some(ItemId::new(
        AccountId::from_hex(creator)?,
        sequence.parse().ok()?,
    ))
}

/// 課題 1 件（タブ区切り・文字列は逃がす）。
fn item_text(item: &Item) -> String {
    [
        item_id_text(item.id()),
        if item.is_deleted() { "1" } else { "0" }.to_owned(),
        item.rank().to_string(),
        item.state().keyword().to_owned(),
        item.due().map_or_else(|| NONE.to_owned(), Date::text),
        escape(item.title()),
        escape(item.assignee()),
        item.labels()
            .iter()
            .map(|label| escape(label))
            .collect::<Vec<String>>()
            .join(","),
        escape(item.body()),
        item.last_attachment().to_string(),
        item.attachments()
            .iter()
            .map(|attachment| {
                format!(
                    "{}:{}:{}:{}",
                    attachment.sequence(),
                    attachment.bytes(),
                    attachment.digest_hex(),
                    escape(attachment.name())
                )
            })
            .collect::<Vec<String>>()
            .join(","),
    ]
    .join("\t")
}

/// `<連番>:<大きさ>:<SHA-256>:<名前>` を `,` で結んだもの（逃がした `,` は割らない）から。
fn parse_attachments(text: &str) -> Option<Vec<Attachment>> {
    if text.is_empty() {
        return Some(Vec::new());
    }
    journal_record::decode_labels(text)
        .ok()?
        .into_iter()
        .map(|entry| {
            let mut parts = entry.splitn(4, ':');
            let sequence = parts.next()?.parse().ok()?;
            let bytes = parts.next()?.parse().ok()?;
            let digest = hex::decode::<32>(parts.next()?)?;
            let name = parts.next()?;
            Some(Attachment::new(sequence, String::from(name), bytes, digest))
        })
        .collect()
}

pub(crate) fn parse_item(text: &str) -> Option<Item> {
    let fields: Vec<&str> = text.split('\t').collect();
    if fields.len() != 11 {
        return None;
    }
    let field = |index: usize| fields.get(index).copied().unwrap_or("");
    let labels = if field(7).is_empty() {
        Vec::new()
    } else {
        journal_record::decode_labels(field(7)).ok()?
    };
    let due = match field(4) {
        NONE => None,
        date => Some(Date::parse(date)?),
    };
    Some(
        Item::created(parse_item_id(field(0))?)
            .with(FieldValue::Rank(field(2).parse().ok()?))
            .with(FieldValue::State(ItemState::from_keyword(field(3))?))
            .with(FieldValue::Due(due))
            .with(FieldValue::Title(unescape(field(5))?))
            .with(FieldValue::Assignee(unescape(field(6))?))
            .with(FieldValue::Labels(labels))
            .with(FieldValue::Body(unescape(field(8))?))
            .with_deleted(field(1) == "1")
            .with_attachments(parse_attachments(field(10))?, field(9).parse().ok()?),
    )
}

/// 状態の文字列と索引のバイト列から。
///
/// # Errors
///
/// 形が違う。
pub fn parse(text: &str, index_bytes: &[u8]) -> Result<StoredSnapshot, StoreFailure> {
    let body = checked_body(text, index_bytes).ok_or(StoreFailure::BadSnapshot)?;
    let mut reading = Reading::empty();
    let mut versioned = false;
    for line in body.lines() {
        let (key, value) = line.split_once(" = ").ok_or(StoreFailure::BadSnapshot)?;
        if key == "version" {
            versioned = true;
        }
        reading.take(key, value).ok_or(StoreFailure::BadSnapshot)?;
    }
    if !versioned {
        return Err(StoreFailure::BadSnapshot);
    }
    let index = decode_index(index_bytes).ok_or(StoreFailure::BadSnapshot)?;
    reading.into_snapshot(index)
}

/// 索引のバイト列。
#[must_use]
pub fn encode_index(index: &Index) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (item, places) in index {
        for packed in places {
            let (journal, offset) = unpack(*packed);
            bytes.extend_from_slice(&item.creator().number().to_be_bytes());
            bytes.extend_from_slice(&item.sequence().to_be_bytes());
            bytes.push(journal);
            bytes.extend_from_slice(&[0, 0, 0]);
            bytes.extend_from_slice(&offset.to_be_bytes());
        }
    }
    bytes
}

/// 索引のバイト列から。長さが 32 の倍数でなければ `None`。
#[must_use]
pub fn decode_index(bytes: &[u8]) -> Option<Index> {
    if !bytes.len().is_multiple_of(RECORD_BYTES) {
        return None;
    }
    let mut index: Index = BTreeMap::new();
    for record in bytes.as_chunks::<RECORD_BYTES>().0 {
        let creator = u128::from_be_bytes(record.get(0..16)?.try_into().ok()?);
        let sequence = u32::from_be_bytes(record.get(16..20)?.try_into().ok()?);
        let journal = *record.get(20)?;
        let offset = u64::from_be_bytes(record.get(24..32)?.try_into().ok()?);
        index
            .entry(ItemId::new(AccountId::new(creator), sequence))
            .or_default()
            .push(pack(journal, offset));
    }
    Some(index)
}

#[cfg(test)]
mod tests {
    use nene_docket_application::undo_state::UndoState;
    use nene_docket_core::docket::Docket;
    use nene_docket_core::op::Op;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;

    const ME: AccountId = AccountId::new(0x11);
    const OTHER: AccountId = AccountId::new(0x22);

    fn sample() -> StoredSnapshot {
        let item = ItemId::new(OTHER, 1);
        let create = Op::new(
            OpId::new(LogicalClock::new(1), OTHER),
            WallClock::new(0),
            OpKind::Create(item),
        );
        let docket = Docket::empty()
            .applied(&create)
            .applied(&Op::new(
                OpId::new(LogicalClock::new(2), ME),
                WallClock::new(0),
                OpKind::Set {
                    item,
                    value: FieldValue::Title(String::from("題\t名,改行\n")),
                },
            ))
            .applied(&Op::new(
                OpId::new(LogicalClock::new(3), ME),
                WallClock::new(0),
                OpKind::Set {
                    item,
                    value: FieldValue::Labels(vec![String::from("a,b"), String::from("c")]),
                },
            ))
            .applied(&Op::new(
                OpId::new(LogicalClock::new(4), ME),
                WallClock::new(0),
                OpKind::Attach {
                    item,
                    attachment: Attachment::new(2, String::from("a,b:c.png"), 5, [1_u8; 32]),
                },
            ));
        let mut index = BTreeMap::new();
        index.insert(item, vec![pack(0, 0), pack(1, 120)]);
        let own = OpId::new(LogicalClock::new(2), ME);
        StoredSnapshot::new(
            LedgerSnapshot::new(
                (
                    docket,
                    LogicalClock::new(3),
                    4,
                    Some(OpId::new(LogicalClock::new(3), ME)),
                ),
                UndoState::new(vec![own], Vec::new(), Vec::new()),
                vec![(own, item)],
            ),
            vec![
                ((ME, JournalVersion::One), 240),
                ((OTHER, JournalVersion::Two), 90),
            ],
            (Some(String::from("abcd")), 1, 2),
            index,
        )
    }

    #[test]
    fn encode_then_parse_round_trips_through_text_and_bytes() {
        let snapshot = sample();
        let text = encode(&snapshot);
        assert!(text.starts_with("version = 1\nclock = 3\nsequence = 4\nlast = 3:"));
        assert!(text.contains("\npermissions = abcd\nrejected = 1\nunverified = 2\norphans = 0\n"));
        assert!(text.contains("\nindex = 2\n"));
        let bytes = encode_index(&snapshot.index);
        assert_eq!(bytes.len(), 64);
        assert_eq!(parse(&text, &bytes).unwrap(), snapshot);
    }

    #[test]
    fn writes_and_reads_the_files_and_refuses_broken_ones() {
        let root =
            std::env::temp_dir().join(format!("nene-docket-store-{}-snapshot", std::process::id()));
        let _ignored = fs::remove_dir_all(&root);
        assert_eq!(read(&root, ME).unwrap(), None);
        let snapshot = sample();
        write(&root, ME, &snapshot).unwrap();
        assert_eq!(read(&root, ME).unwrap(), Some(snapshot.clone()));
        let (state_path, index_path) = paths(&root, ME);
        fs::write(&index_path, [1_u8; 31]).unwrap();
        assert!(matches!(read(&root, ME), Err(StoreFailure::BadSnapshot)));
        fs::write(&index_path, encode_index(&snapshot.index)).unwrap();
        for text in [
            "clock = 1\n",
            "version = 2\n",
            "version = 1\nclock = x\n",
            "version = 1\ncolor = red\n",
            "version = 1\nitem = a\n",
            "version = 1\nundo = what 1:11\n",
            "version = 1\njournal = 11 v1\n",
        ] {
            fs::write(&state_path, text).unwrap();
            assert!(
                matches!(read(&root, ME), Err(StoreFailure::BadSnapshot)),
                "{text}"
            );
        }
        // 索引の件数が合わない。
        fs::write(
            &state_path,
            encode(&snapshot).replace("index = 2", "index = 3"),
        )
        .unwrap();
        assert!(matches!(read(&root, ME), Err(StoreFailure::BadSnapshot)));
        let _cleaned = fs::remove_dir_all(&root);
    }
}
