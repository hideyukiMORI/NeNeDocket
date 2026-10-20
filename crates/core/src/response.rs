//! 部品 1 つに対する、このフレームの判定。

use crate::response_phase::ResponsePhase;

/// [`crate::interaction::Interaction::probe`] が返す。UI はこれを見て描き分け、application は `clicked` で意図を作る。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Response {
    phase: ResponsePhase,
    clicked: bool,
    focused: bool,
}

impl Response {
    /// 判定から作る。
    #[must_use]
    pub const fn new(phase: ResponsePhase, clicked: bool, focused: bool) -> Self {
        Self {
            phase,
            clicked,
            focused,
        }
    }

    /// ポインタとの関係。
    #[must_use]
    pub const fn phase(self) -> ResponsePhase {
        self.phase
    }

    /// この部品の上で押して、この部品の上で離した（クリックが成立した）。
    #[must_use]
    pub const fn clicked(self) -> bool {
        self.clicked
    }

    /// 鍵の入力がこの部品へ行く。
    #[must_use]
    pub const fn focused(self) -> bool {
        self.focused
    }
}
