//! 描くための値の組。UI はこれを描くだけ（ARC-011）。

use alloc::vec::Vec;

use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::language::Language;
use nene_docket_core::theme::Theme;
use nene_docket_core::ui_scale::UiScale;

/// 1 フレームの描画命令と、文字の大きさの元になる倍率と、表示言語（フォントの家族・地域）と、配色。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocketFrame {
    scale: UiScale,
    language: Language,
    theme: Theme,
    commands: Vec<DrawCommand>,
}

impl DocketFrame {
    /// 倍率・言語・配色と命令の列から作る。
    #[must_use]
    pub const fn new(
        (scale, language, theme): (UiScale, Language, Theme),
        commands: Vec<DrawCommand>,
    ) -> Self {
        Self {
            scale,
            language,
            theme,
            commands,
        }
    }

    /// UI の倍率。字の大きさはこれを掛けて決める。
    #[must_use]
    pub const fn scale(&self) -> UiScale {
        self.scale
    }

    /// 本文の字の大きさ（物理画素）。1x で 13（SPECIFICATION 第 2 節）。
    #[must_use]
    pub const fn font_px(&self) -> u16 {
        self.scale.px(13)
    }

    /// 表示言語。UI はこれでフォントの家族（X11）や地域（DirectWrite）を選ぶ。
    #[must_use]
    pub const fn language(&self) -> Language {
        self.language
    }

    /// 配色。UI は `Paint` をこれで色にする。
    #[must_use]
    pub const fn theme(&self) -> Theme {
        self.theme
    }

    /// 描画命令。列の順に描く。
    #[must_use]
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }
}
