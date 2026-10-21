//! keysym と修飾鍵を、core の鍵の意図に読み替える。

use nene_docket_core::cursor_move::CursorMove;
use nene_docket_core::key_intent::KeyIntent;

/// `XK_Escape`。
const ESCAPE: u32 = 0xff1b;
/// `XK_Return` / `XK_KP_Enter`。
const RETURN: u32 = 0xff0d;
const KP_ENTER: u32 = 0xff8d;
/// `XK_BackSpace` / `XK_Delete`。
const BACKSPACE: u32 = 0xff08;
const DELETE: u32 = 0xffff;
/// `XK_Tab` / `XK_ISO_Left_Tab`（Shift+Tab は多くの配列でこれになる）。
const TAB: u32 = 0xff09;
const ISO_LEFT_TAB: u32 = 0xfe20;
/// `XK_Left` / `XK_Up` / `XK_Right` / `XK_Down` / `XK_Home` / `XK_End`。
const LEFT: u32 = 0xff51;
const UP: u32 = 0xff52;
const RIGHT: u32 = 0xff53;
const DOWN: u32 = 0xff54;
const HOME: u32 = 0xff50;
const END: u32 = 0xff57;
/// `XK_a` / `XK_A`。Ctrl と一緒なら全選択。
const LOWER_A: u32 = 0x61;
const UPPER_A: u32 = 0x41;

/// 押されている修飾鍵。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Modifiers {
    /// Shift。
    pub shift: bool,
    /// Ctrl。
    pub control: bool,
}

/// keysym を意図にする。反応しない鍵は `None`。印字できる ASCII と Latin-1 は keysym の値がそのまま文字になる。
#[must_use]
pub fn intent_of(keysym: u32, modifiers: Modifiers) -> Option<KeyIntent> {
    let extend = modifiers.shift;
    match keysym {
        ESCAPE => Some(KeyIntent::Escape),
        RETURN | KP_ENTER => Some(KeyIntent::Enter),
        BACKSPACE => Some(KeyIntent::Backspace),
        DELETE => Some(KeyIntent::Delete),
        ISO_LEFT_TAB => Some(KeyIntent::BackTab),
        TAB if modifiers.shift => Some(KeyIntent::BackTab),
        TAB => Some(KeyIntent::Tab),
        LEFT => Some(moved(CursorMove::Left, extend)),
        RIGHT => Some(moved(CursorMove::Right, extend)),
        HOME => Some(moved(CursorMove::Home, extend)),
        END => Some(moved(CursorMove::End, extend)),
        UP => Some(KeyIntent::Up),
        DOWN => Some(KeyIntent::Down),
        LOWER_A | UPPER_A if modifiers.control => Some(KeyIntent::SelectAll),
        _ if modifiers.control => None,
        0x20..=0x7e | 0xa0..=0xff => char::from_u32(keysym).map(KeyIntent::Char),
        _ => None,
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
    fn maps_editing_keys() {
        assert_eq!(intent_of(ESCAPE, PLAIN), Some(KeyIntent::Escape));
        assert_eq!(intent_of(KP_ENTER, PLAIN), Some(KeyIntent::Enter));
        assert_eq!(intent_of(BACKSPACE, PLAIN), Some(KeyIntent::Backspace));
        assert_eq!(intent_of(DELETE, PLAIN), Some(KeyIntent::Delete));
        assert_eq!(intent_of(TAB, PLAIN), Some(KeyIntent::Tab));
        assert_eq!(intent_of(TAB, SHIFT), Some(KeyIntent::BackTab));
        assert_eq!(intent_of(ISO_LEFT_TAB, SHIFT), Some(KeyIntent::BackTab));
        assert_eq!(intent_of(UP, PLAIN), Some(KeyIntent::Up));
        assert_eq!(intent_of(DOWN, PLAIN), Some(KeyIntent::Down));
    }

    #[test]
    fn shift_extends_the_cursor_moves_and_control_a_selects_all() {
        assert_eq!(
            intent_of(LEFT, PLAIN),
            Some(KeyIntent::Move {
                to: CursorMove::Left,
                extend: false
            })
        );
        assert_eq!(
            intent_of(END, SHIFT),
            Some(KeyIntent::Move {
                to: CursorMove::End,
                extend: true
            })
        );
        assert_eq!(intent_of(LOWER_A, CONTROL), Some(KeyIntent::SelectAll));
        assert_eq!(intent_of(UPPER_A, CONTROL), Some(KeyIntent::SelectAll));
        assert_eq!(intent_of(0x62, CONTROL), None);
    }

    #[test]
    fn printable_characters_pass_through_and_others_do_not() {
        assert_eq!(intent_of(0x23, PLAIN), Some(KeyIntent::Char('#')));
        assert_eq!(intent_of(0x20, PLAIN), Some(KeyIntent::Char(' ')));
        assert_eq!(intent_of(0xe9, PLAIN), Some(KeyIntent::Char('é')));
        assert_eq!(intent_of(0xffe1, PLAIN), None);
        assert_eq!(intent_of(0x7f, PLAIN), None);
    }
}
