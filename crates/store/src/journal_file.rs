//! ジャーナルのファイルを、位置から伸びた分だけ読む。

use std::fs::File;
use std::io::{BufRead as _, BufReader, Read as _, Seek as _, SeekFrom};
use std::path::Path;

use crate::journal_read::{Entry, JournalRead};
use crate::journal_record;
use crate::store_failure::StoreFailure;

/// `offset` から末尾まで読み、完全な行を操作にする。改行の無い末尾は読まずに残す。
/// 壊れた行は飛ばして位置と理由を返す（黙って直さない・ADR 0004 の 9）。
///
/// # Errors
///
/// ファイルを開けない・読めない。無いファイルも失敗（呼ぶ側が列挙したものだけを読む）。
pub fn read_since(path: &Path, offset: u64) -> Result<JournalRead, StoreFailure> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = Vec::new();
    BufReader::new(file).read_to_end(&mut bytes)?;
    Ok(parse(&bytes, offset))
}

/// バイト列を行に割る。`base` はその先頭のファイル内の位置。
#[must_use]
pub fn parse(bytes: &[u8], base: u64) -> JournalRead {
    let mut entries = Vec::new();
    let mut offsets = Vec::new();
    let mut rejected = Vec::new();
    let mut position = base;
    let mut rest = bytes;
    while let Some(end) = rest.iter().position(|&byte| byte == b'\n') {
        let (line, after) = rest.split_at(end);
        let text = String::from_utf8_lossy(line);
        match journal_record::decode(text.trim_end_matches('\r')) {
            Ok(entry) => {
                entries.push(entry);
                offsets.push(position);
            }
            Err(failure) => rejected.push((position, failure)),
        }
        position = position
            .saturating_add(u64::try_from(end).unwrap_or(0))
            .saturating_add(1);
        rest = after.get(1..).unwrap_or(&[]);
    }
    JournalRead::new((entries, offsets), position, !rest.is_empty(), rejected)
}

/// 1 行だけを位置から読む（索引から引く・ADR 0015）。位置が行の先頭でない・読めない行なら失敗。
///
/// # Errors
///
/// ファイルを開けない・読めない・その位置に完全な行が無い・行が読めない。
pub fn read_line_at(path: &Path, offset: u64) -> Result<Entry, StoreFailure> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut line = Vec::new();
    BufReader::new(file).read_until(b'\n', &mut line)?;
    if line.last() != Some(&b'\n') {
        return Err(StoreFailure::BadIndex);
    }
    let text = String::from_utf8_lossy(&line);
    journal_record::decode(text.trim_end_matches(['\n', '\r']))
        .map_err(|_failure| StoreFailure::BadIndex)
}

/// 末尾の `bytes` バイトの中の完全な行を読む（新しい操作から順に見るため・ADR 0015）。
/// 先頭の切れた行は飛ばす（ファイルの先頭から読んだときを除く）。
///
/// # Errors
///
/// ファイルを開けない・読めない。
pub fn read_tail(path: &Path, bytes: u64) -> Result<JournalRead, StoreFailure> {
    let mut file = File::open(path)?;
    let length = file.metadata()?.len();
    let start = length.saturating_sub(bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut buffer = Vec::new();
    BufReader::new(file).read_to_end(&mut buffer)?;
    if start == 0 {
        return Ok(parse(&buffer, 0));
    }
    let Some(first_break) = buffer.iter().position(|&byte| byte == b'\n') else {
        return Ok(parse(&[], length));
    };
    let base = start
        .saturating_add(u64::try_from(first_break).unwrap_or(0))
        .saturating_add(1);
    Ok(parse(
        buffer.get(first_break.saturating_add(1)..).unwrap_or(&[]),
        base,
    ))
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op::Op;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;
    use crate::record_failure::RecordFailure;

    fn op(clock: u64) -> Op {
        let account = AccountId::new(5);
        Op::new(
            OpId::new(LogicalClock::new(clock), account),
            WallClock::new(0),
            OpKind::Create(ItemId::new(account, clock.try_into().unwrap())),
        )
    }

    #[test]
    fn complete_lines_are_read_and_a_partial_tail_is_left_for_next_time() {
        let first = journal_record::encode(&op(1));
        let second = journal_record::encode(&op(2));
        let text = format!("{first}\n{second}\n{}", &second[..10]);
        let read = parse(text.as_bytes(), 0);
        assert_eq!(read.ops(), [op(1), op(2)]);
        assert!(read.has_partial_tail());
        assert_eq!(
            read.next_offset(),
            u64::try_from(first.len() + second.len() + 2).unwrap()
        );
        assert!(read.rejected().is_empty());
        let again = parse(
            &text.as_bytes()[usize::try_from(read.next_offset()).unwrap()..],
            read.next_offset(),
        );
        assert!(again.entries().is_empty());
        assert!(again.has_partial_tail());
    }

    #[test]
    fn a_broken_line_in_the_middle_is_reported_with_its_position() {
        let first = journal_record::encode(&op(1));
        let text = format!("{first}\ngarbage\r\n{}\n", journal_record::encode(&op(3)));
        let read = parse(text.as_bytes(), 100);
        assert_eq!(read.entries().len(), 2);
        assert_eq!(
            read.rejected(),
            [(
                100 + u64::try_from(first.len()).unwrap() + 1,
                RecordFailure::WrongFieldCount
            )]
        );
        assert!(!read.has_partial_tail());
        assert_eq!(parse(b"", 7).next_offset(), 7);
    }

    #[test]
    fn read_since_reads_a_real_file_from_an_offset() {
        let directory = std::env::temp_dir().join(format!(
            "nene-docket-store-{}-read-since",
            std::process::id()
        ));
        let _ignored = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("a.v1.log");
        let first = journal_record::encode(&op(1));
        std::fs::write(
            &path,
            format!("{first}\n{}\n", journal_record::encode(&op(2))),
        )
        .unwrap();
        let all = read_since(&path, 0).unwrap();
        assert_eq!(all.entries().len(), 2);
        let tail = read_since(&path, all.next_offset()).unwrap();
        assert!(tail.entries().is_empty());
        let from_second = read_since(&path, u64::try_from(first.len() + 1).unwrap()).unwrap();
        assert_eq!(from_second.ops(), [op(2)]);
        assert!(read_since(&directory.join("missing.v1.log"), 0).is_err());
        let _cleaned = std::fs::remove_dir_all(&directory);
    }
}
