//! 複数行の入力欄の描画命令（本文・FR-001）。行ごとに 1 つの文字の命令を出す。最初に見せる行は呼び手が決める（スクロール・カーソルの追従）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::draw_command::DrawCommand;
use crate::paint::Paint;
use crate::pixel_rect::PixelRect;
use crate::response::Response;
use crate::text_align::TextAlign;
use crate::text_caret::TextCaret;
use crate::text_edit::TextEdit;

/// 縁と文字の間。
pub const PADDING_PX: u16 = 6;

/// 欄の中に収まる行の数（1 以上）。
#[must_use]
pub fn visible_lines(rect: PixelRect, line_px: u16) -> usize {
    usize::from(rect.inset(PADDING_PX).height() / line_px.max(1)).max(1)
}

/// 複数行の欄。フォーカスがあれば強調の縁で、カーソルと選択を行ごとに描く。
/// `line_px` は 1 行の高さ、`first` は最初に見せる行（0 から）。
#[must_use]
pub fn text_area(
    rect: PixelRect,
    edit: &TextEdit,
    response: Response,
    (line_px, first): (u16, usize),
) -> Vec<DrawCommand> {
    let (text, caret) = edit.display();
    let caret = response.focused().then_some(caret);
    let inner = rect.inset(PADDING_PX);
    let visible = usize::from(inner.height() / line_px.max(1)).max(1);
    let mut commands = alloc::vec![
        DrawCommand::Fill {
            rect,
            paint: Paint::Surface
        },
        DrawCommand::Stroke {
            rect,
            paint: if response.focused() {
                Paint::Accent
            } else {
                Paint::Border
            }
        },
        DrawCommand::Clip(rect.inset(1)),
    ];
    let mut start = 0_usize;
    for (number, line) in text.split('\n').enumerate() {
        let length = line.chars().count();
        if let Some(row) = number.checked_sub(first).filter(|row| *row < visible) {
            let y = inner.y().saturating_add(
                u16::try_from(row)
                    .unwrap_or(u16::MAX)
                    .saturating_mul(line_px),
            );
            commands.push(DrawCommand::Text {
                rect: PixelRect::new(inner.x(), y, inner.width(), line_px),
                text: String::from(line),
                align: TextAlign::Start,
                paint: Paint::Ink,
                caret: caret.and_then(|caret| line_caret(caret, start, length)),
            });
        }
        start = start.saturating_add(length).saturating_add(1);
    }
    commands.push(DrawCommand::Unclip);
    commands
}

/// 文字列全体のカーソルを、`start` から `length` 文字の行の中のカーソルに写す。行に掛からなければ `None`。
fn line_caret(caret: TextCaret, start: usize, length: usize) -> Option<TextCaret> {
    let end = start.saturating_add(length);
    let on_line = (start..=end).contains(&caret.cursor());
    let selection_start = caret.start().max(start);
    let selection_end = caret.end().min(end);
    if on_line {
        let cursor = caret.cursor().saturating_sub(start);
        let anchor = if caret.has_selection() {
            if caret.anchor() < caret.cursor() {
                selection_start.saturating_sub(start)
            } else {
                selection_end.saturating_sub(start)
            }
        } else {
            cursor
        };
        return Some(TextCaret::new(cursor, anchor));
    }
    (caret.has_selection() && selection_start < selection_end).then(|| {
        TextCaret::new(
            selection_end.saturating_sub(start),
            selection_start.saturating_sub(start),
        )
        .without_bar()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_intent::KeyIntent;
    use crate::response_phase::ResponsePhase;

    const RECT: PixelRect = PixelRect::new(0, 0, 200, 60);

    fn focused() -> Response {
        Response::new(ResponsePhase::Idle, false, true)
    }

    fn lines(commands: &[DrawCommand]) -> Vec<(String, Option<TextCaret>)> {
        commands
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, caret, .. } => Some((text.clone(), *caret)),
                DrawCommand::Fill { .. }
                | DrawCommand::Stroke { .. }
                | DrawCommand::Clip(_)
                | DrawCommand::Unclip => None,
            })
            .collect()
    }

    #[test]
    fn each_line_is_its_own_text_and_only_the_caret_line_has_a_bar() {
        let edit = TextEdit::showing("ab\ncd\n");
        let shown = lines(&text_area(RECT, &edit, focused(), (16, 0)));
        assert_eq!(shown.len(), 3);
        assert_eq!(shown[0], (String::from("ab"), None));
        assert_eq!(shown[1], (String::from("cd"), None));
        assert_eq!(shown[2], (String::new(), Some(TextCaret::new(0, 0))));
        let unfocused = text_area(
            RECT,
            &edit,
            Response::new(ResponsePhase::Idle, false, false),
            (16, 0),
        );
        assert!(lines(&unfocused).iter().all(|(_, caret)| caret.is_none()));
        assert_eq!(unfocused.len(), 7);
    }

    #[test]
    fn a_selection_across_lines_is_split_and_bars_appear_once() {
        // 選択は "b\ncd\ne" のうち 1..6、カーソルは末尾側。
        let mut edit = TextEdit::showing("ab\ncd\nef").applied(KeyIntent::Move {
            to: crate::cursor_move::CursorMove::Left,
            extend: false,
        });
        for _ in 0_u8..6 {
            edit = edit.applied(KeyIntent::Move {
                to: crate::cursor_move::CursorMove::Left,
                extend: true,
            });
        }
        assert_eq!(edit.caret(), TextCaret::new(1, 7));
        let shown = lines(&text_area(RECT, &edit, focused(), (16, 0)));
        assert_eq!(shown[0].1, Some(TextCaret::new(1, 2)));
        assert_eq!(shown[1].1, Some(TextCaret::new(2, 0).without_bar()));
        assert_eq!(shown[2].1, Some(TextCaret::new(1, 0).without_bar()));
        // 逆向き（カーソルが後ろ）。
        let forward = TextEdit::showing("ab\ncd\nef").applied(KeyIntent::SelectAll);
        let all = lines(&text_area(RECT, &forward, focused(), (16, 0)));
        assert_eq!(all[0].1, Some(TextCaret::new(2, 0).without_bar()));
        assert_eq!(all[2].1, Some(TextCaret::new(2, 0)));
    }

    #[test]
    fn the_first_line_is_honoured_and_the_visible_count_follows_the_rect() {
        let edit = TextEdit::showing("1\n2\n3\n4\n5\n6");
        // 60 - 12 = 48 の高さに 16 の行は 3 つ。
        assert_eq!(visible_lines(RECT, 16), 3);
        assert_eq!(visible_lines(PixelRect::new(0, 0, 10, 10), 16), 1);
        let shown = lines(&text_area(RECT, &edit, focused(), (16, 3)));
        assert_eq!(
            shown
                .iter()
                .map(|(text, _)| text.as_str())
                .collect::<Vec<&str>>(),
            ["4", "5", "6"]
        );
        let top = lines(&text_area(RECT, &edit, focused(), (16, 0)));
        assert_eq!(top.first().map(|(text, _)| text.as_str()), Some("1"));
        assert_eq!(top.len(), 3);
        // カーソルは 6 行目で見えていないので棒は出ない。
        assert!(top.iter().all(|(_, caret)| caret.is_none()));
    }
}
