//! 操作の種類。

use crate::admin_command::AdminCommand;
use crate::attachment::Attachment;
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
    /// 権限を変えた記録（ADR 0014）。台帳の状態は変えず、履歴にだけ出る。戻せない。
    Admin(AdminCommand),
    /// ファイルを添付した（FR-021）。写しは `attachments/` にある。
    Attach {
        /// 課題。
        item: ItemId,
        /// 添付。
        attachment: Attachment,
    },
    /// 添付を外した。写しは残る。`Attach` の逆。
    Detach {
        /// 課題。
        item: ItemId,
        /// 外した添付の連番。
        sequence: u32,
    },
}

impl OpKind {
    /// 触れる課題。権限の記録は課題に触れない。
    #[must_use]
    pub const fn item(&self) -> Option<ItemId> {
        match self {
            Self::Create(item)
            | Self::Delete(item)
            | Self::Restore(item)
            | Self::Set { item, .. }
            | Self::Attach { item, .. }
            | Self::Detach { item, .. } => Some(*item),
            Self::Admin(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_id::AccountId;
    use crate::item_state::ItemState;

    #[test]
    fn every_item_kind_names_its_item_and_the_admin_kind_names_none() {
        let item = ItemId::new(AccountId::new(1), 1);
        assert_eq!(OpKind::Create(item).item(), Some(item));
        assert_eq!(OpKind::Delete(item).item(), Some(item));
        assert_eq!(OpKind::Restore(item).item(), Some(item));
        assert_eq!(
            OpKind::Set {
                item,
                value: FieldValue::State(ItemState::Done)
            }
            .item(),
            Some(item)
        );
        assert_eq!(OpKind::Admin(AdminCommand::Permissions(true)).item(), None);
        assert_eq!(
            OpKind::Attach {
                item,
                attachment: Attachment::new(1, alloc::string::String::from("a"), 0, [0; 32])
            }
            .item(),
            Some(item)
        );
        assert_eq!(OpKind::Detach { item, sequence: 1 }.item(), Some(item));
    }
}
