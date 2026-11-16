//! 曜日（期限の欄に添える・Issue #133）。

/// 月曜から。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    /// 月曜。
    Monday,
    /// 火曜。
    Tuesday,
    /// 水曜。
    Wednesday,
    /// 木曜。
    Thursday,
    /// 金曜。
    Friday,
    /// 土曜。
    Saturday,
    /// 日曜。
    Sunday,
}

impl Weekday {
    /// 月曜からの順。
    pub const ALL: [Self; 7] = [
        Self::Monday,
        Self::Tuesday,
        Self::Wednesday,
        Self::Thursday,
        Self::Friday,
        Self::Saturday,
        Self::Sunday,
    ];
}
