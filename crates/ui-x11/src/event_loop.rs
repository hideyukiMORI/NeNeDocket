//! イベントを `FrameInput` にして `DocketScreen::step` に渡し、描き直す。

use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::frame_input::FrameInput;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::pointer_input::PointerInput;
use x11rb::connection::Connection;
use x11rb::protocol::Event;

use crate::display::Display;
use crate::docket_window::DocketWindow;
use crate::flow::Flow;
use crate::gathering::Gathering;
use crate::ui_failure::UiFailure;

/// 窓を開き、イベントを待ち、画面に渡し、描き直す。閉じる要求で戻る。
///
/// # Errors
///
/// X の要求か cairo が失敗した。
pub fn run(display: &Display, screen: &mut DocketScreen) -> Result<(), UiFailure> {
    let mut window = DocketWindow::open(display)?;
    let mut gathering = Gathering::empty();
    present(&window, screen, PointerInput::absent(), Vec::new())?;
    loop {
        let event = display.connection().wait_for_event()?;
        if step(display, &mut window, &mut gathering, &event) == Flow::Quit {
            return Ok(());
        }
        // 溜まったイベントは 1 回の描き直しにまとめる。押下と離しだけは 1 つずつフレームにする。
        while let Some(pending) = display.connection().poll_for_event()? {
            if step(display, &mut window, &mut gathering, &pending) == Flow::Quit {
                return Ok(());
            }
        }
        for (pointer, keys) in gathering.take_frames() {
            present(&window, screen, pointer, keys)?;
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
    }
    Flow::Continue
}

/// 画面に 1 フレーム渡し、描いて転送する。
fn present(
    window: &DocketWindow<'_>,
    screen: &mut DocketScreen,
    pointer: PointerInput,
    keys: Vec<KeyIntent>,
) -> Result<(), UiFailure> {
    let input = FrameInput::new(window.bounds(), pointer).with_keys(keys);
    let frame = screen.step(&input);
    window.present(&frame)
}
