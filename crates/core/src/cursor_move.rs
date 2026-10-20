//! 入力欄のカーソルの動き。

/// カーソルをどこへ動かすか。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMove {
    /// 1 文字左へ。
    Left,
    /// 1 文字右へ。
    Right,
    /// 先頭へ。
    Home,
    /// 末尾へ。
    End,
}
