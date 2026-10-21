//! 操作 1 つと、ジャーナルの 1 行の相互変換（版 1 と版 2・ADR 0004 の 3・ADR 0010）。
//!
//! 形（タブ区切り・改行なし）:
//! `v1 <時計> <アカウント 16 進> <壁時計> <打ち消す操作 or -> <種類> <課題> [<項目> <値>] <検査和 8 桁>`
//! `v2 <時計> <アカウント 16 進> <壁時計> <打ち消す操作 or -> <種類> <課題> [<項目> <値>] <署名 128 桁> <検査和 8 桁>`
//! 課題は `<作ったアカウント 16 進>:<連番>`、打ち消す操作は `<時計>:<アカウント 16 進>`。
//! 添付（FR-021）は `attach <課題> <連番> <大きさ> <SHA-256 の 16 進> <名前>`（名前は逃がす）と `detach <課題> <連番>`。
//! 権限の記録（ADR 0014）は種類 `admin`、課題は `-`、続いて `<命令> <引数>…`:
//! `permissions on|off`・`allow <アカウント>`・`disallow <アカウント>`・`group <名前> <アカウント>`・`ungroup <名前> <アカウント>`・
//! `succeed <新> <旧>`・`owner <アカウント>`。
//! 検査和は、それより前の欄をタブで結んだバイト列の CRC-32。署名は `v2` から値までをタブで結んだバイト列への Ed25519 署名。

use nene_docket_core::account_id::AccountId;
use nene_docket_core::admin_command::AdminCommand;
use nene_docket_core::attachment::Attachment;
use nene_docket_core::date::Date;
use nene_docket_core::field::Field;
use nene_docket_core::field_value::FieldValue;
use nene_docket_core::hex;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::item_state::ItemState;
use nene_docket_core::logical_clock::LogicalClock;
use nene_docket_core::op::Op;
use nene_docket_core::op_id::OpId;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::public_key::PublicKey;
use nene_docket_core::signature::Signature;
use nene_docket_core::signing_key::SigningKey;
use nene_docket_core::wall_clock::WallClock;

use crate::crc32::crc32;
use crate::escape::{escape, unescape};
use crate::journal_version::JournalVersion;
use crate::record_failure::RecordFailure;
/// 欄の区切り。
const SEPARATOR: char = '\t';
/// 無い値（打ち消す操作・期限）。
const NONE: &str = "-";
/// ラベルの区切り。
const LABEL_SEPARATOR: char = ',';
/// 権限の記録の種類。
const ADMIN: &str = "admin";
/// 権限の有効・無効の値。
const ON: &str = "on";
const OFF: &str = "off";

/// 署名する文（版の印から値まで）。
fn body_of(version: JournalVersion, op: &Op) -> String {
    let mut fields = vec![
        version.tag().to_owned(),
        op.id().clock().tick().to_string(),
        op.id().account().hex(),
        op.wall().seconds().to_string(),
        op.undoes().map_or_else(|| NONE.to_owned(), encode_op_id),
    ];
    fields.extend(encode_kind(op.kind()));
    fields.join("\t")
}

/// 検査和を付けて行にする。
fn with_checksum(body: &str) -> String {
    format!("{body}{SEPARATOR}{:08x}", crc32(body.as_bytes()))
}

/// 操作を版 1 の 1 行にする（改行は付けない）。
#[must_use]
pub fn encode(op: &Op) -> String {
    with_checksum(&body_of(JournalVersion::One, op))
}

/// 操作を鍵で署名して版 2 の 1 行にする（改行は付けない）。
#[must_use]
pub fn encode_signed(op: &Op, key: &SigningKey) -> String {
    let body = body_of(JournalVersion::Two, op);
    let signature = key.sign(body.as_bytes());
    with_checksum(&format!("{body}{SEPARATOR}{}", signature.hex()))
}

/// 版 2 の行の署名が `public` の持ち主のものか。
#[must_use]
pub fn is_signed_by(op: &Op, signature: &Signature, public: &PublicKey) -> bool {
    public.verify(body_of(JournalVersion::Two, op).as_bytes(), signature)
}

/// 1 行を操作（と、版 2 なら署名）に戻す。署名が本物かはここでは見ない（[`is_signed_by`]）。
///
/// # Errors
///
/// 版・欄の数・検査和・数・語・署名の桁のどれかが読めない。
pub fn decode(line: &str) -> Result<(Op, Option<Signature>), RecordFailure> {
    let (body, checksum) = line
        .rsplit_once(SEPARATOR)
        .ok_or(RecordFailure::WrongFieldCount)?;
    let expected = u32::from_str_radix(checksum, 16).map_err(|_error| RecordFailure::BadNumber)?;
    if crc32(body.as_bytes()) != expected {
        return Err(RecordFailure::ChecksumMismatch);
    }
    let mut fields: Vec<&str> = body.split(SEPARATOR).collect();
    let version = fields
        .first()
        .and_then(|tag| JournalVersion::from_tag(tag))
        .ok_or(RecordFailure::UnknownVersion)?;
    let signature = if version.is_signed() {
        let text = fields.pop().ok_or(RecordFailure::WrongFieldCount)?;
        Some(Signature::from_hex(text).ok_or(RecordFailure::BadSignature)?)
    } else {
        None
    };
    let mut cursor = fields.iter().copied().skip(1);
    let mut next = || cursor.next().ok_or(RecordFailure::WrongFieldCount);
    let clock = LogicalClock::new(parse_number(next()?)?);
    let account = parse_account(next()?)?;
    let wall = WallClock::new(parse_number(next()?)?);
    let undoes = match next()? {
        NONE => None,
        text => Some(decode_op_id(text)?),
    };
    let kind = decode_kind(next()?, next()?, &fields)?;
    Ok((
        Op::new(OpId::new(clock, account), wall, kind).undoing_if(undoes),
        signature,
    ))
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
        OpKind::Admin(command) => {
            let mut fields = vec![ADMIN.to_owned(), NONE.to_owned()];
            fields.extend(encode_admin(command));
            fields
        }
        OpKind::Attach { item, attachment } => vec![
            "attach".to_owned(),
            encode_item(*item),
            attachment.sequence().to_string(),
            attachment.bytes().to_string(),
            attachment.digest_hex(),
            escape(attachment.name()),
        ],
        OpKind::Detach { item, sequence } => {
            vec![
                "detach".to_owned(),
                encode_item(*item),
                sequence.to_string(),
            ]
        }
    }
}

/// `attach` の欄（連番・大きさ・SHA-256・名前）。
fn decode_attachment(fields: &[&str]) -> Result<Attachment, RecordFailure> {
    let field = |index: usize| fields.get(index).copied().unwrap_or("");
    let digest = hex::decode::<32>(field(9)).ok_or(RecordFailure::BadValue)?;
    Ok(Attachment::new(
        parse_number(field(7))?,
        unescape(field(10)).ok_or(RecordFailure::BadValue)?,
        parse_number(field(8))?,
        digest,
    ))
}

/// 命令と引数。グループの名前は逃がす。
fn encode_admin(command: &AdminCommand) -> Vec<String> {
    match command {
        AdminCommand::Permissions(on) => vec![
            "permissions".to_owned(),
            (if *on { ON } else { OFF }).to_owned(),
        ],
        AdminCommand::Allow(id) => vec!["allow".to_owned(), id.hex()],
        AdminCommand::Disallow(id) => vec!["disallow".to_owned(), id.hex()],
        AdminCommand::Group(name, id) => vec!["group".to_owned(), escape(name), id.hex()],
        AdminCommand::Ungroup(name, id) => vec!["ungroup".to_owned(), escape(name), id.hex()],
        AdminCommand::Succeed(new, old) => vec!["succeed".to_owned(), new.hex(), old.hex()],
        AdminCommand::Owner(id) => vec!["owner".to_owned(), id.hex()],
    }
}

/// `admin` の行の命令。欄の数は命令で決まる（`fields[7]` が命令、その後が引数）。
fn decode_admin(fields: &[&str]) -> Result<AdminCommand, RecordFailure> {
    let verb = fields.get(7).copied().unwrap_or("");
    let expected_len = match verb {
        "permissions" | "allow" | "disallow" | "owner" => 9,
        "group" | "ungroup" | "succeed" => 10,
        _ => return Err(RecordFailure::UnknownKind),
    };
    if fields.len() != expected_len {
        return Err(RecordFailure::WrongFieldCount);
    }
    let first = fields.get(8).copied().unwrap_or("");
    let second = fields.get(9).copied().unwrap_or("");
    Ok(match verb {
        "permissions" => AdminCommand::Permissions(match first {
            ON => true,
            OFF => false,
            _ => return Err(RecordFailure::BadValue),
        }),
        "allow" => AdminCommand::Allow(parse_account(first)?),
        "disallow" => AdminCommand::Disallow(parse_account(first)?),
        "owner" => AdminCommand::Owner(parse_account(first)?),
        "group" => AdminCommand::Group(
            unescape(first).ok_or(RecordFailure::BadValue)?,
            parse_account(second)?,
        ),
        "ungroup" => AdminCommand::Ungroup(
            unescape(first).ok_or(RecordFailure::BadValue)?,
            parse_account(second)?,
        ),
        _ => AdminCommand::Succeed(parse_account(first)?, parse_account(second)?),
    })
}

/// 種類と課題の欄の後に、`set` なら項目と値が、`admin` なら命令と引数が続く。欄の総数はそれで決まる。
fn decode_kind(kind: &str, item: &str, fields: &[&str]) -> Result<OpKind, RecordFailure> {
    if kind == ADMIN {
        if item != NONE {
            return Err(RecordFailure::BadValue);
        }
        return Ok(OpKind::Admin(decode_admin(fields)?));
    }
    let item = decode_item(item)?;
    let expected_len = match kind {
        "set" => 9,
        "attach" => 11,
        "detach" => 8,
        _ => 7,
    };
    if fields.len() != expected_len {
        return Err(RecordFailure::WrongFieldCount);
    }
    match kind {
        "create" => Ok(OpKind::Create(item)),
        "delete" => Ok(OpKind::Delete(item)),
        "restore" => Ok(OpKind::Restore(item)),
        "attach" => Ok(OpKind::Attach {
            item,
            attachment: decode_attachment(fields)?,
        }),
        "detach" => Ok(OpKind::Detach {
            item,
            sequence: parse_number(fields.get(7).copied().unwrap_or(""))?,
        }),
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
/// `a\\,b,c` のように逃がした区切りを戻しながら割る（スナップショットの課題の行と共有）。
pub(crate) fn decode_labels(text: &str) -> Result<Vec<String>, RecordFailure> {
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

    /// 権限の記録の全部（ADR 0014）。グループの名前は逃がされる。
    fn every_admin() -> Vec<Op> {
        vec![
            op(20, OpKind::Admin(AdminCommand::Permissions(true))),
            op(21, OpKind::Admin(AdminCommand::Permissions(false))),
            op(22, OpKind::Admin(AdminCommand::Allow(B))),
            op(23, OpKind::Admin(AdminCommand::Disallow(B))),
            op(
                24,
                OpKind::Admin(AdminCommand::Group("q\ta,b".to_owned(), B)),
            ),
            op(25, OpKind::Admin(AdminCommand::Ungroup("qa".to_owned(), B))),
            op(26, OpKind::Admin(AdminCommand::Succeed(A, B))),
            op(27, OpKind::Admin(AdminCommand::Owner(B))),
        ]
    }

    #[test]
    fn every_admin_record_round_trips_signed_and_unsigned() {
        let key = SigningKey::from_bytes(&[3_u8; 32]);
        for original in every_admin() {
            let line = encode(&original);
            assert!(line.contains("\tadmin\t-\t"), "{line}");
            assert_eq!(decode(&line), Ok((original.clone(), None)));
            let signed = encode_signed(&original, &key);
            let (decoded, signature) = decode(&signed).unwrap();
            assert_eq!(decoded, original);
            assert!(is_signed_by(&decoded, &signature.unwrap(), &key.public()));
        }
    }

    #[test]
    fn a_bad_admin_record_is_refused_by_value_verb_count_item_or_account() {
        let good = encode(&op(20, OpKind::Admin(AdminCommand::Permissions(true))));
        let body = good.rsplit_once('\t').unwrap().0;
        let rebuilt = |text: &str| with_checksum(text);
        assert_eq!(
            decode(&rebuilt(
                &body.replace("\tpermissions\ton", "\tpermissions\tmaybe")
            )),
            Err(RecordFailure::BadValue)
        );
        assert_eq!(
            decode(&rebuilt(&body.replace("\tpermissions\ton", "\tbless\ton"))),
            Err(RecordFailure::UnknownKind)
        );
        assert_eq!(
            decode(&rebuilt(
                &body.replace("\tpermissions\ton", "\tpermissions\ton\textra")
            )),
            Err(RecordFailure::WrongFieldCount)
        );
        assert_eq!(
            decode(&rebuilt(&body.replace("\tadmin\t-\t", "\tadmin\tx\t"))),
            Err(RecordFailure::BadValue)
        );
        assert_eq!(
            decode(&rebuilt(&body.replace("\tpermissions\ton", "\tallow\tzz"))),
            Err(RecordFailure::BadAccount)
        );
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
            op(
                12,
                OpKind::Attach {
                    item,
                    attachment: Attachment::new(1, "a\tb,c.png".to_owned(), 1024, [0xab_u8; 32]),
                },
            ),
            op(13, OpKind::Detach { item, sequence: 1 }),
            op(12, OpKind::Restore(item)).undoing(OpId::new(LogicalClock::new(11), A)),
        ]
    }

    #[test]
    fn every_kind_and_value_round_trips() {
        for original in every_kind() {
            let line = encode(&original);
            assert!(!line.contains('\n'), "{line}");
            assert_eq!(decode(&line), Ok((original.clone(), None)), "{line}");
        }
    }

    #[test]
    fn signed_lines_round_trip_and_verify_only_with_the_signers_key() {
        let key = SigningKey::from_bytes(&[6_u8; 32]);
        let other = SigningKey::from_bytes(&[7_u8; 32]);
        for original in every_kind() {
            let line = encode_signed(&original, &key);
            let (op, signature) = decode(&line).unwrap();
            assert_eq!(op, original);
            let signature = signature.unwrap();
            assert!(is_signed_by(&op, &signature, &key.public()));
            assert!(!is_signed_by(&op, &signature, &other.public()));
            assert_eq!(line.split('\t').nth_back(1).map(str::len), Some(128));
        }
    }

    #[test]
    fn a_tampered_signed_line_and_an_unsigned_v2_line_are_refused() {
        let key = SigningKey::from_bytes(&[6_u8; 32]);
        let original = op(2, OpKind::Create(ItemId::new(B, 7)));
        let line = encode_signed(&original, &key);
        // 署名の後の値を変え、検査和は合わせ直す（ファイルを直接いじる人の真似）。
        let (body, _) = line.rsplit_once('\t').unwrap();
        let forged = body.replacen("\t2\t", "\t3\t", 1);
        let (op, signature) = decode(&resigned(&forged)).unwrap();
        assert!(!is_signed_by(&op, &signature.unwrap(), &key.public()));
        // 署名の欄が無い版 2 の行。
        let unsigned = encode(&original).replacen("v1", "v2", 1);
        let (unsigned_body, _) = unsigned.rsplit_once('\t').unwrap();
        assert_eq!(
            decode(&resigned(unsigned_body)),
            Err(RecordFailure::BadSignature)
        );
        let short = body.replacen(&key.sign(b"").hex()[..0], "", 1);
        assert!(decode(&resigned(&format!("{short}0"))).is_err());
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
            .map(|(op, _)| op.undoes().map(|id| id.clock().tick())),
            Ok(Some(1))
        );
        assert_eq!(
            decode(&resigned(&format!("v1\t1\t{hex}\t0\tbad\tcreate\t{item}"))),
            Err(RecordFailure::BadNumber)
        );
    }
}
