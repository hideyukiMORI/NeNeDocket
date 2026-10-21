//! 1 回の待ちで届いたイベントを、フレームの入力に集める。

use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pointer_input::PointerInput;

use crate::pending_frame::PendingFrame;
use crate::pointer_tracker::PointerTracker;

/// ポインタの追跡（フレームをまたぐ）と、このまとまりの鍵と、押下・離しのように 1 つずつフレームが要る入力と、読み直しの合図。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Gathering {
    pointer: PointerTracker,
    keys: Vec<KeyIntent>,
    frames: Vec<PointerInput>,
    refresh: bool,
    drop: Option<(PixelPoint, Vec<String>)>,
}

impl Gathering {
    /// 何も届いていない。
    pub(crate) const fn empty() -> Self {
        Self {
            pointer: PointerTracker::outside(),
            keys: Vec::new(),
            frames: Vec::new(),
            refresh: false,
            drop: None,
        }
    }

    /// ポインタの追跡。
    pub(crate) const fn pointer_mut(&mut self) -> &mut PointerTracker {
        &mut self.pointer
    }

    /// 押下か離し。それぞれ 1 フレームに出す。
    pub(crate) fn push_frame(&mut self, input: PointerInput) {
        self.frames.push(input);
    }

    /// 鍵。
    pub(crate) fn push_key(&mut self, key: Option<KeyIntent>) {
        self.keys.extend(key);
    }

    /// 落とされたファイル。同じまとまりに 2 度来れば後の分を足す。
    pub(crate) fn push_drop(&mut self, point: PixelPoint, paths: Vec<String>) {
        match &mut self.drop {
            Some((_, known)) => known.extend(paths),
            None => self.drop = Some((point, paths)),
        }
    }

    /// 読み直しの合図。何度届いても次のまとまりで 1 度だけ読み直す。
    pub(crate) const fn push_refresh(&mut self) {
        self.refresh = true;
    }

    /// 描くべきフレームの入力を、届いた順に取り出す。鍵と読み直しは最初のフレームに乗せ、最後に今の位置のフレームを足す。
    pub(crate) fn take_frames(&mut self) -> Vec<PendingFrame> {
        let mut frames: Vec<PendingFrame> = self
            .frames
            .drain(..)
            .map(PendingFrame::of_pointer)
            .collect();
        frames.push(PendingFrame::of_pointer(self.pointer.current()));
        if let Some(first) = frames.first_mut() {
            *first = PendingFrame::new(
                first.pointer(),
                core::mem::take(&mut self.keys),
                core::mem::take(&mut self.refresh),
            )
            .with_drop(self.drop.take());
        }
        frames
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nene_docket_core::button_phase::ButtonPhase;

    #[test]
    fn keys_ride_the_first_frame_and_the_last_frame_is_the_current_pointer() {
        let mut gathering = Gathering::empty();
        let pressed = gathering.pointer_mut().pressed(1, 5, 5);
        gathering.push_frame(pressed);
        gathering.push_key(Some(KeyIntent::Tab));
        gathering.push_key(None);
        gathering.push_refresh();
        let frames = gathering.take_frames();
        assert_eq!(frames.len(), 2);
        assert_eq!(
            frames
                .first()
                .map(|f| (f.pointer().phase(), f.keys().len(), f.refresh())),
            Some((ButtonPhase::Pressed, 1, true))
        );
        assert_eq!(
            frames.get(1).map(|f| (f.pointer().phase(), f.refresh())),
            Some((ButtonPhase::Held, false))
        );
        let rest = gathering.take_frames();
        assert_eq!(rest.len(), 1);
        assert_eq!(rest.first().map(PendingFrame::refresh), Some(false));
    }
}
