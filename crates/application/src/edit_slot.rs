//! 編集行の文字の欄（題名・担当・期限・ラベル）。欄の文字列と課題の項目を行き来する。閉じた集合（RS-002）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::date::Date;
use nene_docket_core::field::Field;
use nene_docket_core::field_value::FieldValue;
use nene_docket_core::item::Item;

/// 文字で編集する項目。状態は箱で選ぶのでここに無い。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditSlot {
    /// 題名。
    Title,
    /// 担当。
    Assignee,
    /// 期限。`YYYY-MM-DD`、空は期限なし。
    Due,
    /// ラベル。`,` で区切る。前後の空白は落とし、空は無し。
    Labels,
}

/// ラベルの区切り。
const LABEL_SEPARATOR: char = ',';

impl EditSlot {
    /// 編集行に並ぶ順。
    pub const ALL: [Self; 4] = [Self::Title, Self::Assignee, Self::Due, Self::Labels];

    /// 対応する項目。
    #[must_use]
    pub const fn field(self) -> Field {
        match self {
            Self::Title => Field::Title,
            Self::Assignee => Field::Assignee,
            Self::Due => Field::Due,
            Self::Labels => Field::Labels,
        }
    }

    /// 課題のいまの値を、欄に出す文字列にする。
    #[must_use]
    pub fn text_of(self, item: &Item) -> String {
        match self {
            Self::Title => String::from(item.title()),
            Self::Assignee => String::from(item.assignee()),
            Self::Due => item.due().map(Date::text).unwrap_or_default(),
            Self::Labels => item.labels().join(", "),
        }
    }

    /// 欄の文字列を項目の値にする。期限の形が違えば `None`（追記しない・FR-014 で示す）。
    #[must_use]
    pub fn value_of(self, text: &str) -> Option<FieldValue> {
        match self {
            Self::Title => Some(FieldValue::Title(String::from(text))),
            Self::Assignee => Some(FieldValue::Assignee(String::from(text))),
            Self::Due if text.is_empty() => Some(FieldValue::Due(None)),
            Self::Due => Date::parse(text).map(|date| FieldValue::Due(Some(date))),
            Self::Labels => Some(FieldValue::Labels(Self::labels_of(text))),
        }
    }

    /// `a, b` を `["a", "b"]` に。
    fn labels_of(text: &str) -> Vec<String> {
        text.split(LABEL_SEPARATOR)
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .map(String::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::item_id::ItemId;

    use super::*;

    #[test]
    fn text_and_value_round_trip_and_a_bad_date_is_refused() {
        let item = Item::created(ItemId::new(AccountId::new(1), 1))
            .with(FieldValue::Title(String::from("t")))
            .with(FieldValue::Assignee(String::from("a")))
            .with(FieldValue::Due(Date::new(2026, 12, 31)))
            .with(FieldValue::Labels(alloc::vec![
                String::from("bug"),
                String::from("ui")
            ]));
        for slot in EditSlot::ALL {
            let text = slot.text_of(&item);
            assert_eq!(slot.value_of(&text), Some(item.value_of(slot.field())));
        }
        assert_eq!(EditSlot::Due.text_of(&item), "2026-12-31");
        assert_eq!(EditSlot::Due.value_of(""), Some(FieldValue::Due(None)));
        assert_eq!(EditSlot::Due.value_of("2026-13-01"), None);
        assert_eq!(EditSlot::Due.value_of("tomorrow"), None);
        assert_eq!(EditSlot::Labels.text_of(&item), "bug, ui");
        assert_eq!(
            EditSlot::Labels.value_of(" bug ,, ui,"),
            Some(FieldValue::Labels(alloc::vec![
                String::from("bug"),
                String::from("ui")
            ]))
        );
        assert_eq!(
            EditSlot::Labels.value_of(""),
            Some(FieldValue::Labels(Vec::new()))
        );
    }
}
