//! 使う atom をまとめて intern する。

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, ConnectionExt as _};

use crate::ui_failure::UiFailure;

/// 他人のジャーナルを読み直す合図の `ClientMessage` の型。アダプタの刻み（adapters-x11）が同じ名前で intern して送る。
pub const REFRESH_NAME: &[u8] = b"_NENE_DOCKET_REFRESH";

/// 閉じる要求・題・読み直しの合図に使う atom。
#[derive(Debug, Clone, Copy)]
pub struct Atoms {
    /// `WM_PROTOCOLS`。
    pub wm_protocols: Atom,
    /// `WM_DELETE_WINDOW`。閉じるボタンと Alt+F4 は WM からこれで届く。
    pub wm_delete_window: Atom,
    /// `_NET_WM_NAME`。
    pub net_wm_name: Atom,
    /// `UTF8_STRING`。
    pub utf8_string: Atom,
    /// [`REFRESH_NAME`]。届いたフレームで他人のジャーナルを読み直す。
    pub refresh: Atom,
}

/// 1 つの atom を intern する。
fn intern<C: Connection>(connection: &C, name: &[u8]) -> Result<Atom, UiFailure> {
    Ok(connection.intern_atom(false, name)?.reply()?.atom)
}

impl Atoms {
    /// 全部を intern する。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn intern<C: Connection>(connection: &C) -> Result<Self, UiFailure> {
        Ok(Self {
            wm_protocols: intern(connection, b"WM_PROTOCOLS")?,
            wm_delete_window: intern(connection, b"WM_DELETE_WINDOW")?,
            net_wm_name: intern(connection, b"_NET_WM_NAME")?,
            utf8_string: intern(connection, b"UTF8_STRING")?,
            refresh: intern(connection, REFRESH_NAME)?,
        })
    }
}
