//! `NeNe Docket` の振る舞いの調整。画面の状態の所有者・表示フレーム（ARC-003 / ARC-004）。
//!
//! 🔴 `#![no_std]` である。X11・Win32・永続化・ファイル・時刻を知らない。プラットフォームはポートから入る。
//! 画面は即時モード（ADR 0006）: UI が毎フレーム [`frame_input::FrameInput`] を渡し、
//! [`docket_screen::DocketScreen::step`] が意図を状態に反映して [`docket_frame::DocketFrame`] を返す。

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod account;
pub mod account_file;
pub mod clock_port;
pub mod docket_frame;
pub mod docket_screen;
pub mod environment;
pub mod frame_input;
pub mod journal_failure;
pub mod journal_port;
pub mod ledger;
pub mod notice;
pub mod ports;
pub mod undo_stack;
