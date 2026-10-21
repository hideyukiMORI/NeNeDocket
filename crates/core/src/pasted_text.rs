//! 貼り付けた文字列（クリップボード）と IME の確定した文字列を、鍵の意図の列にする（FR-015）。

use alloc::vec::Vec;

use crate::key_intent::KeyIntent;

/// 1 字ずつ [`KeyIntent::Char`] にする。`\r\n` と `\r` は `\n` に揃える（1 行の欄では空白に、複数行の欄では改行になる）。
/// タブは空白にし、ほかの制御文字は捨てる。
#[must_use]
pub fn key_intents(text: &str) -> Vec<KeyIntent> {
    let mut intents = Vec::new();
    let mut after_return = false;
    for character in text.chars() {
        let carried = after_return;
        after_return = character == '\r';
        let mapped = match character {
            '\n' if carried => None,
            '\r' | '\n' => Some('\n'),
            '\t' => Some(' '),
            control if control.is_control() => None,
            printable => Some(printable),
        };
        intents.extend(mapped.map(KeyIntent::Char));
    }
    intents
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chars(intents: &[KeyIntent]) -> alloc::string::String {
        intents
            .iter()
            .map(|intent| match intent {
                KeyIntent::Char(character) => *character,
                KeyIntent::Move { .. }
                | KeyIntent::Backspace
                | KeyIntent::Delete
                | KeyIntent::SelectAll
                | KeyIntent::Enter
                | KeyIntent::Escape
                | KeyIntent::Tab
                | KeyIntent::BackTab
                | KeyIntent::Up
                | KeyIntent::Down
                | KeyIntent::Undo
                | KeyIntent::Redo => '?',
            })
            .collect()
    }

    #[test]
    fn line_breaks_are_normalized_and_control_characters_dropped() {
        assert_eq!(chars(&key_intents("日本\r\n語\rx\ny")), "日本\n語\nx\ny");
        assert_eq!(chars(&key_intents("a\tb\u{1}c\u{7f}")), "a bc");
        assert_eq!(chars(&key_intents("\r\n\r\n")), "\n\n");
        assert!(key_intents("").is_empty());
    }
}
