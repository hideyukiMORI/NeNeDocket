//! 画像を矩形に収める計算（FR-022）。倍率は「収まる大きさ」を 100 とした百分率、`pan` は中央からのずれ（画素）。
//! 返すのは描く先（矩形の中に切った分）と、画素の列のどの部分か。純関数。

use crate::pixel_rect::PixelRect;

/// 倍率の下限と上限（%）。
pub const MIN_ZOOM: u32 = 10;
/// 倍率の上限（%）。
pub const MAX_ZOOM: u32 = 1600;

/// `area` に `width` × `height` の画像を置く。`None` なら何も描かない（画像が無い・全部はみ出した）。
#[must_use]
pub fn place(
    area: PixelRect,
    (width, height): (u32, u32),
    zoom_percent: u32,
    (pan_x, pan_y): (i32, i32),
) -> Option<(PixelRect, PixelRect)> {
    if area.is_empty() || width == 0 || height == 0 {
        return None;
    }
    let fit = (u64::from(area.width()) * 1000 / u64::from(width))
        .min(u64::from(area.height()) * 1000 / u64::from(height))
        .max(1);
    let scale = (fit * u64::from(zoom_percent.clamp(MIN_ZOOM, MAX_ZOOM)) / 100).max(1);
    let dest_width = i64::try_from((u64::from(width) * scale / 1000).max(1)).ok()?;
    let dest_height = i64::try_from((u64::from(height) * scale / 1000).max(1)).ok()?;
    let origin_x =
        i64::from(area.x()) + (i64::from(area.width()) - dest_width) / 2 + i64::from(pan_x);
    let origin_y =
        i64::from(area.y()) + (i64::from(area.height()) - dest_height) / 2 + i64::from(pan_y);
    let left = origin_x.max(i64::from(area.x()));
    let top = origin_y.max(i64::from(area.y()));
    let right = (origin_x + dest_width).min(i64::from(area.x()) + i64::from(area.width()));
    let bottom = (origin_y + dest_height).min(i64::from(area.y()) + i64::from(area.height()));
    if right <= left || bottom <= top {
        return None;
    }
    let dest = PixelRect::new(
        u16::try_from(left).ok()?,
        u16::try_from(top).ok()?,
        u16::try_from(right - left).ok()?,
        u16::try_from(bottom - top).ok()?,
    );
    let source_x = (left - origin_x) * i64::from(width) / dest_width;
    let source_y = (top - origin_y) * i64::from(height) / dest_height;
    let source_width = ((right - left) * i64::from(width) / dest_width).max(1);
    let source_height = ((bottom - top) * i64::from(height) / dest_height).max(1);
    let source = PixelRect::new(
        u16::try_from(source_x.clamp(0, i64::from(width) - 1)).ok()?,
        u16::try_from(source_y.clamp(0, i64::from(height) - 1)).ok()?,
        u16::try_from(source_width.min(i64::from(width))).ok()?,
        u16::try_from(source_height.min(i64::from(height))).ok()?,
    );
    Some((dest, source))
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: PixelRect = PixelRect::new(10, 20, 400, 200);

    #[test]
    fn a_fitting_image_is_centred_and_shows_all_of_itself() {
        let (dest, source) = place(AREA, (800_u32, 200_u32), 100_u32, (0_i32, 0_i32)).unwrap();
        assert_eq!(dest, PixelRect::new(10, 70, 400, 100));
        assert_eq!(source, PixelRect::new(0, 0, 800, 200));
        let (tall, _) = place(AREA, (10_u32, 1000_u32), 100_u32, (0_i32, 0_i32)).unwrap();
        assert_eq!(tall, PixelRect::new(209, 20, 2, 200));
    }

    #[test]
    fn zoom_and_pan_cut_the_visible_part_and_the_source_follows() {
        let (dest, source) = place(AREA, (800_u32, 200_u32), 200_u32, (0_i32, 0_i32)).unwrap();
        assert_eq!(dest, AREA);
        assert_eq!(source, PixelRect::new(200, 0, 400, 200));
        let (panned, moved) = place(AREA, (800_u32, 200_u32), 200_u32, (100_i32, 0_i32)).unwrap();
        assert_eq!(panned, AREA);
        assert_eq!(moved, PixelRect::new(100, 0, 400, 200));
        assert!(place(AREA, (800_u32, 200_u32), 100_u32, (500_i32, 0_i32)).is_none());
        assert!(place(AREA, (0_u32, 10_u32), 100_u32, (0_i32, 0_i32)).is_none());
        assert!(
            place(
                PixelRect::new(0, 0, 0, 0),
                (8_u32, 8_u32),
                100_u32,
                (0_i32, 0_i32)
            )
            .is_none()
        );
        // 倍率は 10〜1600 に収める。
        let (tiny, _) = place(AREA, (800_u32, 200_u32), 1_u32, (0_i32, 0_i32)).unwrap();
        assert_eq!(tiny.width(), 40);
    }
}
