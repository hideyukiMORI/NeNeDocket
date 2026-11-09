//! 表の列の順と、出している列（FR-012）。設定の版 7 に `order = number,title,state,assignee,due,labels,opened,closed`
//! （出している列だけを左から）で残る。出していない列は `TableColumn::ALL` の順で後ろに控える（ADR 0023）。
//! 版 4〜6 の `order`（6 列ちょうど）も読め、移すときに起票日と完了日を右に足す（[`ColumnOrder::with_new_columns`]）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::table_column::TableColumn;

/// 全部の列の数。
const COUNT: usize = TableColumn::ALL.len();
/// 既定で出す列の数（番号・題名・状態・担当・期限・ラベル・起票日・完了日）。
const INITIAL_SHOWN: u8 = 8;

/// 全部の列の並び（前の `shown` 本が出している列・左から）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnOrder {
    columns: [TableColumn; COUNT],
    shown: u8,
}

impl ColumnOrder {
    /// 既定（番号・題名・状態・担当・期限・ラベル・起票日・完了日。起票者と更新日は控える）。
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            columns: TableColumn::ALL,
            shown: INITIAL_SHOWN,
        }
    }

    /// 出している列（左から）。
    #[must_use]
    pub fn columns(self) -> Vec<TableColumn> {
        self.columns
            .into_iter()
            .take(usize::from(self.shown))
            .collect()
    }

    /// `column` を出している列の左から `index` 番目に動かした順（末尾を超えれば末尾）。出していない列なら変わらない。
    #[must_use]
    pub fn moved_to(self, column: TableColumn, index: usize) -> Self {
        let mut shown = self.columns();
        if !shown.contains(&column) {
            return self;
        }
        shown.retain(|known| *known != column);
        shown.insert(index.min(shown.len()), column);
        Self::from_shown(&shown).unwrap_or(self)
    }

    /// 版 6 までの順から移すとき、出していなければ起票日と完了日を右に足す（ADR 0023）。
    #[must_use]
    pub fn with_new_columns(self) -> Self {
        let mut shown = self.columns();
        for column in [TableColumn::Opened, TableColumn::Closed] {
            if !shown.contains(&column) {
                shown.push(column);
            }
        }
        Self::from_shown(&shown).unwrap_or(self)
    }

    /// `number,title,…` から（出している列を左から）。題名が無い・重なり・知らない語なら `None`。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let parsed = text
            .split(',')
            .map(|part| TableColumn::from_keyword(part.trim()))
            .collect::<Option<Vec<TableColumn>>>()?;
        Self::from_shown(&parsed)
    }

    /// `parse` の逆（出している列だけ）。
    #[must_use]
    pub fn text(self) -> String {
        self.columns()
            .iter()
            .map(|column| column.keyword())
            .collect::<Vec<&str>>()
            .join(",")
    }

    /// 出す列（左から）から。残りは `TableColumn::ALL` の順で後ろに控える。題名が無い・重なりなら `None`。
    fn from_shown(shown: &[TableColumn]) -> Option<Self> {
        let distinct = shown
            .iter()
            .enumerate()
            .all(|(index, column)| !shown.iter().skip(index + 1).any(|other| other == column));
        if !distinct || !shown.contains(&TableColumn::Title) {
            return None;
        }
        let hidden = TableColumn::ALL
            .into_iter()
            .filter(|column| !shown.contains(column));
        let all: Vec<TableColumn> = shown.iter().copied().chain(hidden).collect();
        Some(Self {
            columns: all.try_into().ok()?,
            shown: u8::try_from(shown.len()).ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_round_trips_and_moving_keeps_every_column_once() {
        let order = ColumnOrder::initial();
        assert_eq!(
            order.text(),
            "number,title,state,assignee,due,labels,opened,closed"
        );
        assert_eq!(ColumnOrder::parse(&order.text()), Some(order));
        let moved = order.moved_to(TableColumn::State, 0);
        assert_eq!(
            moved.text(),
            "state,number,title,assignee,due,labels,opened,closed"
        );
        let to_end = moved.moved_to(TableColumn::Number, 99);
        assert_eq!(
            to_end.text(),
            "state,title,assignee,due,labels,opened,closed,number"
        );
        assert_eq!(to_end.moved_to(TableColumn::Number, 7), to_end);
        // 出していない列は動かせない。
        assert_eq!(order.moved_to(TableColumn::Opener, 0), order);
    }

    #[test]
    fn a_subset_with_the_title_is_shown_and_the_rest_wait_behind() {
        let order = ColumnOrder::parse("title,updated,opener").unwrap();
        assert_eq!(
            order.columns(),
            [
                TableColumn::Title,
                TableColumn::Updated,
                TableColumn::Opener
            ]
        );
        assert_eq!(order.text(), "title,updated,opener");
    }

    #[test]
    fn an_order_from_version_six_gains_the_opened_and_closed_columns() {
        let legacy = ColumnOrder::parse("state,number,title,assignee,due,labels").unwrap();
        assert_eq!(
            legacy.with_new_columns().text(),
            "state,number,title,assignee,due,labels,opened,closed"
        );
        let initial = ColumnOrder::initial();
        assert_eq!(initial.with_new_columns(), initial);
    }

    #[test]
    fn a_missing_title_a_duplicate_or_an_unknown_column_is_refused() {
        for text in [
            "number,state,assignee,due",
            "number,title,title",
            "number,title,state,assignee,due,rank",
            "",
        ] {
            assert_eq!(ColumnOrder::parse(text), None, "{text}");
        }
    }
}
