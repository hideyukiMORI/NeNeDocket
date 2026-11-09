//! 自動の項目（Issue #117・ADR 0023）。併合から導き、利用者は書かない。閉じた集合（RS-002）。

/// 起票日・起票者・完了日・更新日。`Word::AutoField` の中身にもなる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoField {
    /// 起票日（作る操作の壁時計）。
    Opened,
    /// 起票者（作ったアカウント）。
    Opener,
    /// 完了日（最後に完了へ移した操作の壁時計）。
    Closed,
    /// 更新日（最後に当たった操作の壁時計）。
    Updated,
}
