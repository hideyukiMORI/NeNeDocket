//! 描画命令の色の意味。

/// 何の色で描くか。実際の色は UI が見え（ダーク・ライト）から決める。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paint {
    /// 窓の地。
    Background,
    /// 部品の面（入力欄・ボタン・表の行）。
    Surface,
    /// 部品の縁。
    Border,
    /// 文字と印。
    Ink,
    /// 控えめな文字（ヘッダ・ヒント）。
    Muted,
    /// 強調（フォーカスの縁・押している面・チェックの印）。
    Accent,
    /// 強調の上に置く文字。
    OnAccent,
    /// 選択の背景（文字の選択・光らせている候補・選んだ行）。
    Selection,
}
