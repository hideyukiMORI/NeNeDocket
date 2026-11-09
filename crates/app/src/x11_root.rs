//! X11（Linux）の合成ルート（ARC-006）。設定・アカウント・ジャーナル・時計・刻みを結び、X11 の画面を起動する。

use nene_docket_adapters_x11::appearance;
use nene_docket_adapters_x11::external_opener::ExternalOpener;
use nene_docket_adapters_x11::locale;
use nene_docket_adapters_x11::refresh_ticker::RefreshTicker;
use nene_docket_adapters_x11::saved_account;
use nene_docket_adapters_x11::saved_settings::{self, SettingsStore};
use nene_docket_adapters_x11::stopwatch::Stopwatch;
use nene_docket_adapters_x11::system_clock::SystemClock;
use nene_docket_application::administration_log;
use nene_docket_application::audit_report;
use nene_docket_application::bench_report::BenchReport;
use nene_docket_application::bench_seed;
use nene_docket_application::clock_display::ClockDisplay;
use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::ledger::Ledger;
use nene_docket_application::ports::Ports;
use nene_docket_application::settings::Settings;
use nene_docket_application::stopwatch_port::StopwatchPort;
use nene_docket_store::administration;
use nene_docket_store::bench_folder;
use nene_docket_store::file_journal::FileJournal;
use nene_docket_store::share_probe;
use nene_docket_ui_x11::atoms;
use nene_docket_ui_x11::bench_loop;
use nene_docket_ui_x11::display::Display;
use nene_docket_ui_x11::docket_window::DocketWindow;
use nene_docket_ui_x11::event_loop;

use std::path::Path;

use nene_docket_application::account::Account;
use nene_docket_application::ledger_opening;
use nene_docket_core::admin_command::AdminCommand;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::language::Language;
use nene_docket_core::op_kind::OpKind;

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
            ClockDisplay::of(settings.clock(), &SystemClock::observed()),
        );
    }
    if !arguments.attach().is_empty() {
        return attach_files(&folder, &account, arguments.attach());
    }
    if arguments.probe_share() {
        return print_probe(&folder);
    }
    if let Some((items, bench_root)) = arguments.bench() {
        return print_bench(bench_root, items, &account, &settings);
    }
    let journal = journal_in(&folder, &account)?;
    let settings = saved_settings::remember(settings, &folder)?;
    let display = Display::connect()?;
    let mut screen = screen_on(&display, journal, &account, &settings)?;
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

/// 開いたジャーナルで画面を作る（読み、併合する）。
fn screen_on(
    display: &Display,
    journal: FileJournal,
    account: &Account,
    settings: &Settings,
) -> Result<DocketScreen, LaunchFailure> {
    let ports = Ports::new(
        Box::new(journal),
        Box::new(SystemClock::observed()),
        Box::new(SettingsStore::in_config_directory()?),
        Box::new(ExternalOpener::xdg_open()),
    );
    let environment = display
        .environment()
        .with_language(locale::read())
        .with_appearance(appearance::read());
    Ok(DocketScreen::open(
        environment,
        account.clone(),
        ports,
        settings.clone(),
    ))
}

/// 計測用の台帳を作り、起動とフレームを測って標準出力に出す（`--bench`・Issue #111）。
/// 最近の台帳には覚えない。2 回目の起動は OS のファイルのキャッシュが効いた後の数字。
// 🔴 標準出力へ書いてよいのは合成ルートだけ（RS-014）。この #[expect] が宣言そのものである。
#[expect(
    clippy::print_stdout,
    reason = "RS-014: 端末への出力は合成ルートだけが行う"
)]
fn print_bench(
    folder: &Path,
    items: u32,
    account: &Account,
    settings: &Settings,
) -> Result<(), LaunchFailure> {
    bench_folder::prepare(folder)?;
    let display = Display::connect()?;
    let watch = Stopwatch::started();
    let (mut ledger, mut ports) = open_ledger(folder, account)?;
    let now = ports.clock().now();
    let started = watch.elapsed();
    bench_seed::seed(&mut ledger, ports.journal_mut(), now, items)?;
    let seeded = watch.elapsed();
    drop(ports);
    let mut first = screen_on(&display, journal_in(folder, account)?, account, settings)?;
    let opened = watch.elapsed();
    first.close();
    let checkpointed = watch.elapsed();
    drop(first);
    let mut screen = screen_on(&display, journal_in(folder, account)?, account, settings)?;
    let reopened = watch.elapsed();
    let mut window = DocketWindow::open(&display)?;
    let laps = bench_loop::run(&display, &mut window, &mut screen, &watch)?;
    let bounds = window.bounds();
    let report = BenchReport::new(
        items,
        (
            seeded.saturating_sub(started),
            opened.saturating_sub(seeded),
            checkpointed.saturating_sub(opened),
            reopened.saturating_sub(checkpointed),
        ),
    )
    .with_laps(laps);
    print!(
        "{}",
        report.text("x11", &format!("{}x{}", bounds.width(), bounds.height()))
    );
    Ok(())
}

/// フォルダのジャーナルを開く（名前はフォルダ名）。
fn journal_in(folder: &Path, account: &Account) -> Result<FileJournal, LaunchFailure> {
    Ok(FileJournal::open(
        folder,
        &Arguments::docket_name(folder),
        account,
    )?)
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
fn print_audit(
    folder: &Path,
    account: &Account,
    language: Language,
    display: ClockDisplay,
) -> Result<(), LaunchFailure> {
    let (_ledger, ports) = open_ledger(folder, account)?;
    let ops = ports.journal().all_ops()?;
    print!(
        "{}",
        audit_report::text(&ops, &ports.journal().names(), language, display)
    );
    Ok(())
}

/// 共有フォルダの速さを標準出力に出す（`--probe-share`・Issue #75）。
// 🔴 標準出力へ書いてよいのは合成ルートだけ（RS-014）。この #[expect] が宣言そのものである。
#[expect(
    clippy::print_stdout,
    reason = "RS-014: 端末への出力は合成ルートだけが行う"
)]
fn print_probe(folder: &Path) -> Result<(), LaunchFailure> {
    let probe = share_probe::measure(folder, 20)?;
    println!(
        "share probe ({}; 20 rounds): {}",
        folder.display(),
        probe.describe()
    );
    Ok(())
}

/// ファイルを添付して終わる（`--attach`・FR-021）。写してから操作を追記する。
fn attach_files(
    folder: &Path,
    account: &Account,
    attachments: &[(ItemId, String)],
) -> Result<(), LaunchFailure> {
    let (mut ledger, mut ports) = open_ledger(folder, account)?;
    for (item, source) in attachments {
        let sequence = ledger
            .docket()
            .item(*item)
            .map_or(1, nene_docket_core::item::Item::next_attachment);
        let attachment = ports
            .journal_mut()
            .copy_attachment(*item, sequence, source)?;
        let now = ports.clock().now();
        let op = ledger.compose(
            now,
            OpKind::Attach {
                item: *item,
                attachment,
            },
        );
        ports.journal_mut().append(&op)?;
    }
    ports.journal_mut().sync()?;
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
        Box::new(SystemClock::observed()),
        Box::new(SettingsStore::in_config_directory()?),
        Box::new(ExternalOpener::xdg_open()),
    );
    Ok((ledger, ports))
}
