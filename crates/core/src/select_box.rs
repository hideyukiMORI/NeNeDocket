//! セレクトボックスの描画命令と、開いたときの候補の矩形。

use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::select_state::SelectState;
use crate::text_align::TextAlign;

/// 縁と文字の間。
const PADDING_PX: u16 = 6;
/// 右端の印の幅。
const MARK_PX: u16 = 20;
/// 開いたときの印。
const MARK: &str = "▾";

/// 閉じた箱と、開いていれば下に並ぶ候補。`options` は候補の文字列。
#[must_use]
pub fn select_box(
    rect: PixelRect,
    options: &[&str],
    state: SelectState,
    response: Response,
) -> Vec<DrawCommand> {
    let mut commands = closed_box(rect, options, state, response);
    if state.is_open() {
        for (index, option_rect) in option_rects(rect, options.len()).into_iter().enumerate() {
            let paint = if index == state.highlighted() {
                Paint::Selection
            } else {
                Paint::Surface
            };
            commands.push(DrawCommand::Fill {
                rect: option_rect,
                paint,
            });
            commands.push(DrawCommand::Text {
                rect: option_rect.inset(PADDING_PX),
                text: options.get(index).copied().unwrap_or("").to_owned(),
                align: TextAlign::Start,
                paint: Paint::Ink,
                caret: None,
            });
        }
        commands.push(DrawCommand::Stroke {
            rect: list_rect(rect, options.len()),
            paint: Paint::Border,
        });
    }
    commands
}

/// 開いたときの `index` 番目の候補の矩形。箱の真下に、箱と同じ高さで並ぶ。application はこれで候補を問い合わせる。
#[must_use]
pub fn option_rects(rect: PixelRect, count: usize) -> Vec<PixelRect> {
    (0..count)
        .map(|index| {
            let steps = u16::try_from(index.saturating_add(1)).unwrap_or(u16::MAX);
            PixelRect::new(
                rect.x(),
                rect.y().saturating_add(rect.height().saturating_mul(steps)),
                rect.width(),
                rect.height(),
            )
        })
        .collect()
}

/// 開いたときの候補の一覧全体の矩形。
#[must_use]
pub fn list_rect(rect: PixelRect, count: usize) -> PixelRect {
    let rows = u16::try_from(count).unwrap_or(u16::MAX);
    PixelRect::new(
        rect.x(),
        rect.y().saturating_add(rect.height()),
        rect.width(),
        rect.height().saturating_mul(rows),
    )
}

fn closed_box(
    rect: PixelRect,
    options: &[&str],
    state: SelectState,
    response: Response,
) -> Vec<DrawCommand> {
    let border = if response.focused() {
        Paint::Accent
    } else {
        Paint::Border
    };
    let label = options.get(state.chosen()).copied().unwrap_or("");
    let text_rect = PixelRect::new(
        rect.x().saturating_add(PADDING_PX),
        rect.y(),
        rect.width()
            .saturating_sub(PADDING_PX)
            .saturating_sub(MARK_PX),
        rect.height(),
    );
    let mark_rect = PixelRect::new(
        rect.right().saturating_sub(MARK_PX).saturating_add(1),
        rect.y(),
        MARK_PX,
        rect.height(),
    );
    alloc::vec![
        DrawCommand::Fill {
            rect,
            paint: Paint::Surface
        },
        DrawCommand::Stroke {
            rect,
            paint: border
        },
        DrawCommand::Text {
            rect: text_rect,
            text: label.to_owned(),
            align: TextAlign::Start,
            paint: Paint::Ink,
            caret: None,
        },
        DrawCommand::Text {
            rect: mark_rect,
            text: MARK.to_owned(),
            align: TextAlign::Center,
            paint: Paint::Muted,
            caret: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response_phase::ResponsePhase;

    const RECT: PixelRect = PixelRect::new(10, 10, 100, 24);
    const OPTIONS: [&str; 3] = ["Open", "In progress", "Done"];

    #[test]
    fn closed_box_shows_the_chosen_option_and_the_mark() {
        let commands = select_box(
            RECT,
            &OPTIONS,
            SelectState::closed(3, 2),
            Response::new(ResponsePhase::Idle, false, false),
        );
        assert_eq!(commands.len(), 4);
        assert!(matches!(
            commands.get(2),
            Some(DrawCommand::Text { text, rect, .. }) if text == "Done" && rect.width() == 74
        ));
        assert!(matches!(
            commands.get(3),
            Some(DrawCommand::Text { text, rect, align: TextAlign::Center, .. }) if text == MARK && rect.x() == 90
        ));
    }

    #[test]
    fn open_box_lists_the_options_below_and_highlights_one() {
        let state = SelectState::closed(3, 0)
            .toggled()
            .applied(crate::key_intent::KeyIntent::Down);
        let commands = select_box(
            RECT,
            &OPTIONS,
            state,
            Response::new(ResponsePhase::Idle, false, true),
        );
        assert_eq!(commands.len(), 4 + 3 * 2 + 1);
        assert_eq!(
            commands.get(1),
            Some(&DrawCommand::Stroke {
                rect: RECT,
                paint: Paint::Accent
            })
        );
        assert_eq!(
            commands.get(6),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(10, 58, 100, 24),
                paint: Paint::Selection
            })
        );
        assert_eq!(
            commands.last(),
            Some(&DrawCommand::Stroke {
                rect: PixelRect::new(10, 34, 100, 72),
                paint: Paint::Border
            })
        );
    }

    #[test]
    fn option_rects_stack_under_the_box() {
        let rects = option_rects(RECT, 2);
        assert_eq!(
            rects,
            alloc::vec![
                PixelRect::new(10, 34, 100, 24),
                PixelRect::new(10, 58, 100, 24)
            ]
        );
        assert_eq!(list_rect(RECT, 2), PixelRect::new(10, 34, 100, 48));
        assert!(option_rects(RECT, 0).is_empty());
    }

    #[test]
    fn missing_option_text_is_drawn_as_empty() {
        let commands = select_box(
            RECT,
            &[],
            SelectState::closed(0, 0),
            Response::new(ResponsePhase::Idle, false, false),
        );
        assert!(matches!(
            commands.get(2),
            Some(DrawCommand::Text { text, .. }) if text.is_empty()
        ));
    }
}
