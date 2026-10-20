//! 1 フレーム分のポインタの入力。

use crate::button_phase::ButtonPhase;
use crate::pixel_point::PixelPoint;

/// UI が毎フレーム渡すポインタの状態。位置は窓の外なら `None`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerInput {
    position: Option<PixelPoint>,
    phase: ButtonPhase,
    wheel: i8,
}

impl PointerInput {
    /// 位置・主ボタンの状態・ホイールの刻み（下が正）から作る。
    #[must_use]
    pub const fn new(position: Option<PixelPoint>, phase: ButtonPhase, wheel: i8) -> Self {
        Self {
            position,
            phase,
            wheel,
        }
    }

    /// ポインタが窓の外で、何も起きていないフレーム。
    #[must_use]
    pub const fn absent() -> Self {
        Self::new(None, ButtonPhase::Idle, 0)
    }

    /// 位置。
    #[must_use]
    pub const fn position(self) -> Option<PixelPoint> {
        self.position
    }

    /// 主ボタンの状態。
    #[must_use]
    pub const fn phase(self) -> ButtonPhase {
        self.phase
    }

    /// ホイールの刻み。下へ回せば正。
    #[must_use]
    pub const fn wheel(self) -> i8 {
        self.wheel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absent_pointer_has_no_position_and_no_button() {
        let input = PointerInput::absent();
        assert_eq!(input.position(), None);
        assert_eq!(input.phase(), ButtonPhase::Idle);
        assert_eq!(input.wheel(), 0);
    }
}
