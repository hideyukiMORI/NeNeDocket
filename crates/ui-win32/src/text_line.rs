//! 1 行の字の配置と寸法（DirectWrite）。`pen` が測って描く。

use windows::Win32::Graphics::DirectWrite::IDWriteTextLayout;

/// 配置と、その幅・高さ・ベースラインまでの高さ（物理画素）。
pub(crate) struct TextLine {
    layout: IDWriteTextLayout,
    width: f32,
    height: f32,
    ascent: f32,
}

impl TextLine {
    /// 配置と寸法から。
    pub(crate) const fn new(
        layout: IDWriteTextLayout,
        (width, height, ascent): (f32, f32, f32),
    ) -> Self {
        Self {
            layout,
            width,
            height,
            ascent,
        }
    }

    /// 配置。
    pub(crate) const fn layout(&self) -> &IDWriteTextLayout {
        &self.layout
    }

    /// 幅（末尾の空白を含む）。
    pub(crate) const fn width(&self) -> f32 {
        self.width
    }

    /// 行の高さ。
    pub(crate) const fn height(&self) -> f32 {
        self.height
    }

    /// 上端からベースラインまで。
    pub(crate) const fn ascent(&self) -> f32 {
        self.ascent
    }
}
