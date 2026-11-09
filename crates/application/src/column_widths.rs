//! 表の列の幅（FR-012）。論理画素。題名は残りを埋めるので持たない。設定の版 7 に
//! `columns = 72,110,90,90,120,96,96,80,96`（番号・状態・担当・期限・ラベル・起票日・完了日・起票者・更新日）で残る。
//! 版 3〜6 の 5 つの幅も読み、足した 4 列は既定で補う（ADR 0023）。

use alloc::string::{String, ToString as _};
use alloc::vec::Vec;

use crate::table_column::TableColumn;

/// 列の幅の下限（論理画素）。
const MIN_PX: u16 = 40;
/// 幅を持つ列の数。
const COUNT: usize = 9;
/// 版 6 までの幅の数（番号・状態・担当・期限・ラベル）。
const LEGACY_COUNT: usize = 5;
/// 既定（1x）。番号・状態・担当・期限・ラベル・起票日・完了日・起票者・更新日。
const DEFAULT: [u16; COUNT] = [72, 110, 90, 90, 120, 96, 96, 80, 96];

/// 題名以外の列の幅。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnWidths {
    widths: [u16; COUNT],
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

    /// `72,110,90,90,120,96,96,80,96` から。版 6 までの 5 つなら足した列を既定で補う。数が違う・数でない・40 未満なら `None`。
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
        let mut widths = DEFAULT;
        if parsed.len() != COUNT && parsed.len() != LEGACY_COUNT {
            return None;
        }
        for (slot, width) in widths.iter_mut().zip(parsed) {
            *slot = width;
        }
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
            TableColumn::Opened => Some(5),
            TableColumn::Closed => Some(6),
            TableColumn::Opener => Some(7),
            TableColumn::Updated => Some(8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_and_changes_are_floored() {
        let widths = ColumnWidths::initial();
        assert_eq!(widths.text(), "72,110,90,90,120,96,96,80,96");
        // 版 6 までの 5 つは、足した 4 列を既定で補う。
        assert_eq!(
            ColumnWidths::parse("80,110,90,90,120"),
            Some(widths.with(TableColumn::Number, 80))
        );
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
        for text in [
            "72,110,90,90",
            "72,110,90,90,x",
            "72,110,90,90,10",
            "72,110,90,90,120,96",
        ] {
            assert_eq!(ColumnWidths::parse(text), None, "{text}");
        }
        assert!(TableColumn::Number.is_resizable());
        assert!(!TableColumn::Title.is_resizable());
    }
}
