//! 他人のジャーナルを読み直す刻み（ADR 0004 の 8）。
//!
//! 画面のイベントループは X のイベントを待つだけなので、時間は外から合図にして入れる。
//! 自分の X 接続を持つスレッドが、一定の間隔で窓へ `ClientMessage` を送る。ui-x11 はその型の atom を見て、
//! 次のフレームに読み直しを乗せる。窓が消えて送れなくなればスレッドは終わる。

use std::thread::{self, JoinHandle};
use std::time::Duration;

use x11rb::connection::Connection as _;
use x11rb::protocol::xproto::{Atom, ClientMessageEvent, ConnectionExt as _, EventMask, Window};
use x11rb::rust_connection::RustConnection;

use crate::adapter_failure::AdapterFailure;

/// 走っているスレッド。落とすと（`drop`）待たずに切り離す。
#[derive(Debug)]
pub struct RefreshTicker {
    handle: JoinHandle<()>,
}

impl RefreshTicker {
    /// `window` へ `atom_name` の型の `ClientMessage` を `interval_seconds` 秒ごとに送るスレッドを始める。接続は同期的に作るので、繋げなければここで失敗する。
    ///
    /// # Errors
    ///
    /// X サーバに繋げない・atom を intern できない。
    pub fn start(
        window: Window,
        atom_name: &[u8],
        interval_seconds: u64,
    ) -> Result<Self, AdapterFailure> {
        let interval = Duration::from_secs(interval_seconds);
        let (connection, _screen) = x11rb::connect(None)?;
        let atom = connection.intern_atom(false, atom_name)?.reply()?.atom;
        let handle = thread::spawn(move || tick(&connection, window, atom, interval));
        Ok(Self { handle })
    }

    /// スレッドがまだ走っているか。
    #[must_use]
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }
}

/// 送れなくなるまで刻む。
fn tick(connection: &RustConnection, window: Window, atom: Atom, interval: Duration) {
    loop {
        thread::sleep(interval);
        if send(connection, window, atom).is_err() {
            return;
        }
    }
}

/// 合図を 1 つ送り、窓がまだ在ることを確かめる。
fn send(connection: &RustConnection, window: Window, atom: Atom) -> Result<(), AdapterFailure> {
    let event = ClientMessageEvent::new(32, window, atom, [0_u32; 5]);
    connection
        .send_event(false, window, EventMask::NO_EVENT, event)?
        .check()?;
    connection.flush()?;
    Ok(())
}
