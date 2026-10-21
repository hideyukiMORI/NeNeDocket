//! 表の列の幅（FR-012）。論理画素。題名は残りを埋めるので持たない。設定の版 3 に `columns = 72,110,90,90,120` で残る。

use alloc::string::{String, ToString as _};
use alloc::vec::Vec;

use crate::table_column::TableColumn;

/// 列の幅の下限（論理画素）。
const MIN_PX: u16 = 40;
/// 既定（1x）。番号・状態・担当・期限・ラベル。
const DEFAULT: [u16; 5] = [72, 110, 90, 90, 120];

/// 番号・状態・担当・期限・ラベルの幅。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnWidths {
    widths: [u16; 5],
}

impl ColumnWidths {
    /// 既定の幅。
    #[must_use]
    pub const fn initial() -> Self {
        Self { widths: DEFAULT }
    }

    /// 列の幅。題名は `None`。
    #[must_use]
    pub fn get(self, column: TableColumn) -> Option<u16> {
        Self::slot(column).and_then(|index| self.widths.get(index).copied())
    }

    /// 列の幅を変えた値（40 以上に丸める）。題名なら変わらない。
    #[must_use]
    pub fn with(mut self, column: TableColumn, width: u16) -> Self {
        if let Some(slot) = Self::slot(column).and_then(|index| self.widths.get_mut(index)) {
            *slot = width.max(MIN_PX);
        }
        self
    }

    /// `72,110,90,90,120` から。数が違う・数でない・40 未満なら `None`。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let parsed = text
            .split(',')
            .map(|part| {
                part.trim()
                    .parse::<u16>()
                    .ok()
                    .filter(|width| *width >= MIN_PX)
            })
            .collect::<Option<Vec<u16>>>()?;
        let widths: [u16; 5] = parsed.try_into().ok()?;
        Some(Self { widths })
    }

    /// `parse` の逆。
    #[must_use]
    pub fn text(self) -> String {
        self.widths
            .iter()
            .map(u16::to_string)
            .collect::<Vec<String>>()
            .join(",")
    }

    const fn slot(column: TableColumn) -> Option<usize> {
        match column {
            TableColumn::Number => Some(0),
            TableColumn::Title => None,
            TableColumn::State => Some(1),
            TableColumn::Assignee => Some(2),
            TableColumn::Due => Some(3),
            TableColumn::Labels => Some(4),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_and_changes_are_floored() {
        let widths = ColumnWidths::initial();
        assert_eq!(widths.text(), "72,110,90,90,120");
        assert_eq!(ColumnWidths::parse(&widths.text()), Some(widths));
        assert_eq!(widths.get(TableColumn::Title), None);
        assert_eq!(widths.get(TableColumn::State), Some(110));
        let changed = widths
            .with(TableColumn::State, 10)
            .with(TableColumn::Title, 300);
        assert_eq!(changed.get(TableColumn::State), Some(40));
        assert_eq!(changed.get(TableColumn::Title), None);
    }

    #[test]
    fn the_wrong_count_a_non_number_and_a_narrow_width_are_refused() {
        for text in ["72,110,90,90", "72,110,90,90,x", "72,110,90,90,10"] {
            assert_eq!(ColumnWidths::parse(text), None, "{text}");
        }
        assert!(TableColumn::Number.is_resizable());
        assert!(!TableColumn::Title.is_resizable());
    }
}
