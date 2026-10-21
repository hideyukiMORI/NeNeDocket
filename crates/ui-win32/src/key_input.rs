//! 仮想鍵（`WM_KEYDOWN`）と文字（`WM_CHAR`）を、core の鍵の意図に読み替える。Win32 を呼ばないので Linux でもテストする。

use nene_docket_core::cursor_move::CursorMove;
use nene_docket_core::key_intent::KeyIntent;

/// `VK_BACK` / `VK_TAB` / `VK_RETURN` / `VK_ESCAPE`。
const BACK: u16 = 0x08;
const TAB: u16 = 0x09;
const RETURN: u16 = 0x0D;
const ESCAPE: u16 = 0x1B;
/// `VK_END` / `VK_HOME` / `VK_LEFT` / `VK_UP` / `VK_RIGHT` / `VK_DOWN` / `VK_DELETE`。
const END: u16 = 0x23;
const HOME: u16 = 0x24;
const LEFT: u16 = 0x25;
const UP: u16 = 0x26;
const RIGHT: u16 = 0x27;
const DOWN: u16 = 0x28;
const DELETE: u16 = 0x2E;
/// `A` / `Y` / `Z`（仮想鍵は大文字の ASCII）。Ctrl と一緒なら全選択・Redo・Undo。
const KEY_A: u16 = 0x41;
const KEY_Y: u16 = 0x59;
const KEY_Z: u16 = 0x5A;

/// 押されている修飾鍵。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    /// Shift。
    pub shift: bool,
    /// Ctrl。
    pub control: bool,
}

/// `WM_KEYDOWN` の仮想鍵を意図にする。印字できる字はここでは扱わず、`WM_CHAR` に任せる（`intent_of_char`）。
#[must_use]
pub fn intent_of_key(virtual_key: u16, modifiers: Modifiers) -> Option<KeyIntent> {
    let extend = modifiers.shift;
    match virtual_key {
        ESCAPE => Some(KeyIntent::Escape),
        RETURN => Some(KeyIntent::Enter),
        BACK => Some(KeyIntent::Backspace),
        DELETE => Some(KeyIntent::Delete),
        TAB if modifiers.shift => Some(KeyIntent::BackTab),
        TAB => Some(KeyIntent::Tab),
        LEFT => Some(moved(CursorMove::Left, extend)),
        RIGHT => Some(moved(CursorMove::Right, extend)),
        HOME => Some(moved(CursorMove::Home, extend)),
        END => Some(moved(CursorMove::End, extend)),
        UP => Some(KeyIntent::Up),
        DOWN => Some(KeyIntent::Down),
        KEY_A if modifiers.control => Some(KeyIntent::SelectAll),
        KEY_Z if modifiers.control => Some(KeyIntent::Undo),
        KEY_Y if modifiers.control => Some(KeyIntent::Redo),
        _ => None,
    }
}

/// `WM_CHAR` の字を意図にする。制御文字（Tab・Enter・Backspace・Esc・Ctrl+A など。`WM_KEYDOWN` で扱った）は `None`。
#[must_use]
pub fn intent_of_char(character: char) -> Option<KeyIntent> {
    if character.is_control() {
        None
    } else {
        Some(KeyIntent::Char(character))
    }
}

const fn moved(to: CursorMove, extend: bool) -> KeyIntent {
    KeyIntent::Move { to, extend }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLAIN: Modifiers = Modifiers {
        shift: false,
        control: false,
    };
    const SHIFT: Modifiers = Modifiers {
        shift: true,
        control: false,
    };
    const CONTROL: Modifiers = Modifiers {
        shift: false,
        control: true,
    };

    #[test]
    fn maps_editing_and_navigation_keys() {
        assert_eq!(intent_of_key(ESCAPE, PLAIN), Some(KeyIntent::Escape));
        assert_eq!(intent_of_key(RETURN, PLAIN), Some(KeyIntent::Enter));
        assert_eq!(intent_of_key(BACK, PLAIN), Some(KeyIntent::Backspace));
        assert_eq!(intent_of_key(DELETE, PLAIN), Some(KeyIntent::Delete));
        assert_eq!(intent_of_key(TAB, PLAIN), Some(KeyIntent::Tab));
        assert_eq!(intent_of_key(TAB, SHIFT), Some(KeyIntent::BackTab));
        assert_eq!(intent_of_key(UP, PLAIN), Some(KeyIntent::Up));
        assert_eq!(intent_of_key(DOWN, PLAIN), Some(KeyIntent::Down));
    }

    #[test]
    fn shift_extends_the_cursor_moves() {
        assert_eq!(
            intent_of_key(LEFT, SHIFT),
            Some(KeyIntent::Move {
                to: CursorMove::Left,
                extend: true
            })
        );
        assert_eq!(
            intent_of_key(END, PLAIN),
            Some(KeyIntent::Move {
                to: CursorMove::End,
                extend: false
            })
        );
        assert_eq!(
            intent_of_key(HOME, PLAIN),
            Some(KeyIntent::Move {
                to: CursorMove::Home,
                extend: false
            })
        );
        assert_eq!(
            intent_of_key(RIGHT, PLAIN),
            Some(KeyIntent::Move {
                to: CursorMove::Right,
                extend: false
            })
        );
    }

    #[test]
    fn control_shortcuts_and_plain_letters() {
        assert_eq!(intent_of_key(KEY_A, CONTROL), Some(KeyIntent::SelectAll));
        assert_eq!(intent_of_key(KEY_Z, CONTROL), Some(KeyIntent::Undo));
        assert_eq!(intent_of_key(KEY_Y, CONTROL), Some(KeyIntent::Redo));
        assert_eq!(intent_of_key(KEY_A, PLAIN), None);
        assert_eq!(intent_of_key(0x70, PLAIN), None);
    }

    #[test]
    fn characters_pass_through_and_controls_do_not() {
        assert_eq!(intent_of_char('a'), Some(KeyIntent::Char('a')));
        assert_eq!(intent_of_char('日'), Some(KeyIntent::Char('日')));
        assert_eq!(intent_of_char(' '), Some(KeyIntent::Char(' ')));
        assert_eq!(intent_of_char('\t'), None);
        assert_eq!(intent_of_char('\r'), None);
        assert_eq!(intent_of_char('\u{1}'), None);
        assert_eq!(intent_of_char('\u{7f}'), None);
    }
}
