//! 添付の段に 1 フレーム渡すもの（FR-021 / FR-022）。段は業務の判断を持たないので、課題・印・中身は画面が渡す（ARC-011）。

use nene_docket_core::item::Item;
use nene_docket_core::language::Language;
use nene_docket_core::pixel_rect::PixelRect;

use crate::attachment_view::AttachmentView;

/// 段の矩形・言語・選んだ課題・写しが合わない連番・選んだ添付の中身。
#[derive(Debug, Clone, Copy)]
pub struct AttachmentsContext<'a> {
    area: PixelRect,
    language: Language,
    item: Option<&'a Item>,
    altered: &'a [u32],
    view: Option<&'a AttachmentView>,
}

impl<'a> AttachmentsContext<'a> {
    /// 矩形と言語だけ。
    #[must_use]
    pub const fn new(area: PixelRect, language: Language) -> Self {
        Self {
            area,
            language,
            item: None,
            altered: &[],
            view: None,
        }
    }

    /// 選んだ課題。
    #[must_use]
    pub const fn with_item(mut self, item: Option<&'a Item>) -> Self {
        self.item = item;
        self
    }

    /// 写しが合わない連番。
    #[must_use]
    pub const fn with_altered(mut self, altered: &'a [u32]) -> Self {
        self.altered = altered;
        self
    }

    /// 選んだ添付の中身。
    #[must_use]
    pub const fn with_view(mut self, view: Option<&'a AttachmentView>) -> Self {
        self.view = view;
        self
    }

    /// 段の矩形。
    #[must_use]
    pub const fn area(&self) -> PixelRect {
        self.area
    }

    /// 言語。
    #[must_use]
    pub const fn language(&self) -> Language {
        self.language
    }

    /// 選んだ課題。
    #[must_use]
    pub const fn item(&self) -> Option<&'a Item> {
        self.item
    }

    /// 写しが合わない連番。
    #[must_use]
    pub const fn altered(&self) -> &'a [u32] {
        self.altered
    }

    /// 選んだ添付の中身。
    #[must_use]
    pub const fn view(&self) -> Option<&'a AttachmentView> {
        self.view
    }
}
