//! 添付の種類（FR-022）。名前の拡張子から決める。見せ方（文・表・OS のアプリ）を選ぶ鍵。閉じた集合（RS-002）。

/// 拡張子と種類の表。無い拡張子は「それ以外」。
const KINDS: [(&str, AttachmentKind); 22] = [
    ("txt", AttachmentKind::Text),
    ("md", AttachmentKind::Text),
    ("log", AttachmentKind::Text),
    ("json", AttachmentKind::Text),
    ("toml", AttachmentKind::Text),
    ("yaml", AttachmentKind::Text),
    ("yml", AttachmentKind::Text),
    ("xml", AttachmentKind::Text),
    ("ini", AttachmentKind::Text),
    ("cfg", AttachmentKind::Text),
    ("rs", AttachmentKind::Text),
    ("py", AttachmentKind::Text),
    ("sh", AttachmentKind::Text),
    ("html", AttachmentKind::Text),
    ("htm", AttachmentKind::Text),
    ("svg", AttachmentKind::Text),
    ("csv", AttachmentKind::Csv),
    ("tsv", AttachmentKind::Tsv),
    ("png", AttachmentKind::Image),
    ("jpg", AttachmentKind::Image),
    ("jpeg", AttachmentKind::Image),
    ("", AttachmentKind::Other),
];

/// 文（`text_area` の読み取り専用）・表（CSV / TSV）・画像（PNG / JPEG・Issue #64）・それ以外（OS の既定のアプリ）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttachmentKind {
    /// 文として出す。
    Text,
    /// `,` 区切りの表として出す。
    Csv,
    /// `\t` 区切りの表として出す。
    Tsv,
    /// 画像。
    Image,
    /// OS の既定のアプリで開く。
    Other,
}

impl AttachmentKind {
    /// 名前から。拡張子の大文字小文字は問わない。
    #[must_use]
    pub fn of(name: &str) -> Self {
        let extension = name
            .rsplit_once('.')
            .map_or(alloc::string::String::new(), |(_, extension)| {
                extension.to_ascii_lowercase()
            });
        KINDS
            .iter()
            .find(|(known, _)| *known == extension.as_str())
            .map_or(Self::Other, |(_, kind)| *kind)
    }

    /// 文として読む種類か（文と表）。
    #[must_use]
    pub const fn is_textual(self) -> bool {
        matches!(self, Self::Text | Self::Csv | Self::Tsv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_extension_decides_regardless_of_case() {
        assert_eq!(AttachmentKind::of("note.txt"), AttachmentKind::Text);
        assert_eq!(AttachmentKind::of("README.MD"), AttachmentKind::Text);
        assert_eq!(AttachmentKind::of("sales.CSV"), AttachmentKind::Csv);
        assert_eq!(AttachmentKind::of("a.tsv"), AttachmentKind::Tsv);
        assert_eq!(AttachmentKind::of("photo.JPEG"), AttachmentKind::Image);
    }

    #[test]
    fn unknown_or_missing_extensions_are_other_and_only_text_and_tables_are_textual() {
        assert_eq!(AttachmentKind::of("book.xlsx"), AttachmentKind::Other);
        assert_eq!(AttachmentKind::of("noext"), AttachmentKind::Other);
        assert_eq!(AttachmentKind::of("trailing."), AttachmentKind::Other);
        assert!(AttachmentKind::Csv.is_textual());
        assert!(AttachmentKind::Text.is_textual());
        assert!(!AttachmentKind::Image.is_textual());
    }
}
