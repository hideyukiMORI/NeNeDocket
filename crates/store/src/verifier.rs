//! 権限を有効にした台帳の読み手の検証（FR-010・ADR 0010 の 4）。署名と編集者であることを、行ごとに確かめる。

use std::collections::BTreeMap;
use std::path::Path;

use nene_docket_application::key_record::KeyRecord;
use nene_docket_application::permissions_file;
use nene_docket_core::account_id::AccountId;
use nene_docket_core::op::Op;
use nene_docket_core::permissions::Permissions;
use nene_docket_core::public_key::PublicKey;

use crate::admission_failure::AdmissionFailure;
use crate::journal_read::Entry;
use crate::journal_record;
use crate::key_file;
use crate::permissions_store;
use crate::store_failure::StoreFailure;

/// 権限と、`keys/` の公開鍵。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verifier {
    permissions: Permissions,
    keys: BTreeMap<AccountId, PublicKey>,
}

impl Verifier {
    /// 権限と鍵の記録から。
    #[must_use]
    pub fn new(permissions: Permissions, keys: &[KeyRecord]) -> Self {
        Self {
            permissions,
            keys: keys
                .iter()
                .map(|record| (record.id(), record.public()))
                .collect(),
        }
    }

    /// 台帳のフォルダから読む。`permissions.v1.txt` が無ければ持ち主だけ。あれば持ち主の鍵で本物と確かめる。
    ///
    /// # Errors
    ///
    /// 読めない・形が違う・署名が通らない・持ち主の鍵が `keys/` に無い。
    pub fn load(root: &Path, owner: AccountId, keys: &[KeyRecord]) -> Result<Self, StoreFailure> {
        let permissions = match permissions_store::read(root)? {
            None => Permissions::owner_only(owner),
            Some((permissions, signature)) => {
                let key = keys
                    .iter()
                    .find(|record| record.id() == owner)
                    .ok_or(StoreFailure::UnverifiedPermissions)?;
                if permissions.owner() != owner
                    || !permissions_file::is_authentic(&permissions, &signature, &key.public())
                {
                    return Err(StoreFailure::UnverifiedPermissions);
                }
                permissions
            }
        };
        Ok(Self::new(permissions, keys))
    }

    /// 権限。
    #[must_use]
    pub const fn permissions(&self) -> &Permissions {
        &self.permissions
    }

    /// `file_account` のジャーナルから読んだ 1 行を通すか。
    ///
    /// # Errors
    ///
    /// 署名が無い・他人の名・鍵が無い・署名が通らない・編集者でない。
    pub fn admit(
        &self,
        file_account: AccountId,
        (op, signature): Entry,
    ) -> Result<Op, AdmissionFailure> {
        let signature = signature.ok_or(AdmissionFailure::Unsigned)?;
        let account = op.id().account();
        if account != file_account {
            return Err(AdmissionFailure::ForeignAccount);
        }
        let key = self
            .keys
            .get(&account)
            .ok_or(AdmissionFailure::UnknownKey)?;
        if !journal_record::is_signed_by(&op, &signature, key) {
            return Err(AdmissionFailure::BadSignature);
        }
        if !self.permissions.may_edit(account) {
            return Err(AdmissionFailure::NotAnEditor);
        }
        Ok(op)
    }
}

/// 検証器があれば通し、無ければ（誰でも編集の台帳）そのまま通す。
///
/// # Errors
///
/// [`Verifier::admit`] と同じ。
pub fn admit(
    verifier: Option<&Verifier>,
    file_account: AccountId,
    entry: Entry,
) -> Result<Op, AdmissionFailure> {
    match verifier {
        Some(verifier) => verifier.admit(file_account, entry),
        None => Ok(entry.0),
    }
}

/// `keys/` を読み、権限が有効なら検証器を作る。
///
/// # Errors
///
/// [`key_file::read_all`] と [`Verifier::load`] の失敗。
pub fn load_if_enabled(
    root: &Path,
    owner: Option<AccountId>,
) -> Result<(Vec<KeyRecord>, Option<Verifier>), StoreFailure> {
    let keys = key_file::read_all(root)?;
    let verifier = match owner {
        Some(owner) => Some(Verifier::load(root, owner, &keys)?),
        None => None,
    };
    Ok((keys, verifier))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use nene_docket_application::account::Account;
    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::signing_key::SigningKey;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;

    const OWNER: AccountId = AccountId::new(1);
    const EDITOR: AccountId = AccountId::new(2);
    const STRANGER: AccountId = AccountId::new(3);

    fn account(id: AccountId, seed: u8) -> Account {
        Account::new(id, format!("u{seed}"), SigningKey::from_bytes(&[seed; 32]))
    }

    fn create(account: &Account) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(1), account.id()),
            WallClock::new(0),
            OpKind::Create(ItemId::new(account.id(), 1)),
        )
    }

    fn signed(account: &Account) -> Entry {
        journal_record::decode(&journal_record::encode_signed(
            &create(account),
            account.key(),
        ))
        .unwrap()
    }

    fn verifier() -> Verifier {
        let keys = [
            KeyRecord::of(&account(OWNER, 1)),
            KeyRecord::of(&account(EDITOR, 2)),
            KeyRecord::of(&account(STRANGER, 3)),
        ];
        Verifier::new(Permissions::owner_only(OWNER).allowing(EDITOR), &keys)
    }

    fn scratch(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("nene-docket-store-{}-{name}", std::process::id()));
        let _ignored = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn good_lines_pass_with_or_without_a_verifier() {
        let verifier = verifier();
        let editor = account(EDITOR, 2);
        assert_eq!(verifier.admit(EDITOR, signed(&editor)), Ok(create(&editor)));
        assert_eq!(
            verifier.admit(OWNER, signed(&account(OWNER, 1))),
            Ok(create(&account(OWNER, 1)))
        );
        assert!(verifier.permissions().may_edit(EDITOR));
        assert_eq!(
            admit(None, EDITOR, (create(&editor), None)),
            Ok(create(&editor))
        );
        assert_eq!(
            admit(Some(&verifier), EDITOR, (create(&editor), None)),
            Err(AdmissionFailure::Unsigned)
        );
    }

    #[test]
    fn each_failure_has_a_path() {
        let verifier = verifier();
        let editor = account(EDITOR, 2);
        let unknown = account(AccountId::new(9), 9);
        // 編集者の名で、別の鍵が署名した（なりすまし）。
        let impostor = Account::new(
            EDITOR,
            String::from("x"),
            SigningKey::from_bytes(&[7_u8; 32]),
        );
        let cases = [
            (
                (EDITOR, (create(&editor), None)),
                AdmissionFailure::Unsigned,
            ),
            ((OWNER, signed(&editor)), AdmissionFailure::ForeignAccount),
            (
                (unknown.id(), signed(&unknown)),
                AdmissionFailure::UnknownKey,
            ),
            ((EDITOR, signed(&impostor)), AdmissionFailure::BadSignature),
            (
                (STRANGER, signed(&account(STRANGER, 3))),
                AdmissionFailure::NotAnEditor,
            ),
        ];
        for ((file_account, entry), expected) in cases {
            assert_eq!(verifier.admit(file_account, entry), Err(expected));
            assert!(!expected.to_string().is_empty());
        }
    }

    #[test]
    fn loading_falls_back_to_owner_only_and_reads_a_signed_file() {
        let root = scratch("verifier");
        let owner = account(OWNER, 1);
        assert!(load_if_enabled(&root, None).unwrap().1.is_none());
        let (_, only) = load_if_enabled(&root, Some(OWNER)).unwrap();
        assert_eq!(only.unwrap().permissions(), &Permissions::owner_only(OWNER));
        let permissions = Permissions::owner_only(OWNER).allowing(EDITOR);
        permissions_store::write(&root, &permissions, owner.key()).unwrap();
        key_file::ensure_own(&root, &owner).unwrap();
        let (keys, loaded) = load_if_enabled(&root, Some(OWNER)).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(loaded.unwrap().permissions(), &permissions);
        let _cleaned = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn loading_refuses_a_file_without_the_owners_key_or_signature() {
        let root = scratch("verifier-refused");
        let owner = account(OWNER, 1);
        let permissions = Permissions::owner_only(OWNER).allowing(EDITOR);
        permissions_store::write(&root, &permissions, owner.key()).unwrap();
        // 持ち主の鍵が keys/ に無い。
        assert!(matches!(
            load_if_enabled(&root, Some(OWNER)),
            Err(StoreFailure::UnverifiedPermissions)
        ));
        key_file::ensure_own(&root, &owner).unwrap();
        // 持ち主でない鍵で署名し直した。
        permissions_store::write(&root, &permissions, account(EDITOR, 2).key()).unwrap();
        assert!(matches!(
            load_if_enabled(&root, Some(OWNER)),
            Err(StoreFailure::UnverifiedPermissions)
        ));
        // 持ち主の欄が違う。
        permissions_store::write(&root, &Permissions::owner_only(EDITOR), owner.key()).unwrap();
        assert!(matches!(
            load_if_enabled(&root, Some(OWNER)),
            Err(StoreFailure::UnverifiedPermissions)
        ));
        let _cleaned = std::fs::remove_dir_all(&root);
    }
}
