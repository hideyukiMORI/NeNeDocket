//! 台帳の窓。作る・描いたものを転送する・大きさを覚える。OS の通常の窓（FR-018）。

use nene_docket_application::docket_frame::DocketFrame;
use nene_docket_core::language::Language;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pixel_rect::PixelRect;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ClientMessageEvent, ConnectionExt as _, CreateGCAux, CreateWindowAux, EventMask,
    GcontextWrapper, PropMode, SelectionNotifyEvent, Window, WindowClass, WindowWrapper,
};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::canvas::Canvas;
use crate::clipboard::Clipboard;
use crate::display::Display;
use crate::drop_target::DropTarget;
use crate::font_family;
use crate::surface_cache::SurfaceCache;
use crate::transfer::Transfer;
use crate::ui_failure::UiFailure;

/// 窓の題。
const TITLE: &str = "NeNe Docket";
/// 最初の大きさ。
const INITIAL_WIDTH: u16 = 1120;
const INITIAL_HEIGHT: u16 = 600;

/// 窓とその OS 資源。`Drop` で閉じる（規約 第 4 節）。
#[derive(Debug)]
pub struct DocketWindow<'d> {
    display: &'d Display,
    window: WindowWrapper<&'d RustConnection>,
    gc: GcontextWrapper<&'d RustConnection>,
    bounds: PixelRect,
    /// 直前のフレームの言語と、その字を描ける家族。言語が変わったら選び直す（ADR 0011）。
    font: Option<(Language, &'static str)>,
    /// 添付の画像の surface の覚え（FR-022）。
    surfaces: SurfaceCache,
    /// XDND の受け手（FR-021 の第 2 段）。
    drops: DropTarget,
    /// クリップボードからの貼り付け（FR-015 の逃げ道）。
    clipboard: Clipboard,
}

impl<'d> DocketWindow<'d> {
    /// 窓を作って表示する。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn open(display: &'d Display) -> Result<Self, UiFailure> {
        let connection = display.connection();
        let screen = display.screen()?;
        let visual = display.visual();
        let attributes = CreateWindowAux::new()
            .background_pixel(screen.white_pixel)
            .event_mask(
                EventMask::EXPOSURE
                    | EventMask::STRUCTURE_NOTIFY
                    | EventMask::BUTTON_PRESS
                    | EventMask::BUTTON_RELEASE
                    | EventMask::POINTER_MOTION
                    | EventMask::LEAVE_WINDOW
                    | EventMask::KEY_PRESS,
            );
        let window = WindowWrapper::create_window(
            connection,
            visual.depth(),
            screen.root,
            0,
            0,
            INITIAL_WIDTH,
            INITIAL_HEIGHT,
            0,
            WindowClass::INPUT_OUTPUT,
            visual.visual(),
            &attributes,
        )?;
        let gc = GcontextWrapper::create_gc(connection, window.window(), &CreateGCAux::new())?;
        let opened = Self {
            display,
            window,
            gc,
            bounds: PixelRect::new(0, 0, INITIAL_WIDTH, INITIAL_HEIGHT),
            font: None,
            surfaces: SurfaceCache::empty(),
            drops: DropTarget::idle(),
            clipboard: Clipboard::idle(),
        };
        opened.declare_hints()?;
        connection.map_window(opened.window.window())?;
        connection.flush()?;
        Ok(opened)
    }

    /// この窓の ID。
    #[must_use]
    pub fn window(&self) -> Window {
        self.window.window()
    }

    /// いまの大きさ（左上は 0）。
    #[must_use]
    pub const fn bounds(&self) -> PixelRect {
        self.bounds
    }

    /// WM が大きさを変えた。
    pub(crate) fn resized(&mut self, width: u16, height: u16) {
        self.bounds = PixelRect::new(0, 0, width, height);
    }

    /// フレームを描き切ってから `PutImage` で転送する（ADR 0002）。
    ///
    /// # Errors
    ///
    /// X の要求か cairo が失敗した。
    pub fn present(&mut self, frame: &DocketFrame) -> Result<(), UiFailure> {
        let family = self.font_for(frame.language())?;
        let mut canvas = Canvas::render(
            frame,
            self.bounds,
            (self.display.visual().format(), family),
            &mut self.surfaces,
        )?;
        Transfer::new(
            (self.window.window(), self.gc.gcontext()),
            canvas.width(),
            self.display.visual().depth(),
        )
        .present(self.display.connection(), &mut canvas)
    }

    /// 言語の字を描ける家族。同じ言語なら前回のまま。
    fn font_for(&mut self, language: Language) -> Result<&'static str, UiFailure> {
        if let Some((known, family)) = self.font
            && known == language
        {
            return Ok(family);
        }
        let family = font_family::choose(language)?;
        self.font = Some((language, family));
        Ok(family)
    }

    /// 題と閉じる要求を WM へ伝える。
    fn declare_hints(&self) -> Result<(), UiFailure> {
        let connection = self.display.connection();
        let atoms = self.display.atoms();
        let window = self.window.window();
        connection.change_property32(
            PropMode::REPLACE,
            window,
            atoms.wm_protocols,
            AtomEnum::ATOM,
            &[atoms.wm_delete_window],
        )?;
        connection.change_property8(
            PropMode::REPLACE,
            window,
            atoms.net_wm_name,
            atoms.utf8_string,
            TITLE.as_bytes(),
        )?;
        connection.change_property8(
            PropMode::REPLACE,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            TITLE.as_bytes(),
        )?;
        connection.change_property32(
            PropMode::REPLACE,
            window,
            atoms.xdnd_aware,
            AtomEnum::ATOM,
            &[DropTarget::version()],
        )?;
        Ok(())
    }

    /// XDND の `ClientMessage`。扱ったら真。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn xdnd_message(&mut self, message: &ClientMessageEvent) -> Result<bool, UiFailure> {
        let window = self.window.window();
        self.drops.client_message(self.display, window, message)
    }

    /// クリップボードの変換を求める（Ctrl+V）。`time` は鍵のイベントの時刻。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn request_paste(&mut self, time: u32) -> Result<(), UiFailure> {
        let window = self.window.window();
        self.clipboard.request(self.display, window, time)
    }

    /// クリップボードの変換の答え。文字列が取れれば返す。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn clipboard_selection(
        &mut self,
        event: &SelectionNotifyEvent,
    ) -> Result<Option<String>, UiFailure> {
        let window = self.window.window();
        self.clipboard.selection_notify(self.display, window, event)
    }

    /// 落とす中身の変換の答え。落とされたファイルがあれば位置（窓の座標）とパスの列。
    ///
    /// # Errors
    ///
    /// X の要求が失敗した。
    pub fn xdnd_selection(
        &mut self,
        event: &SelectionNotifyEvent,
    ) -> Result<Option<(PixelPoint, Vec<String>)>, UiFailure> {
        let window = self.window.window();
        self.drops.selection_notify(self.display, window, event)
    }
}
