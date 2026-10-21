//! 設定ファイル（FR-012）の読み書き。設定フォルダの `settings.v1.txt`。形は application の `settings_file`。

use std::fs;
use std::io;
use std::path::Path;

use nene_docket_application::settings::Settings;
use nene_docket_application::settings_file;

use crate::adapter_failure::AdapterFailure;
use crate::config_directory;

/// ファイル名。版は名前にある（ARC-009）。
const FILE_NAME: &str = "settings.v1.txt";

/// 設定を読む。無ければ初期値。読めない形なら失敗（黙って既定値へ落とさない・FR-014）。
///
/// # Errors
///
/// 設定フォルダが決められない・ファイルが読めない形・読めない。
pub fn load() -> Result<Settings, AdapterFailure> {
    load_from(&config_directory::file_path(FILE_NAME)?)
}

/// 開いた台帳を設定の先頭に置いて書く。フォルダは絶対の道にする（相対の道は起動した場所に依る）。
///
/// # Errors
///
/// フォルダの道を解決できない・設定フォルダが決められない・書けない。
pub fn remember(settings: Settings, folder: &Path) -> Result<Settings, AdapterFailure> {
    remember_at(&config_directory::file_path(FILE_NAME)?, settings, folder)
}

fn load_from(path: &Path) -> Result<Settings, AdapterFailure> {
    match fs::read_to_string(path) {
        Ok(text) => settings_file::parse(&text)
            .ok_or_else(|| AdapterFailure::BadSettingsFile(path.to_path_buf())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Settings::initial()),
        Err(error) => Err(AdapterFailure::Io(error)),
    }
}

fn remember_at(path: &Path, settings: Settings, folder: &Path) -> Result<Settings, AdapterFailure> {
    let absolute = fs::canonicalize(folder)?;
    let remembered = settings.opened(&absolute.to_string_lossy());
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    // 設定は台帳でないので追記せず、丸ごと置き換える。
    fs::write(path, settings_file::encode(&remembered))?;
    Ok(remembered)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> std::path::PathBuf {
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
        assert_eq!(load_from(&path).unwrap(), Settings::initial());
        let docket = root.join("docket");
        fs::create_dir_all(&docket).unwrap();
        let remembered = remember_at(&path, Settings::initial(), &docket).unwrap();
        assert!(
            remembered
                .latest()
                .is_some_and(|folder| folder.ends_with("docket") && Path::new(folder).is_absolute())
        );
        assert_eq!(load_from(&path).unwrap(), remembered);
        assert!(remember_at(&path, remembered, &root.join("missing")).is_err());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_broken_file_is_refused_not_replaced() {
        let root = temp("broken");
        let path = root.join(FILE_NAME);
        fs::write(&path, "version = 9\n").unwrap();
        assert!(matches!(
            load_from(&path),
            Err(AdapterFailure::BadSettingsFile(_))
        ));
        fs::remove_dir_all(&root).unwrap();
    }
}
