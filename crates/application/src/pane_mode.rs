//! 右のペインに何を出すか（Issue #124 / #125）。下書きと設定は同時に出ないので、1 つの値で持つ。

use nene_docket_core::text_edit::TextEdit;

/// 課題（選んでいなければ台帳全体の履歴）・下書き・設定。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneMode {
    /// 選んだ課題（選んでいなければ台帳全体の履歴）。
    Item,
    /// 「＋ 課題を追加」の下書きの題名。Enter で初めて追記する。
    Draft(TextEdit),
    /// 設定。
    Settings,
}
