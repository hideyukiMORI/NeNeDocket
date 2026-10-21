//! 画面側で起こりうる失敗。閉じた集合（ARC-010）。

use core::fmt;

use x11rb::errors::{ConnectError, ConnectionError, ReplyError, ReplyOrIdError};

/// X の要求と cairo の失敗を 1 つの型に畳む。合成ルートはこれを 1 行で示して終了する（ARC-006）。
#[derive(Debug)]
pub enum UiFailure {
    /// X サーバに繋げなかった。
    Connect(ConnectError),
    /// 接続が切れた・要求を送れなかった。
    Connection(ConnectionError),
    /// 要求が X のエラーで返った。
    Reply(ReplyError),
    /// 要求が X のエラーで返った、または ID を使い切った。
    ReplyOrId(ReplyOrIdError),
    /// cairo が失敗した。
    Cairo(cairo::Error),
    /// cairo の画素を借りられなかった（描画中の `Context` が残っている）。
    CairoBorrow(cairo::BorrowError),
    /// 既定の画面が無い。
    NoScreen,
    /// 使える visual が無い（根の窓が depth 24 の `TrueColor` でない）。
    NoUsableVisual,
    /// 幾何が X の `i16` / `u16` に収まらない。
    GeometryOutOfRange,
}

impl fmt::Display for UiFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => write!(formatter, "cannot connect to the X server: {error}"),
            Self::Connection(error) => write!(formatter, "X connection failed: {error}"),
            Self::Reply(error) => write!(formatter, "X request failed: {error}"),
            Self::ReplyOrId(error) => write!(formatter, "X request failed: {error}"),
            Self::Cairo(error) => write!(formatter, "cairo failed: {error}"),
            Self::CairoBorrow(error) => write!(formatter, "cairo surface is busy: {error}"),
            Self::NoScreen => formatter.write_str("the X server has no default screen"),
            Self::NoUsableVisual => {
                formatter.write_str("the root window is not a 24-bit TrueColor visual")
            }
            Self::GeometryOutOfRange => formatter.write_str("window geometry exceeds the X limits"),
        }
    }
}

impl From<ConnectError> for UiFailure {
    fn from(error: ConnectError) -> Self {
        Self::Connect(error)
    }
}

impl From<ConnectionError> for UiFailure {
    fn from(error: ConnectionError) -> Self {
        Self::Connection(error)
    }
}

impl From<ReplyError> for UiFailure {
    fn from(error: ReplyError) -> Self {
        Self::Reply(error)
    }
}

impl From<ReplyOrIdError> for UiFailure {
    fn from(error: ReplyOrIdError) -> Self {
        Self::ReplyOrId(error)
    }
}

impl From<cairo::Error> for UiFailure {
    fn from(error: cairo::Error) -> Self {
        Self::Cairo(error)
    }
}

impl From<cairo::BorrowError> for UiFailure {
    fn from(error: cairo::BorrowError) -> Self {
        Self::CairoBorrow(error)
    }
}
