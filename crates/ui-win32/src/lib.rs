//! `NeNe Docket` の画面（Windows）。Win32 の通常の窓に Direct2D / DirectWrite で core の描画命令を描く（ADR 0002 / 0009）。
//!
//! application が作った `DocketFrame` を描き、OS のメッセージを `FrameInput` にして渡すだけ（ARC-011・ADR 0006）。
//! `unsafe` を書ける 2 クレートの 1 つ。Win32 を呼ぶ関数ごとに `#[expect(unsafe_code)]` と SAFETY の注釈を置く。
//! Win32 に触れるモジュールは Windows でだけ組み立て、純粋なもの（鍵の読み替え・ポインタの追跡）は Linux でもテストする。

pub mod char_assembler;
pub mod composition;
#[cfg(windows)]
pub mod docket_window;
#[cfg(windows)]
mod ime;
pub mod key_input;
#[cfg(windows)]
pub mod pen;
pub mod pointer_state;
#[cfg(windows)]
mod text_line;
#[cfg(windows)]
pub mod win32_failure;
#[cfg(windows)]
mod windows_app;
