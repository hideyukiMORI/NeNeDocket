//! 持ち主が台帳の権限を変える（FR-010・ADR 0010 の 3）。`docket.v1.txt` の `permissions` と、署名した `permissions.v1.txt` を書く。
//!
//! 持ち主の無い台帳（Issue #29 より前に作ったもの）は、最初に権限を触ったアカウントが持ち主になる。

use std::path::Path;

use nene_docket_application::account::Account;
use nene_docket_application::admin_command::AdminCommand;
use nene_docket_application::permissions_file;
use nene_docket_core::permissions::Permissions;

use crate::docket_file::DocketFile;
use crate::key_file;
use crate::permissions_store;
use crate::store_failure::StoreFailure;

/// 命令を順に当て、`docket.v1.txt` と `permissions.v1.txt` を書き直す。
///
/// # Errors
///
/// 台帳が無い・持ち主でない・鍵が食い違う・いまの権限ファイルが持ち主の署名でない・読み書きできない。
pub fn apply(
    root: &Path,
    account: &Account,
    commands: &[AdminCommand],
) -> Result<Permissions, StoreFailure> {
    let mut docket = DocketFile::read_from(root)?.ok_or(StoreFailure::NoDocket)?;
    let owner = if let Some(owner) = docket.owner() {
        owner
    } else {
        docket = docket.owned_by(account.id());
        account.id()
    };
    if owner != account.id() {
        return Err(StoreFailure::NotTheOwner);
    }
    if key_file::ensure_own(root, account)? {
        return Err(StoreFailure::KeyMismatch);
    }
    let mut permissions = match permissions_store::read(root)? {
        None => Permissions::owner_only(owner),
        Some((permissions, signature)) => {
            if permissions.owner() != owner
                || !permissions_file::is_authentic(&permissions, &signature, &account.public())
            {
                return Err(StoreFailure::UnverifiedPermissions);
            }
            permissions
        }
    };
    for command in commands {
        match command {
            AdminCommand::Permissions(on) => docket = docket.with_permissions(*on),
            AdminCommand::Allow(id) => permissions = permissions.allowing(*id),
            AdminCommand::Disallow(id) => permissions = permissions.disallowing(*id),
            AdminCommand::Group(name, id) => permissions = permissions.grouped(name, *id),
            AdminCommand::Ungroup(name, id) => permissions = permissions.ungrouped(name, *id),
        }
    }
    docket.write_to(root)?;
    permissions_store::write(root, &permissions, account.key())?;
    Ok(permissions)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::signing_key::SigningKey;

    use super::*;

    const OWNER: AccountId = AccountId::new(1);
    const EDITOR: AccountId = AccountId::new(2);

    fn account(id: AccountId, seed: u8) -> Account {
        Account::new(id, format!("u{seed}"), SigningKey::from_bytes(&[seed; 32]))
    }

    fn scratch(name: &str) -> PathBuf {
        let directory =
            std::env::temp_dir().join(format!("nene-docket-store-{}-{name}", std::process::id()));
        let _ignored = fs::remove_dir_all(&directory);
        DocketFile::named("t").write_to(&directory).unwrap();
        directory
    }

    fn turned_on(root: &Path) -> Permissions {
        apply(
            root,
            &account(OWNER, 1),
            &[
                AdminCommand::Permissions(true),
                AdminCommand::Allow(EDITOR),
                AdminCommand::Group(String::from("qa"), AccountId::new(3)),
            ],
        )
        .unwrap()
    }

    #[test]
    fn the_owner_turns_permissions_on_and_edits_the_lists() {
        let root = scratch("admin");
        let permissions = turned_on(&root);
        assert!(permissions.may_edit(EDITOR));
        assert!(permissions.may_edit(AccountId::new(3)));
        let docket = DocketFile::read_from(&root).unwrap().unwrap();
        assert!(docket.permissions_on());
        assert_eq!(docket.owner(), Some(OWNER));
        let narrowed = apply(
            &root,
            &account(OWNER, 1),
            &[
                AdminCommand::Disallow(EDITOR),
                AdminCommand::Ungroup(String::from("qa"), AccountId::new(3)),
                AdminCommand::Permissions(false),
            ],
        )
        .unwrap();
        assert_eq!(narrowed, Permissions::owner_only(OWNER));
        assert!(
            !DocketFile::read_from(&root)
                .unwrap()
                .unwrap()
                .permissions_on()
        );
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn others_a_twin_key_and_a_foreign_signature_are_refused() {
        let root = scratch("admin-refused");
        let missing = root.join("missing");
        assert!(matches!(
            apply(&missing, &account(OWNER, 1), &[]),
            Err(StoreFailure::NoDocket)
        ));
        turned_on(&root);
        assert!(matches!(
            apply(
                &root,
                &account(EDITOR, 2),
                &[AdminCommand::Permissions(false)]
            ),
            Err(StoreFailure::NotTheOwner)
        ));
        assert!(matches!(
            apply(&root, &account(OWNER, 9), &[]),
            Err(StoreFailure::KeyMismatch)
        ));
        permissions_store::write(
            &root,
            &Permissions::owner_only(OWNER),
            account(EDITOR, 2).key(),
        )
        .unwrap();
        assert!(matches!(
            apply(&root, &account(OWNER, 1), &[]),
            Err(StoreFailure::UnverifiedPermissions)
        ));
        let _ignored = fs::remove_dir_all(&root);
    }
}
