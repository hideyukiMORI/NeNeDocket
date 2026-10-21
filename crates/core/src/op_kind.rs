//! 操作の種類。

use crate::field_value::FieldValue;
use crate::item_id::ItemId;

/// 台帳を変える最小の単位の中身。閉じた集合（RS-002）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpKind {
    /// 課題を作る。項目は空で始まり、続く `Set` が埋める。
    Create(ItemId),
    /// 項目を置く。同じ項目への操作は全順序で後が勝つ。
    Set {
        /// 課題。
        item: ItemId,
        /// 項目と新しい値。
        value: FieldValue,
    },
    /// 課題を消す（表から外す。ジャーナルには残る）。
    Delete(ItemId),
    /// 消した課題を戻す。`Delete` の逆。
    Restore(ItemId),
}

impl OpKind {
    /// 触れる課題。
    #[must_use]
    pub const fn item(&self) -> ItemId {
        match self {
            Self::Create(item)
            | Self::Delete(item)
            | Self::Restore(item)
            | Self::Set { item, .. } => *item,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_id::AccountId;
    use crate::item_state::ItemState;

    #[test]
    fn every_kind_names_its_item() {
        let item = ItemId::new(AccountId::new(1), 1);
        assert_eq!(OpKind::Create(item).item(), item);
        assert_eq!(OpKind::Delete(item).item(), item);
        assert_eq!(OpKind::Restore(item).item(), item);
        assert_eq!(
            OpKind::Set {
                item,
                value: FieldValue::State(ItemState::Done)
            }
            .item(),
            item
        );
    }
}
