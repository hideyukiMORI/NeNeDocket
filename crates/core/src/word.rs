//! 画面に出す文言の名前（FR-019）。UI に文字列を書かず、`wording` の表から言語ごとの文字列を引く（ARC-001）。

use crate::field::Field;
use crate::item_state::ItemState;

/// 画面の語。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Word {
    /// 絞り込みの「すべて」。
    All,
    /// 課題の状態の名前。
    StateName(ItemState),
    /// 項目の名前（表の列・履歴）。
    FieldName(Field),
    /// 「完了も表示」。
    ShowDone,
    /// 「履歴」。
    History,
    /// 「追加」。
    Add,
    /// 「削除」。
    Delete,
    /// 表の番号の列。
    Number,
    /// 履歴の「いつ」。
    When,
    /// 履歴の「誰が」。
    Who,
    /// 履歴の「何を」。
    Change,
    /// 履歴の「作成」。
    Created,
    /// 履歴の「削除」。
    Deleted,
    /// 履歴の「復元」。
    Restored,
    /// 履歴の Undo の印。
    UndoMark,
    /// ジャーナルに書けない。
    CannotWriteJournal,
    /// 他人のジャーナルを読めない。
    CannotReadJournals,
    /// 宛先の無い操作（数が続く）。
    OrphanOperations,
    /// 読めなかった行（数が続く）。
    RejectedLines,
    /// 期限の形が違う。
    NotADate,
    /// 設定を書けない。
    CannotWriteSettings,
    /// 知らないテーマ。
    UnknownTheme,
    /// 言語の選択肢の「OS に従う」。
    FollowSystem,
    /// 署名か権限が通らず捨てた行（数が続く・FR-010）。
    UnverifiedLines,
    /// 台帳にある自分の公開鍵が手元と違う。
    KeyMismatch,
    /// 自分はこの台帳の編集者でない。
    NotAnEditor,
    /// 参加者の段（FR-009）。
    Members,
    /// 参加者の「名前」。
    Name,
    /// 参加者の「役」。
    Role,
    /// 役: 持ち主。
    Owner,
    /// 役: 編集者。
    Editor,
    /// 役: 閲覧のみ。
    Viewer,
    /// 役: 権限が無い台帳では誰でも編集。
    Anyone,
    /// 権限のチェック（有効・無効）。
    Permissions,
    /// 権限を書けない。
    CannotWritePermissions,
    /// 他人の変更を上書きした（履歴で選び直せる・FR-008）。
    Overwrote,
    /// 参加者の編集行の「グループ」（`,` 区切り）。
    Groups,
    /// グループ名に `:` は使えない。
    BadGroupName,
    /// 参加者の編集行の「引き継ぎ先」の箱の、引き継がない候補（閉じた箱の名でもある）。
    NoSuccessor,
    /// 履歴: 権限を有効にした（ADR 0014）。
    PermissionsOn,
    /// 履歴: 権限を無効にした。
    PermissionsOff,
    /// 履歴: 編集者に加えた（名前が続く）。
    Allowed,
    /// 履歴: 編集者から外した。
    Disallowed,
    /// 履歴: グループに入れた（`グループ: 名前` が続く）。
    Grouped,
    /// 履歴: グループから外した。
    Ungrouped,
    /// 履歴: 引き継ぎ（`新 ← 旧` が続く）。
    Succeeded,
    /// 履歴: 持ち主を替えた（名前が続く）。
    OwnerChanged,
}
