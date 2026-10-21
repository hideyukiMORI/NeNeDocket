//! X11（Linux）の合成ルート（ARC-006）。X11 の画面を起動する。

use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::sample_item::SampleItem;
use nene_docket_ui_x11::display::Display;
use nene_docket_ui_x11::event_loop;
use nene_docket_ui_x11::ui_failure::UiFailure;

/// 見本の課題の数。ジャーナル（ADR 0004）が入るまでの仮の台帳。1 万件で表の速さを測る（FR-002）。
const SAMPLE_COUNT: u32 = 10_000;

/// 依存を結んで走らせる。
pub(crate) fn run() -> Result<(), UiFailure> {
    let display = Display::connect()?;
    let mut screen = DocketScreen::open(display.environment(), SampleItem::generate(SAMPLE_COUNT));
    event_loop::run(&display, &mut screen)
}
