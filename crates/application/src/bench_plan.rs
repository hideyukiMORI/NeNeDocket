//! 計測の筋書き（`--bench`・Issue #111）。暖機・何もしない・検索の欄を押す・検索に 1 字足して消す、をフレームの列にする。
//! X11 版も Windows 版も同じ筋書きを回すので、数字は同じ操作の比べになる。

use alloc::vec::Vec;

use nene_docket_core::button_phase::ButtonPhase;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::pointer_input::PointerInput;

use crate::bench_phase::BenchPhase;
use crate::docket_screen::DocketScreen;
use crate::frame_input::FrameInput;

/// 暖機のフレーム数。
const WARMUP: usize = 5;
/// 何もしないフレーム数。
const IDLE: usize = 50;
/// 検索の欄を押して離す 2 フレーム。
const FOCUS: usize = 2;
/// 検索に 1 字足す・消すフレーム数。
const SEARCH: usize = 50;
/// 検索に打つ字。題名の連番に含まれるので、表の一覧が毎回変わる。
const SEARCH_CHAR: char = '1';

/// 筋書き。検索の欄の位置だけを画面から受け取る。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchPlan {
    search: PixelPoint,
}

impl BenchPlan {
    /// 画面の検索の欄の位置から。
    #[must_use]
    pub const fn for_screen(screen: &DocketScreen) -> Self {
        Self {
            search: screen.search_point(),
        }
    }

    /// フレームの数。
    #[must_use]
    pub const fn len(&self) -> usize {
        WARMUP + IDLE + FOCUS + SEARCH
    }

    /// 空ではない（clippy の `len_without_is_empty` の対）。
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// `index` 番目のフレームの段と入力。`bounds` は窓のいまの大きさ。
    #[must_use]
    pub fn frame(&self, index: usize, bounds: PixelRect) -> (BenchPhase, FrameInput) {
        let absent = FrameInput::new(bounds, PointerInput::absent());
        if index < WARMUP {
            return (BenchPhase::Warmup, absent);
        }
        let index = index - WARMUP;
        if index < IDLE {
            return (BenchPhase::Idle, absent);
        }
        let index = index - IDLE;
        if index < FOCUS {
            let phase = if index == 0 {
                ButtonPhase::Pressed
            } else {
                ButtonPhase::Released
            };
            let pointer = PointerInput::new(Some(self.search), phase, 0);
            return (BenchPhase::Focus, FrameInput::new(bounds, pointer));
        }
        let key = if (index - FOCUS).is_multiple_of(2) {
            KeyIntent::Char(SEARCH_CHAR)
        } else {
            KeyIntent::Backspace
        };
        (BenchPhase::Search, absent.with_keys(Vec::from([key])))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOUNDS: PixelRect = PixelRect::new(0, 0, 1120, 600);

    #[test]
    fn the_plan_warms_up_idles_focuses_the_search_and_types() {
        let plan = BenchPlan {
            search: PixelPoint::new(36, 22),
        };
        let phases: Vec<BenchPhase> = (0..plan.len())
            .map(|index| plan.frame(index, BOUNDS).0)
            .collect();
        let count = |wanted: BenchPhase| phases.iter().filter(|phase| **phase == wanted).count();
        assert_eq!(count(BenchPhase::Warmup), WARMUP);
        assert_eq!(count(BenchPhase::Idle), IDLE);
        assert_eq!(count(BenchPhase::Focus), FOCUS);
        assert_eq!(count(BenchPhase::Search), SEARCH);
        let (_, press) = plan.frame(WARMUP + IDLE, BOUNDS);
        assert_eq!(press.pointer().position(), Some(PixelPoint::new(36, 22)));
        assert_eq!(press.pointer().phase(), ButtonPhase::Pressed);
        let (_, typed) = plan.frame(WARMUP + IDLE + FOCUS, BOUNDS);
        assert_eq!(typed.keys(), [KeyIntent::Char('1')]);
        let (_, erased) = plan.frame(WARMUP + IDLE + FOCUS + 1, BOUNDS);
        assert_eq!(erased.keys(), [KeyIntent::Backspace]);
        assert!(!plan.is_empty());
    }
}
