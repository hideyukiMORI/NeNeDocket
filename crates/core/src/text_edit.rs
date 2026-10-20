//! 入力欄の編集状態。文字列・カーソル・選択・確定前の文字列（FR-003 / FR-015）。
//!
//! 位置はすべて**文字**（`char`）の添字で、バイトでは持たない。途中のバイトで切る経路が無い（Issue #4 の受け入れ条件）。
//! 操作は値を返す純関数で、同じ状態と同じ意図から同じ状態が出る（ARC-007）。

use alloc::borrow::ToOwned;
use alloc::string::String;

use crate::cursor_move::CursorMove;
use crate::key_intent::KeyIntent;
use crate::preedit::Preedit;
use crate::text_caret::TextCaret;

/// 1 つの入力欄の、いまの文字列とカーソルと選択と確定前の文字列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    text: String,
    caret: TextCaret,
    preedit: Option<Preedit>,
}

impl TextEdit {
    /// 文字列から始める。カーソルは末尾。
    #[must_use]
    pub fn showing(text: &str) -> Self {
        let end = text.chars().count();
        Self {
            text: text.to_owned(),
            caret: TextCaret::new(end, end),
            preedit: None,
        }
    }

    /// 空から始める。
    #[must_use]
    pub fn empty() -> Self {
        Self::showing("")
    }

    /// 確定した文字列。確定前の文字列は含まない。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// カーソルと選択。
    #[must_use]
    pub const fn caret(&self) -> TextCaret {
        self.caret
    }

    /// 確定前の文字列。
    #[must_use]
    pub const fn preedit(&self) -> Option<&Preedit> {
        self.preedit.as_ref()
    }

    /// 確定前の文字列を置き換えた状態。IME が組み立てを始める・変える・やめるたびに UI が呼ぶ。
    #[must_use]
    pub fn with_preedit(&self, preedit: Option<Preedit>) -> Self {
        Self {
            text: self.text.clone(),
            caret: self.caret,
            preedit,
        }
    }

    /// 鍵の意図を適用した状態。入力欄が扱わない意図（Enter・Esc・Tab・上下）では変わらない。
    #[must_use]
    pub fn applied(&self, intent: KeyIntent) -> Self {
        match intent {
            KeyIntent::Char(character) => {
                let mut inserted = String::new();
                inserted.push(character);
                self.inserted(&inserted)
            }
            KeyIntent::Move { to, extend } => self.moved(to, extend),
            KeyIntent::Backspace => self.erased(CursorMove::Left),
            KeyIntent::Delete => self.erased(CursorMove::Right),
            KeyIntent::SelectAll => self.with_caret(TextCaret::new(self.char_count(), 0)),
            KeyIntent::Enter
            | KeyIntent::Escape
            | KeyIntent::Tab
            | KeyIntent::Up
            | KeyIntent::Down => self.clone(),
        }
    }

    /// 文字列を挿入した状態。選択があればそれを置き換え、確定前の文字列は消える（IME の確定）。
    #[must_use]
    pub fn inserted(&self, addition: &str) -> Self {
        let cleared = self.without_selection();
        let at = cleared.byte_offset(cleared.caret.cursor());
        let (head, tail) = cleared.text.split_at(at);
        let mut text = String::with_capacity(cleared.text.len().saturating_add(addition.len()));
        text.push_str(head);
        text.push_str(addition);
        text.push_str(tail);
        let cursor = cleared
            .caret
            .cursor()
            .saturating_add(addition.chars().count());
        Self {
            text,
            caret: TextCaret::new(cursor, cursor),
            preedit: None,
        }
    }

    /// 描く文字列と、その中のカーソル。確定前の文字列をカーソルの位置に差し込み、カーソルはその中へ移る。
    #[must_use]
    pub fn display(&self) -> (String, TextCaret) {
        let Some(preedit) = &self.preedit else {
            return (self.text.clone(), self.caret);
        };
        let shown = self.without_selection().inserted(preedit.text());
        let base = self.without_selection().caret.cursor();
        let cursor = base.saturating_add(preedit.cursor());
        (shown.text, TextCaret::new(cursor, cursor))
    }

    /// 文字数。
    #[must_use]
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    fn with_caret(&self, caret: TextCaret) -> Self {
        Self {
            text: self.text.clone(),
            caret,
            preedit: self.preedit.clone(),
        }
    }

    fn moved(&self, to: CursorMove, extend: bool) -> Self {
        let caret = self.caret;
        let cursor = match to {
            CursorMove::Left if caret.has_selection() && !extend => caret.start(),
            CursorMove::Right if caret.has_selection() && !extend => caret.end(),
            CursorMove::Left => caret.cursor().saturating_sub(1),
            CursorMove::Right => caret.cursor().saturating_add(1).min(self.char_count()),
            CursorMove::Home => 0,
            CursorMove::End => self.char_count(),
        };
        let anchor = if extend { caret.anchor() } else { cursor };
        self.with_caret(TextCaret::new(cursor, anchor))
    }

    /// 選択があればそれを、無ければカーソルの左か右の 1 文字を消した状態。
    fn erased(&self, side: CursorMove) -> Self {
        if self.caret.has_selection() {
            return self.without_selection();
        }
        let cursor = self.caret.cursor();
        let (start, end) = match side {
            CursorMove::Left => (cursor.saturating_sub(1), cursor),
            CursorMove::Right | CursorMove::Home | CursorMove::End => {
                (cursor, cursor.saturating_add(1).min(self.char_count()))
            }
        };
        self.removed(start, end)
    }

    fn without_selection(&self) -> Self {
        if self.caret.has_selection() {
            self.removed(self.caret.start(), self.caret.end())
        } else {
            self.clone()
        }
    }

    /// 文字の添字 `start..end` を消し、カーソルを `start` に置いた状態。
    fn removed(&self, start: usize, end: usize) -> Self {
        let mut text = self.text.clone();
        text.replace_range(self.byte_offset(start)..self.byte_offset(end), "");
        Self {
            text,
            caret: TextCaret::new(start, start),
            preedit: self.preedit.clone(),
        }
    }

    /// 文字の添字をバイトの位置に直す。末尾を超えていれば末尾。
    fn byte_offset(&self, char_index: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_index)
            .map_or(self.text.len(), |(offset, _)| offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn typed(edit: &TextEdit, text: &str) -> TextEdit {
        text.chars()
            .fold(edit.clone(), |edit, c| edit.applied(KeyIntent::Char(c)))
    }

    fn moved(edit: &TextEdit, to: CursorMove, extend: bool) -> TextEdit {
        edit.applied(KeyIntent::Move { to, extend })
    }

    #[test]
    fn typing_appends_at_the_cursor_in_characters_not_bytes() {
        let edit = typed(&TextEdit::empty(), "日本x");
        assert_eq!(edit.text(), "日本x");
        assert_eq!(edit.caret(), TextCaret::new(3, 3));
        let middle = moved(
            &moved(&edit, CursorMove::Left, false),
            CursorMove::Left,
            false,
        );
        assert_eq!(typed(&middle, "語").text(), "日語本x");
        assert_eq!(edit.char_count(), 3);
    }

    #[test]
    fn backspace_and_delete_remove_one_character_or_the_selection() {
        let edit = TextEdit::showing("あいう");
        assert_eq!(edit.applied(KeyIntent::Backspace).text(), "あい");
        assert_eq!(edit.applied(KeyIntent::Delete).text(), "あいう");
        let home = moved(&edit, CursorMove::Home, false);
        assert_eq!(home.applied(KeyIntent::Delete).text(), "いう");
        assert_eq!(home.applied(KeyIntent::Backspace).text(), "あいう");
        let selected = moved(&home, CursorMove::Right, true);
        let selected = moved(&selected, CursorMove::Right, true);
        assert_eq!(selected.caret(), TextCaret::new(2, 0));
        assert_eq!(selected.applied(KeyIntent::Backspace).text(), "う");
        assert_eq!(selected.applied(KeyIntent::Delete).text(), "う");
        assert_eq!(typed(&selected, "x").text(), "xう");
    }

    #[test]
    fn moving_collapses_a_selection_toward_the_moved_side() {
        let all = TextEdit::showing("abcd").applied(KeyIntent::SelectAll);
        assert_eq!(all.caret(), TextCaret::new(4, 0));
        assert_eq!(
            moved(&all, CursorMove::Left, false).caret(),
            TextCaret::new(0, 0)
        );
        assert_eq!(
            moved(&all, CursorMove::Right, false).caret(),
            TextCaret::new(4, 4)
        );
        assert_eq!(
            moved(&all, CursorMove::Left, true).caret(),
            TextCaret::new(3, 0)
        );
        assert_eq!(
            moved(&all, CursorMove::End, false).caret(),
            TextCaret::new(4, 4)
        );
        let start = moved(&all, CursorMove::Home, false);
        assert_eq!(
            moved(&start, CursorMove::Left, false).caret(),
            TextCaret::new(0, 0)
        );
        assert_eq!(
            moved(&all, CursorMove::Right, true).caret(),
            TextCaret::new(4, 0)
        );
    }

    #[test]
    fn preedit_is_shown_at_the_cursor_and_replaced_by_the_commit() {
        let edit = moved(&TextEdit::showing("ab"), CursorMove::Left, false);
        let composing = edit.with_preedit(Some(Preedit::new("にほ", 1)));
        let (shown, caret) = composing.display();
        assert_eq!(shown, "aにほb");
        assert_eq!(caret, TextCaret::new(2, 2));
        assert_eq!(composing.text(), "ab");
        assert_eq!(composing.preedit().map(Preedit::text), Some("にほ"));
        let committed = composing.inserted("日本");
        assert_eq!(committed.text(), "a日本b");
        assert_eq!(committed.preedit(), None);
        assert_eq!(committed.caret(), TextCaret::new(3, 3));
        assert_eq!(edit.display(), ("ab".to_owned(), TextCaret::new(1, 1)));
    }

    #[test]
    fn preedit_replaces_the_selection_when_committed() {
        let selected = TextEdit::showing("abc").applied(KeyIntent::SelectAll);
        let composing = selected.with_preedit(Some(Preedit::new("x", 1)));
        assert_eq!(composing.display().0, "x");
        assert_eq!(composing.inserted("y").text(), "y");
    }

    #[test]
    fn other_intents_leave_the_edit_alone() {
        let edit = TextEdit::showing("a");
        for intent in [
            KeyIntent::Enter,
            KeyIntent::Escape,
            KeyIntent::Tab,
            KeyIntent::Up,
            KeyIntent::Down,
        ] {
            assert_eq!(edit.applied(intent), edit);
        }
    }
}
