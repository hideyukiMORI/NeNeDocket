//! 1 フレーム分の入力。UI が OS のイベントから作る。

use alloc::vec::Vec;

use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::pointer_input::PointerInput;
use nene_docket_core::preedit::Preedit;

/// 窓の大きさ・ポインタ・このフレームまでに押された鍵・IME の確定前の文字列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameInput {
    bounds: PixelRect,
    pointer: PointerInput,
    keys: Vec<KeyIntent>,
    preedit: Option<Preedit>,
}

impl FrameInput {
    /// 窓の大きさとポインタから。鍵と確定前の文字列は無し。
    #[must_use]
    pub const fn new(bounds: PixelRect, pointer: PointerInput) -> Self {
        Self {
            bounds,
            pointer,
            keys: Vec::new(),
            preedit: None,
        }
    }

    /// 鍵を添える。
    #[must_use]
    pub fn with_keys(self, keys: Vec<KeyIntent>) -> Self {
        Self { keys, ..self }
    }

    /// 確定前の文字列を添える。
    #[must_use]
    pub fn with_preedit(self, preedit: Option<Preedit>) -> Self {
        Self { preedit, ..self }
    }

    /// 窓の大きさ。
    #[must_use]
    pub const fn bounds(&self) -> PixelRect {
        self.bounds
    }

    /// ポインタ。
    #[must_use]
    pub const fn pointer(&self) -> PointerInput {
        self.pointer
    }

    /// 鍵。押された順。
    #[must_use]
    pub fn keys(&self) -> &[KeyIntent] {
        &self.keys
    }

    /// 確定前の文字列。
    #[must_use]
    pub const fn preedit(&self) -> Option<&Preedit> {
        self.preedit.as_ref()
    }
}
