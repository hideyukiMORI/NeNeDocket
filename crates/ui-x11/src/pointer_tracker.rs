//! X のポインタのイベントを、フレームごとの `PointerInput` に写す。

use nene_docket_core::button_phase::ButtonPhase;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pointer_input::PointerInput;

/// 左ボタン・ホイールの上下。
const BUTTON_LEFT: u8 = 1;
const WHEEL_UP: u8 = 4;
const WHEEL_DOWN: u8 = 5;

/// 直前の位置と、左ボタンが押されたままか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PointerTracker {
    position: Option<PixelPoint>,
    down: bool,
}

impl PointerTracker {
    /// 窓の外・離れている。
    pub(crate) const fn outside() -> Self {
        Self {
            position: None,
            down: false,
        }
    }

    /// 動いた。位置を覚え、押されたままなら `Held`。
    pub(crate) fn moved(&mut self, x: i16, y: i16) -> PointerInput {
        self.position = PixelPoint::from_event(i32::from(x), i32::from(y));
        PointerInput::new(self.position, self.idle_phase(), 0)
    }

    /// 窓を出た。
    pub(crate) fn left(&mut self) -> PointerInput {
        self.position = None;
        PointerInput::new(None, self.idle_phase(), 0)
    }

    /// ボタンが押された。左なら `Pressed`、ホイールなら刻み。
    pub(crate) fn pressed(&mut self, button: u8, x: i16, y: i16) -> PointerInput {
        self.position = PixelPoint::from_event(i32::from(x), i32::from(y));
        match button {
            BUTTON_LEFT => {
                self.down = true;
                PointerInput::new(self.position, ButtonPhase::Pressed, 0)
            }
            WHEEL_UP => PointerInput::new(self.position, self.idle_phase(), -1),
            WHEEL_DOWN => PointerInput::new(self.position, self.idle_phase(), 1),
            _ => PointerInput::new(self.position, self.idle_phase(), 0),
        }
    }

    /// ボタンが離された。左なら `Released`。
    pub(crate) fn released(&mut self, button: u8, x: i16, y: i16) -> PointerInput {
        self.position = PixelPoint::from_event(i32::from(x), i32::from(y));
        if button == BUTTON_LEFT && self.down {
            self.down = false;
            return PointerInput::new(self.position, ButtonPhase::Released, 0);
        }
        PointerInput::new(self.position, self.idle_phase(), 0)
    }

    /// 何も起きていないフレーム（露出・鍵）に渡す入力。
    pub(crate) const fn current(self) -> PointerInput {
        PointerInput::new(self.position, self.idle_phase(), 0)
    }

    const fn idle_phase(self) -> ButtonPhase {
        if self.down {
            ButtonPhase::Held
        } else {
            ButtonPhase::Idle
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_hold_release_follow_the_left_button() {
        let mut tracker = PointerTracker::outside();
        assert_eq!(tracker.current(), PointerInput::absent());
        let pressed = tracker.pressed(1, 10, 20);
        assert_eq!(pressed.phase(), ButtonPhase::Pressed);
        assert_eq!(pressed.position(), Some(PixelPoint::new(10, 20)));
        assert_eq!(tracker.moved(12, 20).phase(), ButtonPhase::Held);
        assert_eq!(tracker.current().phase(), ButtonPhase::Held);
        assert_eq!(tracker.released(1, 12, 20).phase(), ButtonPhase::Released);
        assert_eq!(tracker.current().phase(), ButtonPhase::Idle);
        assert_eq!(tracker.released(1, 12, 20).phase(), ButtonPhase::Idle);
    }

    #[test]
    fn wheel_and_other_buttons_do_not_press() {
        let mut tracker = PointerTracker::outside();
        assert_eq!(tracker.pressed(4, 1, 1).wheel(), -1);
        assert_eq!(tracker.pressed(5, 1, 1).wheel(), 1);
        assert_eq!(tracker.pressed(3, 1, 1).phase(), ButtonPhase::Idle);
        assert_eq!(tracker.left().position(), None);
        assert_eq!(tracker.moved(-5, 3).position(), None);
    }
}
