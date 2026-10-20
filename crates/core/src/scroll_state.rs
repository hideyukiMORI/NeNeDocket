//! 縦スクロールの状態。

use crate::pixel_rect::PixelRect;

/// 中身の高さ・見えている高さ・いまの位置（物理画素）。位置は常に `0..=max_offset` に収まる。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollState {
    content: u32,
    viewport: u32,
    offset: u32,
}

impl ScrollState {
    /// つまみの最小の長さ。
    const MIN_THUMB_PX: u32 = 16;

    /// 中身と見えている高さから、先頭で始める。
    #[must_use]
    pub const fn top(content: u32, viewport: u32) -> Self {
        Self {
            content,
            viewport,
            offset: 0,
        }
    }

    /// いまの位置。
    #[must_use]
    pub const fn offset(self) -> u32 {
        self.offset
    }

    /// 中身の高さ。
    #[must_use]
    pub const fn content(self) -> u32 {
        self.content
    }

    /// 見えている高さ。
    #[must_use]
    pub const fn viewport(self) -> u32 {
        self.viewport
    }

    /// いちばん下まで送った位置。中身が収まっていれば 0。
    #[must_use]
    pub const fn max_offset(self) -> u32 {
        self.content.saturating_sub(self.viewport)
    }

    /// 中身と見えている高さを変え、位置を収まる範囲に寄せた状態。
    #[must_use]
    pub const fn resized(self, content: u32, viewport: u32) -> Self {
        Self {
            content,
            viewport,
            offset: 0,
        }
        .scrolled_to(self.offset)
    }

    /// `offset` へ送った状態。範囲を超えれば端で止まる。
    #[must_use]
    pub const fn scrolled_to(self, offset: u32) -> Self {
        let max = self.max_offset();
        Self {
            offset: if offset > max { max } else { offset },
            ..self
        }
    }

    /// `delta` だけ送った状態（下が正）。
    #[must_use]
    pub const fn scrolled_by(self, delta: i32) -> Self {
        let offset = if delta < 0_i32 {
            self.offset.saturating_sub(delta.unsigned_abs())
        } else {
            self.offset.saturating_add(delta.unsigned_abs())
        };
        self.scrolled_to(offset)
    }

    /// 縦の軌道 `track` の中のつまみの矩形。中身が収まっていれば軌道いっぱい。
    #[must_use]
    pub fn thumb(self, track: PixelRect) -> PixelRect {
        let track_height = u32::from(track.height());
        if self.content <= self.viewport || self.content == 0 {
            return track;
        }
        let proportional =
            u64::from(track_height) * u64::from(self.viewport) / u64::from(self.content);
        let length = proportional
            .max(u64::from(Self::MIN_THUMB_PX))
            .min(u64::from(track_height));
        let travel = u64::from(track_height).saturating_sub(length);
        let top = travel * u64::from(self.offset) / u64::from(self.max_offset().max(1));
        let y = u16::try_from(u64::from(track.y()).saturating_add(top)).unwrap_or(u16::MAX);
        let height = u16::try_from(length).unwrap_or(u16::MAX);
        PixelRect::new(track.x(), y, track.width(), height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrolling_stays_within_the_content() {
        let state = ScrollState::top(1000, 300);
        assert_eq!(state.max_offset(), 700);
        assert_eq!(state.scrolled_by(50).offset(), 50);
        assert_eq!(state.scrolled_by(-50).offset(), 0);
        assert_eq!(state.scrolled_by(5000).offset(), 700);
        assert_eq!(state.scrolled_to(9_999).offset(), 700);
        assert_eq!(ScrollState::top(100, 300).scrolled_by(10).offset(), 0);
        assert_eq!((state.content(), state.viewport()), (1000, 300));
    }

    #[test]
    fn resizing_keeps_the_offset_in_range() {
        let state = ScrollState::top(1000, 300).scrolled_to(700);
        assert_eq!(state.resized(500, 300).offset(), 200);
        assert_eq!(state.resized(1000, 1000).offset(), 0);
    }

    #[test]
    fn thumb_is_proportional_and_moves_with_the_offset() {
        let track = PixelRect::new(100, 0, 8, 300);
        let state = ScrollState::top(1200, 300);
        assert_eq!(state.thumb(track), PixelRect::new(100, 0, 8, 75));
        assert_eq!(
            state.scrolled_to(900).thumb(track),
            PixelRect::new(100, 225, 8, 75)
        );
        assert_eq!(
            state.scrolled_to(450).thumb(track),
            PixelRect::new(100, 112, 8, 75)
        );
    }

    #[test]
    fn thumb_fills_the_track_when_nothing_scrolls_and_never_shrinks_below_the_minimum() {
        let track = PixelRect::new(0, 0, 8, 300);
        assert_eq!(ScrollState::top(200, 300).thumb(track), track);
        assert_eq!(ScrollState::top(0, 300).thumb(track), track);
        assert_eq!(ScrollState::top(1_000_000, 300).thumb(track).height(), 16);
    }
}
