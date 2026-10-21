//! 操作 1 つと、ジャーナルの 1 行の相互変換（版 1・ADR 0004 の 3）。
//!
//! 形（タブ区切り・改行なし）:
//! `v1 <時計> <アカウント 16 進> <壁時計> <打ち消す操作 or -> <種類> <課題> [<項目> <値>] <検査和 8 桁>`
//! 課題は `<作ったアカウント 16 進>:<連番>`、打ち消す操作は `<時計>:<アカウント 16 進>`。
//! 検査和は、それより前の欄をタブで結んだバイト列の CRC-32。

use nene_docket_core::account_id::AccountId;
use nene_docket_core::date::Date;
use nene_docket_core::field::Field;
use nene_docket_core::field_value::FieldValue;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::item_state::ItemState;
use nene_docket_core::logical_clock::LogicalClock;
use nene_docket_core::op::Op;
use nene_docket_core::op_id::OpId;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::wall_clock::WallClock;

use crate::crc32::crc32;
use crate::escape::{escape, unescape};
use crate::record_failure::RecordFailure;

/// 版の印。
const VERSION: &str = "v1";
/// 欄の区切り。
const SEPARATOR: char = '\t';
/// 無い値（打ち消す操作・期限）。
const NONE: &str = "-";
/// ラベルの区切り。
const LABEL_SEPARATOR: char = ',';

/// 操作を 1 行にする（改行は付けない）。
#[must_use]
pub fn encode(op: &Op) -> String {
    let mut fields = vec![
        VERSION.to_owned(),
        op.id().clock().tick().to_string(),
        op.id().account().hex(),
        op.wall().seconds().to_string(),
        op.undoes().map_or_else(|| NONE.to_owned(), encode_op_id),
    ];
    fields.extend(encode_kind(op.kind()));
    let body = fields.join("\t");
    format!("{body}{SEPARATOR}{:08x}", crc32(body.as_bytes()))
}

/// 1 行を操作に戻す。
///
/// # Errors
///
/// 版・欄の数・検査和・数・語のどれかが読めない。
pub fn decode(line: &str) -> Result<Op, RecordFailure> {
    let (body, checksum) = line
        .rsplit_once(SEPARATOR)
        .ok_or(RecordFailure::WrongFieldCount)?;
    let expected = u32::from_str_radix(checksum, 16).map_err(|_error| RecordFailure::BadNumber)?;
    if crc32(body.as_bytes()) != expected {
        return Err(RecordFailure::ChecksumMismatch);
    }
    let fields: Vec<&str> = body.split(SEPARATOR).collect();
    let mut cursor = fields.iter().copied();
    let mut next = || cursor.next().ok_or(RecordFailure::WrongFieldCount);
    if next()? != VERSION {
        return Err(RecordFailure::UnknownVersion);
    }
    let clock = LogicalClock::new(parse_number(next()?)?);
    let account = parse_account(next()?)?;
    let wall = WallClock::new(parse_number(next()?)?);
    let undoes = match next()? {
        NONE => None,
        text => Some(decode_op_id(text)?),
    };
    let kind = decode_kind(next()?, next()?, &fields)?;
    Ok(Op::new(OpId::new(clock, account), wall, kind).undoing_if(undoes))
}

fn encode_op_id(id: OpId) -> String {
    format!("{}:{}", id.clock().tick(), id.account().hex())
}

fn decode_op_id(text: &str) -> Result<OpId, RecordFailure> {
    let (clock, account) = text.split_once(':').ok_or(RecordFailure::BadNumber)?;
    Ok(OpId::new(
        LogicalClock::new(parse_number(clock)?),
        parse_account(account)?,
    ))
}

fn encode_item(item: ItemId) -> String {
    format!("{}:{}", item.creator().hex(), item.sequence())
}

fn decode_item(text: &str) -> Result<ItemId, RecordFailure> {
    let (creator, sequence) = text.split_once(':').ok_or(RecordFailure::BadNumber)?;
    Ok(ItemId::new(
        parse_account(creator)?,
        parse_number(sequence)?,
    ))
}

fn encode_kind(kind: &OpKind) -> Vec<String> {
    match kind {
        OpKind::Create(item) => vec!["create".to_owned(), encode_item(*item)],
        OpKind::Delete(item) => vec!["delete".to_owned(), encode_item(*item)],
        OpKind::Restore(item) => vec!["restore".to_owned(), encode_item(*item)],
        OpKind::Set { item, value } => vec![
            "set".to_owned(),
            encode_item(*item),
            value.field().keyword().to_owned(),
            encode_value(value),
        ],
    }
}

/// 種類と課題の欄の後に、`set` なら項目と値が続く。欄の総数はそれで決まる。
fn decode_kind(kind: &str, item: &str, fields: &[&str]) -> Result<OpKind, RecordFailure> {
    let item = decode_item(item)?;
    let expected_len = if kind == "set" { 9 } else { 7 };
    if fields.len() != expected_len {
        return Err(RecordFailure::WrongFieldCount);
    }
    match kind {
        "create" => Ok(OpKind::Create(item)),
        "delete" => Ok(OpKind::Delete(item)),
        "restore" => Ok(OpKind::Restore(item)),
        "set" => {
            let field = Field::from_keyword(fields.get(7).copied().unwrap_or(""))
                .ok_or(RecordFailure::UnknownField)?;
            let value = decode_value(field, fields.get(8).copied().unwrap_or(""))?;
            Ok(OpKind::Set { item, value })
        }
        _ => Err(RecordFailure::UnknownKind),
    }
}

fn encode_value(value: &FieldValue) -> String {
    match value {
        FieldValue::Title(text) | FieldValue::Body(text) | FieldValue::Assignee(text) => {
            escape(text)
        }
        FieldValue::State(state) => state.keyword().to_owned(),
        FieldValue::Due(due) => due.map_or_else(|| NONE.to_owned(), Date::text),
        FieldValue::Labels(labels) => labels
            .iter()
            .map(|label| escape(label))
            .collect::<Vec<String>>()
            .join(","),
        FieldValue::Rank(rank) => rank.to_string(),
    }
}

fn decode_value(field: Field, text: &str) -> Result<FieldValue, RecordFailure> {
    let unescaped = || unescape(text).ok_or(RecordFailure::BadValue);
    Ok(match field {
        Field::Title => FieldValue::Title(unescaped()?),
        Field::Body => FieldValue::Body(unescaped()?),
        Field::Assignee => FieldValue::Assignee(unescaped()?),
        Field::State => {
            FieldValue::State(ItemState::from_keyword(text).ok_or(RecordFailure::BadValue)?)
        }
        Field::Due if text == NONE => FieldValue::Due(None),
        Field::Due => FieldValue::Due(Some(Date::parse(text).ok_or(RecordFailure::BadValue)?)),
        Field::Labels if text.is_empty() => FieldValue::Labels(Vec::new()),
        Field::Labels => FieldValue::Labels(decode_labels(text)?),
        Field::Rank => FieldValue::Rank(parse_number(text)?),
    })
}

/// `,` で区切る。逃がした `\,` は区切らない。
fn decode_labels(text: &str) -> Result<Vec<String>, RecordFailure> {
    let mut labels = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in text.chars() {
        if escaped {
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            current.push(character);
            escaped = true;
        } else if character == LABEL_SEPARATOR {
            labels.push(unescape(&current).ok_or(RecordFailure::BadValue)?);
            current.clear();
        } else {
            current.push(character);
        }
    }
    labels.push(unescape(&current).ok_or(RecordFailure::BadValue)?);
    Ok(labels)
}

fn parse_number<N: core::str::FromStr>(text: &str) -> Result<N, RecordFailure> {
    text.parse().map_err(|_error| RecordFailure::BadNumber)
}

fn parse_account(text: &str) -> Result<AccountId, RecordFailure> {
    AccountId::from_hex(text).ok_or(RecordFailure::BadAccount)
}

/// `undoing` を `Option` で付ける小さな橋。
trait UndoingIf {
    fn undoing_if(self, target: Option<OpId>) -> Self;
}

impl UndoingIf for Op {
    fn undoing_if(self, target: Option<OpId>) -> Self {
        match target {
            Some(id) => self.undoing(id),
            None => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: AccountId = AccountId::new(0xA1);
    const B: AccountId = AccountId::new(0xB2);

    fn op(clock: u64, kind: OpKind) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), A),
            WallClock::new(1_700_000_000),
            kind,
        )
    }

    fn every_kind() -> Vec<Op> {
        let item = ItemId::new(B, 7);
        let set = |value: FieldValue| OpKind::Set { item, value };
        vec![
            op(1, OpKind::Create(item)),
            op(
                2,
                set(FieldValue::Title(
                    "題\t名\n改行 \\ 逃がし,カンマ".to_owned(),
                )),
            ),
            op(3, set(FieldValue::Body(String::new()))),
            op(4, set(FieldValue::State(ItemState::InProgress))),
            op(5, set(FieldValue::Assignee("mori".to_owned()))),
            op(6, set(FieldValue::Due(Date::new(2026, 12, 31)))),
            op(7, set(FieldValue::Due(None))),
            op(
                8,
                set(FieldValue::Labels(vec![
                    "core".to_owned(),
                    "a,b".to_owned(),
                ])),
            ),
            op(9, set(FieldValue::Labels(Vec::new()))),
            op(10, set(FieldValue::Rank(-42))),
            op(11, OpKind::Delete(item)),
            op(12, OpKind::Restore(item)).undoing(OpId::new(LogicalClock::new(11), A)),
        ]
    }

    #[test]
    fn every_kind_and_value_round_trips() {
        for original in every_kind() {
            let line = encode(&original);
            assert!(!line.contains('\n'), "{line}");
            assert_eq!(decode(&line).as_ref(), Ok(&original), "{line}");
        }
    }

    #[test]
    fn the_line_has_the_documented_shape() {
        let line = encode(&op(2, OpKind::Create(ItemId::new(B, 7))));
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 8);
        assert_eq!(fields.first(), Some(&"v1"));
        assert_eq!(fields.get(1), Some(&"2"));
        assert_eq!(fields.get(2).map(|f| f.len()), Some(32));
        assert_eq!(fields.get(4), Some(&"-"));
        assert_eq!(fields.get(5), Some(&"create"));
        assert_eq!(fields.get(7).map(|f| f.len()), Some(8));
    }

    #[test]
    fn corruption_is_reported_not_guessed() {
        let line = encode(&op(
            2,
            OpKind::Set {
                item: ItemId::new(B, 7),
                value: FieldValue::State(ItemState::Done),
            },
        ));
        let flipped = line.replacen("done", "dome", 1);
        assert_eq!(decode(&flipped), Err(RecordFailure::ChecksumMismatch));
        let (body, _) = line.rsplit_once('\t').unwrap();
        let truncated = &body[..body.len().saturating_sub(3)];
        assert!(decode(truncated).is_err());
        assert_eq!(decode("v2\tx"), Err(RecordFailure::BadNumber));
        assert_eq!(decode(""), Err(RecordFailure::WrongFieldCount));
    }

    fn resigned(body: &str) -> String {
        format!("{body}\t{:08x}", crc32(body.as_bytes()))
    }

    #[test]
    fn each_failure_kind_has_a_path() {
        let hex = A.hex();
        let item = format!("{hex}:1");
        assert_eq!(
            decode(&resigned(&format!("v9\t1\t{hex}\t0\t-\tcreate\t{item}"))),
            Err(RecordFailure::UnknownVersion)
        );
        assert_eq!(
            decode(&resigned(&format!("v1\tx\t{hex}\t0\t-\tcreate\t{item}"))),
            Err(RecordFailure::BadNumber)
        );
        assert_eq!(
            decode(&resigned(&format!("v1\t1\tzz\t0\t-\tcreate\t{item}"))),
            Err(RecordFailure::BadAccount)
        );
        assert_eq!(
            decode(&resigned(&format!("v1\t1\t{hex}\t0\t-\tsplit\t{item}"))),
            Err(RecordFailure::UnknownKind)
        );
    }

    #[test]
    fn each_value_failure_has_a_path() {
        let hex = A.hex();
        let item = format!("{hex}:1");
        assert_eq!(
            decode(&resigned(&format!(
                "v1\t1\t{hex}\t0\t-\tset\t{item}\towner\tx"
            ))),
            Err(RecordFailure::UnknownField)
        );
        assert_eq!(
            decode(&resigned(&format!(
                "v1\t1\t{hex}\t0\t-\tset\t{item}\tdue\t2026-13-01"
            ))),
            Err(RecordFailure::BadValue)
        );
        assert_eq!(
            decode(&resigned(&format!(
                "v1\t1\t{hex}\t0\t-\tset\t{item}\ttitle\ta\\x"
            ))),
            Err(RecordFailure::BadValue)
        );
        assert_eq!(
            decode(&resigned(&format!(
                "v1\t1\t{hex}\t0\t-\tcreate\t{item}\textra"
            ))),
            Err(RecordFailure::WrongFieldCount)
        );
        assert_eq!(
            decode(&resigned(&format!(
                "v1\t1\t{hex}\t0\t1:{hex}\tcreate\t{item}"
            )))
            .map(|op| op.undoes().map(|id| id.clock().tick())),
            Ok(Some(1))
        );
        assert_eq!(
            decode(&resigned(&format!("v1\t1\t{hex}\t0\tbad\tcreate\t{item}"))),
            Err(RecordFailure::BadNumber)
        );
    }
}
