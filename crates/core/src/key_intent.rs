//! 鍵の意図。

use crate::cursor_move::CursorMove;

/// UI が鍵の押下を OS の keysym から写した、部品への意図。閉じた集合（RS-002）。
///
/// 文字は確定した 1 文字（IME の確定を含む）。確定前の文字列は [`crate::preedit::Preedit`] で別に渡す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyIntent {
    /// 確定した 1 文字。
    Char(char),
    /// カーソルの移動。`extend` なら選択を伸ばす（Shift）。
    Move {
        /// どこへ。
        to: CursorMove,
        /// 選択を伸ばすか。
        extend: bool,
    },
    /// カーソルの左を消す。選択があればそれを消す。
    Backspace,
    /// カーソルの右を消す。選択があればそれを消す。
    Delete,
    /// 全選択（Ctrl+A）。
    SelectAll,
    /// 確定（Enter）。
    Enter,
    /// 取り消し（Esc）。
    Escape,
    /// 次の部品へ（Tab）。
    Tab,
    /// 上へ（セレクトボックスの候補）。
    Up,
    /// 下へ。
    Down,
}
