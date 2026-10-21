//! 添付の写しを文として読んだ結果（FR-022）。文字コードは store が決める（UTF-8・UTF-16 の BOM・`Shift_JIS`）。

use alloc::string::String;

/// 読んだ文と、先頭だけで切ったか。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentText {
    text: String,
    truncated: bool,
}

impl AttachmentText {
    /// 値から。
    #[must_use]
    pub const fn new(text: String, truncated: bool) -> Self {
        Self { text, truncated }
    }

    /// 文。
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// 先頭だけか（続きがある）。
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        self.truncated
    }

    /// 文を取り出す。
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }
}
