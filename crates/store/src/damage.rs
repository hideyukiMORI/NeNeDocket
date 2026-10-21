//! テストで「壊れたファイル」を決定的に作る道具（Issue #77）。切り詰め・ビット反転・0 埋め・空・一時名の残骸。
//! 乱数は種で決まる線形合同法（外部クレートを足さない・ARC-013）。production では使わない。

use std::fs;
use std::io;
use std::path::Path;

/// 種から決まる疑似乱数（Numerical Recipes の定数）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Lcg {
    state: u64,
}

impl Lcg {
    pub(crate) const fn seeded(seed: u64) -> Self {
        Self { state: seed }
    }

    /// `0..bound` の値。`bound` が 0 なら 0。
    pub(crate) fn below(&mut self, bound: u64) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        if bound == 0 {
            return 0;
        }
        (self.state >> 33_u32) % bound
    }
}

/// 末尾を落として `keep` バイトにする。
pub(crate) fn truncate(path: &Path, keep: u64) -> io::Result<()> {
    let file = fs::OpenOptions::new().write(true).open(path)?;
    file.set_len(keep)
}

/// `index` バイト目の `bit` 番目を反転する。範囲外なら何もしない。
pub(crate) fn flip_bit(path: &Path, index: usize, bit: u8) -> io::Result<()> {
    let mut bytes = fs::read(path)?;
    if let Some(byte) = bytes.get_mut(index) {
        *byte ^= 1_u8 << (bit % 8);
        fs::write(path, bytes)?;
    }
    Ok(())
}

/// 末尾に 0 を `count` バイト足す（NTFS / NAS で停電の後に残る形）。
pub(crate) fn zero_tail(path: &Path, count: usize) -> io::Result<()> {
    let mut file = fs::OpenOptions::new().append(true).open(path)?;
    io::Write::write_all(&mut file, &vec![0_u8; count])
}

/// 中身を空にする。
pub(crate) fn empty(path: &Path) -> io::Result<()> {
    fs::write(path, b"")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use nene_docket_application::account::Account;
    use nene_docket_application::journal_port::JournalPort as _;
    use nene_docket_application::ledger_opening;
    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op::Op;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::signing_key::SigningKey;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;
    use crate::file_journal::FileJournal;
    use crate::journal_file;
    use crate::journal_version::JournalVersion;
    use crate::own_journal;
    use crate::record_failure::RecordFailure;
    use crate::store_failure::StoreFailure;

    const ME: AccountId = AccountId::new(0x11);
    const OTHER: AccountId = AccountId::new(0x22);

    fn account(id: AccountId, seed: u8) -> Account {
        Account::new(id, format!("u{seed}"), SigningKey::from_bytes(&[seed; 32]))
    }

    fn create(clock: u64, who: AccountId) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), who),
            WallClock::new(0),
            OpKind::Create(ItemId::new(who, u32::try_from(clock).unwrap())),
        )
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("nene-docket-damage-{name}"));
        let _ignored = fs::remove_dir_all(&root);
        root
    }

    /// 自分が 3 件、他人が 3 件書いた台帳。他人のジャーナルの場所を返す。
    fn docket_with_two_writers(root: &Path) -> std::path::PathBuf {
        let mut theirs = FileJournal::open(root, "Team", &account(OTHER, 2)).unwrap();
        for clock in 1_u64..=3 {
            theirs.append(&create(clock, OTHER)).unwrap();
        }
        theirs.sync().unwrap();
        let mut mine = FileJournal::open(root, "Team", &account(ME, 1)).unwrap();
        for clock in 4_u64..=6 {
            mine.append(&create(clock, ME)).unwrap();
        }
        mine.sync().unwrap();
        mine.folder()
            .journal_directory()
            .join(own_journal::file_name(OTHER, JournalVersion::One))
    }

    /// 開いて読み直し、届いた操作と壊れた行の数。
    fn reopen(root: &Path) -> (Vec<Op>, usize) {
        let mut journal = FileJournal::open(root, "Team", &account(ME, 1)).unwrap();
        let ops = journal.refresh_settled().unwrap();
        (ops, journal.rejected_lines())
    }

    #[test]
    fn the_generator_is_deterministic_and_bounded() {
        let mut first = Lcg::seeded(7);
        let mut second = Lcg::seeded(7);
        let draws: Vec<u64> = (0_u32..5).map(|_| first.below(100)).collect();
        assert_eq!(
            draws,
            (0_u32..5).map(|_| second.below(100)).collect::<Vec<_>>()
        );
        assert!(draws.iter().all(|value| *value < 100));
        assert_eq!(Lcg::seeded(1).below(0), 0);
    }

    #[test]
    fn a_truncated_other_journal_yields_exactly_the_whole_lines_before_the_cut() {
        let root = temp("truncate");
        let theirs = docket_with_two_writers(&root);
        let whole = fs::read(&theirs).unwrap();
        let line_ends: Vec<usize> = whole
            .iter()
            .enumerate()
            .filter(|(_, byte)| **byte == b'\n')
            .map(|(index, _)| index + 1)
            .collect();
        let mut lcg = Lcg::seeded(3);
        for _ in 0_u32..40 {
            let keep = lcg.below(u64::try_from(whole.len()).unwrap() + 1);
            fs::write(&theirs, &whole).unwrap();
            truncate(&theirs, keep).unwrap();
            let expected = line_ends
                .iter()
                .filter(|end| u64::try_from(**end).unwrap() <= keep)
                .count();
            let (ops, rejected) = reopen(&root);
            let others = ops.iter().filter(|op| op.id().account() == OTHER).count();
            assert_eq!(others, expected, "keep {keep}");
            assert_eq!(rejected, 0, "書きかけは壊れに数えない（keep {keep}）");
        }
        let _cleaned = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_bit_flip_never_panics_and_never_admits_an_altered_operation() {
        let root = temp("flip");
        let theirs = docket_with_two_writers(&root);
        let whole = fs::read(&theirs).unwrap();
        let originals: Vec<Op> = (1_u64..=3).map(|clock| create(clock, OTHER)).collect();
        let mut lcg = Lcg::seeded(11);
        for _ in 0_u32..60 {
            let index = usize::try_from(lcg.below(u64::try_from(whole.len()).unwrap())).unwrap();
            let bit = u8::try_from(lcg.below(8)).unwrap();
            fs::write(&theirs, &whole).unwrap();
            flip_bit(&theirs, index, bit).unwrap();
            let (ops, rejected) = reopen(&root);
            let others: Vec<&Op> = ops.iter().filter(|op| op.id().account() == OTHER).collect();
            assert!(
                others.iter().all(|op| originals.contains(op)),
                "index {index} bit {bit}"
            );
            assert!(others.len() + rejected <= 3, "index {index} bit {bit}");
            assert!(others.len() + rejected >= 1, "index {index} bit {bit}");
        }
        let _cleaned = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_zero_filled_tail_is_a_pending_line_until_the_writer_closes_it() {
        let root = temp("zeros");
        let theirs = docket_with_two_writers(&root);
        zero_tail(&theirs, 512).unwrap();
        let (ops, rejected) = reopen(&root);
        assert_eq!(
            ops.iter().filter(|op| op.id().account() == OTHER).count(),
            3
        );
        assert_eq!(rejected, 0);
        // 書き手が開き直すと改行を足し（own_journal）、0 の行は壊れとして数えられ、次の行は読める。
        let mut theirs_again = FileJournal::open(&root, "Team", &account(OTHER, 2)).unwrap();
        theirs_again.append(&create(9, OTHER)).unwrap();
        theirs_again.sync().unwrap();
        let (ops_after, rejected_after) = reopen(&root);
        assert_eq!(
            ops_after
                .iter()
                .filter(|op| op.id().account() == OTHER)
                .count(),
            4
        );
        assert_eq!(rejected_after, 1);
        let _cleaned = fs::remove_dir_all(&root);
    }

    #[test]
    fn an_emptied_journal_reads_as_nothing_and_a_partial_own_line_is_closed_on_reopen() {
        let root = temp("empty");
        let theirs = docket_with_two_writers(&root);
        empty(&theirs).unwrap();
        let (ops, rejected) = reopen(&root);
        assert_eq!(
            ops.iter().filter(|op| op.id().account() == OTHER).count(),
            0
        );
        assert_eq!(rejected, 0);
        // 自分のジャーナルの最後の行が途中で切れている（追記の途中で落ちた）。
        let mine = root
            .join("journal")
            .join(own_journal::file_name(ME, JournalVersion::One));
        let whole = fs::read(&mine).unwrap();
        truncate(&mine, u64::try_from(whole.len() - 7).unwrap()).unwrap();
        let mut reopened = FileJournal::open(&root, "Team", &account(ME, 1)).unwrap();
        assert_eq!(reopened.rejected_lines(), 1);
        reopened.append(&create(7, ME)).unwrap();
        reopened.sync().unwrap();
        drop(reopened);
        let read = journal_file::read_since(&mine, 0).unwrap();
        assert_eq!(read.ops().len(), 3, "切れた行の後の追記が繋がらない");
        assert_eq!(read.rejected().len(), 1);
        assert!(!read.has_partial_tail());
        let _cleaned = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_stalled_partial_tail_is_counted_after_four_refreshes_and_uncounted_when_it_grows() {
        let root = temp("stall");
        let theirs = docket_with_two_writers(&root);
        let mut mine = FileJournal::open(&root, "Team", &account(ME, 1)).unwrap();
        mine.refresh_settled().unwrap();
        let mut partial = fs::read(&theirs).unwrap();
        partial.extend_from_slice(b"v1\t9\t0000");
        fs::write(&theirs, &partial).unwrap();
        for _ in 0_u32..3 {
            mine.refresh_settled().unwrap();
            assert_eq!(mine.rejected_lines(), 0);
        }
        mine.refresh_settled().unwrap();
        assert_eq!(mine.rejected_lines(), 1);
        mine.refresh_settled().unwrap();
        assert_eq!(mine.rejected_lines(), 1, "2 度は数えない");
        // 書き手が続きを書いた: 壊れた行として読まれ、「止まった」の数えは消える（1 のまま）。
        let mut finished = fs::read(&theirs).unwrap();
        finished.extend_from_slice(b"0000\n");
        fs::write(&theirs, &finished).unwrap();
        mine.refresh_settled().unwrap();
        assert_eq!(mine.rejected_lines(), 1);
        assert!(FileJournal::open(&root, "Team", &account(ME, 1)).is_ok());
        let _cleaned = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_snapshot_pair_from_two_generations_is_discarded_and_the_ledger_is_read_in_full() {
        let root = temp("half-rename");
        docket_with_two_writers(&root);
        let me = account(ME, 1);
        let state = root.join("snapshot").join(format!("{}.v1.txt", ME.hex()));
        let index = root.join("snapshot").join(format!("{}.v1.idx", ME.hex()));
        // 1 世代目を書き、索引を取っておく。
        let mut mine = FileJournal::open(&root, "Team", &me).unwrap();
        let (first, _) = ledger_opening::open(&mut mine, me.clone());
        mine.checkpoint(&first.snapshot()).unwrap();
        mine.finish_checkpoint().unwrap();
        let old_index = fs::read(&index).unwrap();
        // 操作を 1 つ足し、開き直して（1 世代目から戻り、新しい行を読む）2 世代目を書く。
        mine.append(&create(7, ME)).unwrap();
        mine.sync().unwrap();
        drop(mine);
        let mut next = FileJournal::open(&root, "Team", &me).unwrap();
        assert!(next.is_resumed());
        let (second, _) = ledger_opening::open(&mut next, me.clone());
        next.checkpoint(&second.snapshot()).unwrap();
        next.finish_checkpoint().unwrap();
        drop(next);
        // 状態は新しく、索引だけ古い（rename の途中で落ちた形）→ 検査和が合わず捨て、全部読んで同じ台帳。
        fs::write(&index, old_index).unwrap();
        let mut damaged = FileJournal::open(&root, "Team", &me).unwrap();
        assert!(!damaged.is_resumed());
        let (ledger, failure) = ledger_opening::open(&mut damaged, me);
        assert!(failure.is_none());
        assert_eq!(ledger.docket(), second.docket());
        assert_ne!(second.docket(), first.docket());
        assert!(!state.with_extension("txt.tmp").exists());
        let _cleaned = fs::remove_dir_all(&root);
    }

    #[test]
    fn damaged_docket_keys_and_snapshot_files_fail_loudly_or_are_discarded() {
        let root = temp("files");
        docket_with_two_writers(&root);
        // スナップショットを書いてから壊す → 検査和が合わず捨てて全部読む（台帳は同じ）。
        let mut mine = FileJournal::open(&root, "Team", &account(ME, 1)).unwrap();
        let (baseline, _) = ledger_opening::open(&mut mine, account(ME, 1));
        mine.checkpoint(&baseline.snapshot()).unwrap();
        drop(mine);
        let state = root.join("snapshot").join(format!("{}.v1.txt", ME.hex()));
        let index = root.join("snapshot").join(format!("{}.v1.idx", ME.hex()));
        let mut lcg = Lcg::seeded(5);
        for path in [&state, &index] {
            let untouched = fs::read(path).unwrap();
            let length = fs::metadata(path).unwrap().len();
            flip_bit(path, usize::try_from(lcg.below(length)).unwrap(), 3).unwrap();
            let mut damaged = FileJournal::open(&root, "Team", &account(ME, 1)).unwrap();
            assert!(!damaged.is_resumed(), "{}", path.display());
            let (ledger, failure) = ledger_opening::open(&mut damaged, account(ME, 1));
            assert!(failure.is_none());
            assert_eq!(ledger.docket(), baseline.docket(), "{}", path.display());
            fs::write(path, untouched).unwrap();
        }
        // docket.v1.txt と keys/ の壊れは起動を止めて示す（黙って既定値へ落とさない・FR-014）。
        let docket = root.join("docket.v1.txt");
        let original = fs::read(&docket).unwrap();
        truncate(&docket, 3).unwrap();
        assert!(matches!(
            FileJournal::open(&root, "Team", &account(ME, 1)),
            Err(StoreFailure::BadDocketFile | StoreFailure::UnknownDocketVersion(_))
        ));
        fs::write(&docket, original).unwrap();
        let key = fs::read_dir(root.join("keys"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| !path.to_string_lossy().contains(&ME.hex()))
            .unwrap();
        empty(&key).unwrap();
        assert!(matches!(
            FileJournal::open(&root, "Team", &account(ME, 1)),
            Err(StoreFailure::BadKeyFile(_))
        ));
        assert_eq!(RecordFailure::Unfinished.describe(), "unfinished last line");
        let _cleaned = fs::remove_dir_all(&root);
    }
}
