//! `NeNe Docket` の画面（Linux）。x11rb で通常の窓を持ち、core の描画命令を cairo の `ImageSurface` に描いて `PutImage` で転送する。
//!
//! application が作った `DocketFrame` を描き、OS のイベントを `FrameInput` にして渡すだけ（ARC-011・ADR 0006）。
//! 可変性の隔離区画（ARC-005）。業務状態は複製しない。
//! 時刻・環境・ファイルには到達しない（ARC-007 / CNF-012）。

pub mod atoms;
pub mod canvas;
pub mod display;
pub mod docket_window;
pub mod event_loop;
mod flow;
pub mod font_family;
mod gathering;
pub mod key_input;
pub mod key_map;
pub mod palette;
mod pointer_tracker;
mod transfer;
pub mod ui_failure;
pub mod window_visual;
