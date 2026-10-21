//! スナップショットから戻した部品（ADR 0015）。`FileJournal::open` が自分のスナップショットから、ジャーナルの名と読み位置・
//! 自分の行の数え・課題ごとの索引を取り出す。無ければ全部空。

use std::collections::BTreeMap;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::op::Op;

use crate::journal_version::JournalVersion;
use crate::snapshot_file::{Index, StoredSnapshot};

/// ジャーナルの名（アカウントと版）。
type JournalName = (AccountId, JournalVersion);

/// スナップショットから戻した部品（無ければ空）。
pub(crate) struct Resumed {
    journals: Vec<JournalName>,
    other_offsets: BTreeMap<JournalName, u64>,
    own_counts: (usize, usize),
    index: Index,
    offsets: Vec<(JournalName, u64)>,
}

impl Resumed {
    pub(crate) fn from_stored(stored: Option<&StoredSnapshot>, own: AccountId) -> Self {
        let Some(stored) = stored else {
            return Self {
                journals: Vec::new(),
                other_offsets: BTreeMap::new(),
                own_counts: (0, 0),
                index: BTreeMap::new(),
                offsets: Vec::new(),
            };
        };
        Self {
            journals: stored.journals().iter().map(|(known, _)| *known).collect(),
            other_offsets: stored
                .journals()
                .iter()
                .filter(|((other, _), _)| *other != own)
                .map(|(known, offset)| (*known, *offset))
                .collect(),
            own_counts: stored.own_counts(),
            index: stored.clone().into_index(),
            offsets: stored.journals().to_vec(),
        }
    }

    /// 自分のジャーナルの壊れた行と検証で捨てた行の数。
    pub(crate) const fn own_counts(&self) -> (usize, usize) {
        self.own_counts
    }

    /// ジャーナルの名の列・他人の読み位置・索引を取り出す。
    pub(crate) fn into_parts(self) -> (Vec<JournalName>, BTreeMap<JournalName, u64>, Index) {
        (self.journals, self.other_offsets, self.index)
    }

    /// そのジャーナルの読み終えた位置（無ければ 0）。
    pub(crate) fn offset_of(&self, name: JournalName) -> u64 {
        self.offsets
            .iter()
            .find(|(known, _)| *known == name)
            .map_or(0, |(_, offset)| *offset)
    }
}

/// ジャーナルの番号（無ければ末尾に足す）。
pub(crate) fn journal_number(journals: &mut Vec<JournalName>, name: JournalName) -> u8 {
    let position = journals
        .iter()
        .position(|known| *known == name)
        .unwrap_or_else(|| {
            journals.push(name);
            journals.len().saturating_sub(1)
        });
    u8::try_from(position).unwrap_or(u8::MAX)
}

/// 課題に触れる操作を索引に入れる。
pub(crate) fn remember(index: &mut Index, op: &Op, journal: u8, offset: u64) {
    if let Some(item) = op.kind().item() {
        index
            .entry(item)
            .or_default()
            .push(crate::snapshot_file::pack(journal, offset));
    }
}
