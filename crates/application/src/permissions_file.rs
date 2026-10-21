//! 台帳の `permissions.v1.txt` の形（FR-010・ADR 0010）。持ち主が署名する。
//!
//! `version = 1` / `owner = <account>` / `editor = <account>`（0 行以上）/ `group = <名前>: <account>, …`（0 行以上）/
//! `signature = <16 進 128 桁>`。署名は `signature` より前の行をそのまま結んだバイト列への、持ち主の Ed25519 署名。
//! 形はここ（純関数）、読み書きは store。検証に使う持ち主の公開鍵は `keys/` から来る。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::permissions::Permissions;
use nene_docket_core::public_key::PublicKey;
use nene_docket_core::signature::Signature;
use nene_docket_core::signing_key::SigningKey;

/// この形の版。
const VERSION: &str = "1";

/// 署名する文（`signature` の行より前）。
#[must_use]
pub fn body(permissions: &Permissions) -> String {
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
    text
}

/// 持ち主の鍵で署名した、書く文字列。
#[must_use]
pub fn encode(permissions: &Permissions, owner: &SigningKey) -> String {
    let text = body(permissions);
    let signature = owner.sign(text.as_bytes());
    alloc::format!("{text}signature = {}\n", signature.hex())
}

/// 文字列から権限と署名を読む。版が違う・欄が読めない・署名が無いなら `None`。署名が本物かは [`is_authentic`] で。
#[must_use]
pub fn parse(text: &str) -> Option<(Permissions, Signature)> {
    let mut version = None;
    let mut owner = None;
    let mut editors = Vec::new();
    let mut groups = Vec::new();
    let mut signature = None;
    for line in text.lines() {
        let (key, value) = line.split_once('=')?;
        let value = value.trim();
        match key.trim() {
            "version" => version = Some(value),
            "owner" => owner = Some(AccountId::from_hex(value)?),
            "editor" => editors.push(AccountId::from_hex(value)?),
            "group" => groups.push(parse_group(value)?),
            "signature" => signature = Some(Signature::from_hex(value)?),
            _ => return None,
        }
    }
    if version != Some(VERSION) {
        return None;
    }
    Some((Permissions::new(owner?, editors, groups), signature?))
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

/// 署名が持ち主の公開鍵で通るか。持ち主の鍵は `keys/` の、`permissions.owner()` の分。
#[must_use]
pub fn is_authentic(permissions: &Permissions, signature: &Signature, owner: &PublicKey) -> bool {
    owner.verify(body(permissions).as_bytes(), signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: AccountId = AccountId::new(1);
    const A: AccountId = AccountId::new(2);
    const B: AccountId = AccountId::new(3);

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
        let text = encode(&sample(), &owner_key());
        assert!(text.starts_with("version = 1\nowner = "));
        assert!(text.contains(&alloc::format!("\ngroup = qa: {}, {}\n", A.hex(), B.hex())));
        let (permissions, signature) = parse(&text).unwrap();
        assert_eq!(permissions, sample());
        assert!(is_authentic(
            &permissions,
            &signature,
            &owner_key().public()
        ));
        assert!(!is_authentic(
            &permissions,
            &signature,
            &SigningKey::from_bytes(&[5_u8; 32]).public()
        ));
        assert!(!is_authentic(
            &permissions.allowing(B),
            &signature,
            &owner_key().public()
        ));
        assert_eq!(
            parse(&encode(&Permissions::owner_only(OWNER), &owner_key())).map(|(p, _)| p),
            Some(Permissions::owner_only(OWNER))
        );
    }

    #[test]
    fn a_missing_signature_a_bad_group_or_an_unknown_key_is_refused() {
        let good = encode(&sample(), &owner_key());
        assert_eq!(parse(&body(&sample())), None);
        assert_eq!(parse(&good.replacen("version = 1", "version = 2", 1)), None);
        assert_eq!(parse(&good.replacen("group = qa:", "group = :", 1)), None);
        assert_eq!(parse(&good.replacen("group = qa:", "group = qa", 1)), None);
        assert_eq!(parse(&good.replacen("editor = ", "editor = x", 1)), None);
        assert_eq!(parse(&alloc::format!("{good}extra = 1\n")), None);
        assert_eq!(parse("owner"), None);
    }
}
