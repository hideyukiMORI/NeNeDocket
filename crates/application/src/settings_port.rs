//! 設定のポート（ARC-003）。画面で変えた言語と配色を残す。実装はアダプタ。

use crate::settings::Settings;
use crate::settings_failure::SettingsFailure;

/// 設定を丸ごと書く。
pub trait SettingsPort {
    /// 設定を書く（設定は台帳でないので追記しない）。
    ///
    /// # Errors
    ///
    /// 書けない。
    fn save(&mut self, settings: &Settings) -> Result<(), SettingsFailure>;
}
