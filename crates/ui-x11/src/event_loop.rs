//! イベントを `FrameInput` にして `DocketScreen::step` に渡し、描き直す。

use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::frame_input::FrameInput;
use nene_docket_core::pointer_input::PointerInput;
use x11rb::connection::Connection;
use x11rb::protocol::Event;

use crate::display::Display;
use crate::docket_window::DocketWindow;
use crate::flow::Flow;
use crate::gathering::Gathering;
use crate::pending_frame::PendingFrame;
use crate::ui_failure::UiFailure;

/// 開いた窓でイベントを待ち、画面に渡し、描き直す。閉じる要求で戻る。
/// 読み直しの合図（[`crate::atoms::REFRESH_NAME`] の `ClientMessage`）は、次のフレームに読み直しを乗せる。
///
/// # Errors
///
/// X の要求か cairo が失敗した。
pub fn run(
    display: &Display,
    window: &mut DocketWindow<'_>,
    screen: &mut DocketScreen,
) -> Result<(), UiFailure> {
    let mut gathering = Gathering::empty();
    present(
        window,
        screen,
        &PendingFrame::of_pointer(PointerInput::absent()),
    )?;
    loop {
        let event = display.connection().wait_for_event()?;
        if step(display, window, &mut gathering, &event) == Flow::Quit {
            return Ok(());
        }
        // 溜まったイベントは 1 回の描き直しにまとめる。押下と離しだけは 1 つずつフレームにする。
        while let Some(pending) = display.connection().poll_for_event()? {
            if step(display, window, &mut gathering, &pending) == Flow::Quit {
                return Ok(());
            }
        }
        for frame in gathering.take_frames() {
            present(window, screen, &frame)?;
        }
    }
}

/// 1 つのイベント。X のイベントは開いた集合なので `if let` の連なりで書く（規約 第 4 節）。
fn step(
    display: &Display,
    window: &mut DocketWindow<'_>,
    gathering: &mut Gathering,
    event: &Event,
) -> Flow {
    if let Event::ConfigureNotify(configure) = event {
        window.resized(configure.width, configure.height);
    } else if let Event::ButtonPress(press) = event {
        let input = gathering
            .pointer_mut()
            .pressed(press.detail, press.event_x, press.event_y);
        gathering.push_frame(input);
    } else if let Event::ButtonRelease(release) = event {
        let input =
            gathering
                .pointer_mut()
                .released(release.detail, release.event_x, release.event_y);
        gathering.push_frame(input);
    } else if let Event::MotionNotify(motion) = event {
        gathering
            .pointer_mut()
            .moved(motion.event_x, motion.event_y);
    } else if let Event::LeaveNotify(_) = event {
        gathering.pointer_mut().left();
    } else if let Event::KeyPress(key) = event {
        gathering.push_key(display.key_intent(key));
    } else if let Event::DestroyNotify(_) = event {
        // WM の無い X（Xvfb）では閉じる要求が来ず、窓が直接消される。消えた窓には描けないので戻る。
        return Flow::Quit;
    } else if let Event::ClientMessage(message) = event {
        let atoms = display.atoms();
        if message.type_ == atoms.wm_protocols
            && message.data.as_data32().first() == Some(&atoms.wm_delete_window)
        {
            return Flow::Quit;
        }
        if message.type_ == atoms.refresh {
            gathering.push_refresh();
        }
    }
    Flow::Continue
}

/// 画面に 1 フレーム渡し、描いて転送する。
fn present(
    window: &mut DocketWindow<'_>,
    screen: &mut DocketScreen,
    pending: &PendingFrame,
) -> Result<(), UiFailure> {
    let mut input =
        FrameInput::new(window.bounds(), pending.pointer()).with_keys(pending.keys().to_vec());
    if pending.refresh() {
        input = input.with_refresh();
    }
    let frame = screen.step(&input);
    window.present(&frame)
}
