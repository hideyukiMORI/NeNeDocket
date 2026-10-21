//! 併合した台帳。課題の集合と、作られた順（ADR 0004）。正本ではなく、ジャーナルからの派生値。

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::item::Item;
use crate::item_id::ItemId;
use crate::op::Op;
use crate::op_kind::OpKind;

/// 課題（ID 順）と、作られた順の ID の列と、宛先の無い操作の数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Docket {
    items: BTreeMap<ItemId, Item>,
    creation_order: Vec<ItemId>,
    orphans: usize,
}

impl Docket {
    /// 空の台帳。
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            items: BTreeMap::new(),
            creation_order: Vec::new(),
            orphans: 0,
        }
    }

    /// 作られた順の課題と、宛先の無い操作の数から（スナップショットから戻す・ADR 0015）。
    #[must_use]
    pub fn from_parts(items: Vec<Item>, orphans: usize) -> Self {
        let mut docket = Self::empty();
        docket.orphans = orphans;
        for item in items {
            docket = docket.replaced(item);
        }
        docket
    }

    /// 課題 1 件を置き換えた台帳。無ければ作られた順の末尾に足す。
    /// 全順序で前の操作が後から届いたとき、その課題の操作だけから作り直して置く（ADR 0015 の 2）。
    #[must_use]
    pub fn replaced(mut self, item: Item) -> Self {
        let id = item.id();
        if self.items.insert(id, item).is_none() {
            self.creation_order.push(id);
        }
        self
    }

    /// 操作を 1 つ適用した台帳。`merge` が全順序に並べた後で呼ぶ。
    /// 作られていない課題への操作は、黙って落とさず「宛先の無い操作」として数える（FR-014 の元）。
    #[must_use]
    pub fn applied(mut self, op: &Op) -> Self {
        match op.kind() {
            OpKind::Create(id) => {
                if !self.items.contains_key(id) {
                    self.items.insert(*id, Item::created(*id));
                    self.creation_order.push(*id);
                }
            }
            OpKind::Set { item, value } => match self.items.remove(item) {
                Some(found) => {
                    self.items.insert(*item, found.with(value.clone()));
                }
                None => self.orphans = self.orphans.saturating_add(1),
            },
            OpKind::Delete(id) | OpKind::Restore(id) => match self.items.remove(id) {
                Some(found) => {
                    let deleted = matches!(op.kind(), OpKind::Delete(_));
                    self.items.insert(*id, found.with_deleted(deleted));
                }
                None => self.orphans = self.orphans.saturating_add(1),
            },
            OpKind::Admin(_) => {}
        }
        self
    }

    /// 課題。無ければ `None`。
    #[must_use]
    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id)
    }

    /// 作られた順の課題。消したものも含む。
    #[must_use]
    pub fn items_in_creation_order(&self) -> Vec<&Item> {
        self.creation_order
            .iter()
            .filter_map(|id| self.items.get(id))
            .collect()
    }

    /// 作られた順の、消していない課題。表に出すもの。
    #[must_use]
    pub fn live_items(&self) -> Vec<&Item> {
        self.items_in_creation_order()
            .into_iter()
            .filter(|item| !item.is_deleted())
            .collect()
    }

    /// 課題の数（消したものも含む）。
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// 課題が 1 つも無いか。
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// 宛先の無い操作の数。0 でなければ利用者に示す（FR-014）。
    #[must_use]
    pub const fn orphans(&self) -> usize {
        self.orphans
    }
}
