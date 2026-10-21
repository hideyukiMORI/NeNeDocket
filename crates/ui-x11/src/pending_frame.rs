//! まだ画面に渡していない 1 フレーム分の入力。ポインタ・鍵・読み直しの合図・落とされたファイル。

use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pointer_input::PointerInput;

/// `FrameInput` の、窓の大きさを除いた部分。窓の大きさは渡す瞬間の値を使う。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingFrame {
    pointer: PointerInput,
    keys: Vec<KeyIntent>,
    refresh: bool,
    drop: Option<(PixelPoint, Vec<String>)>,
}

impl PendingFrame {
    /// 3 つから。
    pub(crate) const fn new(pointer: PointerInput, keys: Vec<KeyIntent>, refresh: bool) -> Self {
        Self {
            pointer,
            keys,
            refresh,
            drop: None,
        }
    }

    /// 落とされたファイルを添える。
    pub(crate) fn with_drop(self, drop: Option<(PixelPoint, Vec<String>)>) -> Self {
        Self { drop, ..self }
    }

    /// ポインタだけ。鍵は無く、読み直さない。
    pub(crate) const fn of_pointer(pointer: PointerInput) -> Self {
        Self::new(pointer, Vec::new(), false)
    }

    /// ポインタ。
    pub(crate) const fn pointer(&self) -> PointerInput {
        self.pointer
    }

    /// 鍵。
    pub(crate) fn keys(&self) -> &[KeyIntent] {
        &self.keys
    }

    /// 読み直すか。
    pub(crate) const fn refresh(&self) -> bool {
        self.refresh
    }

    /// 落とされたファイル。
    pub(crate) const fn drop(&self) -> Option<&(PixelPoint, Vec<String>)> {
        self.drop.as_ref()
    }
}
