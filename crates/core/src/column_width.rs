//! 段の中の列の幅。

use alloc::vec::Vec;

use crate::pixel_rect::PixelRect;

/// 列の幅の決め方。閉じた集合（RS-002）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnWidth {
    /// 物理画素で固定。
    Fixed(u16),
    /// 残りを `Fill` の列で等分する。
    Fill,
}

/// `rect` を `widths` の列に左から割る。列と列の間は `gap`。`Fill` は固定分と隙間を引いた残りを等分する。
/// 残りが無ければ `Fill` の幅は 0。右端に余りが出れば最後の `Fill` に足す。
#[must_use]
pub fn split(rect: PixelRect, widths: &[ColumnWidth], gap: u16) -> Vec<PixelRect> {
    let fixed: u16 = widths
        .iter()
        .map(|width| match width {
            ColumnWidth::Fixed(px) => *px,
            ColumnWidth::Fill => 0,
        })
        .fold(0, u16::saturating_add);
    let fills = widths
        .iter()
        .filter(|width| matches!(width, ColumnWidth::Fill))
        .count();
    let gaps =
        gap.saturating_mul(u16::try_from(widths.len().saturating_sub(1)).unwrap_or(u16::MAX));
    let remaining = rect.width().saturating_sub(fixed).saturating_sub(gaps);
    let fill_count = u16::try_from(fills).unwrap_or(u16::MAX).max(1);
    let fill_width = remaining / fill_count;
    let leftover = remaining - fill_width * u16::try_from(fills).unwrap_or(0);
    let last_fill = widths
        .iter()
        .rposition(|width| matches!(width, ColumnWidth::Fill));
    let mut x = rect.x();
    widths
        .iter()
        .enumerate()
        .map(|(index, width)| {
            let px = match width {
                ColumnWidth::Fixed(px) => *px,
                ColumnWidth::Fill if last_fill == Some(index) => {
                    fill_width.saturating_add(leftover)
                }
                ColumnWidth::Fill => fill_width,
            };
            let column = PixelRect::new(x, rect.y(), px, rect.height());
            x = x.saturating_add(px).saturating_add(gap);
            column
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_and_fill_columns_share_the_width() {
        let rect = PixelRect::new(10, 5, 100, 20);
        let columns = split(
            rect,
            &[ColumnWidth::Fixed(20), ColumnWidth::Fill, ColumnWidth::Fill],
            4,
        );
        assert_eq!(
            columns,
            alloc::vec![
                PixelRect::new(10, 5, 20, 20),
                PixelRect::new(34, 5, 36, 20),
                PixelRect::new(74, 5, 36, 20),
            ]
        );
    }

    #[test]
    fn leftover_goes_to_the_last_fill_and_overflow_gives_zero_width() {
        let rect = PixelRect::new(0, 0, 101, 10);
        let columns = split(rect, &[ColumnWidth::Fill, ColumnWidth::Fill], 0);
        assert_eq!(columns.iter().map(|c| c.width()).sum::<u16>(), 101);
        assert_eq!(columns.get(1).map(|c| c.width()), Some(51));
        let cramped = split(
            PixelRect::new(0, 0, 10, 10),
            &[ColumnWidth::Fixed(30), ColumnWidth::Fill],
            2,
        );
        assert_eq!(cramped.get(1).map(|c| c.width()), Some(0));
        assert!(split(rect, &[], 0).is_empty());
    }
}
