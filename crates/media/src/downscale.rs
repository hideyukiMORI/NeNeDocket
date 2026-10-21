//! 大きな画像を整数の倍率で縮める（箱の平均）。記憶と描画の量を抑える（大容量でも軽い・FR-017）。

use nene_docket_core::bitmap::Bitmap;

/// 長い辺が `max_side` に収まる最小の整数の倍率。収まっていれば 1。
#[must_use]
pub fn factor_for(width: u32, height: u32, max_side: u32) -> u32 {
    let longest = width.max(height);
    longest.div_ceil(max_side.max(1)).max(1)
}

/// `factor` × `factor` の箱の平均で縮める。端の欠けた箱はある分だけで平均する。倍率 1 ならそのまま。
#[must_use]
pub fn shrink(bitmap: &Bitmap, factor: u32) -> Bitmap {
    if factor <= 1 {
        return bitmap.clone();
    }
    let width = bitmap.width().div_ceil(factor);
    let height = bitmap.height().div_ceil(factor);
    let mut rgba = Vec::with_capacity(usize::try_from(width * height * 4).unwrap_or(0));
    for y in 0..height {
        for x in 0..width {
            rgba.extend_from_slice(&average(bitmap, (x * factor, y * factor), factor));
        }
    }
    Bitmap::new(width, height, rgba).unwrap_or_else(|| bitmap.clone())
}

/// 箱の中の画素の平均。
fn average(bitmap: &Bitmap, (left, top): (u32, u32), factor: u32) -> [u8; 4] {
    let mut sum = [0_u64; 4];
    let mut count = 0_u64;
    for y in top..(top.saturating_add(factor)).min(bitmap.height()) {
        for x in left..(left.saturating_add(factor)).min(bitmap.width()) {
            count += add_pixel(bitmap, (x, y), &mut sum);
        }
    }
    let count = count.max(1);
    sum.map(|total| u8::try_from((total + count / 2) / count).unwrap_or(u8::MAX))
}

/// 1 画素を足す。足せたら 1。
fn add_pixel(bitmap: &Bitmap, (x, y): (u32, u32), sum: &mut [u64; 4]) -> u64 {
    let Some(pixel) = bitmap.pixel(x, y) else {
        return 0;
    };
    for (total, channel) in sum.iter_mut().zip(pixel) {
        *total += u64::from(channel);
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_factor_fits_the_longest_side_and_boxes_are_averaged() {
        assert_eq!(factor_for(4000, 3000, 2048), 2);
        assert_eq!(factor_for(2048, 10, 2048), 1);
        assert_eq!(factor_for(2049, 10, 2048), 2);
        assert_eq!(factor_for(10, 9000, 2048), 5);
        let bitmap = Bitmap::new(
            3,
            2,
            vec![
                0, 0, 0, 255, 100, 100, 100, 255, 200, 0, 0, 255, //
                200, 200, 200, 255, 100, 100, 100, 255, 0, 0, 0, 255,
            ],
        )
        .unwrap();
        let small = shrink(&bitmap, 2);
        assert_eq!((small.width(), small.height()), (2, 1));
        assert_eq!(small.pixel(0, 0), Some([100, 100, 100, 255]));
        assert_eq!(small.pixel(1, 0), Some([100, 0, 0, 255]));
        assert_eq!(shrink(&bitmap, 1), bitmap);
    }
}
