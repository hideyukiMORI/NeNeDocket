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
pub mod account_reading;
pub mod admin_command;
pub mod admission_report;
pub mod body_pane;
pub mod clock_port;
pub mod column_order;
pub mod column_widths;
pub mod docket_frame;
pub mod docket_screen;
pub mod drag_target;
pub mod edit_slot;
pub mod environment;
pub mod frame_input;
pub mod history_change;
pub mod history_line;
pub mod history_pane;
pub mod journal_failure;
pub mod journal_port;
pub mod key_record;
pub mod ledger;
pub mod member;
pub mod member_role;
pub mod members_action;
pub mod members_pane;
pub mod notice;
pub mod pane_height;
pub mod permissions_file;
pub mod permissions_reading;
pub mod ports;
pub mod settings;
pub mod settings_failure;
pub mod settings_file;
pub mod settings_port;
pub mod table_column;
pub mod undo_stack;
