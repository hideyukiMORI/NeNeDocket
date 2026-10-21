//! セレクトボックスの状態（FR-002 の絞り込み・FR-001 の状態と担当の選択）。

use crate::key_intent::KeyIntent;

/// 候補の数と選んでいる番号と、開いているかと、開いているときに光らせている番号。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectState {
    count: usize,
    chosen: usize,
    open: bool,
    highlighted: usize,
}

impl SelectState {
    /// 候補の数と選んでいる番号から、閉じた状態で始める。番号が数を超えていれば最後の候補。
    #[must_use]
    pub const fn closed(count: usize, chosen: usize) -> Self {
        let chosen = Self::clamp(count, chosen);
        Self {
            count,
            chosen,
            open: false,
            highlighted: chosen,
        }
    }

    /// 候補の数。
    #[must_use]
    pub const fn count(self) -> usize {
        self.count
    }

    /// 選んでいる番号。
    #[must_use]
    pub const fn chosen(self) -> usize {
        self.chosen
    }

    /// 候補の一覧が開いているか。
    #[must_use]
    pub const fn is_open(self) -> bool {
        self.open
    }

    /// 開いているときに光らせている番号。
    #[must_use]
    pub const fn highlighted(self) -> usize {
        self.highlighted
    }

    /// 開閉を切り替えた状態（クリック）。開くときは選んでいる候補を光らせる。
    #[must_use]
    pub const fn toggled(self) -> Self {
        Self {
            open: !self.open,
            highlighted: self.chosen,
            ..self
        }
    }

    /// `index` の候補を選んで閉じた状態（候補のクリック）。
    #[must_use]
    pub const fn chosen_at(self, index: usize) -> Self {
        Self::closed(self.count, index)
    }

    /// 鍵の意図を適用した状態。閉じていれば上下で直接選び、開いていれば上下で光らせ Enter で決め Esc でやめる。
    #[must_use]
    pub const fn applied(self, intent: KeyIntent) -> Self {
        match intent {
            KeyIntent::Up => self.stepped(true),
            KeyIntent::Down => self.stepped(false),
            KeyIntent::Enter if self.open => self.chosen_at(self.highlighted),
            KeyIntent::Enter => self.toggled(),
            KeyIntent::Escape if self.open => Self::closed(self.count, self.chosen),
            KeyIntent::Escape
            | KeyIntent::Char(_)
            | KeyIntent::Move { .. }
            | KeyIntent::Backspace
            | KeyIntent::Delete
            | KeyIntent::SelectAll
            | KeyIntent::Tab
            | KeyIntent::BackTab => self,
        }
    }

    const fn stepped(self, up: bool) -> Self {
        let current = if self.open {
            self.highlighted
        } else {
            self.chosen
        };
        let next = if up {
            current.saturating_sub(1)
        } else {
            Self::clamp(self.count, current.saturating_add(1))
        };
        if self.open {
            Self {
                highlighted: next,
                ..self
            }
        } else {
            Self::closed(self.count, next)
        }
    }

    const fn clamp(count: usize, index: usize) -> usize {
        if count == 0 {
            0
        } else if index >= count {
            count - 1
        } else {
            index
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chosen_index_is_clamped_to_the_candidates() {
        assert_eq!(SelectState::closed(3, 7).chosen(), 2);
        assert_eq!(SelectState::closed(0, 7).chosen(), 0);
        assert_eq!(SelectState::closed(3, 1).count(), 3);
    }

    #[test]
    fn arrows_choose_directly_while_closed() {
        let state = SelectState::closed(3, 0);
        let down = state.applied(KeyIntent::Down);
        assert_eq!(down.chosen(), 1);
        assert!(!down.is_open());
        assert_eq!(
            down.applied(KeyIntent::Down)
                .applied(KeyIntent::Down)
                .chosen(),
            2
        );
        assert_eq!(state.applied(KeyIntent::Up).chosen(), 0);
    }

    #[test]
    fn open_list_highlights_and_enter_commits_escape_cancels() {
        let open = SelectState::closed(3, 0).toggled();
        assert!(open.is_open());
        let moved = open.applied(KeyIntent::Down).applied(KeyIntent::Down);
        assert_eq!(moved.highlighted(), 2);
        assert_eq!(moved.chosen(), 0);
        let committed = moved.applied(KeyIntent::Enter);
        assert_eq!(committed.chosen(), 2);
        assert!(!committed.is_open());
        let cancelled = moved.applied(KeyIntent::Escape);
        assert_eq!(cancelled.chosen(), 0);
        assert!(!cancelled.is_open());
        assert_eq!(moved.applied(KeyIntent::Up).highlighted(), 1);
    }

    #[test]
    fn enter_opens_a_closed_box_and_click_chooses() {
        let state = SelectState::closed(3, 1);
        assert!(state.applied(KeyIntent::Enter).is_open());
        assert_eq!(state.applied(KeyIntent::Enter).highlighted(), 1);
        assert_eq!(state.toggled().chosen_at(2).chosen(), 2);
        assert!(!state.toggled().chosen_at(2).is_open());
        assert_eq!(state.applied(KeyIntent::Escape), state);
        assert_eq!(state.applied(KeyIntent::Tab), state);
        assert_eq!(state.applied(KeyIntent::Char('a')), state);
    }
}
