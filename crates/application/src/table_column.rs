//! 表の列。閉じた集合（RS-002）。幅を持つのは題名以外（題名は残りを埋める）。自動の項目 4 つは Issue #117・ADR 0023。

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
    /// 起票日（自動の項目・Issue #117）。
    Opened,
    /// 起票者（作ったアカウント）。
    Opener,
    /// 完了日。
    Closed,
    /// 更新日。
    Updated,
}

impl TableColumn {
    /// 全部の列（既定の左からの順）。
    pub const ALL: [Self; 10] = [
        Self::Number,
        Self::Title,
        Self::State,
        Self::Assignee,
        Self::Due,
        Self::Labels,
        Self::Opened,
        Self::Closed,
        Self::Opener,
        Self::Updated,
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
            Self::Opened => "opened",
            Self::Opener => "opener",
            Self::Closed => "closed",
            Self::Updated => "updated",
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
