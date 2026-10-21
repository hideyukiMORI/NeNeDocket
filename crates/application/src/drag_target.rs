//! ヘッダの境か、表と下の段の境をドラッグ中に、どちらを動かしているか（FR-012）。閉じた集合（RS-002）。

use crate::table_column::TableColumn;

/// 動かしている境。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragTarget {
    /// 列の幅（その列）。
    Column(TableColumn),
    /// 表の下の段の高さ。
    Panes,
}
