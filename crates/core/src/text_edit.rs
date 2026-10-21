//! 入力欄の編集状態。文字列・カーソル・選択・確定前の文字列（FR-003 / FR-015）。
//!
//! 位置はすべて**文字**（`char`）の添字で、バイトでは持たない。途中のバイトで切る経路が無い（Issue #4 の受け入れ条件）。
//! 操作は値を返す純関数で、同じ状態と同じ意図から同じ状態が出る（ARC-007）。
//! 文字列は `\n` で行に分かれる。Home / End は行の中で動き、複数行の欄（[`crate::text_area`]）では Enter が改行を挿し、上下が隣の行へ動く。

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

    /// 複数行の欄として鍵の意図を適用した状態。Enter は改行を挿し、上下は隣の行の同じ桁へ。ほかは [`Self::applied`]。
    #[must_use]
    pub fn applied_in_area(&self, intent: KeyIntent) -> Self {
        match intent {
            KeyIntent::Enter | KeyIntent::Char('\n') => self.inserted("\n"),
            KeyIntent::Up => self.moved_by_line(-1_i8),
            KeyIntent::Down => self.moved_by_line(1_i8),
            KeyIntent::Char(_)
            | KeyIntent::Move { .. }
            | KeyIntent::Backspace
            | KeyIntent::Delete
            | KeyIntent::SelectAll
            | KeyIntent::Escape
            | KeyIntent::Tab
            | KeyIntent::BackTab
            | KeyIntent::Undo
            | KeyIntent::Redo => self.applied(intent),
        }
    }

    /// 文字の添字がある行（0 から）と、行の中の桁（文字数）。
    #[must_use]
    pub fn position_of(&self, index: usize) -> (usize, usize) {
        let mut line = 0_usize;
        let mut column = 0_usize;
        for character in self.text.chars().take(index) {
            if character == '\n' {
                line = line.saturating_add(1);
                column = 0;
            } else {
                column = column.saturating_add(1);
            }
        }
        (line, column)
    }

    /// 行と桁の文字の添字。行が無ければ末尾、桁が行より長ければ行の末尾。
    #[must_use]
    pub fn index_of(&self, line: usize, column: usize) -> usize {
        let mut index = 0_usize;
        let mut current = 0_usize;
        for character in self.text.chars() {
            if current == line {
                break;
            }
            if character == '\n' {
                current = current.saturating_add(1);
            }
            index = index.saturating_add(1);
        }
        let length = self
            .text
            .chars()
            .skip(index)
            .take_while(|character| *character != '\n')
            .count();
        index.saturating_add(column.min(length))
    }

    /// 鍵の意図を適用した状態。入力欄が扱わない意図（Enter・Esc・Tab・上下）では変わらない。
    /// 改行の字（貼り付け・IME の確定で届く）は 1 行の欄では空白にする（複数行の欄は [`Self::applied_in_area`]）。
    #[must_use]
    pub fn applied(&self, intent: KeyIntent) -> Self {
        match intent {
            KeyIntent::Char('\n') => self.inserted(" "),
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
            | KeyIntent::BackTab
            | KeyIntent::Up
            | KeyIntent::Down
            | KeyIntent::Undo
            | KeyIntent::Redo => self.clone(),
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
        let end = base.saturating_add(preedit.char_count());
        (
            shown.text,
            TextCaret::new(cursor, cursor).with_underline(base, end),
        )
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
            CursorMove::Home => {
                let (line, _) = self.position_of(caret.cursor());
                self.index_of(line, 0)
            }
            CursorMove::End => {
                let (line, _) = self.position_of(caret.cursor());
                self.index_of(line, usize::MAX)
            }
        };
        let anchor = if extend { caret.anchor() } else { cursor };
        self.with_caret(TextCaret::new(cursor, anchor))
    }

    /// 隣の行の同じ桁へ。最初の行から上は先頭、最後の行から下は末尾。選択は畳む。
    fn moved_by_line(&self, delta: i8) -> Self {
        let (line, column) = self.position_of(self.caret.cursor());
        let cursor = if delta < 0 {
            match line.checked_sub(1) {
                Some(above) => self.index_of(above, column),
                None => 0,
            }
        } else if line.saturating_add(1) < self.line_count() {
            self.index_of(line.saturating_add(1), column)
        } else {
            self.char_count()
        };
        self.with_caret(TextCaret::new(cursor, cursor))
    }

    /// 行の数（空でも 1）。
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.text
            .chars()
            .filter(|character| *character == '\n')
            .count()
            .saturating_add(1)
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
        assert_eq!(caret, TextCaret::new(2, 2).with_underline(1, 3));
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
    fn a_newline_character_becomes_a_space_in_a_field_and_a_line_break_in_an_area() {
        let edit = TextEdit::showing("ab").applied(KeyIntent::Char('\n'));
        assert_eq!(edit.text(), "ab ");
        let area = TextEdit::showing("ab").applied_in_area(KeyIntent::Char('\n'));
        assert_eq!(area.text(), "ab\n");
    }

    #[test]
    fn lines_and_columns_map_both_ways() {
        let edit = TextEdit::showing("ab\ncde\nf");
        assert_eq!(edit.line_count(), 3);
        assert_eq!(edit.position_of(edit.caret().cursor()), (2, 1));
        assert_eq!(edit.index_of(1, 1), 4);
        assert_eq!(edit.index_of(1, 9), 6);
        assert_eq!(edit.index_of(7, 0), 8);
        assert_eq!(TextEdit::empty().line_count(), 1);
    }

    #[test]
    fn home_and_end_move_within_the_current_line() {
        let up = TextEdit::showing("ab\ncde\nf").applied_in_area(KeyIntent::Up);
        assert_eq!(up.caret(), TextCaret::new(4, 4));
        let home = moved(&up, CursorMove::Home, false);
        assert_eq!(home.caret(), TextCaret::new(3, 3));
        let end = moved(&up, CursorMove::End, true);
        assert_eq!(end.caret(), TextCaret::new(6, 4));
    }

    #[test]
    fn walking_past_the_first_or_last_line_lands_on_the_ends() {
        let top = TextEdit::showing("ab\ncde\nf")
            .applied_in_area(KeyIntent::Up)
            .applied_in_area(KeyIntent::Up)
            .applied_in_area(KeyIntent::Up);
        assert_eq!(top.caret(), TextCaret::new(0, 0));
        let bottom = top
            .applied_in_area(KeyIntent::Down)
            .applied_in_area(KeyIntent::Down)
            .applied_in_area(KeyIntent::Down);
        assert_eq!(bottom.caret(), TextCaret::new(8, 8));
    }

    #[test]
    fn enter_in_the_area_inserts_a_line_break_and_tab_is_left_alone() {
        let home = moved(
            &TextEdit::showing("ab\ncde\nf").applied_in_area(KeyIntent::Up),
            CursorMove::Home,
            false,
        );
        let broken = home.applied_in_area(KeyIntent::Enter);
        assert_eq!(broken.text(), "ab\n\ncde\nf");
        assert_eq!(broken.caret(), TextCaret::new(4, 4));
        assert_eq!(
            broken.applied_in_area(KeyIntent::Char('x')).text(),
            "ab\n\nxcde\nf"
        );
        assert_eq!(broken.applied_in_area(KeyIntent::Tab), broken);
        assert_eq!(TextEdit::empty().line_count(), 1);
    }

    #[test]
    fn other_intents_leave_the_edit_alone() {
        let edit = TextEdit::showing("a");
        for intent in [
            KeyIntent::Enter,
            KeyIntent::Escape,
            KeyIntent::Tab,
            KeyIntent::BackTab,
            KeyIntent::Up,
            KeyIntent::Down,
            KeyIntent::Undo,
            KeyIntent::Redo,
        ] {
            assert_eq!(edit.applied(intent), edit);
        }
    }
}
