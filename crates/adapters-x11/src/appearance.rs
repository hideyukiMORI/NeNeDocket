//! OS の明暗（ADR 0012）。X11 には標準の観測が無いので分からない（`system` はライト）。

use nene_docket_core::appearance::Appearance;

/// OS の明暗。X11 では常に `None`。デスクトップごとの設定（GNOME の `color-scheme` など）を読む経路は別の Issue。
#[must_use]
pub const fn read() -> Option<Appearance> {
    None
}
