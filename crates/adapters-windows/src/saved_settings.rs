//! 設定ファイル（FR-012）の読み書き。設定フォルダ（`%APPDATA%`）の `settings.v4.txt`。形は application の `settings_file`。X11 版と同じ手続き。
//!
//! 版 4 が無ければ版 3（`settings.v3.txt`）、版 2、版 1 の順に読む（無い鍵は既定で補う）。書くのは版 4 だけで、前の版は消さない（ARC-009 の移行）。

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use nene_docket_application::settings::Settings;
use nene_docket_application::settings_failure::SettingsFailure;
use nene_docket_application::settings_file;
use nene_docket_application::settings_port::SettingsPort;

use crate::adapter_failure::AdapterFailure;
use crate::config_directory;

/// いま書く版のファイル名。版は名前にある（ARC-009）。
const FILE_NAME: &str = "settings.v4.txt";
/// 読めるだけの前の版のファイル名（新しい順）。
const PREVIOUS_FILE_NAMES: [&str; 3] = ["settings.v3.txt", "settings.v2.txt", "settings.v1.txt"];

/// 設定を読む。版 4、無ければ版 3・2・1 の順、どれも無ければ初期値。読めない形なら失敗（黙って既定値へ落とさない・FR-014）。
///
/// # Errors
///
/// 設定フォルダが決められない・ファイルが読めない形・読めない。
pub fn load() -> Result<Settings, AdapterFailure> {
    let mut paths = vec![config_directory::file_path(FILE_NAME)?];
    for name in PREVIOUS_FILE_NAMES {
        paths.push(config_directory::file_path(name)?);
    }
    load_chain(&paths)
}

/// 開いた台帳を設定の先頭に置いて書く。フォルダは絶対の道にする（相対の道は起動した場所に依る）。
///
/// # Errors
///
/// フォルダの道を解決できない・設定フォルダが決められない・書けない。
pub fn remember(settings: Settings, folder: &Path) -> Result<Settings, AdapterFailure> {
    remember_at(&config_directory::file_path(FILE_NAME)?, settings, folder)
}

/// 画面が変えた設定（言語・配色）を書く、application のポートの実装。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsStore {
    path: PathBuf,
}

impl SettingsStore {
    /// 設定フォルダの版 2 のファイルへ。
    ///
    /// # Errors
    ///
    /// 設定フォルダが決められない。
    pub fn in_config_directory() -> Result<Self, AdapterFailure> {
        Ok(Self {
            path: config_directory::file_path(FILE_NAME)?,
        })
    }
}

impl SettingsPort for SettingsStore {
    fn save(&mut self, settings: &Settings) -> Result<(), SettingsFailure> {
        write_to(&self.path, settings).map_err(|_failure| SettingsFailure::Unwritable)
    }
}

/// 新しい版から順に、在る最初のファイルを読む。どれも無ければ初期値。
fn load_chain(paths: &[PathBuf]) -> Result<Settings, AdapterFailure> {
    for path in paths {
        match fs::read_to_string(path) {
            Ok(text) => {
                return settings_file::parse(&text)
                    .ok_or_else(|| AdapterFailure::BadSettingsFile(path.clone()));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(AdapterFailure::Io(error)),
        }
    }
    Ok(Settings::initial())
}

fn remember_at(path: &Path, settings: Settings, folder: &Path) -> Result<Settings, AdapterFailure> {
    let absolute = fs::canonicalize(folder)?;
    let remembered = settings.opened(&absolute.to_string_lossy());
    write_to(path, &remembered)?;
    Ok(remembered)
}

/// 設定は台帳でないので追記せず、丸ごと置き換える。
fn write_to(path: &Path, settings: &Settings) -> Result<(), AdapterFailure> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    fs::write(path, settings_file::encode(settings))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use nene_docket_core::language::Language;
    use nene_docket_core::language_choice::LanguageChoice;

    use super::*;

    fn temp(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("nene-docket-saved-settings-{name}"));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn missing_is_initial_and_remember_writes_the_absolute_folder_first() {
        let root = temp("remember");
        let path = root.join("config").join(FILE_NAME);
        let chain = [path.clone(), root.join("config").join("settings.v2.txt")];
        assert_eq!(load_chain(&chain).unwrap(), Settings::initial());
        let docket = root.join("docket");
        fs::create_dir_all(&docket).unwrap();
        let remembered = remember_at(&path, Settings::initial(), &docket).unwrap();
        assert!(remembered.latest().is_some_and(|folder| folder.ends_with("docket")
            && Path::new(folder).is_absolute()));
        assert_eq!(load_chain(&chain).unwrap(), remembered);
        assert!(remember_at(&path, remembered, &root.join("missing")).is_err());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn older_versions_are_read_in_order_and_the_store_writes_version_four() {
        let root = temp("migrate");
        let path = root.join(FILE_NAME);
        let version_two = root.join("settings.v2.txt");
        let previous = root.join("settings.v1.txt");
        let chain = [path.clone(), version_two.clone(), previous.clone()];
        fs::write(
            &previous,
            "version = 1\nrefresh_seconds = 5\nrecent = /old\n",
        )
        .unwrap();
        let loaded = load_chain(&chain).unwrap();
        assert_eq!(loaded.refresh_seconds(), 5);
        assert_eq!(loaded.latest(), Some("/old"));
        assert_eq!(loaded.language(), LanguageChoice::System);
        let mut store = SettingsStore { path: path.clone() };
        let changed = loaded.with_language(LanguageChoice::Fixed(Language::Japanese));
        assert_eq!(store.save(&changed), Ok(()));
        assert!(
            fs::read_to_string(&path)
                .unwrap()
                .starts_with("version = 4\n")
        );
        assert!(previous.exists());
        assert_eq!(load_chain(&chain).unwrap(), changed);
        // 版 3 が無く版 2 があれば版 2 が版 1 より先。
        fs::remove_file(&path).unwrap();
        fs::write(
            &version_two,
            "version = 2\nrefresh_seconds = 7\nlanguage = ko\ntheme = system\n",
        )
        .unwrap();
        assert_eq!(load_chain(&chain).unwrap().refresh_seconds(), 7);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_broken_file_is_refused_not_replaced_and_an_unwritable_store_fails() {
        let root = temp("broken");
        let path = root.join(FILE_NAME);
        fs::write(&path, "version = 9\n").unwrap();
        assert!(matches!(
            load_chain(std::slice::from_ref(&path)),
            Err(AdapterFailure::BadSettingsFile(_))
        ));
        let mut store = SettingsStore {
            path: root.join("missing-dir-file").join("x").join(FILE_NAME),
        };
        fs::write(root.join("missing-dir-file"), "a file, not a directory").unwrap();
        assert_eq!(
            store.save(&Settings::initial()),
            Err(SettingsFailure::Unwritable)
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
