//! XDND（版 5）の受け手（FR-021 の第 2 段）。`XdndEnter` → `XdndPosition`（`XdndStatus` を返す）→ `XdndDrop`（選択を変換して読む）→ `XdndFinished`。
//! 受けるのは `text/uri-list` だけ。送り手の窓・位置（ルートの座標）・時刻を覚える。

use nene_docket_core::pixel_point::PixelPoint;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConnectionExt as _, EventMask, SelectionNotifyEvent, Window,
};

use crate::display::Display;
use crate::ui_failure::UiFailure;
use crate::uri_list;

/// XDND の版。
const VERSION: u32 = 5;
/// 読む property の最大の長さ（32 ビット語）。
const PROPERTY_WORDS: u32 = 1 << 20;

/// 送り手の窓・`text/uri-list` を出しているか・最後の位置（ルートの座標）・落とした時刻。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DropTarget {
    source: Option<Window>,
    offered: bool,
    root_position: (i16, i16),
    time: u32,
}

impl DropTarget {
    /// 何も来ていない。
    pub(crate) const fn idle() -> Self {
        Self {
            source: None,
            offered: false,
            root_position: (0, 0),
            time: 0,
        }
    }

    /// XDND の `ClientMessage` を扱う。扱ったら真（他の `ClientMessage` なら偽）。
    pub(crate) fn client_message(
        &mut self,
        display: &Display,
        window: Window,
        message: &ClientMessageEvent,
    ) -> Result<bool, UiFailure> {
        let atoms = display.atoms();
        let words = message.data.as_data32();
        if message.type_ == atoms.xdnd_enter {
            self.enter(display, words)?;
        } else if message.type_ == atoms.xdnd_position {
            self.position(display, window, words)?;
        } else if message.type_ == atoms.xdnd_drop {
            self.drop(display, window, words)?;
        } else if message.type_ == atoms.xdnd_leave {
            *self = Self::idle();
        } else {
            return Ok(false);
        }
        Ok(true)
    }

    /// 送り手の窓と、出している型。3 つを超えるなら `XdndTypeList` を読む。
    fn enter(&mut self, display: &Display, words: [u32; 5]) -> Result<(), UiFailure> {
        let atoms = display.atoms();
        let [source, flags, first, second, third] = words;
        let listed = [first, second, third].contains(&atoms.text_uri_list);
        let offered = if flags & 1 == 1 {
            let reply = display
                .connection()
                .get_property(
                    false,
                    source,
                    atoms.xdnd_type_list,
                    AtomEnum::ATOM,
                    0,
                    PROPERTY_WORDS,
                )?
                .reply()?;
            reply
                .value32()
                .is_some_and(|mut types| types.any(|atom| atom == atoms.text_uri_list))
        } else {
            listed
        };
        *self = Self {
            source: Some(source),
            offered,
            root_position: (0, 0),
            time: 0,
        };
        Ok(())
    }

    /// 位置を覚え、受けるかどうかを `XdndStatus` で返す。
    fn position(
        &mut self,
        display: &Display,
        window: Window,
        words: [u32; 5],
    ) -> Result<(), UiFailure> {
        let atoms = display.atoms();
        let [source, _flags, packed, _time, _action] = words;
        if self.source != Some(source) {
            return Ok(());
        }
        self.root_position = (
            i16::try_from(packed >> 16_u32).unwrap_or(0),
            i16::try_from(packed & 0xFFFF).unwrap_or(0),
        );
        let action = if self.offered {
            atoms.xdnd_action_copy
        } else {
            0
        };
        send(
            display,
            source,
            atoms.xdnd_status,
            [window, u32::from(self.offered), 0, 0, action],
        )
    }

    /// 落とされた。受けるなら選択の変換を求める（答えは `SelectionNotify`）。受けないなら終わりを返す。
    fn drop(
        &mut self,
        display: &Display,
        window: Window,
        words: [u32; 5],
    ) -> Result<(), UiFailure> {
        let atoms = display.atoms();
        let [source, _flags, time, _unused, _unused2] = words;
        if self.source != Some(source) {
            return Ok(());
        }
        if !self.offered {
            self.finish(display, window, false)?;
            return Ok(());
        }
        self.time = time;
        display.connection().convert_selection(
            window,
            atoms.xdnd_selection,
            atoms.text_uri_list,
            atoms.drop_property,
            time,
        )?;
        display.connection().flush()?;
        Ok(())
    }

    /// 変換の答え。この受け手のものなら property を読んでパスにし、位置を窓の座標にして返す。
    pub(crate) fn selection_notify(
        &mut self,
        display: &Display,
        window: Window,
        event: &SelectionNotifyEvent,
    ) -> Result<Option<(PixelPoint, Vec<String>)>, UiFailure> {
        let atoms = display.atoms();
        if event.selection != atoms.xdnd_selection || self.source.is_none() {
            return Ok(None);
        }
        if event.property != atoms.drop_property {
            self.finish(display, window, false)?;
            return Ok(None);
        }
        let reply = display
            .connection()
            .get_property(
                true,
                window,
                atoms.drop_property,
                AtomEnum::ANY,
                0,
                PROPERTY_WORDS,
            )?
            .reply()?;
        let paths = uri_list::file_paths(&String::from_utf8_lossy(&reply.value));
        let root = display.screen()?.root;
        let (root_x, root_y) = self.root_position;
        let local = display
            .connection()
            .translate_coordinates(root, window, root_x, root_y)?
            .reply()?;
        let point = PixelPoint::new(
            u16::try_from(local.dst_x).unwrap_or(0),
            u16::try_from(local.dst_y).unwrap_or(0),
        );
        self.finish(display, window, !paths.is_empty())?;
        Ok(Some((point, paths)))
    }

    /// `XdndFinished` を送り、忘れる。
    fn finish(
        &mut self,
        display: &Display,
        window: Window,
        accepted: bool,
    ) -> Result<(), UiFailure> {
        let atoms = display.atoms();
        if let Some(source) = self.source {
            let action = if accepted { atoms.xdnd_action_copy } else { 0 };
            send(
                display,
                source,
                atoms.xdnd_finished,
                [window, u32::from(accepted), action, 0, 0],
            )?;
        }
        *self = Self::idle();
        Ok(())
    }

    /// 版（`XdndAware` に書く値）。
    pub(crate) const fn version() -> u32 {
        VERSION
    }
}

/// 送り手の窓に 32 ビット × 5 の `ClientMessage` を送る。
fn send(display: &Display, to: Window, kind: u32, words: [u32; 5]) -> Result<(), UiFailure> {
    let event = ClientMessageEvent::new(32, to, kind, words);
    display
        .connection()
        .send_event(false, to, EventMask::NO_EVENT, event)?;
    display.connection().flush()?;
    Ok(())
}
