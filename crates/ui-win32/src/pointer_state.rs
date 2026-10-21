//! ポインタの追跡（フレームをまたぐ）。窓の中の座標と左ボタンの状態から `PointerInput` を作る。Win32 を呼ばないので Linux でもテストする。

use nene_docket_core::button_phase::ButtonPhase;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pointer_input::PointerInput;

/// ホイール 1 刻みの `WHEEL_DELTA`。
const WHEEL_DELTA: i32 = 120;

/// いまの位置と、左ボタンを押しているか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerState {
    position: Option<PixelPoint>,
    down: bool,
}

impl PointerState {
    /// 窓の外。
    #[must_use]
    pub const fn outside() -> Self {
        Self {
            position: None,
            down: false,
        }
    }

    /// 動いた（`WM_MOUSEMOVE`）。
    pub fn moved(&mut self, x: i32, y: i32) -> PointerInput {
        self.position = PixelPoint::from_event(x, y);
        PointerInput::new(self.position, self.idle_phase(), 0)
    }

    /// 窓から出た（`WM_MOUSELEAVE`）。
    pub fn left(&mut self) -> PointerInput {
        self.position = None;
        PointerInput::new(None, self.idle_phase(), 0)
    }

    /// 左ボタンを押した（`WM_LBUTTONDOWN`）。
    pub fn pressed(&mut self, x: i32, y: i32) -> PointerInput {
        self.position = PixelPoint::from_event(x, y);
        self.down = true;
        PointerInput::new(self.position, ButtonPhase::Pressed, 0)
    }

    /// 左ボタンを離した（`WM_LBUTTONUP`）。押していなければただの位置。
    pub fn released(&mut self, x: i32, y: i32) -> PointerInput {
        self.position = PixelPoint::from_event(x, y);
        if self.down {
            self.down = false;
            return PointerInput::new(self.position, ButtonPhase::Released, 0);
        }
        PointerInput::new(self.position, self.idle_phase(), 0)
    }

    /// ホイール（`WM_MOUSEWHEEL`）。`delta` は `WHEEL_DELTA` 単位で、正が上。core は下が正。
    pub fn wheeled(&mut self, (x, y): (i32, i32), delta: i16) -> PointerInput {
        self.position = PixelPoint::from_event(x, y);
        let notches = -(i32::from(delta) / WHEEL_DELTA);
        let wheel =
            i8::try_from(notches.clamp(i32::from(i8::MIN), i32::from(i8::MAX))).unwrap_or(0);
        PointerInput::new(self.position, self.idle_phase(), wheel)
    }

    /// いまの位置の入力（押下も離しも無い）。
    #[must_use]
    pub const fn current(self) -> PointerInput {
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
        let mut state = PointerState::outside();
        assert_eq!(state.current(), PointerInput::absent());
        let pressed = state.pressed(10, 20);
        assert_eq!(pressed.phase(), ButtonPhase::Pressed);
        assert_eq!(pressed.position(), Some(PixelPoint::new(10, 20)));
        assert_eq!(state.moved(12, 20).phase(), ButtonPhase::Held);
        assert_eq!(state.released(12, 21).phase(), ButtonPhase::Released);
        assert_eq!(state.released(12, 21).phase(), ButtonPhase::Idle);
        assert_eq!(state.left(), PointerInput::absent());
    }

    #[test]
    fn the_wheel_is_in_notches_with_down_positive_and_negative_coordinates_are_outside() {
        let mut state = PointerState::outside();
        assert_eq!(state.wheeled((5_i32, 5_i32), 120).wheel(), -1);
        assert_eq!(state.wheeled((5_i32, 5_i32), -240).wheel(), 2);
        assert_eq!(state.wheeled((5_i32, 5_i32), 60).wheel(), 0);
        assert_eq!(state.moved(-1, 5).position(), None);
    }
}
