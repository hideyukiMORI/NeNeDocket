//! UI の倍率（FR-016・nene-ruler ADR 0011 の継承）。

/// 1x か 2x か。整数だけにするので、1 px の線が半端な画素に割れない。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiScale {
    /// 1 倍。dpi が分からないときもこれ。
    Single,
    /// 2 倍。
    Double,
}

impl UiScale {
    /// 1 倍の基準の dpi。
    pub const BASE_DPI: u32 = 96;
    /// これ以上の dpi は 2 倍。1.5 倍（96 × 1.5）で丸めを切り替える。
    pub const DOUBLE_FROM_DPI: u32 = 144;

    /// 掛ける数。
    #[must_use]
    pub const fn factor(self) -> u16 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
        }
    }

    /// 1x の寸法に倍率を掛ける。
    #[must_use]
    pub const fn px(self, base: u16) -> u16 {
        base.saturating_mul(self.factor())
    }

    /// dpi を最も近い整数倍に丸める。2 倍より大きくはしない。
    #[must_use]
    pub const fn from_dpi(dpi: u32) -> Self {
        if dpi >= Self::DOUBLE_FROM_DPI {
            Self::Double
        } else {
            Self::Single
        }
    }

    /// デスクトップの dpi から決める。分からなければ 1 倍。
    #[must_use]
    pub fn choose(desktop_dpi: Option<u32>) -> Self {
        desktop_dpi.map_or(Self::Single, Self::from_dpi)
    }

    /// X のリソース文字列（ルート窓の `RESOURCE_MANAGER`）から `Xft.dpi` の整数部を読む。
    /// 無い・読めないなら `None`。
    #[must_use]
    pub fn xft_dpi_in(resources: &str) -> Option<u32> {
        resources.lines().find_map(|line| {
            let value = line.strip_prefix("Xft.dpi:")?.trim();
            let digits = value.split('.').next()?;
            digits.parse::<u32>().ok().filter(|dpi| *dpi > 0)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_at_one_and_a_half() {
        assert_eq!(UiScale::from_dpi(96), UiScale::Single);
        assert_eq!(UiScale::from_dpi(143), UiScale::Single);
        assert_eq!(UiScale::from_dpi(144), UiScale::Double);
        assert_eq!(UiScale::Single.factor(), 1);
        assert_eq!(UiScale::Double.px(28), 56);
        assert_eq!(UiScale::choose(None), UiScale::Single);
        assert_eq!(UiScale::choose(Some(192)), UiScale::Double);
    }

    #[test]
    fn reads_xft_dpi_from_the_resources() {
        assert_eq!(
            UiScale::xft_dpi_in("Xcursor.size:\t24\nXft.dpi:\t192\n"),
            Some(192)
        );
        assert_eq!(UiScale::xft_dpi_in("Xft.dpi: 144.5"), Some(144));
        assert_eq!(UiScale::xft_dpi_in("Xft.dpi:\tlarge"), None);
        assert_eq!(UiScale::xft_dpi_in("Xft.dpi:\t0"), None);
        assert_eq!(UiScale::xft_dpi_in(""), None);
    }
}
