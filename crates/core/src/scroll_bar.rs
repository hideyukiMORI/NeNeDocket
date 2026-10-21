//! 縦スクロールバーの描画命令。

use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::response_phase::ResponsePhase;
use crate::scroll_state::ScrollState;
use crate::ui_scale::UiScale;

/// 軌道の線とつまみ。掴んでいる間はつまみを強調で塗る。線とつまみの幅は軌道の幅から、つまみの最小の長さは `scale` から（FR-016）。
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
    // 押せる幅は軌道のまま。見せるのは中央の細い線と、その 1/4 の幅のつまみだけ（Issue #133・1x で 1 px と 3 px）。
    let line = (track.width() / 12).max(1);
    let thumb_width = (track.width() / 4).max(1);
    let center = |width: u16| {
        track
            .x()
            .saturating_add(track.width().saturating_sub(width) / 2)
    };
    let mut commands = alloc::vec![DrawCommand::Fill {
        rect: PixelRect::new(center(line), track.y(), line, track.height()),
        paint: Paint::Border,
    }];
    if state.max_offset() > 0 {
        let thumb = state.thumb(track, scale);
        commands.push(DrawCommand::Fill {
            rect: PixelRect::new(center(thumb_width), thumb.y(), thumb_width, thumb.height()),
            paint: thumb_paint,
        });
    }
    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_thin_line_and_a_thin_thumb_follow_the_state_and_the_pointer() {
        let track = PixelRect::new(100, 0, 12, 300);
        let state = ScrollState::top(1200, 300);
        let idle = scroll_bar(
            track,
            state,
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Single,
        );
        assert_eq!(
            idle.first(),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(105, 0, 1, 300),
                paint: Paint::Border
            })
        );
        assert_eq!(
            idle.get(1),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(104, 0, 3, 75),
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
    fn nothing_to_scroll_shows_only_the_line_and_a_doubled_track_doubles_the_widths() {
        let track = PixelRect::new(0, 0, 24, 300);
        let fits = scroll_bar(
            track,
            ScrollState::top(200, 300),
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Double,
        );
        assert_eq!(fits.len(), 1);
        let long = scroll_bar(
            track,
            ScrollState::top(1_000_000, 300),
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Double,
        );
        // 2 倍では線 2・つまみ 6、つまみの最小の長さも 2 倍（64）。
        assert!(matches!(
            long.as_slice(),
            [
                DrawCommand::Fill { rect: line, .. },
                DrawCommand::Fill { rect: thumb, .. },
            ] if line.width() == 2 && thumb.width() == 6 && thumb.height() == 64
        ));
    }
}
