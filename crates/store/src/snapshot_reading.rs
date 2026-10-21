//! スナップショットの状態の行を読み取る途中の値（ADR 0015）。形は `snapshot_file`。

use nene_docket_core::account_id::AccountId;
use nene_docket_core::item::Item;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::op_id::OpId;

use nene_docket_application::ledger_snapshot::LedgerSnapshot;
use nene_docket_application::undo_state::UndoState;
use nene_docket_core::docket::Docket;
use nene_docket_core::logical_clock::LogicalClock;

use crate::journal_version::JournalVersion;
use crate::snapshot_file::{self, Index, NONE, StoredSnapshot, VERSION};
use crate::store_failure::StoreFailure;

/// 読み取り中の値。
pub(crate) struct Reading {
    clock: Option<u64>,
    sequence: Option<u32>,
    last: Option<OpId>,
    last_seen: bool,
    permissions: Option<String>,
    permissions_seen: bool,
    rejected: Option<usize>,
    unverified: Option<usize>,
    orphans: Option<usize>,
    journals: Vec<((AccountId, JournalVersion), u64)>,
    items: Vec<Item>,
    done: Vec<OpId>,
    undone: Vec<(OpId, OpId)>,
    map: Vec<(OpId, OpId)>,
    own: Vec<(OpId, ItemId)>,
    index_count: Option<usize>,
}

impl Reading {
    pub(crate) fn empty() -> Self {
        Self {
            clock: None,
            sequence: None,
            last: None,
            last_seen: false,
            permissions: None,
            permissions_seen: false,
            rejected: None,
            unverified: None,
            orphans: None,
            journals: Vec::new(),
            items: Vec::new(),
            done: Vec::new(),
            undone: Vec::new(),
            map: Vec::new(),
            own: Vec::new(),
            index_count: None,
        }
    }

    pub(crate) fn take(&mut self, key: &str, value: &str) -> Option<()> {
        match key {
            "version" => (value == VERSION).then_some(())?,
            "clock" => self.clock = Some(value.parse().ok()?),
            "sequence" => self.sequence = Some(value.parse().ok()?),
            "rejected" => self.rejected = Some(value.parse().ok()?),
            "unverified" => self.unverified = Some(value.parse().ok()?),
            "orphans" => self.orphans = Some(value.parse().ok()?),
            "index" => self.index_count = Some(value.parse().ok()?),
            "last" => {
                self.last = match value {
                    NONE => None,
                    text => Some(snapshot_file::parse_op_id(text)?),
                };
                self.last_seen = true;
            }
            "permissions" => {
                self.permissions = match value {
                    NONE => None,
                    text => Some(text.to_owned()),
                };
                self.permissions_seen = true;
            }
            _ => return self.take_listed(key, value),
        }
        Some(())
    }

    /// 繰り返す鍵（ジャーナル・課題・Undo・自分の操作）。
    fn take_listed(&mut self, key: &str, value: &str) -> Option<()> {
        match key {
            "journal" => {
                let mut parts = value.split(' ');
                let account = AccountId::from_hex(parts.next()?)?;
                let version = JournalVersion::from_tag(parts.next()?)?;
                let offset = parts.next()?.parse().ok()?;
                if parts.next().is_some() {
                    return None;
                }
                self.journals.push(((account, version), offset));
            }
            "item" => self.items.push(snapshot_file::parse_item(value)?),
            "undo" => self.take_undo(value)?,
            "own" => {
                let (id, item) = value.split_once(' ')?;
                self.own.push((
                    snapshot_file::parse_op_id(id)?,
                    snapshot_file::parse_item_id(item)?,
                ));
            }
            _ => return None,
        }
        Some(())
    }

    fn take_undo(&mut self, value: &str) -> Option<()> {
        let mut parts = value.split(' ');
        match parts.next()? {
            "done" => self.done.push(snapshot_file::parse_op_id(parts.next()?)?),
            "undone" => self.undone.push((
                snapshot_file::parse_op_id(parts.next()?)?,
                snapshot_file::parse_op_id(parts.next()?)?,
            )),
            "map" => self.map.push((
                snapshot_file::parse_op_id(parts.next()?)?,
                snapshot_file::parse_op_id(parts.next()?)?,
            )),
            _ => return None,
        }
        if parts.next().is_some() {
            return None;
        }
        Some(())
    }

    /// 揃った値からスナップショットへ。足りない鍵・索引の件数の不一致は失敗。
    pub(crate) fn into_snapshot(self, index: Index) -> Result<StoredSnapshot, StoreFailure> {
        let count: usize = index.values().map(Vec::len).sum();
        if self.index_count != Some(count) {
            return Err(StoreFailure::BadSnapshot);
        }
        let ledger = LedgerSnapshot::new(
            (
                Docket::from_parts(self.items, self.orphans.ok_or(StoreFailure::BadSnapshot)?),
                LogicalClock::new(self.clock.ok_or(StoreFailure::BadSnapshot)?),
                self.sequence.ok_or(StoreFailure::BadSnapshot)?,
                self.last_seen
                    .then_some(self.last)
                    .ok_or(StoreFailure::BadSnapshot)?,
            ),
            UndoState::new(self.done, self.undone, self.map),
            self.own,
        );
        Ok(StoredSnapshot::new(
            ledger,
            self.journals,
            (
                self.permissions_seen
                    .then_some(self.permissions)
                    .ok_or(StoreFailure::BadSnapshot)?,
                self.rejected.ok_or(StoreFailure::BadSnapshot)?,
                self.unverified.ok_or(StoreFailure::BadSnapshot)?,
            ),
            index,
        ))
    }
}
