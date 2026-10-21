//! `NeNe Docket` の中核。値・不変条件・閉じた選択肢と、部品の核（ARC-003 / ADR 0006）。
//!
//! 🔴 `#![no_std]` である。X11・Win32・cairo・Direct2D・ファイル・時刻・環境変数には構文的に到達できない。
//! 外部クレートへの依存は 0（ARC-013 / CNF-013）。
//!
//! 部品の核は即時モードで書く。UI は毎フレーム、ポインタと鍵の入力を [`interaction::Interaction`] に渡し、
//! 部品ごとに矩形と ID を問い合わせて [`response::Response`] を受け取り、[`draw_command::DrawCommand`] の列を描く。
//! 台帳の中核（ADR 0004）もここにある: 課題・操作・論理時計と、操作の集合から台帳を作る純関数 [`merge::merge`]。
//! 幾何はすべて物理画素で、窓の左上を原点とする `u16` の座標で表す（FR-016）。
//! 文字の幅は core では測らない。カーソルの位置は文字の添字で渡し、UI がフォントで位置に直す（ADR 0006）。

#![no_std]

extern crate alloc;

#[cfg(test)]
extern crate std;

pub mod account_id;
pub mod button;
pub mod button_phase;
pub mod checkbox;
pub mod column_width;
pub mod cursor_move;
pub mod date;
pub mod docket;
pub mod draw_command;
pub mod field;
pub mod field_value;
pub mod interaction;
pub mod item;
pub mod item_id;
pub mod item_state;
pub mod key_intent;
pub mod layout;
pub mod logical_clock;
pub mod merge;
pub mod op;
pub mod op_id;
pub mod op_kind;
pub mod paint;
pub mod pixel_point;
pub mod pixel_rect;
pub mod pointer_input;
pub mod preedit;
pub mod response;
pub mod response_phase;
pub mod scroll_bar;
pub mod scroll_state;
pub mod select_box;
pub mod select_state;
pub mod table_layout;
pub mod text_align;
pub mod text_caret;
pub mod text_edit;
pub mod text_field;
pub mod ui_scale;
pub mod wall_clock;
pub mod widget_id;
