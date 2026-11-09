//! F1 の案内の文言の名前（Issue #125・運用の筋書きの 6 段・docs/design/2026-11-09-toolbar-and-settings.md 第 5 節）。
//! `Word::Help` の中身。閉じた集合（RS-002）。

/// 6 段（始める・足す・割り振る・毎日・定例・閉じる）の名・すること・飛ぶ先と、見出しと鍵の一覧。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpWord {
    /// 見出し。
    Title,
    /// 鍵の一覧の 1 行。
    Keys,
    /// 0 段目の名。
    Name0,
    /// 0 段目で何をするか。
    What0,
    /// 0 段目から飛ぶ先（ボタンの字）。
    Go0,
    /// 1 段目の名。
    Name1,
    /// 1 段目で何をするか。
    What1,
    /// 1 段目から飛ぶ先（ボタンの字）。
    Go1,
    /// 2 段目の名。
    Name2,
    /// 2 段目で何をするか。
    What2,
    /// 2 段目から飛ぶ先（ボタンの字）。
    Go2,
    /// 3 段目の名。
    Name3,
    /// 3 段目で何をするか。
    What3,
    /// 3 段目から飛ぶ先（ボタンの字）。
    Go3,
    /// 4 段目の名。
    Name4,
    /// 4 段目で何をするか。
    What4,
    /// 4 段目から飛ぶ先（ボタンの字）。
    Go4,
    /// 5 段目の名。
    Name5,
    /// 5 段目で何をするか。
    What5,
    /// 5 段目から飛ぶ先（ボタンの字）。
    Go5,
}

impl HelpWord {
    /// 段ごとの（名・すること・飛ぶ先）。
    pub const STEPS: [(Self, Self, Self); 6] = [
        (Self::Name0, Self::What0, Self::Go0),
        (Self::Name1, Self::What1, Self::Go1),
        (Self::Name2, Self::What2, Self::Go2),
        (Self::Name3, Self::What3, Self::Go3),
        (Self::Name4, Self::What4, Self::Go4),
        (Self::Name5, Self::What5, Self::Go5),
    ];
}
