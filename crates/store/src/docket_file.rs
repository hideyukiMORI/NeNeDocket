//! `docket.v1.txt`。台帳の版と名前と、持ち主と権限の有無（ARC-009・ADR 0010）。
//!
//! `owner` と `permissions` は Issue #29 で足した鍵で、版は 1 のまま（前の読み手は知らない鍵を無視する）。

use std::fs;
use std::path::Path;

use nene_docket_core::account_id::AccountId;

use crate::store_failure::StoreFailure;

/// ファイル名。版はファイル名にも出す。
pub const FILE_NAME: &str = "docket.v1.txt";
/// この版。
const VERSION: &str = "1";

/// 名前と、持ち主（作ったアカウント・無い台帳もある）と、権限を有効にしているか。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocketFile {
    name: String,
    owner: Option<AccountId>,
    permissions: bool,
}

impl DocketFile {
    /// 名前から。持ち主は無く、権限は無効。改行は空白にする（1 行 1 鍵の形を守る）。
    #[must_use]
    pub fn named(name: &str) -> Self {
        Self {
            name: name.replace(['\n', '\r'], " "),
            owner: None,
            permissions: false,
        }
    }

    /// 持ち主を置いたもの。
    #[must_use]
    pub const fn owned_by(mut self, owner: AccountId) -> Self {
        self.owner = Some(owner);
        self
    }

    /// 権限を有効（無効）にしたもの。持ち主が要る。
    #[must_use]
    pub const fn with_permissions(mut self, on: bool) -> Self {
        self.permissions = on;
        self
    }

    /// 名前。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 持ち主。
    #[must_use]
    pub const fn owner(&self) -> Option<AccountId> {
        self.owner
    }

    /// 権限を有効にしているか。
    #[must_use]
    pub const fn permissions_on(&self) -> bool {
        self.permissions
    }

    /// 書く文字列。`key = value` の行。
    #[must_use]
    pub fn text(&self) -> String {
        let head = format!("version = {VERSION}\nname = {}\n", self.name);
        match self.owner {
            Some(owner) => format!(
                "{head}owner = {}\npermissions = {}\n",
                owner.hex(),
                if self.permissions { "on" } else { "off" }
            ),
            None => head,
        }
    }

    /// 読む。版が違えば `UnknownDocketVersion`、形が違えば `BadDocketFile`（持ち主の無い `permissions = on` も）。
    ///
    /// # Errors
    ///
    /// 版が違う・形が読めない。
    pub fn parse(text: &str) -> Result<Self, StoreFailure> {
        let mut version = None;
        let mut name = None;
        let mut owner = None;
        let mut permissions = false;
        for line in text.lines() {
            let (key, value) = line.split_once('=').ok_or(StoreFailure::BadDocketFile)?;
            let value = value.trim();
            match key.trim() {
                "version" => version = Some(value.to_owned()),
                "name" => name = Some(value.to_owned()),
                "owner" => {
                    owner = Some(AccountId::from_hex(value).ok_or(StoreFailure::BadDocketFile)?);
                }
                "permissions" => {
                    permissions = match value {
                        "on" => true,
                        "off" => false,
                        _ => return Err(StoreFailure::BadDocketFile),
                    };
                }
                _ => {}
            }
        }
        let version = version.ok_or(StoreFailure::BadDocketFile)?;
        if version != VERSION {
            return Err(StoreFailure::UnknownDocketVersion(version));
        }
        if permissions && owner.is_none() {
            return Err(StoreFailure::BadDocketFile);
        }
        Ok(Self {
            name: Self::named(&name.ok_or(StoreFailure::BadDocketFile)?).name,
            owner,
            permissions,
        })
    }

    /// フォルダの `docket.v1.txt` を読む。無ければ `None`。
    ///
    /// # Errors
    ///
    /// 読めない・版が違う・形が違う。
    pub fn read_from(folder: &Path) -> Result<Option<Self>, StoreFailure> {
        match fs::read_to_string(folder.join(FILE_NAME)) {
            Ok(text) => Self::parse(&text).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(StoreFailure::Io(error)),
        }
    }

    /// フォルダに書く。作るときと、持ち主が権限を切り替えるとき（`administration`）だけ。
    ///
    /// # Errors
    ///
    /// 書けない。
    pub fn write_to(&self, folder: &Path) -> Result<(), StoreFailure> {
        fs::create_dir_all(folder)?;
        fs::write(folder.join(FILE_NAME), self.text())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: AccountId = AccountId::new(0xAB);

    #[test]
    fn text_round_trips_and_strips_newlines_from_the_name() {
        let file = DocketFile::named("team\ndocket");
        assert_eq!(file.name(), "team docket");
        assert_eq!(file.text(), "version = 1\nname = team docket\n");
        assert_eq!(DocketFile::parse(&file.text()).unwrap(), file);
        assert_eq!(
            DocketFile::parse("name = x\nversion = 1\nextra = y\n")
                .unwrap()
                .name(),
            "x"
        );
        let owned = DocketFile::named("t")
            .owned_by(OWNER)
            .with_permissions(true);
        assert_eq!(
            owned.text(),
            format!(
                "version = 1\nname = t\nowner = {}\npermissions = on\n",
                OWNER.hex()
            )
        );
        assert_eq!(DocketFile::parse(&owned.text()).unwrap(), owned);
        assert!(owned.permissions_on());
        assert_eq!(owned.owner(), Some(OWNER));
        let off = owned.with_permissions(false);
        assert!(!DocketFile::parse(&off.text()).unwrap().permissions_on());
    }

    #[test]
    fn unknown_versions_and_broken_files_are_reported() {
        assert!(matches!(
            DocketFile::parse("version = 2\nname = x\n"),
            Err(StoreFailure::UnknownDocketVersion(v)) if v == "2"
        ));
        for text in [
            "no equals",
            "name = x\n",
            "version = 1\n",
            "version = 1\nname = x\nowner = zz\n",
            "version = 1\nname = x\npermissions = on\n",
            &format!(
                "version = 1\nname = x\nowner = {}\npermissions = maybe\n",
                OWNER.hex()
            ),
        ] {
            assert!(
                matches!(DocketFile::parse(text), Err(StoreFailure::BadDocketFile)),
                "{text}"
            );
        }
    }

    #[test]
    fn reads_and_writes_a_folder() {
        let folder = std::env::temp_dir().join(format!(
            "nene-docket-store-{}-docket-file",
            std::process::id()
        ));
        let _ignored = fs::remove_dir_all(&folder);
        assert!(DocketFile::read_from(&folder).unwrap().is_none());
        DocketFile::named("t").write_to(&folder).unwrap();
        assert_eq!(
            DocketFile::read_from(&folder)
                .unwrap()
                .map(|f| f.name().to_owned()),
            Some("t".to_owned())
        );
        let _cleaned = fs::remove_dir_all(&folder);
    }
}
