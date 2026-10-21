//! 併合した課題 1 件（FR-001）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::date::Date;
use crate::field_value::FieldValue;
use crate::item_id::ItemId;
use crate::item_state::ItemState;
use crate::op_kind::OpKind;

/// 題名・本文・状態・担当・期限・ラベル・並び順と、消されているか。すべて併合の派生値で、正本はジャーナル。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    id: ItemId,
    title: String,
    body: String,
    state: ItemState,
    assignee: String,
    due: Option<Date>,
    labels: Vec<String>,
    rank: i64,
    deleted: bool,
}

impl Item {
    /// `Create` の直後。項目は空、状態は `Open`。
    #[must_use]
    pub const fn created(id: ItemId) -> Self {
        Self {
            id,
            title: String::new(),
            body: String::new(),
            state: ItemState::Open,
            assignee: String::new(),
            due: None,
            labels: Vec::new(),
            rank: 0,
            deleted: false,
        }
    }

    /// 項目を置いた状態。
    #[must_use]
    pub fn with(mut self, value: FieldValue) -> Self {
        match value {
            FieldValue::Title(title) => self.title = title,
            FieldValue::Body(body) => self.body = body,
            FieldValue::State(state) => self.state = state,
            FieldValue::Assignee(assignee) => self.assignee = assignee,
            FieldValue::Due(due) => self.due = due,
            FieldValue::Labels(labels) => self.labels = labels,
            FieldValue::Rank(rank) => self.rank = rank,
        }
        self
    }

    /// 操作の種類を当てた課題（`Docket::applied` の 1 件版）。権限の記録は何もしない。
    #[must_use]
    pub fn applied(self, kind: &OpKind) -> Self {
        match kind {
            OpKind::Create(_) | OpKind::Admin(_) => self,
            OpKind::Set { value, .. } => self.with(value.clone()),
            OpKind::Delete(_) => self.with_deleted(true),
            OpKind::Restore(_) => self.with_deleted(false),
        }
    }

    /// 消した・戻した状態。
    #[must_use]
    pub fn with_deleted(self, deleted: bool) -> Self {
        Self { deleted, ..self }
    }

    /// いまの値を、項目の名前から取り出す。逆の操作（Undo）を作るのに使う。
    #[must_use]
    pub fn value_of(&self, field: crate::field::Field) -> FieldValue {
        match field {
            crate::field::Field::Title => FieldValue::Title(self.title.clone()),
            crate::field::Field::Body => FieldValue::Body(self.body.clone()),
            crate::field::Field::State => FieldValue::State(self.state),
            crate::field::Field::Assignee => FieldValue::Assignee(self.assignee.clone()),
            crate::field::Field::Due => FieldValue::Due(self.due),
            crate::field::Field::Labels => FieldValue::Labels(self.labels.clone()),
            crate::field::Field::Rank => FieldValue::Rank(self.rank),
        }
    }

    /// ID。
    #[must_use]
    pub const fn id(&self) -> ItemId {
        self.id
    }

    /// 題名。
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 本文。
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// 状態。
    #[must_use]
    pub const fn state(&self) -> ItemState {
        self.state
    }

    /// 担当。
    #[must_use]
    pub fn assignee(&self) -> &str {
        &self.assignee
    }

    /// 期限。
    #[must_use]
    pub const fn due(&self) -> Option<Date> {
        self.due
    }

    /// ラベル。
    #[must_use]
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// 並び順。
    #[must_use]
    pub const fn rank(&self) -> i64 {
        self.rank
    }

    /// 消されているか。
    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_id::AccountId;
    use crate::field::Field;

    fn filled() -> Item {
        Item::created(ItemId::new(AccountId::new(1), 1))
            .with(FieldValue::Title(String::from("題")))
            .with(FieldValue::Body(String::from("本文")))
            .with(FieldValue::State(ItemState::Done))
            .with(FieldValue::Assignee(String::from("mori")))
            .with(FieldValue::Due(Date::new(2026, 12, 31)))
            .with(FieldValue::Labels(alloc::vec![String::from("core")]))
            .with(FieldValue::Rank(-5))
            .with_deleted(true)
    }

    #[test]
    fn values_set_and_read_back() {
        let item = filled();
        assert_eq!(item.title(), "題");
        assert_eq!(item.body(), "本文");
        assert_eq!(item.state(), ItemState::Done);
        assert_eq!(item.assignee(), "mori");
        assert_eq!(item.due(), Date::new(2026, 12, 31));
        assert_eq!(item.labels(), [String::from("core")]);
        assert_eq!(item.rank(), -5);
        assert!(item.is_deleted());
        assert_eq!(item.id(), ItemId::new(AccountId::new(1), 1));
    }

    #[test]
    fn value_of_reads_every_field_and_a_new_item_is_open() {
        let item = filled();
        for field in Field::ALL {
            assert_eq!(item.value_of(field).field(), field);
        }
        assert_eq!(
            item.value_of(Field::Title),
            FieldValue::Title(String::from("題"))
        );
        assert_eq!(
            Item::created(ItemId::new(AccountId::new(1), 1)).state(),
            ItemState::Open
        );
    }
}
