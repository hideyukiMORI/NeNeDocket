//! 課題の項目と、その新しい値。操作は項目単位で勝敗が付く（FR-008・ADR 0004 の 6）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::date::Date;
use crate::field::Field;
use crate::item_state::ItemState;

/// 項目ごとの値。閉じた集合（RS-002）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldValue {
    /// 題名。
    Title(String),
    /// 本文。項目単位で勝敗が付く（文字単位の併合はしない）。
    Body(String),
    /// 状態。
    State(ItemState),
    /// 担当。空なら無し。
    Assignee(String),
    /// 期限。`None` は無し。
    Due(Option<Date>),
    /// ラベルの集合。全体で 1 つの値。
    Labels(Vec<String>),
    /// 並び順。小さいほど上。
    Rank(i64),
}

impl FieldValue {
    /// どの項目か。
    #[must_use]
    pub const fn field(&self) -> Field {
        match self {
            Self::Title(_) => Field::Title,
            Self::Body(_) => Field::Body,
            Self::State(_) => Field::State,
            Self::Assignee(_) => Field::Assignee,
            Self::Due(_) => Field::Due,
            Self::Labels(_) => Field::Labels,
            Self::Rank(_) => Field::Rank,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_value_names_its_field() {
        assert_eq!(FieldValue::Title(String::new()).field(), Field::Title);
        assert_eq!(FieldValue::Body(String::new()).field(), Field::Body);
        assert_eq!(FieldValue::State(ItemState::Done).field(), Field::State);
        assert_eq!(FieldValue::Assignee(String::new()).field(), Field::Assignee);
        assert_eq!(FieldValue::Due(None).field(), Field::Due);
        assert_eq!(FieldValue::Labels(Vec::new()).field(), Field::Labels);
        assert_eq!(FieldValue::Rank(0).field(), Field::Rank);
    }
}
