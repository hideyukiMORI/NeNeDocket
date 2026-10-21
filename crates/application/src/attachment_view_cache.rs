//! 添付の段に出す中身の覚え（FR-022）。鍵（台帳の世代・課題・連番）が同じ間はファイルを読み直さない。

use nene_docket_core::item_id::ItemId;

use crate::attachment_view::AttachmentView;

/// 鍵と、読んだ中身（読めなかったなら `None`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentViewCache {
    key: Option<(u64, ItemId, u32)>,
    view: Option<AttachmentView>,
}

impl AttachmentViewCache {
    /// 何も覚えていない。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            key: None,
            view: None,
        }
    }

    /// この鍵の中身を覚えているか（読めなかったことも覚える）。
    #[must_use]
    pub fn holds(&self, key: (u64, ItemId, u32)) -> bool {
        self.key == Some(key)
    }

    /// 覚える。
    pub fn remember(&mut self, key: (u64, ItemId, u32), view: Option<AttachmentView>) {
        self.key = Some(key);
        self.view = view;
    }

    /// 忘れる（選んでいない）。
    pub fn clear(&mut self) {
        self.key = None;
        self.view = None;
    }

    /// 覚えている中身。
    #[must_use]
    pub const fn view(&self) -> Option<&AttachmentView> {
        self.view.as_ref()
    }
}
