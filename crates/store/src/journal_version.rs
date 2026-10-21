//! ジャーナルの版。版 1 は検査和だけ、版 2 は署名も付く（ADR 0004 の 3・ADR 0010）。

/// 行の先頭の印とファイル名の `.vN.log` に出る。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum JournalVersion {
    /// `v1`。誰でも編集できる台帳。
    One,
    /// `v2`。権限を有効にした台帳。各行に Ed25519 の署名。
    Two,
}

impl JournalVersion {
    /// 行の先頭の印。
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::One => "v1",
            Self::Two => "v2",
        }
    }

    /// ファイル名の末尾。
    #[must_use]
    pub const fn file_suffix(self) -> &'static str {
        match self {
            Self::One => ".v1.log",
            Self::Two => ".v2.log",
        }
    }

    /// 印から。知らない印は `None`。
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        [Self::One, Self::Two]
            .into_iter()
            .find(|version| version.tag() == tag)
    }

    /// ファイル名から（アカウントの 16 進と版）。
    #[must_use]
    pub fn split_file_name(file_name: &str) -> Option<(&str, Self)> {
        [Self::One, Self::Two].into_iter().find_map(|version| {
            file_name
                .strip_suffix(version.file_suffix())
                .map(|stem| (stem, version))
        })
    }

    /// 行に署名が要るか。
    #[must_use]
    pub const fn is_signed(self) -> bool {
        matches!(self, Self::Two)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_and_suffixes_round_trip() {
        assert_eq!(JournalVersion::from_tag("v1"), Some(JournalVersion::One));
        assert_eq!(JournalVersion::from_tag("v2"), Some(JournalVersion::Two));
        assert_eq!(JournalVersion::from_tag("v3"), None);
        assert_eq!(
            JournalVersion::split_file_name("ab.v2.log"),
            Some(("ab", JournalVersion::Two))
        );
        assert_eq!(
            JournalVersion::split_file_name("ab.v1.log"),
            Some(("ab", JournalVersion::One))
        );
        assert_eq!(JournalVersion::split_file_name("ab.txt"), None);
        assert!(JournalVersion::Two.is_signed());
        assert!(!JournalVersion::One.is_signed());
    }
}
