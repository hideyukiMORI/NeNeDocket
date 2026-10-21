//! 履歴の 1 行の「何を・何から何へ」（FR-006）。文言は表示言語で引く。

use alloc::format;
use alloc::string::String;

use nene_docket_core::date::Date;
use nene_docket_core::field::Field;
use nene_docket_core::field_value::FieldValue;
use nene_docket_core::language::Language;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

/// 操作の種類と、`Set` なら前後の値。閉じた集合（RS-002）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryChange {
    /// 作った。
    Created,
    /// 消した。
    Deleted,
    /// 戻した。
    Restored,
    /// 項目を置いた。
    Set {
        /// 項目。
        field: Field,
        /// 置く前の値。
        before: FieldValue,
        /// 置いた値。
        after: FieldValue,
    },
}

impl HistoryChange {
    /// 文言。`Set` は `<項目> : <前> -> <後>`。
    #[must_use]
    pub fn describe(&self, language: Language) -> String {
        match self {
            Self::Created => String::from(wording::text(language, Word::Created)),
            Self::Deleted => String::from(wording::text(language, Word::Deleted)),
            Self::Restored => String::from(wording::text(language, Word::Restored)),
            Self::Set {
                field,
                before,
                after,
            } => format!(
                "{} : {} -> {}",
                wording::text(language, Word::FieldName(*field)),
                value_text(language, before),
                value_text(language, after)
            ),
        }
    }
}

/// 本文の改行を 1 行に収める印。
const LINE_BREAK_MARK: &str = "↵";

/// 値の文言。状態は表示言語の語、期限は `YYYY-MM-DD`、ラベルは `, ` 区切り、本文の改行は `↵`。
fn value_text(language: Language, value: &FieldValue) -> String {
    match value {
        FieldValue::Title(text) | FieldValue::Assignee(text) => text.clone(),
        FieldValue::Body(text) => text.replace('\n', LINE_BREAK_MARK),
        FieldValue::State(state) => String::from(wording::text(language, Word::StateName(*state))),
        FieldValue::Due(due) => due.map(Date::text).unwrap_or_default(),
        FieldValue::Labels(labels) => labels.join(", "),
        FieldValue::Rank(rank) => format!("{rank}"),
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::item_state::ItemState;

    use super::*;

    #[test]
    fn every_value_kind_and_every_change_has_a_text() {
        let set = |before: FieldValue, after: FieldValue| HistoryChange::Set {
            field: after.field(),
            before,
            after,
        };
        let cases = [
            (
                set(
                    FieldValue::Body(String::new()),
                    FieldValue::Body(String::from("body")),
                ),
                "Body :  -> body",
            ),
            (
                set(
                    FieldValue::Assignee(String::new()),
                    FieldValue::Assignee(String::from("rina")),
                ),
                "Assignee :  -> rina",
            ),
            (
                set(
                    FieldValue::Due(None),
                    FieldValue::Due(Date::new(2026, 1, 2)),
                ),
                "Due :  -> 2026-01-02",
            ),
            (
                set(
                    FieldValue::Labels(alloc::vec![]),
                    FieldValue::Labels(alloc::vec![String::from("x"), String::from("y")]),
                ),
                "Labels :  -> x, y",
            ),
            (
                set(FieldValue::Rank(0), FieldValue::Rank(-3)),
                "Rank : 0 -> -3",
            ),
            (
                set(
                    FieldValue::State(ItemState::Open),
                    FieldValue::State(ItemState::Done),
                ),
                "State : Open -> Done",
            ),
            (HistoryChange::Created, "created"),
            (HistoryChange::Deleted, "deleted"),
            (HistoryChange::Restored, "restored"),
        ];
        for (change, english) in &cases {
            assert_eq!(change.describe(Language::English), *english);
            for language in Language::ALL {
                assert!(!change.describe(language).is_empty());
            }
        }
        assert_eq!(
            cases[5].0.describe(Language::Japanese),
            "状態 : 未着手 -> 完了"
        );
    }
}
