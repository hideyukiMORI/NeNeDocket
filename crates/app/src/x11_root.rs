//! X11（Linux）の合成ルート（ARC-006）。アカウント・ジャーナル・時計・刻みを結び、X11 の画面を起動する。

use nene_docket_adapters_x11::refresh_ticker::RefreshTicker;
use nene_docket_adapters_x11::saved_account;
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

/// 他人のジャーナルを読み直す間隔（秒。ADR 0004 の 8。決め打ちしない）。
const REFRESH_INTERVAL_SECONDS: u64 = 3;

/// 依存を結んで走らせる。
pub(crate) fn run(arguments: &Arguments) -> Result<(), LaunchFailure> {
    let account = saved_account::load_or_create()?;
    let journal = FileJournal::open(arguments.folder(), &arguments.docket_name(), account.id())?;
    let ports = Ports::new(Box::new(journal), Box::new(SystemClock));
    let display = Display::connect()?;
    let mut screen = DocketScreen::open(display.environment(), account, ports);
    let mut window = DocketWindow::open(&display)?;
    let ticker = RefreshTicker::start(
        window.window(),
        atoms::REFRESH_NAME,
        REFRESH_INTERVAL_SECONDS,
    )?;
    event_loop::run(&display, &mut window, &mut screen)?;
    drop(ticker);
    Ok(())
}
