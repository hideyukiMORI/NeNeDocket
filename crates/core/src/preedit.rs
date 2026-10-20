//! IME が組み立てている確定前の文字列（FR-015）。

use alloc::borrow::ToOwned;
use alloc::string::String;

/// 確定前の文字列と、その中のカーソルの位置（文字の添字）。UI が IME から受け取り、入力欄の編集状態に渡す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Preedit {
    text: String,
    cursor: usize,
}

impl Preedit {
    /// 文字列とカーソルから作る。カーソルが文字数を超えていれば末尾に寄せる。
    #[must_use]
    pub fn new(text: &str, cursor: usize) -> Self {
        let count = text.chars().count();
        Self {
            text: text.to_owned(),
            cursor: if cursor > count { count } else { cursor },
        }
    }

    /// 文字列。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// カーソル（文字の添字）。
    #[must_use]
    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    /// 文字数。
    #[must_use]
    pub fn char_count(&self) -> usize {
        self.text.chars().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_is_clamped_to_the_character_count() {
        let preedit = Preedit::new("にほ", 9);
        assert_eq!(preedit.cursor(), 2);
        assert_eq!(preedit.char_count(), 2);
        assert_eq!(preedit.text(), "にほ");
        assert_eq!(Preedit::new("ab", 1).cursor(), 1);
    }
}
