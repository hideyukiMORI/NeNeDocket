//! Windows の合成ルート（ARC-006・ADR 0009）。設定・アカウント・ジャーナル・時計を結び、Windows の画面を起動する。

use nene_docket_adapters_windows::appearance;
use nene_docket_adapters_windows::desktop_dpi;
use nene_docket_adapters_windows::locale;
use nene_docket_adapters_windows::saved_account;
use nene_docket_adapters_windows::saved_settings::{self, SettingsStore};
use nene_docket_adapters_windows::system_clock::SystemClock;
use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::environment::Environment;
use nene_docket_application::ports::Ports;
use nene_docket_store::file_journal::FileJournal;
use nene_docket_ui_win32::docket_window;

use crate::arguments::Arguments;
use crate::launch_failure::LaunchFailure;

/// 依存を結んで走らせる。dpi 対応を先に宣言してから dpi を読む（FR-016）。読み直しの刻みは窓のタイマー。
pub(crate) fn run(arguments: &Arguments) -> Result<(), LaunchFailure> {
    docket_window::declare_dpi_awareness();
    let settings = saved_settings::load()?;
    let folder = arguments.folder_or(settings.latest());
    let account = saved_account::load_or_create()?;
    let journal = FileJournal::open(&folder, &Arguments::docket_name(&folder), account.id())?;
    let settings = saved_settings::remember(settings, &folder)?;
    let ports = Ports::new(
        Box::new(journal),
        Box::new(SystemClock),
        Box::new(SettingsStore::in_config_directory()?),
    );
    let environment = Environment::new(desktop_dpi::read())
        .with_language(locale::read())
        .with_appearance(appearance::read());
    let screen = DocketScreen::open(environment, account, ports, settings.clone());
    docket_window::run(screen, settings.refresh_seconds())?;
    Ok(())
}
