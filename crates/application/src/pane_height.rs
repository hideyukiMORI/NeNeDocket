//! 表の下の段（履歴・本文・参加者）の高さ（FR-012）。論理画素。設定の版 5 に `pane_px = auto` か `pane_px = <高さ>` で残る。
//! `auto` は表の残りから決める（段 1 つなら 2/5、2 つなら 1/4 ずつ、3 つなら 1/5 ずつ）。

use alloc::string::{String, ToString as _};

/// 段の高さの下限（論理画素・行 2 つ分）。
const MIN_PX: u16 = 56;
/// 自動の語。
const AUTO: &str = "auto";

/// 自動か、利用者が境をドラッグして決めた 1 段の高さ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneHeight {
    /// 表の残りから決める。
    Auto,
    /// 1 段の高さ（論理画素・56 以上）。
    Fixed(u16),
}

impl PaneHeight {
    /// 下限（論理画素）。
    pub const MIN_PX: u16 = MIN_PX;

    /// 高さを決めた値（56 以上に丸める）。
    #[must_use]
    pub fn fixed(px: u16) -> Self {
        Self::Fixed(px.max(MIN_PX))
    }

    /// `auto` か `56` 以上の数から。それ以外は `None`。
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.trim();
        if text == AUTO {
            return Some(Self::Auto);
        }
        text.parse::<u16>()
            .ok()
            .filter(|px| *px >= MIN_PX)
            .map(Self::Fixed)
    }

    /// `parse` の逆。
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Self::Auto => String::from(AUTO),
            Self::Fixed(px) => px.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_auto_and_numbers_at_or_above_the_minimum_and_round_trips() {
        assert_eq!(PaneHeight::parse("auto"), Some(PaneHeight::Auto));
        assert_eq!(PaneHeight::parse(" 200 "), Some(PaneHeight::Fixed(200)));
        assert_eq!(PaneHeight::parse("56"), Some(PaneHeight::Fixed(56)));
        assert_eq!(PaneHeight::parse("55"), None);
        assert_eq!(PaneHeight::parse("tall"), None);
        assert_eq!(PaneHeight::parse(""), None);
        assert_eq!(PaneHeight::fixed(10), PaneHeight::Fixed(56));
        for value in [PaneHeight::Auto, PaneHeight::Fixed(123)] {
            assert_eq!(PaneHeight::parse(&value.text()), Some(value));
        }
    }
}
