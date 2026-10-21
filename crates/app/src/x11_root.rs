//! X11（Linux）の合成ルート（ARC-006）。設定・アカウント・ジャーナル・時計・刻みを結び、X11 の画面を起動する。

use nene_docket_adapters_x11::appearance;
use nene_docket_adapters_x11::locale;
use nene_docket_adapters_x11::refresh_ticker::RefreshTicker;
use nene_docket_adapters_x11::saved_account;
use nene_docket_adapters_x11::saved_settings::{self, SettingsStore};
use nene_docket_adapters_x11::system_clock::SystemClock;
use nene_docket_application::administration_log;
use nene_docket_application::audit_report;
use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::ledger::Ledger;
use nene_docket_application::ports::Ports;
use nene_docket_store::administration;
use nene_docket_store::file_journal::FileJournal;
use nene_docket_ui_x11::atoms;
use nene_docket_ui_x11::display::Display;
use nene_docket_ui_x11::docket_window::DocketWindow;
use nene_docket_ui_x11::event_loop;

use std::path::Path;

use nene_docket_application::account::Account;
use nene_docket_application::ledger_opening;
use nene_docket_core::admin_command::AdminCommand;
use nene_docket_core::language::Language;

use crate::arguments::Arguments;
use crate::launch_failure::LaunchFailure;

/// 依存を結んで走らせる。権限の命令があれば書いて終わる（画面は開かない）。台帳のフォルダは引数、無ければ設定の最近の台帳（FR-012）。開けたら設定に覚える。
/// OS の言語と明暗は起動時に 1 度だけ観測して環境に添える（ADR 0011 / 0012）。
pub(crate) fn run(arguments: &Arguments) -> Result<(), LaunchFailure> {
    let settings = saved_settings::load()?;
    let folder = arguments.folder_or(settings.latest());
    let account = saved_account::load_or_create()?;
    if !arguments.admin().is_empty() {
        administration::apply(&folder, &account, arguments.admin())?;
        record_administration(&folder, &account, arguments.admin())?;
        return Ok(());
    }
    if arguments.audit() {
        return print_audit(
            &folder,
            &account,
            settings.language().resolve(locale::read()),
        );
    }
    let journal = FileJournal::open(&folder, &Arguments::docket_name(&folder), &account)?;
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
    screen.close();
    Ok(())
}

/// 権限を書いた後、その命令を自分のジャーナルに記録する（監査・ADR 0014）。
fn record_administration(
    folder: &Path,
    account: &Account,
    commands: &[AdminCommand],
) -> Result<(), LaunchFailure> {
    let (mut ledger, mut ports) = open_ledger(folder, account)?;
    administration_log::record(&mut ledger, &mut ports, commands)?;
    Ok(())
}

/// 台帳全体の時系列を標準出力に出す（`--audit`・ADR 0014）。
// 🔴 標準出力へ書いてよいのは合成ルートだけ（RS-014）。この #[expect] が宣言そのものである。
#[expect(
    clippy::print_stdout,
    reason = "RS-014: 端末への出力は合成ルートだけが行う"
)]
fn print_audit(folder: &Path, account: &Account, language: Language) -> Result<(), LaunchFailure> {
    let (_ledger, ports) = open_ledger(folder, account)?;
    let ops = ports.journal().all_ops()?;
    print!(
        "{}",
        audit_report::text(&ops, &ports.journal().names(), language)
    );
    Ok(())
}

/// 台帳を開いて台帳の状態とポートを返す（画面は開かない）。スナップショットがあればそれを使う（ADR 0015）。
fn open_ledger(folder: &Path, account: &Account) -> Result<(Ledger, Ports), LaunchFailure> {
    let mut journal = FileJournal::open(folder, &Arguments::docket_name(folder), account)?;
    let (ledger, failure) = ledger_opening::open(&mut journal, account.clone());
    if let Some(failure) = failure {
        return Err(failure.into());
    }
    let ports = Ports::new(
        Box::new(journal),
        Box::new(SystemClock),
        Box::new(SettingsStore::in_config_directory()?),
    );
    Ok((ledger, ports))
}
