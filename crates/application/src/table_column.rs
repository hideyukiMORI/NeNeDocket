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
}
