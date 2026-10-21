//! 1 フレーム分の入力。UI が OS のイベントから作る。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::pointer_input::PointerInput;
use nene_docket_core::preedit::Preedit;

/// 窓の大きさ・ポインタ・このフレームまでに押された鍵・IME の確定前の文字列・他人のジャーナルを読み直す合図・落とされたファイル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameInput {
    bounds: PixelRect,
    pointer: PointerInput,
    keys: Vec<KeyIntent>,
    preedit: Option<Preedit>,
    refresh: bool,
    drop: Option<(PixelPoint, Vec<String>)>,
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
            refresh: false,
            drop: None,
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

    /// 他人のジャーナルを読み直す合図を添える。UI が数秒ごとに立てる（ADR 0004 の 8）。
    #[must_use]
    pub fn with_refresh(self) -> Self {
        Self {
            refresh: true,
            ..self
        }
    }

    /// 落とされたファイルのパスの列と、落とした位置（窓の座標）を添える（FR-021・ドラッグ＆ドロップ）。
    #[must_use]
    pub fn with_drop(self, point: PixelPoint, paths: Vec<String>) -> Self {
        Self {
            drop: Some((point, paths)),
            ..self
        }
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

    /// このフレームで他人のジャーナルを読み直すか。
    #[must_use]
    pub const fn refresh(&self) -> bool {
        self.refresh
    }

    /// 落とされたファイル（位置と、パスの列）。
    #[must_use]
    pub fn drop(&self) -> Option<(PixelPoint, &[String])> {
        self.drop
            .as_ref()
            .map(|(point, paths)| (*point, paths.as_slice()))
    }
}
