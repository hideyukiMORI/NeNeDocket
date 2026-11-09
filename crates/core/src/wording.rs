//! 文言の表（FR-019・ADR 0011）。6 言語 × `Word`。UI はここからだけ文字列を得る。
//!
//! 翻訳は短く、窓の中に収める（FR-014）。言語を足すときは `Language` と、ここの関数を 1 つ足す。

use crate::attachment_word::AttachmentWord;
use crate::auto_field::AutoField;
use crate::field::Field;
use crate::help_word::HelpWord;
use crate::item_state::ItemState;
use crate::language::Language;
use crate::toolbar_word::ToolbarWord;
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
        Word::StateName(state) => japanese_state(state),
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
        Word::Members => "参加者",
        Word::Name => "名前",
        Word::Role => "役",
        Word::Owner => "持ち主",
        Word::Editor => "編集者",
        Word::Viewer => "閲覧のみ",
        Word::Anyone => "誰でも編集",
        Word::Permissions => "権限",
        Word::CannotWritePermissions => "権限を書けません",
        Word::Overwrote => "他の人の変更を上書きしました。履歴で選び直せます",
        Word::Groups => "グループ",
        Word::BadGroupName => "グループ名に「:」は使えません",
        Word::NoSuccessor => "引き継ぎ先なし",
        Word::PermissionsOn => "権限を有効にした",
        Word::PermissionsOff => "権限を無効にした",
        Word::Allowed => "編集者に加えた",
        Word::Disallowed => "編集者から外した",
        Word::Grouped => "グループに入れた",
        Word::Ungrouped => "グループから外した",
        Word::Succeeded => "引き継ぎ",
        Word::OwnerChanged => "持ち主を替えた",
        Word::Attachment(word) => japanese_attachment(word),
        Word::AutoField(field) => japanese_auto_field(field),
        Word::Toolbar(word) => japanese_toolbar(word),
        Word::Help(word) => japanese_help(word),
    }
}

const fn japanese_attachment(word: AttachmentWord) -> &'static str {
    match word {
        AttachmentWord::Attachments => "添付",
        AttachmentWord::Path => "パス",
        AttachmentWord::Attach => "添付する",
        AttachmentWord::Detach => "外す",
        AttachmentWord::Size => "大きさ",
        AttachmentWord::Attached => "添付した",
        AttachmentWord::Detached => "添付を外した",
        AttachmentWord::CannotRead => "添付のファイルを読めません",
        AttachmentWord::CannotWrite => "添付の写しを置けません",
        AttachmentWord::Altered => "中身が記録と合いません",
        AttachmentWord::Open => "開く",
        AttachmentWord::Truncated => "先頭だけを出しています",
        AttachmentWord::CannotOpen => "OS のアプリで開けません",
        AttachmentWord::CannotDecode => "画像を読めません",
    }
}

const fn english(word: Word) -> &'static str {
    match word {
        Word::All => "All",
        Word::StateName(state) => english_state(state),
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
        Word::Members => "Members",
        Word::Name => "Name",
        Word::Role => "Role",
        Word::Owner => "Owner",
        Word::Editor => "Editor",
        Word::Viewer => "Read only",
        Word::Anyone => "Anyone may edit",
        Word::Permissions => "Permissions",
        Word::CannotWritePermissions => "permissions: cannot write",
        Word::Overwrote => "overwrote a change by someone else; pick again from the history",
        Word::Groups => "Groups",
        Word::BadGroupName => "a group name cannot contain \":\"",
        Word::NoSuccessor => "No successor",
        Word::PermissionsOn => "turned permissions on",
        Word::PermissionsOff => "turned permissions off",
        Word::Allowed => "added an editor",
        Word::Disallowed => "removed an editor",
        Word::Grouped => "added to a group",
        Word::Ungrouped => "removed from a group",
        Word::Succeeded => "succession",
        Word::OwnerChanged => "changed the owner",
        Word::Attachment(word) => english_attachment(word),
        Word::AutoField(field) => english_auto_field(field),
        Word::Toolbar(word) => english_toolbar(word),
        Word::Help(word) => english_help(word),
    }
}

const fn english_attachment(word: AttachmentWord) -> &'static str {
    match word {
        AttachmentWord::Attachments => "Attachments",
        AttachmentWord::Path => "Path",
        AttachmentWord::Attach => "Attach",
        AttachmentWord::Detach => "Detach",
        AttachmentWord::Size => "Size",
        AttachmentWord::Attached => "attached",
        AttachmentWord::Detached => "detached",
        AttachmentWord::CannotRead => "cannot read the file to attach",
        AttachmentWord::CannotWrite => "cannot store the attachment",
        AttachmentWord::Altered => "contents differ from the record",
        AttachmentWord::Open => "Open",
        AttachmentWord::Truncated => "Showing the beginning only",
        AttachmentWord::CannotOpen => "Cannot open with the system application",
        AttachmentWord::CannotDecode => "Cannot decode the image",
    }
}

const fn simplified_chinese(word: Word) -> &'static str {
    match word {
        Word::All => "全部",
        Word::StateName(state) => simplified_chinese_state(state),
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
        Word::Members => "成员",
        Word::Name => "名称",
        Word::Role => "角色",
        Word::Owner => "所有者",
        Word::Editor => "编辑者",
        Word::Viewer => "只读",
        Word::Anyone => "任何人可编辑",
        Word::Permissions => "权限",
        Word::CannotWritePermissions => "无法写入权限",
        Word::Overwrote => "覆盖了他人的改动，可在历史中重新选择",
        Word::Groups => "组",
        Word::BadGroupName => "组名不能包含“:”",
        Word::NoSuccessor => "无继承者",
        Word::PermissionsOn => "启用了权限",
        Word::PermissionsOff => "停用了权限",
        Word::Allowed => "添加了编辑者",
        Word::Disallowed => "移除了编辑者",
        Word::Grouped => "加入了组",
        Word::Ungrouped => "移出了组",
        Word::Succeeded => "继承",
        Word::OwnerChanged => "更换了所有者",
        Word::Attachment(word) => simplified_chinese_attachment(word),
        Word::AutoField(field) => simplified_chinese_auto_field(field),
        Word::Toolbar(word) => simplified_chinese_toolbar(word),
        Word::Help(word) => simplified_chinese_help(word),
    }
}

const fn simplified_chinese_attachment(word: AttachmentWord) -> &'static str {
    match word {
        AttachmentWord::Attachments => "附件",
        AttachmentWord::Path => "路径",
        AttachmentWord::Attach => "添加附件",
        AttachmentWord::Detach => "移除",
        AttachmentWord::Size => "大小",
        AttachmentWord::Attached => "添加了附件",
        AttachmentWord::Detached => "移除了附件",
        AttachmentWord::CannotRead => "无法读取要附加的文件",
        AttachmentWord::CannotWrite => "无法保存附件",
        AttachmentWord::Altered => "内容与记录不符",
        AttachmentWord::Open => "打开",
        AttachmentWord::Truncated => "仅显示开头部分",
        AttachmentWord::CannotOpen => "无法用系统应用打开",
        AttachmentWord::CannotDecode => "无法读取图像",
    }
}

const fn traditional_chinese(word: Word) -> &'static str {
    match word {
        Word::All => "全部",
        Word::StateName(state) => traditional_chinese_state(state),
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
        Word::Members => "成員",
        Word::Name => "名稱",
        Word::Role => "角色",
        Word::Owner => "擁有者",
        Word::Editor => "編輯者",
        Word::Viewer => "唯讀",
        Word::Anyone => "任何人可編輯",
        Word::Permissions => "權限",
        Word::CannotWritePermissions => "無法寫入權限",
        Word::Overwrote => "覆蓋了他人的改動，可在歷史中重新選擇",
        Word::Groups => "群組",
        Word::BadGroupName => "群組名稱不能包含「:」",
        Word::NoSuccessor => "無繼承者",
        Word::PermissionsOn => "啟用了權限",
        Word::PermissionsOff => "停用了權限",
        Word::Allowed => "新增了編輯者",
        Word::Disallowed => "移除了編輯者",
        Word::Grouped => "加入了群組",
        Word::Ungrouped => "移出了群組",
        Word::Succeeded => "繼承",
        Word::OwnerChanged => "更換了擁有者",
        Word::Attachment(word) => traditional_chinese_attachment(word),
        Word::AutoField(field) => traditional_chinese_auto_field(field),
        Word::Toolbar(word) => traditional_chinese_toolbar(word),
        Word::Help(word) => traditional_chinese_help(word),
    }
}

const fn traditional_chinese_attachment(word: AttachmentWord) -> &'static str {
    match word {
        AttachmentWord::Attachments => "附件",
        AttachmentWord::Path => "路徑",
        AttachmentWord::Attach => "新增附件",
        AttachmentWord::Detach => "移除",
        AttachmentWord::Size => "大小",
        AttachmentWord::Attached => "新增了附件",
        AttachmentWord::Detached => "移除了附件",
        AttachmentWord::CannotRead => "無法讀取要附加的檔案",
        AttachmentWord::CannotWrite => "無法保存附件",
        AttachmentWord::Altered => "內容與記錄不符",
        AttachmentWord::Open => "開啟",
        AttachmentWord::Truncated => "僅顯示開頭部分",
        AttachmentWord::CannotOpen => "無法用系統應用程式開啟",
        AttachmentWord::CannotDecode => "無法讀取圖像",
    }
}

const fn korean(word: Word) -> &'static str {
    match word {
        Word::All => "전체",
        Word::StateName(state) => korean_state(state),
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
        Word::Members => "참가자",
        Word::Name => "이름",
        Word::Role => "역할",
        Word::Owner => "소유자",
        Word::Editor => "편집자",
        Word::Viewer => "읽기 전용",
        Word::Anyone => "누구나 편집 가능",
        Word::Permissions => "권한",
        Word::CannotWritePermissions => "권한을 쓸 수 없습니다",
        Word::Overwrote => "다른 사람의 변경을 덮어썼습니다. 기록에서 다시 선택할 수 있습니다",
        Word::Groups => "그룹",
        Word::BadGroupName => "그룹 이름에 \":\"를 쓸 수 없습니다",
        Word::NoSuccessor => "인계 대상 없음",
        Word::PermissionsOn => "권한을 켰음",
        Word::PermissionsOff => "권한을 껐음",
        Word::Allowed => "편집자를 추가함",
        Word::Disallowed => "편집자를 제거함",
        Word::Grouped => "그룹에 추가함",
        Word::Ungrouped => "그룹에서 제거함",
        Word::Succeeded => "인계",
        Word::OwnerChanged => "소유자를 변경함",
        Word::Attachment(word) => korean_attachment(word),
        Word::AutoField(field) => korean_auto_field(field),
        Word::Toolbar(word) => korean_toolbar(word),
        Word::Help(word) => korean_help(word),
    }
}

const fn korean_attachment(word: AttachmentWord) -> &'static str {
    match word {
        AttachmentWord::Attachments | AttachmentWord::Attach => "첨부",
        AttachmentWord::Path => "경로",
        AttachmentWord::Detach => "제거",
        AttachmentWord::Size => "크기",
        AttachmentWord::Attached => "첨부함",
        AttachmentWord::Detached => "첨부를 제거함",
        AttachmentWord::CannotRead => "첨부할 파일을 읽을 수 없습니다",
        AttachmentWord::CannotWrite => "첨부를 저장할 수 없습니다",
        AttachmentWord::Altered => "내용이 기록과 다릅니다",
        AttachmentWord::Open => "열기",
        AttachmentWord::Truncated => "앞부분만 표시합니다",
        AttachmentWord::CannotOpen => "시스템 앱으로 열 수 없습니다",
        AttachmentWord::CannotDecode => "이미지를 읽을 수 없습니다",
    }
}

const fn spanish(word: Word) -> &'static str {
    match word {
        Word::All => "Todos",
        Word::StateName(state) => spanish_state(state),
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
        Word::Members => "Miembros",
        Word::Name => "Nombre",
        Word::Role => "Rol",
        Word::Owner => "Propietario",
        Word::Editor => "Editor",
        Word::Viewer => "Solo lectura",
        Word::Anyone => "Cualquiera puede editar",
        Word::Permissions => "Permisos",
        Word::CannotWritePermissions => "permisos: no se puede escribir",
        Word::Overwrote => "se sobrescribió un cambio ajeno; elija de nuevo en el historial",
        Word::Groups => "Grupos",
        Word::BadGroupName => "un nombre de grupo no puede contener \":\"",
        Word::NoSuccessor => "Sin sucesor",
        Word::PermissionsOn => "activó los permisos",
        Word::PermissionsOff => "desactivó los permisos",
        Word::Allowed => "añadió un editor",
        Word::Disallowed => "quitó un editor",
        Word::Grouped => "añadió a un grupo",
        Word::Ungrouped => "quitó de un grupo",
        Word::Succeeded => "sucesión",
        Word::OwnerChanged => "cambió el propietario",
        Word::Attachment(word) => spanish_attachment(word),
        Word::AutoField(field) => spanish_auto_field(field),
        Word::Toolbar(word) => spanish_toolbar(word),
        Word::Help(word) => spanish_help(word),
    }
}

const fn spanish_attachment(word: AttachmentWord) -> &'static str {
    match word {
        AttachmentWord::Attachments => "Adjuntos",
        AttachmentWord::Path => "Ruta",
        AttachmentWord::Attach => "Adjuntar",
        AttachmentWord::Detach => "Quitar",
        AttachmentWord::Size => "Tamaño",
        AttachmentWord::Attached => "adjuntó",
        AttachmentWord::Detached => "quitó un adjunto",
        AttachmentWord::CannotRead => "no se puede leer el archivo a adjuntar",
        AttachmentWord::CannotWrite => "no se puede guardar el adjunto",
        AttachmentWord::Altered => "el contenido no coincide con el registro",
        AttachmentWord::Open => "Abrir",
        AttachmentWord::Truncated => "Solo se muestra el comienzo",
        AttachmentWord::CannotOpen => "No se puede abrir con la aplicación del sistema",
        AttachmentWord::CannotDecode => "No se puede leer la imagen",
    }
}

const fn japanese_auto_field(field: AutoField) -> &'static str {
    match field {
        AutoField::Opened => "起票日",
        AutoField::Opener => "起票者",
        AutoField::Closed => "完了日",
        AutoField::Updated => "更新日",
    }
}

const fn english_auto_field(field: AutoField) -> &'static str {
    match field {
        AutoField::Opened => "Opened",
        AutoField::Opener => "Opened by",
        AutoField::Closed => "Closed",
        AutoField::Updated => "Updated",
    }
}

const fn simplified_chinese_auto_field(field: AutoField) -> &'static str {
    match field {
        AutoField::Opened => "创建日期",
        AutoField::Opener => "创建人",
        AutoField::Closed => "完成日期",
        AutoField::Updated => "更新日期",
    }
}

const fn traditional_chinese_auto_field(field: AutoField) -> &'static str {
    match field {
        AutoField::Opened => "建立日期",
        AutoField::Opener => "建立者",
        AutoField::Closed => "完成日期",
        AutoField::Updated => "更新日期",
    }
}

const fn korean_auto_field(field: AutoField) -> &'static str {
    match field {
        AutoField::Opened => "등록일",
        AutoField::Opener => "등록자",
        AutoField::Closed => "완료일",
        AutoField::Updated => "수정일",
    }
}

const fn spanish_auto_field(field: AutoField) -> &'static str {
    match field {
        AutoField::Opened => "Abierto",
        AutoField::Opener => "Abierto por",
        AutoField::Closed => "Cerrado",
        AutoField::Updated => "Actualizado",
    }
}

const fn japanese_state(state: ItemState) -> &'static str {
    match state {
        ItemState::Open => "未着手",
        ItemState::InProgress => "対応中",
        ItemState::Review => "確認中",
        ItemState::OnHold => "保留",
        ItemState::Done => "完了",
        ItemState::Cancelled => "中止",
    }
}

const fn english_state(state: ItemState) -> &'static str {
    match state {
        ItemState::Open => "Open",
        ItemState::InProgress => "In progress",
        ItemState::Review => "Review",
        ItemState::OnHold => "On hold",
        ItemState::Done => "Done",
        ItemState::Cancelled => "Cancelled",
    }
}

const fn simplified_chinese_state(state: ItemState) -> &'static str {
    match state {
        ItemState::Open => "待处理",
        ItemState::InProgress => "进行中",
        ItemState::Review => "待确认",
        ItemState::OnHold => "搁置",
        ItemState::Done => "已完成",
        ItemState::Cancelled => "已取消",
    }
}

const fn traditional_chinese_state(state: ItemState) -> &'static str {
    match state {
        ItemState::Open => "待處理",
        ItemState::InProgress => "進行中",
        ItemState::Review => "待確認",
        ItemState::OnHold => "擱置",
        ItemState::Done => "已完成",
        ItemState::Cancelled => "已取消",
    }
}

const fn korean_state(state: ItemState) -> &'static str {
    match state {
        ItemState::Open => "열림",
        ItemState::InProgress => "진행 중",
        ItemState::Review => "확인 중",
        ItemState::OnHold => "보류",
        ItemState::Done => "완료",
        ItemState::Cancelled => "취소",
    }
}

const fn spanish_state(state: ItemState) -> &'static str {
    match state {
        ItemState::Open => "Abierto",
        ItemState::InProgress => "En curso",
        ItemState::Review => "En revisión",
        ItemState::OnHold => "En espera",
        ItemState::Done => "Hecho",
        ItemState::Cancelled => "Cancelado",
    }
}

const fn japanese_toolbar(word: ToolbarWord) -> &'static str {
    match word {
        ToolbarWord::FilterOpen => "未完了",
        ToolbarWord::FilterOverdue => "期限超過",
        ToolbarWord::FilterUnassigned => "未割り当て",
        ToolbarWord::FilterReview => "確認待ち",
        ToolbarWord::SearchHint => "題名・本文で検索  Ctrl+F",
        ToolbarWord::Showing => "{shown} 件を表示（全 {total} 件）",
        ToolbarWord::Settings => "設定",
        ToolbarWord::AddItem => "＋ 課題を追加",
        ToolbarWord::NewItem => "新しい課題",
        ToolbarWord::DraftHint => "Enter で追加・Esc で取り消し",
        ToolbarWord::AddedOutside => "{number} を追加しました。今の絞り込みでは表示されません",
        ToolbarWord::ClearFilter => "絞り込みを外す",
        ToolbarWord::Close => "閉じる",
        ToolbarWord::Language => "表示言語",
        ToolbarWord::Theme => "配色",
        ToolbarWord::MembersToggle => "参加者と権限",
        ToolbarWord::Apply => "適用",
        ToolbarWord::MineSection => "自分だけ（この PC）",
        ToolbarWord::DocketSection => "この台帳（全員に効く）",
        ToolbarWord::Columns => "表の列",
    }
}

const fn english_toolbar(word: ToolbarWord) -> &'static str {
    match word {
        ToolbarWord::FilterOpen => "Open",
        ToolbarWord::FilterOverdue => "Overdue",
        ToolbarWord::FilterUnassigned => "Unassigned",
        ToolbarWord::FilterReview => "In review",
        ToolbarWord::SearchHint => "Search titles and text  Ctrl+F",
        ToolbarWord::Showing => "Showing {shown} of {total}",
        ToolbarWord::Settings => "Settings",
        ToolbarWord::AddItem => "+ New issue",
        ToolbarWord::NewItem => "New issue",
        ToolbarWord::DraftHint => "Enter to add · Esc to cancel",
        ToolbarWord::AddedOutside => "Added {number}. The current filter hides it",
        ToolbarWord::ClearFilter => "Clear filter",
        ToolbarWord::Close => "Close",
        ToolbarWord::Language => "Language",
        ToolbarWord::Theme => "Colors",
        ToolbarWord::MembersToggle => "Members and permissions",
        ToolbarWord::Apply => "Apply",
        ToolbarWord::MineSection => "Just me (this PC)",
        ToolbarWord::DocketSection => "This docket (everyone)",
        ToolbarWord::Columns => "Table columns",
    }
}

const fn simplified_chinese_toolbar(word: ToolbarWord) -> &'static str {
    match word {
        ToolbarWord::FilterOpen => "未完成",
        ToolbarWord::FilterOverdue => "已逾期",
        ToolbarWord::FilterUnassigned => "未分配",
        ToolbarWord::FilterReview => "待确认",
        ToolbarWord::SearchHint => "搜索标题和正文  Ctrl+F",
        ToolbarWord::Showing => "显示 {shown} 条（共 {total} 条）",
        ToolbarWord::Settings => "设置",
        ToolbarWord::AddItem => "＋ 新建问题",
        ToolbarWord::NewItem => "新问题",
        ToolbarWord::DraftHint => "Enter 添加 · Esc 取消",
        ToolbarWord::AddedOutside => "已添加 {number}。当前筛选下不显示",
        ToolbarWord::ClearFilter => "清除筛选",
        ToolbarWord::Close => "关闭",
        ToolbarWord::Language => "显示语言",
        ToolbarWord::Theme => "配色",
        ToolbarWord::MembersToggle => "成员与权限",
        ToolbarWord::Apply => "应用",
        ToolbarWord::MineSection => "仅自己（本机）",
        ToolbarWord::DocketSection => "此台账（所有人）",
        ToolbarWord::Columns => "表格列",
    }
}

const fn traditional_chinese_toolbar(word: ToolbarWord) -> &'static str {
    match word {
        ToolbarWord::FilterOpen => "未完成",
        ToolbarWord::FilterOverdue => "已逾期",
        ToolbarWord::FilterUnassigned => "未指派",
        ToolbarWord::FilterReview => "待確認",
        ToolbarWord::SearchHint => "搜尋標題與內文  Ctrl+F",
        ToolbarWord::Showing => "顯示 {shown} 筆（共 {total} 筆）",
        ToolbarWord::Settings => "設定",
        ToolbarWord::AddItem => "＋ 新增問題",
        ToolbarWord::NewItem => "新問題",
        ToolbarWord::DraftHint => "Enter 新增 · Esc 取消",
        ToolbarWord::AddedOutside => "已新增 {number}。目前的篩選不會顯示",
        ToolbarWord::ClearFilter => "清除篩選",
        ToolbarWord::Close => "關閉",
        ToolbarWord::Language => "顯示語言",
        ToolbarWord::Theme => "配色",
        ToolbarWord::MembersToggle => "成員與權限",
        ToolbarWord::Apply => "套用",
        ToolbarWord::MineSection => "僅自己（本機）",
        ToolbarWord::DocketSection => "此台帳（所有人）",
        ToolbarWord::Columns => "表格欄",
    }
}

const fn korean_toolbar(word: ToolbarWord) -> &'static str {
    match word {
        ToolbarWord::FilterOpen => "미완료",
        ToolbarWord::FilterOverdue => "기한 초과",
        ToolbarWord::FilterUnassigned => "미배정",
        ToolbarWord::FilterReview => "확인 대기",
        ToolbarWord::SearchHint => "제목·본문 검색  Ctrl+F",
        ToolbarWord::Showing => "{shown}건 표시 (전체 {total}건)",
        ToolbarWord::Settings => "설정",
        ToolbarWord::AddItem => "＋ 이슈 추가",
        ToolbarWord::NewItem => "새 이슈",
        ToolbarWord::DraftHint => "Enter 추가 · Esc 취소",
        ToolbarWord::AddedOutside => "{number}을(를) 추가했습니다. 현재 필터에서는 보이지 않습니다",
        ToolbarWord::ClearFilter => "필터 해제",
        ToolbarWord::Close => "닫기",
        ToolbarWord::Language => "표시 언어",
        ToolbarWord::Theme => "배색",
        ToolbarWord::MembersToggle => "참가자와 권한",
        ToolbarWord::Apply => "적용",
        ToolbarWord::MineSection => "나만 (이 PC)",
        ToolbarWord::DocketSection => "이 대장 (모두에게)",
        ToolbarWord::Columns => "표 열",
    }
}

const fn spanish_toolbar(word: ToolbarWord) -> &'static str {
    match word {
        ToolbarWord::FilterOpen => "Abiertas",
        ToolbarWord::FilterOverdue => "Vencidas",
        ToolbarWord::FilterUnassigned => "Sin asignar",
        ToolbarWord::FilterReview => "En revisión",
        ToolbarWord::SearchHint => "Buscar en títulos y textos  Ctrl+F",
        ToolbarWord::Showing => "Mostrando {shown} de {total}",
        ToolbarWord::Settings => "Ajustes",
        ToolbarWord::AddItem => "+ Nueva incidencia",
        ToolbarWord::NewItem => "Nueva incidencia",
        ToolbarWord::DraftHint => "Enter para añadir · Esc para cancelar",
        ToolbarWord::AddedOutside => "Se añadió {number}. El filtro actual la oculta",
        ToolbarWord::ClearFilter => "Quitar filtro",
        ToolbarWord::Close => "Cerrar",
        ToolbarWord::Language => "Idioma",
        ToolbarWord::Theme => "Colores",
        ToolbarWord::MembersToggle => "Miembros y permisos",
        ToolbarWord::Apply => "Aplicar",
        ToolbarWord::MineSection => "Solo yo (este PC)",
        ToolbarWord::DocketSection => "Este registro (todos)",
        ToolbarWord::Columns => "Columnas",
    }
}

const fn japanese_help(word: HelpWord) -> &'static str {
    match word {
        HelpWord::Title => "使い方 — 週 1 回定例をするチームの 1 週間",
        HelpWord::Keys => {
            "Ctrl+N 追加 · Ctrl+F 検索 · Ctrl+Z / Ctrl+Y 戻す・やり直す · Tab 次へ · Esc 閉じる · F1 この案内"
        }
        HelpWord::Name0 => "始める",
        HelpWord::What0 => "共有フォルダに台帳を作り、場所をチームに伝える",
        HelpWord::Go0 => "設定を開く",
        HelpWord::Name1 => "足す",
        HelpWord::What1 => "気づいたら誰でも、題名だけで課題を足す",
        HelpWord::Go1 => "＋ 課題を追加",
        HelpWord::Name2 => "割り振る",
        HelpWord::What2 => "担当の決まっていない課題に担当と期限を付ける",
        HelpWord::Go2 => "未割り当て",
        HelpWord::Name3 => "毎日",
        HelpWord::What3 => "自分の分を対応中にし、対応策を本文に書く",
        HelpWord::Go3 => "未完了",
        HelpWord::Name4 => "定例",
        HelpWord::What4 => "期限超過と、変わった所を全員で見る",
        HelpWord::Go4 => "期限超過",
        HelpWord::Name5 => "閉じる",
        HelpWord::What5 => "担当が確認中にし、確認者が完了にする",
        HelpWord::Go5 => "確認待ち",
    }
}

const fn english_help(word: HelpWord) -> &'static str {
    match word {
        HelpWord::Title => "How a team runs a week with weekly meetings",
        HelpWord::Keys => {
            "Ctrl+N add · Ctrl+F search · Ctrl+Z / Ctrl+Y undo, redo · Tab next · Esc close · F1 this help"
        }
        HelpWord::Name0 => "Start",
        HelpWord::What0 => "Create the docket in a shared folder and tell the team where it is",
        HelpWord::Go0 => "Open settings",
        HelpWord::Name1 => "Add",
        HelpWord::What1 => "Anyone adds an issue with just a title",
        HelpWord::Go1 => "+ New issue",
        HelpWord::Name2 => "Assign",
        HelpWord::What2 => "Give unassigned issues an assignee and a due date",
        HelpWord::Go2 => "Unassigned",
        HelpWord::Name3 => "Daily",
        HelpWord::What3 => "Set your issues to In progress and write the fix in the text",
        HelpWord::Go3 => "Open",
        HelpWord::Name4 => "Meeting",
        HelpWord::What4 => "Review overdue issues and what changed, together",
        HelpWord::Go4 => "Overdue",
        HelpWord::Name5 => "Close",
        HelpWord::What5 => "The assignee sets In review; the reviewer sets Done",
        HelpWord::Go5 => "In review",
    }
}

const fn simplified_chinese_help(word: HelpWord) -> &'static str {
    match word {
        HelpWord::Title => "使用方法 — 每周例会团队的一周",
        HelpWord::Keys => {
            "Ctrl+N 添加 · Ctrl+F 搜索 · Ctrl+Z / Ctrl+Y 撤销、重做 · Tab 下一个 · Esc 关闭 · F1 本帮助"
        }
        HelpWord::Name0 => "开始",
        HelpWord::What0 => "在共享文件夹中创建台账，并告诉团队位置",
        HelpWord::Go0 => "打开设置",
        HelpWord::Name1 => "添加",
        HelpWord::What1 => "任何人只写标题即可添加问题",
        HelpWord::Go1 => "＋ 新建问题",
        HelpWord::Name2 => "分配",
        HelpWord::What2 => "为未分配的问题指定负责人和期限",
        HelpWord::Go2 => "未分配",
        HelpWord::Name3 => "每天",
        HelpWord::What3 => "把自己的问题设为进行中，在正文写对策",
        HelpWord::Go3 => "未完成",
        HelpWord::Name4 => "例会",
        HelpWord::What4 => "大家一起查看逾期和变更",
        HelpWord::Go4 => "已逾期",
        HelpWord::Name5 => "关闭",
        HelpWord::What5 => "负责人设为待确认，确认人设为已完成",
        HelpWord::Go5 => "待确认",
    }
}

const fn traditional_chinese_help(word: HelpWord) -> &'static str {
    match word {
        HelpWord::Title => "使用方式 — 每週例會團隊的一週",
        HelpWord::Keys => {
            "Ctrl+N 新增 · Ctrl+F 搜尋 · Ctrl+Z / Ctrl+Y 復原、重做 · Tab 下一個 · Esc 關閉 · F1 本說明"
        }
        HelpWord::Name0 => "開始",
        HelpWord::What0 => "在共用資料夾建立台帳，並告訴團隊位置",
        HelpWord::Go0 => "開啟設定",
        HelpWord::Name1 => "新增",
        HelpWord::What1 => "任何人只寫標題就能新增問題",
        HelpWord::Go1 => "＋ 新增問題",
        HelpWord::Name2 => "指派",
        HelpWord::What2 => "為未指派的問題指定負責人和期限",
        HelpWord::Go2 => "未指派",
        HelpWord::Name3 => "每天",
        HelpWord::What3 => "把自己的問題設為進行中，在內文寫對策",
        HelpWord::Go3 => "未完成",
        HelpWord::Name4 => "例會",
        HelpWord::What4 => "大家一起查看逾期與變更",
        HelpWord::Go4 => "已逾期",
        HelpWord::Name5 => "結案",
        HelpWord::What5 => "負責人設為待確認，確認者設為已完成",
        HelpWord::Go5 => "待確認",
    }
}

const fn korean_help(word: HelpWord) -> &'static str {
    match word {
        HelpWord::Title => "사용법 — 주 1회 정례 회의 팀의 1주",
        HelpWord::Keys => {
            "Ctrl+N 추가 · Ctrl+F 검색 · Ctrl+Z / Ctrl+Y 실행 취소·다시 실행 · Tab 다음 · Esc 닫기 · F1 이 도움말"
        }
        HelpWord::Name0 => "시작",
        HelpWord::What0 => "공유 폴더에 대장을 만들고 위치를 팀에 알린다",
        HelpWord::Go0 => "설정 열기",
        HelpWord::Name1 => "추가",
        HelpWord::What1 => "누구나 제목만으로 이슈를 추가한다",
        HelpWord::Go1 => "＋ 이슈 추가",
        HelpWord::Name2 => "배정",
        HelpWord::What2 => "미배정 이슈에 담당자와 기한을 정한다",
        HelpWord::Go2 => "미배정",
        HelpWord::Name3 => "매일",
        HelpWord::What3 => "내 이슈를 진행 중으로 하고 대책을 본문에 쓴다",
        HelpWord::Go3 => "미완료",
        HelpWord::Name4 => "정례 회의",
        HelpWord::What4 => "기한 초과와 바뀐 점을 함께 본다",
        HelpWord::Go4 => "기한 초과",
        HelpWord::Name5 => "종료",
        HelpWord::What5 => "담당자가 확인 중으로, 확인자가 완료로 한다",
        HelpWord::Go5 => "확인 대기",
    }
}

const fn spanish_help(word: HelpWord) -> &'static str {
    match word {
        HelpWord::Title => "Cómo trabaja una semana un equipo con reunión semanal",
        HelpWord::Keys => {
            "Ctrl+N añadir · Ctrl+F buscar · Ctrl+Z / Ctrl+Y deshacer, rehacer · Tab siguiente · Esc cerrar · F1 esta ayuda"
        }
        HelpWord::Name0 => "Empezar",
        HelpWord::What0 => "Crea el registro en una carpeta compartida y avisa al equipo",
        HelpWord::Go0 => "Abrir ajustes",
        HelpWord::Name1 => "Añadir",
        HelpWord::What1 => "Cualquiera añade una incidencia solo con el título",
        HelpWord::Go1 => "+ Nueva incidencia",
        HelpWord::Name2 => "Asignar",
        HelpWord::What2 => "Da responsable y fecha a las incidencias sin asignar",
        HelpWord::Go2 => "Sin asignar",
        HelpWord::Name3 => "Cada día",
        HelpWord::What3 => "Pon tus incidencias en curso y escribe la solución en el texto",
        HelpWord::Go3 => "Abiertas",
        HelpWord::Name4 => "Reunión",
        HelpWord::What4 => "Revisad juntos lo vencido y lo que cambió",
        HelpWord::Go4 => "Vencidas",
        HelpWord::Name5 => "Cerrar",
        HelpWord::What5 => "El responsable pone En revisión; quien revisa, Hecho",
        HelpWord::Go5 => "En revisión",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 全部の語（`Word` の閉じた集合を 1 か所で列挙する。増えたらここも増える）。
    const WORDS: [Word; 116] = [
        Word::Toolbar(ToolbarWord::Apply),
        Word::Toolbar(ToolbarWord::MineSection),
        Word::Toolbar(ToolbarWord::DocketSection),
        Word::Toolbar(ToolbarWord::Columns),
        Word::Help(HelpWord::Title),
        Word::Help(HelpWord::Keys),
        Word::Help(HelpWord::Name0),
        Word::Help(HelpWord::What0),
        Word::Help(HelpWord::Go0),
        Word::Help(HelpWord::Name1),
        Word::Help(HelpWord::What1),
        Word::Help(HelpWord::Go1),
        Word::Help(HelpWord::Name2),
        Word::Help(HelpWord::What2),
        Word::Help(HelpWord::Go2),
        Word::Help(HelpWord::Name3),
        Word::Help(HelpWord::What3),
        Word::Help(HelpWord::Go3),
        Word::Help(HelpWord::Name4),
        Word::Help(HelpWord::What4),
        Word::Help(HelpWord::Go4),
        Word::Help(HelpWord::Name5),
        Word::Help(HelpWord::What5),
        Word::Help(HelpWord::Go5),
        Word::Toolbar(ToolbarWord::FilterOpen),
        Word::Toolbar(ToolbarWord::FilterOverdue),
        Word::Toolbar(ToolbarWord::FilterUnassigned),
        Word::Toolbar(ToolbarWord::FilterReview),
        Word::Toolbar(ToolbarWord::SearchHint),
        Word::Toolbar(ToolbarWord::Showing),
        Word::Toolbar(ToolbarWord::Settings),
        Word::Toolbar(ToolbarWord::AddItem),
        Word::Toolbar(ToolbarWord::NewItem),
        Word::Toolbar(ToolbarWord::DraftHint),
        Word::Toolbar(ToolbarWord::AddedOutside),
        Word::Toolbar(ToolbarWord::ClearFilter),
        Word::Toolbar(ToolbarWord::Close),
        Word::Toolbar(ToolbarWord::Language),
        Word::Toolbar(ToolbarWord::Theme),
        Word::Toolbar(ToolbarWord::MembersToggle),
        Word::All,
        Word::StateName(ItemState::Open),
        Word::StateName(ItemState::InProgress),
        Word::StateName(ItemState::Review),
        Word::StateName(ItemState::OnHold),
        Word::StateName(ItemState::Done),
        Word::StateName(ItemState::Cancelled),
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
        Word::AutoField(AutoField::Opened),
        Word::AutoField(AutoField::Opener),
        Word::AutoField(AutoField::Closed),
        Word::AutoField(AutoField::Updated),
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
        Word::Members,
        Word::Name,
        Word::Role,
        Word::Owner,
        Word::Editor,
        Word::Viewer,
        Word::Anyone,
        Word::Permissions,
        Word::CannotWritePermissions,
        Word::Overwrote,
        Word::Groups,
        Word::BadGroupName,
        Word::NoSuccessor,
        Word::PermissionsOn,
        Word::PermissionsOff,
        Word::Allowed,
        Word::Disallowed,
        Word::Grouped,
        Word::Ungrouped,
        Word::Succeeded,
        Word::OwnerChanged,
        Word::Attachment(AttachmentWord::Attachments),
        Word::Attachment(AttachmentWord::Path),
        Word::Attachment(AttachmentWord::Attach),
        Word::Attachment(AttachmentWord::Detach),
        Word::Attachment(AttachmentWord::Size),
        Word::Attachment(AttachmentWord::Attached),
        Word::Attachment(AttachmentWord::Detached),
        Word::Attachment(AttachmentWord::CannotRead),
        Word::Attachment(AttachmentWord::CannotWrite),
        Word::Attachment(AttachmentWord::Altered),
        Word::Attachment(AttachmentWord::Open),
        Word::Attachment(AttachmentWord::Truncated),
        Word::Attachment(AttachmentWord::CannotOpen),
        Word::Attachment(AttachmentWord::CannotDecode),
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
