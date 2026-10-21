//! 操作（op）。ジャーナルの 1 レコードであり、台帳を変える最小の単位（ADR 0004 の 3）。

use crate::op_id::OpId;
use crate::op_kind::OpKind;
use crate::wall_clock::WallClock;

/// ID・壁時計・打ち消す操作の ID（Undo / Redo なら）・種類。不変。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Op {
    id: OpId,
    wall: WallClock,
    undoes: Option<OpId>,
    kind: OpKind,
}

impl Op {
    /// 値から作る。
    #[must_use]
    pub const fn new(id: OpId, wall: WallClock, kind: OpKind) -> Self {
        Self {
            id,
            wall,
            undoes: None,
            kind,
        }
    }

    /// この操作が打ち消す操作を添える。Undo は逆の操作を追記する（ADR 0004 の 4）。Redo は Undo を打ち消す。
    #[must_use]
    pub fn undoing(self, target: OpId) -> Self {
        Self {
            undoes: Some(target),
            ..self
        }
    }

    /// ID。
    #[must_use]
    pub const fn id(&self) -> OpId {
        self.id
    }

    /// 壁時計。表示にだけ使う。
    #[must_use]
    pub const fn wall(&self) -> WallClock {
        self.wall
    }

    /// 打ち消す操作の ID。
    #[must_use]
    pub const fn undoes(&self) -> Option<OpId> {
        self.undoes
    }

    /// 種類。
    #[must_use]
    pub const fn kind(&self) -> &OpKind {
        &self.kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_id::AccountId;
    use crate::item_id::ItemId;
    use crate::logical_clock::LogicalClock;

    #[test]
    fn undoing_records_the_target() {
        let account = AccountId::new(1);
        let first = OpId::new(LogicalClock::new(1), account);
        let second = OpId::new(LogicalClock::new(2), account);
        let item = ItemId::new(account, 1);
        let op = Op::new(second, WallClock::new(100), OpKind::Delete(item)).undoing(first);
        assert_eq!(op.undoes(), Some(first));
        assert_eq!(op.id(), second);
        assert_eq!(op.wall().seconds(), 100);
        assert_eq!(op.kind(), &OpKind::Delete(item));
        assert_eq!(
            Op::new(first, WallClock::new(0), OpKind::Create(item)).undoes(),
            None
        );
    }
}
