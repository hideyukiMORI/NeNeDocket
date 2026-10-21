//! 表の列の順（FR-012）。6 列の並び替え。設定の版 4 に `order = number,title,state,assignee,due,labels` で残る。

use alloc::string::String;
use alloc::vec::Vec;

use crate::table_column::TableColumn;

/// 左からの並び。6 列すべてを 1 度ずつ含む。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnOrder {
    columns: [TableColumn; 6],
}

impl ColumnOrder {
    /// 既定の順（番号・題名・状態・担当・期限・ラベル）。
    #[must_use]
    pub const fn initial() -> Self {
        Self {
            columns: TableColumn::ALL,
        }
    }

    /// 左からの列。
    #[must_use]
    pub const fn columns(self) -> [TableColumn; 6] {
        self.columns
    }

    /// `column` を左から `index` 番目に動かした順（末尾を超えれば末尾）。
    #[must_use]
    pub fn moved_to(self, column: TableColumn, index: usize) -> Self {
        let mut rest: Vec<TableColumn> = self
            .columns
            .into_iter()
            .filter(|known| *known != column)
            .collect();
        rest.insert(index.min(rest.len()), column);
        let columns: [TableColumn; 6] = rest.try_into().unwrap_or(TableColumn::ALL);
        Self { columns }
    }

    /// `number,title,…` から。6 列すべてが 1 度ずつ無ければ `None`。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let parsed = text
            .split(',')
            .map(|part| TableColumn::from_keyword(part.trim()))
            .collect::<Option<Vec<TableColumn>>>()?;
        let columns: [TableColumn; 6] = parsed.try_into().ok()?;
        let complete = TableColumn::ALL
            .iter()
            .all(|column| columns.contains(column));
        complete.then_some(Self { columns })
    }

    /// `parse` の逆。
    #[must_use]
    pub fn text(self) -> String {
        self.columns
            .iter()
            .map(|column| column.keyword())
            .collect::<Vec<&str>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_round_trips_and_moving_keeps_every_column_once() {
        let order = ColumnOrder::initial();
        assert_eq!(order.text(), "number,title,state,assignee,due,labels");
        assert_eq!(ColumnOrder::parse(&order.text()), Some(order));
        let moved = order.moved_to(TableColumn::State, 0);
        assert_eq!(moved.text(), "state,number,title,assignee,due,labels");
        let to_end = moved.moved_to(TableColumn::Number, 99);
        assert_eq!(to_end.text(), "state,title,assignee,due,labels,number");
        assert_eq!(to_end.moved_to(TableColumn::Number, 5), to_end);
    }

    #[test]
    fn a_missing_a_duplicate_or_an_unknown_column_is_refused() {
        for text in [
            "number,title,state,assignee,due",
            "number,number,state,assignee,due,labels",
            "number,title,state,assignee,due,rank",
            "",
        ] {
            assert_eq!(ColumnOrder::parse(text), None, "{text}");
        }
    }
}
