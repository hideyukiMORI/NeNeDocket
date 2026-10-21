//! 表の列。閉じた集合（RS-002）。幅を持つのは題名以外（題名は残りを埋める）。

/// 表の列（左から）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableColumn {
    /// 番号。
    Number,
    /// 題名（残りを埋める）。
    Title,
    /// 状態。
    State,
    /// 担当。
    Assignee,
    /// 期限。
    Due,
    /// ラベル。
    Labels,
}

impl TableColumn {
    /// 左からの順。
    pub const ALL: [Self; 6] = [
        Self::Number,
        Self::Title,
        Self::State,
        Self::Assignee,
        Self::Due,
        Self::Labels,
    ];

    /// 幅を設定で変えられるか（題名は残りを埋めるので変えられない）。
    #[must_use]
    pub const fn is_resizable(self) -> bool {
        !matches!(self, Self::Title)
    }

    /// 設定に書く語。
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Number => "number",
            Self::Title => "title",
            Self::State => "state",
            Self::Assignee => "assignee",
            Self::Due => "due",
            Self::Labels => "labels",
        }
    }

    /// 設定の語から。知らない語は `None`。
    #[must_use]
    pub fn from_keyword(text: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|column| column.keyword() == text)
    }
}
