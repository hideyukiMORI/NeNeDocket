//! 計測の筋書き（`--bench`・Issue #111）を窓で回す。窓が出るのを待ち、筋書きのフレームを 1 つずつ
//! `DocketScreen::step` → 描き切る → 転送して受け取り終える、に分けて測る。

use nene_docket_application::bench_phase::BenchPhase;
use nene_docket_application::bench_plan::BenchPlan;
use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::frame_lap::FrameLap;
use nene_docket_application::stopwatch_port::StopwatchPort;
use x11rb::connection::Connection;
use x11rb::protocol::Event;

use crate::display::Display;
use crate::docket_window::DocketWindow;
use crate::ui_failure::UiFailure;

/// 筋書きを回し、フレームごとの段と区間を返す。窓の大きさが変われば次のフレームから従う。
///
/// # Errors
///
/// X の要求か cairo が失敗した。
pub fn run(
    display: &Display,
    window: &mut DocketWindow<'_>,
    screen: &mut DocketScreen,
    watch: &dyn StopwatchPort,
) -> Result<Vec<(BenchPhase, FrameLap)>, UiFailure> {
    wait_until_exposed(display, window)?;
    let plan = BenchPlan::for_screen(screen);
    let mut laps = Vec::with_capacity(plan.len());
    for index in 0..plan.len() {
        while let Some(event) = display.connection().poll_for_event()? {
            follow_size(window, &event);
        }
        let (phase, input) = plan.frame(index, window.bounds());
        let started = watch.elapsed();
        let frame = screen.step(&input);
        let stepped = watch.elapsed();
        let (encode, present) = window.present_measured(&frame, watch)?;
        laps.push((
            phase,
            FrameLap::new(stepped.saturating_sub(started), encode, present),
        ));
    }
    Ok(laps)
}

/// 最初の `Expose` まで待つ（それより前の転送は捨てられ、測っても意味が無い）。
fn wait_until_exposed(display: &Display, window: &mut DocketWindow<'_>) -> Result<(), UiFailure> {
    loop {
        let event = display.connection().wait_for_event()?;
        follow_size(window, &event);
        if let Event::Expose(_) = event {
            return Ok(());
        }
    }
}

/// X のイベントは開いた集合なので `if let` で書く（規約 第 4 節）。
fn follow_size(window: &mut DocketWindow<'_>, event: &Event) {
    if let Event::ConfigureNotify(configure) = event {
        window.resized(configure.width, configure.height);
    }
}
