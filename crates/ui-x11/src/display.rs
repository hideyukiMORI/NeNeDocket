//! X への接続と、窓を作る前に決まること（画面・atom・visual・キーボード配列・`Xft.dpi`）。

use nene_docket_application::environment::Environment;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::ui_scale::UiScale;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt as _, KeyButMask, KeyPressEvent, Screen, Window,
};
use x11rb::rust_connection::RustConnection;

use crate::atoms::Atoms;
use crate::key_input::{self, Modifiers};
use crate::key_map::KeyMap;
use crate::ui_failure::UiFailure;
use crate::window_visual::WindowVisual;

/// 接続と、その画面で決まったもの。
#[derive(Debug)]
pub struct Display {
    connection: RustConnection,
    screen_index: usize,
    atoms: Atoms,
    visual: WindowVisual,
    key_map: KeyMap,
    xft_dpi: Option<u32>,
}

impl Display {
    /// `DISPLAY` に繋ぐ。
    ///
    /// # Errors
    ///
    /// 繋げない・画面が無い・根の窓が depth 24 の `TrueColor` でない・X の要求が失敗した。
    pub fn connect() -> Result<Self, UiFailure> {
        let (connection, screen_index) = x11rb::connect(None)?;
        let atoms = Atoms::intern(&connection)?;
        let screen = connection
            .setup()
            .roots
            .get(screen_index)
            .ok_or(UiFailure::NoScreen)?;
        let visual = WindowVisual::root(screen).ok_or(UiFailure::NoUsableVisual)?;
        let key_map = Self::key_map(&connection)?;
        let xft_dpi = Self::xft_dpi(&connection, screen.root)?;
        Ok(Self {
            connection,
            screen_index,
            atoms,
            visual,
            key_map,
            xft_dpi,
        })
    }

    /// ルート窓の `RESOURCE_MANAGER`（`xrdb` が置く文字列）の `Xft.dpi`。無ければ `None`（FR-016）。
    fn xft_dpi(connection: &RustConnection, root: Window) -> Result<Option<u32>, UiFailure> {
        let reply = connection
            .get_property(
                false,
                root,
                AtomEnum::RESOURCE_MANAGER,
                AtomEnum::STRING,
                0,
                u32::MAX / 4,
            )?
            .reply()?;
        Ok(UiScale::xft_dpi_in(&String::from_utf8_lossy(&reply.value)))
    }

    /// キーボード配列を読む。
    fn key_map(connection: &RustConnection) -> Result<KeyMap, UiFailure> {
        let setup = connection.setup();
        let first = setup.min_keycode;
        let count = setup.max_keycode.saturating_sub(first).saturating_add(1);
        let mapping = connection.get_keyboard_mapping(first, count)?.reply()?;
        Ok(KeyMap::from_mapping(first, &mapping))
    }

    /// 環境の事実。application が起動時に一度取り込む（ARC-004）。
    #[must_use]
    pub const fn environment(&self) -> Environment {
        Environment::new(self.xft_dpi)
    }

    /// 押された鍵を意図に読み替える。反応しない鍵は `None`。
    #[must_use]
    pub fn key_intent(&self, key: &KeyPressEvent) -> Option<KeyIntent> {
        let modifiers = Modifiers {
            shift: key.state.contains(KeyButMask::SHIFT),
            control: key.state.contains(KeyButMask::CONTROL),
        };
        let keysym = self.key_map.keysym(key.detail, modifiers.shift)?;
        key_input::intent_of(keysym, modifiers)
    }

    /// 接続。
    #[must_use]
    pub const fn connection(&self) -> &RustConnection {
        &self.connection
    }

    /// 既定の画面。
    ///
    /// # Errors
    ///
    /// 接続時に在った画面が消えることは無いが、型の上では `Option` なので失敗として返す。
    pub fn screen(&self) -> Result<&Screen, UiFailure> {
        self.connection
            .setup()
            .roots
            .get(self.screen_index)
            .ok_or(UiFailure::NoScreen)
    }

    /// intern 済みの atom。
    #[must_use]
    pub const fn atoms(&self) -> Atoms {
        self.atoms
    }

    /// 窓に使う visual。
    #[must_use]
    pub const fn visual(&self) -> WindowVisual {
        self.visual
    }
}
