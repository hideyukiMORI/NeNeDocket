//! X11（Linux）の合成ルート（ARC-006）。設定・アカウント・ジャーナル・時計・刻みを結び、X11 の画面を起動する。

use nene_docket_adapters_x11::appearance;
use nene_docket_adapters_x11::locale;
use nene_docket_adapters_x11::refresh_ticker::RefreshTicker;
use nene_docket_adapters_x11::saved_account;
use nene_docket_adapters_x11::saved_settings::{self, SettingsStore};
use nene_docket_adapters_x11::system_clock::SystemClock;
use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::ports::Ports;
use nene_docket_store::file_journal::FileJournal;
use nene_docket_ui_x11::atoms;
use nene_docket_ui_x11::display::Display;
use nene_docket_ui_x11::docket_window::DocketWindow;
use nene_docket_ui_x11::event_loop;

use crate::arguments::Arguments;
use crate::launch_failure::LaunchFailure;

/// 依存を結んで走らせる。台帳のフォルダは引数、無ければ設定の最近の台帳（FR-012）。開けたら設定に覚える。
/// OS の言語と明暗は起動時に 1 度だけ観測して環境に添える（ADR 0011 / 0012）。
pub(crate) fn run(arguments: &Arguments) -> Result<(), LaunchFailure> {
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
    let display = Display::connect()?;
    let environment = display
        .environment()
        .with_language(locale::read())
        .with_appearance(appearance::read());
    let mut screen = DocketScreen::open(environment, account, ports, settings.clone());
    let mut window = DocketWindow::open(&display)?;
    let ticker = RefreshTicker::start(
        window.window(),
        atoms::REFRESH_NAME,
        settings.refresh_seconds(),
    )?;
    event_loop::run(&display, &mut window, &mut screen)?;
    drop(ticker);
    Ok(())
}
