//! クリップボードからの貼り付け（Ctrl+V・FR-015 の逃げ道・ADR 0020）。`CLIPBOARD` の `UTF8_STRING` を `ConvertSelection` で求め、
//! 答え（`SelectionNotify`）の property を読む。XDND の受け手（`drop_target`）と同じ仕組み。`INCR`（分割の転送）は受けない。

use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt as _, SelectionNotifyEvent, Window};

use crate::display::Display;
use crate::ui_failure::UiFailure;

/// 読む property の最大の長さ（32 ビット語・256 KiB）。
const PROPERTY_WORDS: u32 = 1 << 16;

/// 求めた変換の答えを待っているか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Clipboard {
    waiting: bool,
}

impl Clipboard {
    /// 何も求めていない。
    pub(crate) const fn idle() -> Self {
        Self { waiting: false }
    }

    /// `CLIPBOARD` を `UTF8_STRING` に変換して自分の property に置くよう求める。答えは `SelectionNotify`。
    pub(crate) fn request(
        &mut self,
        display: &Display,
        window: Window,
        time: u32,
    ) -> Result<(), UiFailure> {
        let atoms = display.atoms();
        display.connection().convert_selection(
            window,
            atoms.clipboard,
            atoms.utf8_string,
            atoms.paste_property,
            time,
        )?;
        display.connection().flush()?;
        self.waiting = true;
        Ok(())
    }

    /// 変換の答え。クリップボードのものなら property を読んで（消して）文字列にする。持ち主が無い・型が違う・`INCR` なら `None`。
    pub(crate) fn selection_notify(
        &mut self,
        display: &Display,
        window: Window,
        event: &SelectionNotifyEvent,
    ) -> Result<Option<String>, UiFailure> {
        let atoms = display.atoms();
        if event.selection != atoms.clipboard || !self.waiting {
            return Ok(None);
        }
        self.waiting = false;
        if event.property != atoms.paste_property {
            return Ok(None);
        }
        let reply = display
            .connection()
            .get_property(
                true,
                window,
                atoms.paste_property,
                AtomEnum::ANY,
                0,
                PROPERTY_WORDS,
            )?
            .reply()?;
        if reply.type_ == atoms.incr || reply.format != 8 {
            return Ok(None);
        }
        Ok(Some(String::from_utf8_lossy(&reply.value).into_owned()))
    }
}
