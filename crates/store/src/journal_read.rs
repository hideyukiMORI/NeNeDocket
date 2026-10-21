//! ジャーナルを読んだ結果。読めた操作と、次に読み始める位置と、読めなかった行。

use nene_docket_core::op::Op;
use nene_docket_core::signature::Signature;

use crate::record_failure::RecordFailure;

/// 読めた 1 行: 操作と、版 2 なら署名。
pub type Entry = (Op, Option<Signature>);

/// 完全な行だけを読む。末尾の改行の無い断片は「書きかけ」として次回に回す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalRead {
    entries: Vec<Entry>,
    /// 各行の先頭のバイト位置（`entries` と同じ並び・索引に使う・ADR 0015）。
    offsets: Vec<u64>,
    next_offset: u64,
    partial_tail: bool,
    rejected: Vec<(u64, RecordFailure)>,
}

impl JournalRead {
    /// 値から作る。
    #[must_use]
    pub fn new(
        (entries, offsets): (Vec<Entry>, Vec<u64>),
        next_offset: u64,
        partial_tail: bool,
        rejected: Vec<(u64, RecordFailure)>,
    ) -> Self {
        Self {
            entries,
            offsets,
            next_offset,
            partial_tail,
            rejected,
        }
    }

    /// 読めた行。ファイルの順。
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// 読めた操作（署名は落とす）。ファイルの順。
    #[must_use]
    pub fn ops(&self) -> Vec<Op> {
        self.entries.iter().map(|(op, _)| op.clone()).collect()
    }

    /// 行を取り出す。
    #[must_use]
    pub fn into_entries(self) -> Vec<Entry> {
        self.entries
    }

    /// 行と、その先頭のバイト位置を取り出す。
    #[must_use]
    pub fn into_located_entries(self) -> Vec<(u64, Entry)> {
        self.offsets.into_iter().zip(self.entries).collect()
    }

    /// 次に読み始めるバイト位置（最後の完全な行の直後）。
    #[must_use]
    pub const fn next_offset(&self) -> u64 {
        self.next_offset
    }

    /// 末尾に改行の無い断片があったか（書きかけ、または落ちた直後）。
    #[must_use]
    pub const fn has_partial_tail(&self) -> bool {
        self.partial_tail
    }

    /// 読めなかった行の、バイト位置と理由。完全な行なのに壊れている。利用者に示す（FR-014）。
    #[must_use]
    pub fn rejected(&self) -> &[(u64, RecordFailure)] {
        &self.rejected
    }
}
