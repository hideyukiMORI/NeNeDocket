//! `WM_CHAR` の UTF-16 の単位を字に組み立てる。サロゲートの対は 2 つのメッセージで届く。Win32 を呼ばないので Linux でもテストする。

/// 待っている上位サロゲート。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharAssembler {
    high: Option<u16>,
}

impl CharAssembler {
    /// 何も待っていない。
    #[must_use]
    pub const fn empty() -> Self {
        Self { high: None }
    }

    /// 単位を 1 つ受ける。字が完成すれば返す。不正な並び（下位だけ・上位の後に上位）は捨てる。
    pub fn push(&mut self, unit: u16) -> Option<char> {
        match (self.high.take(), unit) {
            (_, 0xD800..=0xDBFF) => {
                self.high = Some(unit);
                None
            }
            (Some(high), 0xDC00..=0xDFFF) => char::decode_utf16([high, unit]).next()?.ok(),
            (None, 0xDC00..=0xDFFF) => None,
            (_, single) => char::from_u32(u32::from(single)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bmp_characters_come_straight_and_pairs_join() {
        let mut assembler = CharAssembler::empty();
        assert_eq!(assembler.push(0x65), Some('e'));
        assert_eq!(assembler.push(0x65E5), Some('日'));
        assert_eq!(assembler.push(0xD83D), None);
        assert_eq!(assembler.push(0xDE00), Some('😀'));
        assert_eq!(assembler, CharAssembler::empty());
    }

    #[test]
    fn stray_surrogates_are_dropped() {
        let mut assembler = CharAssembler::empty();
        assert_eq!(assembler.push(0xDE00), None);
        assert_eq!(assembler.push(0xD83D), None);
        assert_eq!(assembler.push(0xD83D), None);
        assert_eq!(assembler.push(0x41), Some('A'));
    }
}
