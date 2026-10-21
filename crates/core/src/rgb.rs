//! 8 ビットの 3 色。見えの色の値。

/// 赤・緑・青（0〜255）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    red: u8,
    green: u8,
    blue: u8,
}

impl Rgb {
    /// 3 色から。
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// 赤。
    #[must_use]
    pub const fn red(self) -> u8 {
        self.red
    }

    /// 緑。
    #[must_use]
    pub const fn green(self) -> u8 {
        self.green
    }

    /// 青。
    #[must_use]
    pub const fn blue(self) -> u8 {
        self.blue
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_read_back() {
        let ink = Rgb::new(1, 2, 3);
        assert_eq!((ink.red(), ink.green(), ink.blue()), (1, 2, 3));
    }
}
