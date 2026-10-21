//! 持ち主が台帳の権限を変える（FR-010・ADR 0010 の 3）。`docket.v1.txt` の `permissions` と、署名した `permissions.v1.txt` を書く。
//!
//! 持ち主の無い台帳（Issue #29 より前に作ったもの）は、最初に権限を触ったアカウントが持ち主になる。
//! 持ち主の交代（ADR 0013 の 4）: `Owner` は前の持ち主の鍵で署名し `handover` を残す。新しい持ち主は最初の操作
//! （台帳を開くことも含む・[`adopt`]）で署名し直す。`docket.v1.txt` の持ち主の欄が自分なら、署名が通らない
//! `permissions.v1.txt` でも中身を受け入れて署名し直す（持ち主が鍵を失くしてフォルダを手で直した後）。

use std::path::Path;

use nene_docket_application::account::Account;
use nene_docket_application::permissions_file;
use nene_docket_core::account_id::AccountId;
use nene_docket_core::admin_command::AdminCommand;
use nene_docket_core::permissions::Permissions;

use crate::docket_file::DocketFile;
use crate::key_file;
use crate::permissions_store;
use crate::store_failure::StoreFailure;

/// 命令を順に当て、`docket.v1.txt` と `permissions.v1.txt` を書き直す。
///
/// # Errors
///
/// 台帳が無い・持ち主でない・鍵が食い違う・読み書きできない。
pub fn apply(
    root: &Path,
    account: &Account,
    commands: &[AdminCommand],
) -> Result<Permissions, StoreFailure> {
    let (mut docket, mut permissions) = current(root, account)?;
    let previous = permissions.owner();
    for command in commands {
        match command {
            AdminCommand::Permissions(on) => docket = docket.with_permissions(*on),
            AdminCommand::Allow(id) => permissions = permissions.allowing(*id),
            AdminCommand::Disallow(id) => permissions = permissions.disallowing(*id),
            AdminCommand::Group(name, id) => permissions = permissions.grouped(name, *id),
            AdminCommand::Ungroup(name, id) => permissions = permissions.ungrouped(name, *id),
            AdminCommand::Succeed(new, old) => permissions = permissions.succeeding(*new, *old),
            AdminCommand::Owner(new) => {
                docket = docket.owned_by(*new);
                permissions = permissions.handed_to(*new);
            }
        }
    }
    let handover = (permissions.owner() != previous).then_some(previous);
    docket.write_to(root)?;
    permissions_store::write(root, &permissions, account.key(), handover)?;
    Ok(permissions)
}

/// 持ち主の最初の操作として、自分の鍵で署名していない `permissions.v1.txt` を署名し直す。
/// 持ち主でない・権限が無効・既に自分の署名なら何もしない。署名し直したら真。
///
/// # Errors
///
/// 台帳が無い・鍵が食い違う・読み書きできない。
pub fn adopt(root: &Path, account: &Account) -> Result<bool, StoreFailure> {
    let docket = DocketFile::read_from(root)?.ok_or(StoreFailure::NoDocket)?;
    if !docket.permissions_on() || docket.owner() != Some(account.id()) {
        return Ok(false);
    }
    let signed_by_me = permissions_store::read(root)?.is_some_and(|reading| {
        reading.handover().is_none()
            && reading.permissions().owner() == account.id()
            && permissions_file::is_authentic(&reading, &account.public())
    });
    if signed_by_me {
        return Ok(false);
    }
    apply(root, account, &[])?;
    Ok(true)
}

/// `docket.v1.txt`（持ち主が無ければ自分を書く）と、いまの権限（ファイルが無ければ持ち主だけ・持ち主の欄が
/// 自分でなければ自分に替える）。持ち主でなければ失敗。
fn current(root: &Path, account: &Account) -> Result<(DocketFile, Permissions), StoreFailure> {
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
    let permissions = permissions_store::read(root)?.map_or_else(
        || Permissions::owner_only(owner),
        |reading| adopted(reading.into_permissions(), owner),
    );
    Ok((docket, permissions))
}

/// 持ち主の欄が `owner` でない権限は、`owner` に替える（前の持ち主は編集者に残る）。
fn adopted(permissions: Permissions, owner: AccountId) -> Permissions {
    if permissions.owner() == owner {
        permissions
    } else {
        permissions.handed_to(owner)
    }
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

    /// 持ち主が引き継ぎを 1 つ書き、持ち主を `EDITOR` に替える。
    fn handed_over(root: &Path) -> Permissions {
        turned_on(root);
        key_file::ensure_own(root, &account(EDITOR, 2)).unwrap();
        apply(
            root,
            &account(OWNER, 1),
            &[
                AdminCommand::Succeed(EDITOR, AccountId::new(3)),
                AdminCommand::Owner(EDITOR),
            ],
        )
        .unwrap()
    }

    #[test]
    fn the_owner_hands_the_docket_over_signing_with_the_previous_key() {
        let root = scratch("admin-handover");
        let handed = handed_over(&root);
        assert_eq!(handed.owner(), EDITOR);
        assert_eq!(handed.editors(), [OWNER]);
        assert_eq!(handed.successions(), [(EDITOR, AccountId::new(3))]);
        assert_eq!(
            DocketFile::read_from(&root).unwrap().unwrap().owner(),
            Some(EDITOR)
        );
        let reading = permissions_store::read(&root).unwrap().unwrap();
        assert_eq!(reading.handover(), Some(OWNER));
        assert!(permissions_file::is_authentic(
            &reading,
            &account(OWNER, 1).public()
        ));
        // 前の持ち主はもう変えられない。
        assert!(matches!(
            apply(&root, &account(OWNER, 1), &[]),
            Err(StoreFailure::NotTheOwner)
        ));
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_next_owner_signs_again_on_first_use_and_the_handover_is_gone() {
        let root = scratch("admin-adopt");
        let handed = handed_over(&root);
        let next = account(EDITOR, 2);
        assert!(adopt(&root, &next).unwrap());
        let resigned = permissions_store::read(&root).unwrap().unwrap();
        assert_eq!(resigned.handover(), None);
        assert_eq!(resigned.permissions(), &handed);
        assert!(permissions_file::is_authentic(&resigned, &next.public()));
        assert!(!adopt(&root, &next).unwrap());
        assert!(!adopt(&root, &account(OWNER, 1)).unwrap());
        let _ignored = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_hand_edited_owner_adopts_the_lists_and_signs_them_on_first_use() {
        let root = scratch("admin-lost-key");
        let next = account(EDITOR, 2);
        let before = turned_on(&root);
        // 持ち主が鍵を失くし、フォルダを書ける人が docket.v1.txt の持ち主を手で直した。
        let docket = DocketFile::read_from(&root).unwrap().unwrap();
        docket.owned_by(EDITOR).write_to(&root).unwrap();
        assert!(adopt(&root, &next).unwrap());
        let resigned = permissions_store::read(&root).unwrap().unwrap();
        assert_eq!(resigned.permissions(), &before.handed_to(EDITOR));
        assert_eq!(resigned.handover(), None);
        assert!(permissions_file::is_authentic(&resigned, &next.public()));
        // 権限が無効な台帳では何もしない。
        apply(&root, &next, &[AdminCommand::Permissions(false)]).unwrap();
        assert!(!adopt(&root, &next).unwrap());
        assert!(matches!(
            adopt(&root.join("missing"), &next),
            Err(StoreFailure::NoDocket)
        ));
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
        // 持ち主の署名でないファイルは、持ち主が中身を受け入れて署名し直す（ADR 0013 の 4）。
        permissions_store::write(
            &root,
            &Permissions::owner_only(OWNER).allowing(AccountId::new(5)),
            account(EDITOR, 2).key(),
            None,
        )
        .unwrap();
        let accepted = apply(&root, &account(OWNER, 1), &[]).unwrap();
        assert!(accepted.may_edit(AccountId::new(5)));
        let reading = permissions_store::read(&root).unwrap().unwrap();
        assert!(permissions_file::is_authentic(
            &reading,
            &account(OWNER, 1).public()
        ));
        let _ignored = fs::remove_dir_all(&root);
    }
}
