//! 文言の表（FR-019・ADR 0011）。6 言語 × `Word`。UI はここからだけ文字列を得る。
//!
//! 翻訳は短く、窓の中に収める（FR-014）。言語を足すときは `Language` と、ここの関数を 1 つ足す。

use crate::field::Field;
use crate::item_state::ItemState;
use crate::language::Language;
use crate::word::Word;

/// 言語ごとの文字列。
#[must_use]
pub const fn text(language: Language, word: Word) -> &'static str {
    match language {
        Language::Japanese => japanese(word),
        Language::English => english(word),
        Language::SimplifiedChinese => simplified_chinese(word),
        Language::TraditionalChinese => traditional_chinese(word),
        Language::Korean => korean(word),
        Language::Spanish => spanish(word),
    }
}

const fn japanese(word: Word) -> &'static str {
    match word {
        Word::All => "すべて",
        Word::StateName(ItemState::Open) => "未着手",
        Word::StateName(ItemState::InProgress) => "着手中",
        Word::StateName(ItemState::Done) => "完了",
        Word::FieldName(Field::Title) => "題名",
        Word::FieldName(Field::Body) => "本文",
        Word::FieldName(Field::State) => "状態",
        Word::FieldName(Field::Assignee) => "担当",
        Word::FieldName(Field::Due) => "期限",
        Word::FieldName(Field::Labels) => "ラベル",
        Word::FieldName(Field::Rank) => "並び順",
        Word::ShowDone => "完了も表示",
        Word::History => "履歴",
        Word::Add => "追加",
        Word::Delete | Word::Deleted => "削除",
        Word::Number => "#",
        Word::When => "日時",
        Word::Who => "誰が",
        Word::Change => "変更",
        Word::Created => "作成",
        Word::Restored => "復元",
        Word::UndoMark => "取り消し: ",
        Word::CannotWriteJournal => "ジャーナルに書けません",
        Word::CannotReadJournals => "他の人のジャーナルを読めません",
        Word::OrphanOperations => "宛先の無い操作",
        Word::RejectedLines => "読めなかった行",
        Word::NotADate => "期限は YYYY-MM-DD の形です。保存していません",
        Word::CannotWriteSettings => "設定を書けません",
        Word::UnknownTheme => "知らないテーマです",
        Word::FollowSystem => "OS に従う",
        Word::UnverifiedLines => "署名か権限が通らない行",
        Word::KeyMismatch => "台帳にある自分の公開鍵が手元と違います",
        Word::NotAnEditor => "この台帳の編集者ではありません。変更は無視されます",
    }
}

const fn english(word: Word) -> &'static str {
    match word {
        Word::All => "All",
        Word::StateName(ItemState::Open) => "Open",
        Word::StateName(ItemState::InProgress) => "In progress",
        Word::StateName(ItemState::Done) => "Done",
        Word::FieldName(Field::Title) => "Title",
        Word::FieldName(Field::Body) => "Body",
        Word::FieldName(Field::State) => "State",
        Word::FieldName(Field::Assignee) => "Assignee",
        Word::FieldName(Field::Due) => "Due",
        Word::FieldName(Field::Labels) => "Labels",
        Word::FieldName(Field::Rank) => "Rank",
        Word::ShowDone => "Show done",
        Word::History => "History",
        Word::Add => "Add",
        Word::Delete => "Delete",
        Word::Number => "#",
        Word::When => "When",
        Word::Who => "Who",
        Word::Change => "Change",
        Word::Created => "created",
        Word::Deleted => "deleted",
        Word::Restored => "restored",
        Word::UndoMark => "undo: ",
        Word::CannotWriteJournal => "journal: cannot write",
        Word::CannotReadJournals => "journal: cannot read others",
        Word::OrphanOperations => "operations on unknown items",
        Word::RejectedLines => "unreadable journal lines",
        Word::NotADate => "due: not a date (YYYY-MM-DD), not saved",
        Word::CannotWriteSettings => "settings: cannot write",
        Word::UnknownTheme => "unknown theme",
        Word::FollowSystem => "System",
        Word::UnverifiedLines => "lines failing signature or permission",
        Word::KeyMismatch => "my key in this docket differs from mine",
        Word::NotAnEditor => "not an editor of this docket; changes are ignored",
    }
}

const fn simplified_chinese(word: Word) -> &'static str {
    match word {
        Word::All => "全部",
        Word::StateName(ItemState::Open) => "待处理",
        Word::StateName(ItemState::InProgress) => "进行中",
        Word::StateName(ItemState::Done) => "已完成",
        Word::FieldName(Field::Title) => "标题",
        Word::FieldName(Field::Body) => "正文",
        Word::FieldName(Field::State) => "状态",
        Word::FieldName(Field::Assignee) => "负责人",
        Word::FieldName(Field::Due) => "截止日期",
        Word::FieldName(Field::Labels) => "标签",
        Word::FieldName(Field::Rank) => "排序",
        Word::ShowDone => "显示已完成",
        Word::History => "历史",
        Word::Add => "添加",
        Word::Delete | Word::Deleted => "删除",
        Word::Number => "#",
        Word::When => "时间",
        Word::Who => "谁",
        Word::Change => "变更",
        Word::Created => "创建",
        Word::Restored => "恢复",
        Word::UndoMark => "撤销: ",
        Word::CannotWriteJournal => "无法写入日志",
        Word::CannotReadJournals => "无法读取他人的日志",
        Word::OrphanOperations => "目标不存在的操作",
        Word::RejectedLines => "无法读取的行",
        Word::NotADate => "截止日期须为 YYYY-MM-DD，未保存",
        Word::CannotWriteSettings => "无法写入设置",
        Word::UnknownTheme => "未知的主题",
        Word::FollowSystem => "跟随系统",
        Word::UnverifiedLines => "签名或权限未通过的行",
        Word::KeyMismatch => "台账中的本人公钥与本机不一致",
        Word::NotAnEditor => "不是此台账的编辑者，改动将被忽略",
    }
}

const fn traditional_chinese(word: Word) -> &'static str {
    match word {
        Word::All => "全部",
        Word::StateName(ItemState::Open) => "待處理",
        Word::StateName(ItemState::InProgress) => "進行中",
        Word::StateName(ItemState::Done) => "已完成",
        Word::FieldName(Field::Title) => "標題",
        Word::FieldName(Field::Body) => "內文",
        Word::FieldName(Field::State) => "狀態",
        Word::FieldName(Field::Assignee) => "負責人",
        Word::FieldName(Field::Due) => "截止日期",
        Word::FieldName(Field::Labels) => "標籤",
        Word::FieldName(Field::Rank) => "排序",
        Word::ShowDone => "顯示已完成",
        Word::History => "歷史",
        Word::Add => "新增",
        Word::Delete | Word::Deleted => "刪除",
        Word::Number => "#",
        Word::When => "時間",
        Word::Who => "誰",
        Word::Change => "變更",
        Word::Created => "建立",
        Word::Restored => "恢復",
        Word::UndoMark => "復原: ",
        Word::CannotWriteJournal => "無法寫入日誌",
        Word::CannotReadJournals => "無法讀取他人的日誌",
        Word::OrphanOperations => "目標不存在的操作",
        Word::RejectedLines => "無法讀取的行",
        Word::NotADate => "截止日期須為 YYYY-MM-DD，未儲存",
        Word::CannotWriteSettings => "無法寫入設定",
        Word::UnknownTheme => "未知的主題",
        Word::FollowSystem => "跟隨系統",
        Word::UnverifiedLines => "簽名或權限未通過的行",
        Word::KeyMismatch => "帳冊中的本人公鑰與本機不一致",
        Word::NotAnEditor => "不是此帳冊的編輯者，改動將被忽略",
    }
}

const fn korean(word: Word) -> &'static str {
    match word {
        Word::All => "전체",
        Word::StateName(ItemState::Open) => "열림",
        Word::StateName(ItemState::InProgress) => "진행 중",
        Word::StateName(ItemState::Done) => "완료",
        Word::FieldName(Field::Title) => "제목",
        Word::FieldName(Field::Body) => "본문",
        Word::FieldName(Field::State) => "상태",
        Word::FieldName(Field::Assignee) => "담당자",
        Word::FieldName(Field::Due) => "기한",
        Word::FieldName(Field::Labels) => "라벨",
        Word::FieldName(Field::Rank) => "순서",
        Word::ShowDone => "완료 표시",
        Word::History => "기록",
        Word::Add => "추가",
        Word::Delete | Word::Deleted => "삭제",
        Word::Number => "#",
        Word::When => "일시",
        Word::Who => "누가",
        Word::Change => "변경",
        Word::Created => "생성",
        Word::Restored => "복원",
        Word::UndoMark => "실행 취소: ",
        Word::CannotWriteJournal => "저널에 쓸 수 없음",
        Word::CannotReadJournals => "다른 사람의 저널을 읽을 수 없음",
        Word::OrphanOperations => "대상이 없는 작업",
        Word::RejectedLines => "읽을 수 없는 줄",
        Word::NotADate => "기한은 YYYY-MM-DD 형식이어야 합니다. 저장되지 않음",
        Word::CannotWriteSettings => "설정을 쓸 수 없음",
        Word::UnknownTheme => "알 수 없는 테마",
        Word::FollowSystem => "시스템",
        Word::UnverifiedLines => "서명 또는 권한이 맞지 않는 행",
        Word::KeyMismatch => "장부에 있는 내 공개 키가 이 기기와 다릅니다",
        Word::NotAnEditor => "이 장부의 편집자가 아닙니다. 변경은 무시됩니다",
    }
}

const fn spanish(word: Word) -> &'static str {
    match word {
        Word::All => "Todos",
        Word::StateName(ItemState::Open) => "Abierto",
        Word::StateName(ItemState::InProgress) => "En curso",
        Word::StateName(ItemState::Done) => "Hecho",
        Word::FieldName(Field::Title) => "Título",
        Word::FieldName(Field::Body) => "Cuerpo",
        Word::FieldName(Field::State) => "Estado",
        Word::FieldName(Field::Assignee) => "Responsable",
        Word::FieldName(Field::Due) => "Fecha límite",
        Word::FieldName(Field::Labels) => "Etiquetas",
        Word::FieldName(Field::Rank) => "Orden",
        Word::ShowDone => "Mostrar hechos",
        Word::History => "Historial",
        Word::Add => "Añadir",
        Word::Delete => "Eliminar",
        Word::Number => "#",
        Word::When => "Cuándo",
        Word::Who => "Quién",
        Word::Change => "Cambio",
        Word::Created => "creado",
        Word::Deleted => "eliminado",
        Word::Restored => "restaurado",
        Word::UndoMark => "deshacer: ",
        Word::CannotWriteJournal => "diario: no se puede escribir",
        Word::CannotReadJournals => "diario: no se pueden leer los ajenos",
        Word::OrphanOperations => "operaciones sobre elementos desconocidos",
        Word::RejectedLines => "líneas ilegibles del diario",
        Word::NotADate => "fecha límite: no es una fecha (AAAA-MM-DD), no guardada",
        Word::CannotWriteSettings => "configuración: no se puede escribir",
        Word::UnknownTheme => "tema desconocido",
        Word::FollowSystem => "Sistema",
        Word::UnverifiedLines => "líneas sin firma o permiso válidos",
        Word::KeyMismatch => "mi clave en este registro no coincide con la local",
        Word::NotAnEditor => "no es editor de este registro; los cambios se ignoran",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 全部の語（`Word` の閉じた集合を 1 か所で列挙する。増えたらここも増える）。
    const WORDS: [Word; 34] = [
        Word::All,
        Word::StateName(ItemState::Open),
        Word::StateName(ItemState::InProgress),
        Word::StateName(ItemState::Done),
        Word::FieldName(Field::Title),
        Word::FieldName(Field::Body),
        Word::FieldName(Field::State),
        Word::FieldName(Field::Assignee),
        Word::FieldName(Field::Due),
        Word::FieldName(Field::Labels),
        Word::FieldName(Field::Rank),
        Word::ShowDone,
        Word::History,
        Word::Add,
        Word::Delete,
        Word::Number,
        Word::When,
        Word::Who,
        Word::Change,
        Word::Created,
        Word::Deleted,
        Word::Restored,
        Word::UndoMark,
        Word::CannotWriteJournal,
        Word::CannotReadJournals,
        Word::OrphanOperations,
        Word::RejectedLines,
        Word::NotADate,
        Word::CannotWriteSettings,
        Word::UnknownTheme,
        Word::FollowSystem,
        Word::UnverifiedLines,
        Word::KeyMismatch,
        Word::NotAnEditor,
    ];

    #[test]
    fn every_language_has_every_word_and_the_states_differ() {
        for language in Language::ALL {
            for word in WORDS {
                assert!(!text(language, word).is_empty(), "{language:?} {word:?}");
            }
            let states: alloc::vec::Vec<&str> = ItemState::ALL
                .iter()
                .map(|state| text(language, Word::StateName(*state)))
                .collect();
            assert_ne!(states[0], states[1]);
            assert_ne!(states[1], states[2]);
        }
        assert_eq!(text(Language::Japanese, Word::Add), "追加");
        assert_eq!(
            text(Language::English, Word::StateName(ItemState::InProgress)),
            "In progress"
        );
        assert_eq!(text(Language::Korean, Word::Number), "#");
    }
}
