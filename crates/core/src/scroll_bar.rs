//! 縦スクロールバーの描画命令。

use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::response_phase::ResponsePhase;
use crate::scroll_state::ScrollState;
use crate::ui_scale::UiScale;

/// つまみの、軌道の縁からの引っ込み（1x）。
const THUMB_INSET_PX: u16 = 2;

/// 軌道とつまみ。掴んでいる間はつまみを強調で塗る。寸法は `scale` を掛ける（FR-016）。
#[must_use]
pub fn scroll_bar(
    track: PixelRect,
    state: ScrollState,
    response: Response,
    scale: UiScale,
) -> Vec<DrawCommand> {
    let thumb_paint = match response.phase() {
        ResponsePhase::Held => Paint::Accent,
        ResponsePhase::Hovered => Paint::Ink,
        ResponsePhase::Idle => Paint::Muted,
    };
    alloc::vec![
        DrawCommand::Fill {
            rect: track,
            paint: Paint::Surface
        },
        DrawCommand::Fill {
            rect: state.thumb(track, scale).inset(scale.px(THUMB_INSET_PX)),
            paint: thumb_paint
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumb_follows_the_state_and_the_pointer() {
        let track = PixelRect::new(100, 0, 12, 300);
        let state = ScrollState::top(1200, 300);
        let idle = scroll_bar(
            track,
            state,
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Single,
        );
        assert_eq!(
            idle.get(1),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(102, 2, 8, 71),
                paint: Paint::Muted
            })
        );
        let held = scroll_bar(
            track,
            state,
            Response::new(ResponsePhase::Held, false, false),
            UiScale::Single,
        );
        assert!(matches!(
            held.get(1),
            Some(DrawCommand::Fill {
                paint: Paint::Accent,
                ..
            })
        ));
        let hovered = scroll_bar(
            track,
            state,
            Response::new(ResponsePhase::Hovered, false, false),
            UiScale::Single,
        );
        assert!(matches!(
            hovered.get(1),
            Some(DrawCommand::Fill {
                paint: Paint::Ink,
                ..
            })
        ));
    }

    #[test]
    fn double_scale_doubles_the_inset_and_the_minimum_thumb() {
        let track = PixelRect::new(100, 0, 24, 300);
        let commands = scroll_bar(
            track,
            ScrollState::top(1_000_000, 300),
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Double,
        );
        assert_eq!(
            commands.get(1),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(104, 4, 16, 24),
                paint: Paint::Muted
            })
        );
    }
}
