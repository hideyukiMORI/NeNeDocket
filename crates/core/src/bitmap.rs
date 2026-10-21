//! 画素の列（FR-022）。幅・高さ・RGBA 8 ビット（行の順・上から）。media が作り、UI が cairo / Direct2D に渡す。

use alloc::vec::Vec;

/// 幅 × 高さ × 4 バイトの画素。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitmap {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

impl Bitmap {
    /// 値から。長さが幅 × 高さ × 4 でなければ `None`。
    #[must_use]
    pub fn new(width: u32, height: u32, rgba: Vec<u8>) -> Option<Self> {
        let expected = usize::try_from(
            u64::from(width)
                .checked_mul(u64::from(height))?
                .checked_mul(4)?,
        )
        .ok()?;
        (rgba.len() == expected && width > 0 && height > 0).then_some(Self {
            width,
            height,
            rgba,
        })
    }

    /// 幅（画素）。
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// 高さ（画素）。
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// RGBA の列。
    #[must_use]
    pub fn rgba(&self) -> &[u8] {
        &self.rgba
    }

    /// 1 画素の `[r, g, b, a]`。範囲外は `None`。
    #[must_use]
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let offset = usize::try_from(u64::from(y) * u64::from(self.width) + u64::from(x)).ok()? * 4;
        let slice = self.rgba.get(offset..offset.checked_add(4)?)?;
        <[u8; 4]>::try_from(slice).ok()
    }

    /// cairo の `ARgb32` と Direct2D の `B8G8R8A8 premultiplied` が読む並び: 画素ごとに `[b, g, r, a]`、色は α を掛けたもの。
    #[must_use]
    pub fn premultiplied_bgra(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.rgba.len());
        for pixel in self.rgba.as_chunks::<4>().0 {
            let [red, green, blue, alpha] = *pixel;
            let scale = |channel: u8| {
                u8::try_from((u16::from(channel) * u16::from(alpha) + 127) / 255).unwrap_or(u8::MAX)
            };
            out.extend_from_slice(&[scale(blue), scale(green), scale(red), alpha]);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_length_must_match_and_pixels_are_read_in_row_order() {
        assert!(Bitmap::new(2, 1, alloc::vec![0_u8; 4]).is_none());
        assert!(Bitmap::new(0, 1, Vec::new()).is_none());
        let bitmap = Bitmap::new(
            2,
            2,
            alloc::vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        )
        .unwrap();
        assert_eq!(bitmap.pixel(1, 0), Some([5, 6, 7, 8]));
        assert_eq!(bitmap.pixel(0, 1), Some([9, 10, 11, 12]));
        assert_eq!(bitmap.pixel(2, 0), None);
        assert_eq!(
            (bitmap.width(), bitmap.height(), bitmap.rgba().len()),
            (2, 2, 16)
        );
    }

    #[test]
    fn premultiplied_bgra_swaps_channels_and_scales_by_alpha() {
        let bitmap = Bitmap::new(2, 1, alloc::vec![255, 0, 0, 255, 200, 100, 50, 128]).unwrap();
        assert_eq!(
            bitmap.premultiplied_bgra(),
            [0, 0, 255, 255, 25, 50, 100, 128]
        );
    }
}
