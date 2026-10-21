//! セレクトボックスの描画命令と、開いたときの候補の矩形。

use alloc::borrow::ToOwned;
use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::select_state::SelectState;
use crate::text_align::TextAlign;
use crate::ui_scale::UiScale;

/// 縁と文字の間（1x）。
const PADDING_PX: u16 = 6;
/// 右端の印の幅（1x）。
const MARK_PX: u16 = 20;
/// 印（下向きの三角）の段の幅（1x）。上から順。1 段の高さは 1x で 1 画素。字形に頼らず矩形で描く（フォントに依らない）。
const MARK_ROWS: [u16; 4] = [7, 5, 3, 1];

/// 閉じた箱と、開いていれば下に並ぶ候補。`options` は候補の文字列。寸法は `scale` を掛ける（FR-016）。
#[must_use]
pub fn select_box(
    rect: PixelRect,
    (options, state): (&[&str], SelectState),
    response: Response,
    scale: UiScale,
) -> Vec<DrawCommand> {
    let mut commands = closed_box(rect, (options, state), response, scale);
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
                rect: option_rect.inset(scale.px(PADDING_PX)),
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
    (options, state): (&[&str], SelectState),
    response: Response,
    scale: UiScale,
) -> Vec<DrawCommand> {
    let (padding, mark_px) = (scale.px(PADDING_PX), scale.px(MARK_PX));
    let border = if response.focused() {
        Paint::Accent
    } else {
        Paint::Border
    };
    let label = options.get(state.chosen()).copied().unwrap_or("");
    let text_rect = PixelRect::new(
        rect.x().saturating_add(padding),
        rect.y(),
        rect.width().saturating_sub(padding).saturating_sub(mark_px),
        rect.height(),
    );
    let mark_rect = PixelRect::new(
        rect.right().saturating_sub(mark_px).saturating_add(1),
        rect.y(),
        mark_px,
        rect.height(),
    );
    let mut commands = alloc::vec![
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
    ];
    commands.extend(mark(mark_rect, scale));
    commands
}

/// 下向きの三角。`area` の中央に、細くなる段を重ねる。段の幅と高さは `scale` を掛ける。
fn mark(area: PixelRect, scale: UiScale) -> Vec<DrawCommand> {
    let row_px = scale.px(1);
    let rows = u16::try_from(MARK_ROWS.len())
        .unwrap_or(u16::MAX)
        .saturating_mul(row_px);
    let top = area
        .y()
        .saturating_add(area.height().saturating_sub(rows) / 2);
    let centre = area.x().saturating_add(area.width() / 2);
    MARK_ROWS
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let width = scale.px(*width);
            DrawCommand::Fill {
                rect: PixelRect::new(
                    centre.saturating_sub(width / 2),
                    top.saturating_add(
                        u16::try_from(index)
                            .unwrap_or(u16::MAX)
                            .saturating_mul(row_px),
                    ),
                    width,
                    row_px,
                ),
                paint: Paint::Muted,
            }
        })
        .collect()
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
            (&OPTIONS, SelectState::closed(3, 2)),
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Single,
        );
        assert_eq!(commands.len(), 3 + 4);
        assert!(matches!(
            commands.get(2),
            Some(DrawCommand::Text { text, rect, .. }) if text == "Done" && rect.width() == 74
        ));
        assert_eq!(
            commands.get(3),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(97, 20, 7, 1),
                paint: Paint::Muted
            })
        );
        assert_eq!(
            commands.get(6),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(100, 23, 1, 1),
                paint: Paint::Muted
            })
        );
    }

    #[test]
    fn open_box_lists_the_options_below_and_highlights_one() {
        let state = SelectState::closed(3, 0)
            .toggled()
            .applied(crate::key_intent::KeyIntent::Down);
        let commands = select_box(
            RECT,
            (&OPTIONS, state),
            Response::new(ResponsePhase::Idle, false, true),
            UiScale::Single,
        );
        assert_eq!(commands.len(), 7 + 3 * 2 + 1);
        assert_eq!(
            commands.get(1),
            Some(&DrawCommand::Stroke {
                rect: RECT,
                paint: Paint::Accent
            })
        );
        assert_eq!(
            commands.get(9),
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
            (&[], SelectState::closed(0, 0)),
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Single,
        );
        assert!(matches!(
            commands.get(2),
            Some(DrawCommand::Text { text, .. }) if text.is_empty()
        ));
    }

    #[test]
    fn double_scale_doubles_the_padding_and_the_mark() {
        let rect = PixelRect::new(10, 10, 200, 48);
        let commands = select_box(
            rect,
            (&OPTIONS, SelectState::closed(3, 2)),
            Response::new(ResponsePhase::Idle, false, false),
            UiScale::Double,
        );
        assert_eq!(commands.len(), 3 + 4);
        assert!(matches!(
            commands.get(2),
            Some(DrawCommand::Text { rect: label, .. }) if label.x() == 22 && label.width() == 148
        ));
        assert_eq!(
            commands.get(3),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(183, 30, 14, 2),
                paint: Paint::Muted
            })
        );
        assert_eq!(
            commands.get(6),
            Some(&DrawCommand::Fill {
                rect: PixelRect::new(189, 36, 2, 2),
                paint: Paint::Muted
            })
        );
    }
}
