//! 台帳の `permissions.v1.txt` の形（FR-010・ADR 0010）。持ち主が署名する。
//!
//! `version = 1` / `owner = <account>` / `editor = <account>`（0 行以上）/ `group = <名前>: <account>, …`（0 行以上）/
//! `succeeds = <新しい account>: <古い account>`（0 行以上・ADR 0013）/ `handover = <前の持ち主>`（交代の直後だけ）/
//! `signature = <16 進 128 桁>`。署名は `signature` より前の行をそのまま結んだバイト列への Ed25519 署名で、
//! 署名するのは持ち主（`handover` があれば前の持ち主）。形はここ（純関数）、読み書きは store。検証に使う公開鍵は `keys/` から来る。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::permissions::Permissions;
use nene_docket_core::public_key::PublicKey;
use nene_docket_core::signature::Signature;
use nene_docket_core::signing_key::SigningKey;

use crate::permissions_reading::PermissionsReading;

/// この形の版。
const VERSION: &str = "1";

/// 署名する文（`signature` の行より前）。`handover` があればその行も入る。
#[must_use]
pub fn body(permissions: &Permissions, handover: Option<AccountId>) -> String {
    let mut text = alloc::format!(
        "version = {VERSION}\nowner = {}\n",
        permissions.owner().hex()
    );
    for editor in permissions.editors() {
        text.push_str("editor = ");
        text.push_str(&editor.hex());
        text.push('\n');
    }
    for (name, members) in permissions.groups() {
        let listed: Vec<String> = members.iter().map(|member| member.hex()).collect();
        text.push_str("group = ");
        text.push_str(name);
        text.push_str(": ");
        text.push_str(&listed.join(", "));
        text.push('\n');
    }
    for (new, old) in permissions.successions() {
        text.push_str("succeeds = ");
        text.push_str(&new.hex());
        text.push_str(": ");
        text.push_str(&old.hex());
        text.push('\n');
    }
    if let Some(previous) = handover {
        text.push_str("handover = ");
        text.push_str(&previous.hex());
        text.push('\n');
    }
    text
}

/// 署名した、書く文字列。`signer` は持ち主（交代の直後は前の持ち主で、`handover` にその ID）。
#[must_use]
pub fn encode(
    permissions: &Permissions,
    signer: &SigningKey,
    handover: Option<AccountId>,
) -> String {
    let text = body(permissions, handover);
    let signature = signer.sign(text.as_bytes());
    alloc::format!("{text}signature = {}\n", signature.hex())
}

/// 文字列から権限と署名を読む。版が違う・欄が読めない・署名が無いなら `None`。署名が本物かは [`is_authentic`] で。
#[must_use]
pub fn parse(text: &str) -> Option<PermissionsReading> {
    let mut version = None;
    let mut owner = None;
    let mut editors = Vec::new();
    let mut groups = Vec::new();
    let mut successions = Vec::new();
    let mut handover = None;
    let mut signature = None;
    for line in text.lines() {
        let (key, value) = line.split_once('=')?;
        let value = value.trim();
        match key.trim() {
            "version" => version = Some(value),
            "owner" => owner = Some(AccountId::from_hex(value)?),
            "editor" => editors.push(AccountId::from_hex(value)?),
            "group" => groups.push(parse_group(value)?),
            "succeeds" => successions.push(parse_succession(value)?),
            "handover" => handover = Some(AccountId::from_hex(value)?),
            "signature" => signature = Some(Signature::from_hex(value)?),
            _ => return None,
        }
    }
    if version != Some(VERSION) {
        return None;
    }
    Some(PermissionsReading::new(
        Permissions::new(owner?, editors, groups).with_successions(successions),
        signature?,
        handover,
    ))
}

/// `<新しい account>: <古い account>`。
fn parse_succession(value: &str) -> Option<(AccountId, AccountId)> {
    let (new, old) = value.split_once(':')?;
    Some((
        AccountId::from_hex(new.trim())?,
        AccountId::from_hex(old.trim())?,
    ))
}

/// `<名前>: <account>, …`。名前は空でなく、`:` を含まない。
fn parse_group(value: &str) -> Option<(String, Vec<AccountId>)> {
    let (name, members) = value.split_once(':')?;
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let members = members
        .split(',')
        .map(|member| AccountId::from_hex(member.trim()))
        .collect::<Option<Vec<AccountId>>>()?;
    Some((String::from(name), members))
}

/// 読んだ権限の署名が `signer` の公開鍵で通るか。`signer` は持ち主の鍵、交代の直後は `handover` の鍵。
#[must_use]
pub fn is_authentic(reading: &PermissionsReading, signer: &PublicKey) -> bool {
    signer.verify(
        body(reading.permissions(), reading.handover()).as_bytes(),
        reading.signature(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: AccountId = AccountId::new(1);
    const A: AccountId = AccountId::new(2);
    const B: AccountId = AccountId::new(3);
    const C: AccountId = AccountId::new(4);

    fn owner_key() -> SigningKey {
        SigningKey::from_bytes(&[4_u8; 32])
    }

    fn sample() -> Permissions {
        Permissions::owner_only(OWNER)
            .allowing(A)
            .grouped("qa", B)
            .grouped("qa", A)
    }

    #[test]
    fn encode_then_parse_round_trips_and_the_signature_is_authentic() {
        let text = encode(&sample(), &owner_key(), None);
        assert!(text.starts_with("version = 1\nowner = "));
        assert!(text.contains(&alloc::format!("\ngroup = qa: {}, {}\n", A.hex(), B.hex())));
        let reading = parse(&text).unwrap();
        assert_eq!(reading.permissions(), &sample());
        assert_eq!(reading.handover(), None);
        assert!(is_authentic(&reading, &owner_key().public()));
        assert!(!is_authentic(
            &reading,
            &SigningKey::from_bytes(&[5_u8; 32]).public()
        ));
        let altered = PermissionsReading::new(
            reading.permissions().clone().allowing(B),
            *reading.signature(),
            None,
        );
        assert!(!is_authentic(&altered, &owner_key().public()));
        assert_eq!(
            parse(&encode(&Permissions::owner_only(OWNER), &owner_key(), None))
                .map(PermissionsReading::into_permissions),
            Some(Permissions::owner_only(OWNER))
        );
    }

    #[test]
    fn successions_and_a_handover_round_trip_and_the_handover_signs() {
        let previous = SigningKey::from_bytes(&[6_u8; 32]);
        let permissions = sample().succeeding(A, B);
        let text = encode(&permissions, &previous, Some(C));
        assert!(text.contains(&alloc::format!("\nsucceeds = {}: {}\n", A.hex(), B.hex())));
        assert!(text.contains(&alloc::format!("\nhandover = {}\n", C.hex())));
        let reading = parse(&text).unwrap();
        assert_eq!(reading.permissions(), &permissions);
        assert_eq!(reading.handover(), Some(C));
        assert!(is_authentic(&reading, &previous.public()));
        assert!(!is_authentic(&reading, &owner_key().public()));
        assert_eq!(
            parse(&text.replacen("succeeds = ", "succeeds = x", 1)),
            None
        );
        assert_eq!(parse(&text.replacen(": ", " ", 1)), None);
    }

    #[test]
    fn a_missing_signature_a_bad_group_or_an_unknown_key_is_refused() {
        let good = encode(&sample(), &owner_key(), None);
        assert_eq!(parse(&body(&sample(), None)), None);
        assert_eq!(parse(&good.replacen("version = 1", "version = 2", 1)), None);
        assert_eq!(parse(&good.replacen("group = qa:", "group = :", 1)), None);
        assert_eq!(parse(&good.replacen("group = qa:", "group = qa", 1)), None);
        assert_eq!(parse(&good.replacen("editor = ", "editor = x", 1)), None);
        assert_eq!(parse(&alloc::format!("{good}extra = 1\n")), None);
        assert_eq!(parse("owner"), None);
    }
}
