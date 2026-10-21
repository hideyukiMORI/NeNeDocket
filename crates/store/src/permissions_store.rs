//! 台帳の `permissions.v1.txt` の読み書き（FR-010・ADR 0010）。形は application の `permissions_file`。

use std::fs;
use std::io;
use std::path::Path;

use nene_docket_application::permissions_file;
use nene_docket_core::permissions::Permissions;
use nene_docket_core::signature::Signature;
use nene_docket_core::signing_key::SigningKey;

use crate::store_failure::StoreFailure;

/// ファイル名。版は名前にある。
pub const FILE_NAME: &str = "permissions.v1.txt";

/// 読む。無ければ `None`。署名が本物かはここでは見ない（持ち主の公開鍵は `keys/` にある）。
///
/// # Errors
///
/// 読めない・形が違う。
pub fn read(root: &Path) -> Result<Option<(Permissions, Signature)>, StoreFailure> {
    match fs::read_to_string(root.join(FILE_NAME)) {
        Ok(text) => permissions_file::parse(&text)
            .map(Some)
            .ok_or(StoreFailure::BadPermissionsFile),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(StoreFailure::Io(error)),
    }
}

/// 持ち主の鍵で署名して書く（上書き。ジャーナルでない）。
///
/// # Errors
///
/// 書けない。
pub fn write(
    root: &Path,
    permissions: &Permissions,
    owner: &SigningKey,
) -> Result<(), StoreFailure> {
    fs::create_dir_all(root)?;
    fs::write(
        root.join(FILE_NAME),
        permissions_file::encode(permissions, owner),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;

    use super::*;

    #[test]
    fn writes_then_reads_and_reports_a_broken_file() {
        let root = std::env::temp_dir().join(format!(
            "nene-docket-store-{}-permissions",
            std::process::id()
        ));
        let _ignored = fs::remove_dir_all(&root);
        assert!(read(&root).unwrap().is_none());
        let key = SigningKey::from_bytes(&[8_u8; 32]);
        let permissions = Permissions::owner_only(AccountId::new(1)).allowing(AccountId::new(2));
        write(&root, &permissions, &key).unwrap();
        let (read_back, signature) = read(&root).unwrap().unwrap();
        assert_eq!(read_back, permissions);
        assert!(permissions_file::is_authentic(
            &read_back,
            &signature,
            &key.public()
        ));
        fs::write(root.join(FILE_NAME), "version = 1\n").unwrap();
        assert!(matches!(read(&root), Err(StoreFailure::BadPermissionsFile)));
        let _cleaned = fs::remove_dir_all(&root);
    }
}
