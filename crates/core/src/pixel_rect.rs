//! 窓の中の矩形。

use crate::pixel_point::PixelPoint;

/// 窓の左上を原点とする物理画素の矩形。X の `Rectangle` と同じ形で、Direct2D と cairo へも `f32` / `f64` で写せる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
}

impl PixelRect {
    /// 左上と大きさから作る。
    #[must_use]
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// 左端。
    #[must_use]
    pub const fn x(self) -> u16 {
        self.x
    }

    /// 上端。
    #[must_use]
    pub const fn y(self) -> u16 {
        self.y
    }

    /// 幅。
    #[must_use]
    pub const fn width(self) -> u16 {
        self.width
    }

    /// 高さ。
    #[must_use]
    pub const fn height(self) -> u16 {
        self.height
    }

    /// 右端の画素（含む）。幅 0 なら左端と同じ。
    #[must_use]
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width.saturating_sub(1))
    }

    /// 下端の画素（含む）。高さ 0 なら上端と同じ。
    #[must_use]
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height.saturating_sub(1))
    }

    /// 幅か高さが 0 か。
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// 点を含むか。幅か高さが 0 なら何も含まない。
    #[must_use]
    pub const fn contains(self, point: PixelPoint) -> bool {
        !self.is_empty()
            && point.x() >= self.x
            && point.x() <= self.right()
            && point.y() >= self.y
            && point.y() <= self.bottom()
    }

    /// 四辺を `amount` だけ内側に寄せた矩形。寄せきれなければ幅か高さが 0 になる。
    #[must_use]
    pub const fn inset(self, amount: u16) -> Self {
        let shrink = amount.saturating_mul(2);
        Self::new(
            self.x.saturating_add(amount),
            self.y.saturating_add(amount),
            self.width.saturating_sub(shrink),
            self.height.saturating_sub(shrink),
        )
    }

    /// 2 つの矩形の重なり。重ならなければ幅と高さが 0 の矩形（左上は `self` のもの）。
    #[must_use]
    pub const fn intersection(self, other: Self) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::new(self.x, self.y, 0, 0);
        }
        let left = if self.x > other.x { self.x } else { other.x };
        let top = if self.y > other.y { self.y } else { other.y };
        let right = if self.right() < other.right() {
            self.right()
        } else {
            other.right()
        };
        let bottom = if self.bottom() < other.bottom() {
            self.bottom()
        } else {
            other.bottom()
        };
        if right < left || bottom < top {
            return Self::new(self.x, self.y, 0, 0);
        }
        Self::new(left, top, right - left + 1, bottom - top + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contains_its_edges_but_not_beyond() {
        let rect = PixelRect::new(10, 20, 5, 3);
        assert!(rect.contains(PixelPoint::new(10, 20)));
        assert!(rect.contains(PixelPoint::new(14, 22)));
        assert!(!rect.contains(PixelPoint::new(15, 22)));
        assert!(!rect.contains(PixelPoint::new(14, 23)));
        assert!(!rect.contains(PixelPoint::new(9, 20)));
        assert_eq!(
            (rect.x(), rect.y(), rect.width(), rect.height()),
            (10, 20, 5, 3)
        );
    }

    #[test]
    fn empty_rect_contains_nothing() {
        assert!(!PixelRect::new(1, 1, 0, 5).contains(PixelPoint::new(1, 1)));
        assert_eq!(PixelRect::new(1, 1, 0, 5).right(), 1);
        assert_eq!(PixelRect::new(1, 1, 5, 0).bottom(), 1);
    }

    #[test]
    fn inset_shrinks_from_every_side_and_stops_at_zero() {
        assert_eq!(
            PixelRect::new(10, 10, 20, 10).inset(2),
            PixelRect::new(12, 12, 16, 6)
        );
        assert!(PixelRect::new(10, 10, 3, 3).inset(2).is_empty());
    }

    #[test]
    fn intersection_keeps_the_overlap_only() {
        let a = PixelRect::new(0, 0, 10, 10);
        let b = PixelRect::new(5, 5, 10, 10);
        assert_eq!(a.intersection(b), PixelRect::new(5, 5, 5, 5));
        assert_eq!(b.intersection(a), PixelRect::new(5, 5, 5, 5));
        assert!(a.intersection(PixelRect::new(10, 0, 5, 5)).is_empty());
        assert!(a.intersection(PixelRect::new(0, 20, 5, 5)).is_empty());
        assert!(a.intersection(PixelRect::new(2, 2, 0, 5)).is_empty());
    }
}
