//! 起動時に一度だけ取り込む環境の事実（ARC-004 の表）。

use nene_docket_core::ui_scale::UiScale;

/// デスクトップの dpi（X11 は `Xft.dpi`、Windows はモニタの dpi）。無ければ 1 倍。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    desktop_dpi: Option<u32>,
}

impl Environment {
    /// デスクトップの dpi から作る。分からなければ `None`。
    #[must_use]
    pub const fn new(desktop_dpi: Option<u32>) -> Self {
        Self { desktop_dpi }
    }

    /// UI の倍率（FR-016）。
    #[must_use]
    pub fn scale(self) -> UiScale {
        UiScale::choose(self.desktop_dpi)
    }

    /// デスクトップの dpi。
    #[must_use]
    pub const fn desktop_dpi(self) -> Option<u32> {
        self.desktop_dpi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scale_follows_the_dpi() {
        assert_eq!(Environment::new(None).scale(), UiScale::Single);
        assert_eq!(Environment::new(Some(192)).scale(), UiScale::Double);
        assert_eq!(Environment::new(Some(192)).desktop_dpi(), Some(192));
    }
}
