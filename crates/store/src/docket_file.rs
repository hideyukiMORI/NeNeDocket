//! `docket.v1.txt`。台帳の版と名前（ARC-009）。

use std::fs;
use std::path::Path;

use crate::store_failure::StoreFailure;

/// ファイル名。版はファイル名にも出す。
pub const FILE_NAME: &str = "docket.v1.txt";
/// この版。
const VERSION: &str = "1";

/// 名前だけ。版は 1 に固定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocketFile {
    name: String,
}

impl DocketFile {
    /// 名前から。改行は空白にする（1 行 1 鍵の形を守る）。
    #[must_use]
    pub fn named(name: &str) -> Self {
        Self {
            name: name.replace(['\n', '\r'], " "),
        }
    }

    /// 名前。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 書く文字列。`key = value` の 2 行。
    #[must_use]
    pub fn text(&self) -> String {
        format!("version = {VERSION}\nname = {}\n", self.name)
    }

    /// 読む。版が違えば `UnknownDocketVersion`、形が違えば `BadDocketFile`。
    ///
    /// # Errors
    ///
    /// 版が違う・形が読めない。
    pub fn parse(text: &str) -> Result<Self, StoreFailure> {
        let mut version = None;
        let mut name = None;
        for line in text.lines() {
            let (key, value) = line.split_once('=').ok_or(StoreFailure::BadDocketFile)?;
            match key.trim() {
                "version" => version = Some(value.trim().to_owned()),
                "name" => name = Some(value.trim().to_owned()),
                _ => {}
            }
        }
        let version = version.ok_or(StoreFailure::BadDocketFile)?;
        if version != VERSION {
            return Err(StoreFailure::UnknownDocketVersion(version));
        }
        Ok(Self::named(&name.ok_or(StoreFailure::BadDocketFile)?))
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

    /// フォルダに書く（無いときだけ呼ぶ。上書きは版を変える判断なので別）。
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
    }

    #[test]
    fn unknown_versions_and_broken_files_are_reported() {
        assert!(matches!(
            DocketFile::parse("version = 2\nname = x\n"),
            Err(StoreFailure::UnknownDocketVersion(v)) if v == "2"
        ));
        assert!(matches!(
            DocketFile::parse("no equals"),
            Err(StoreFailure::BadDocketFile)
        ));
        assert!(matches!(
            DocketFile::parse("name = x\n"),
            Err(StoreFailure::BadDocketFile)
        ));
        assert!(matches!(
            DocketFile::parse("version = 1\n"),
            Err(StoreFailure::BadDocketFile)
        ));
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
