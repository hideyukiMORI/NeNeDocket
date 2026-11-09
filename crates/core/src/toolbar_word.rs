//! 道具の行・下書き・設定の文言の名前（Issue #124・docs/design/2026-11-09-toolbar-and-settings.md）。`Word::Toolbar` の中身。閉じた集合（RS-002）。
//! `{shown}`・`{total}`・`{number}` を含む文言は、呼び手が数や番号に置き換える。

/// 道具の行と、それが開くものの語。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolbarWord {
    /// 絞り込みの札「未完了」。
    FilterOpen,
    /// 絞り込みの札「期限超過」。
    FilterOverdue,
    /// 絞り込みの札「未割り当て」。
    FilterUnassigned,
    /// 絞り込みの札「確認待ち」。
    FilterReview,
    /// 検索の欄の案内の字。
    SearchHint,
    /// 「{shown} 件を表示（全 {total} 件）」。
    Showing,
    /// 「設定」。
    Settings,
    /// 「＋ 課題を追加」。
    AddItem,
    /// 下書きの見出し「新しい課題」。
    NewItem,
    /// 下書きの案内「Enter で追加・Esc で取り消し」。
    DraftHint,
    /// 「{number} を追加しました。今の絞り込みでは表示されません」。
    AddedOutside,
    /// 「絞り込みを外す」。
    ClearFilter,
    /// 設定のペインの「閉じる」。
    Close,
    /// 設定の「表示言語」。
    Language,
    /// 設定の「配色」。
    Theme,
    /// 設定の「参加者と権限」（表の下の段を出し入れする）。
    MembersToggle,
}
