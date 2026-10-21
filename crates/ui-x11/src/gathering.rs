//! 1 回の待ちで届いたイベントを、フレームの入力に集める。

use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::pointer_input::PointerInput;

use crate::pointer_tracker::PointerTracker;

/// ポインタの追跡（フレームをまたぐ）と、このまとまりの鍵と、押下・離しのように 1 つずつフレームが要る入力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Gathering {
    pointer: PointerTracker,
    keys: Vec<KeyIntent>,
    frames: Vec<PointerInput>,
}

impl Gathering {
    /// 何も届いていない。
    pub(crate) const fn empty() -> Self {
        Self {
            pointer: PointerTracker::outside(),
            keys: Vec::new(),
            frames: Vec::new(),
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

    /// 描くべきフレームの入力を、届いた順に取り出す。鍵は最初のフレームに乗せ、最後に今の位置のフレームを足す。
    pub(crate) fn take_frames(&mut self) -> Vec<(PointerInput, Vec<KeyIntent>)> {
        let mut frames: Vec<(PointerInput, Vec<KeyIntent>)> = self
            .frames
            .drain(..)
            .map(|input| (input, Vec::new()))
            .collect();
        frames.push((self.pointer.current(), Vec::new()));
        if let Some(first) = frames.first_mut() {
            first.1 = core::mem::take(&mut self.keys);
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
        let frames = gathering.take_frames();
        assert_eq!(frames.len(), 2);
        assert_eq!(
            frames.first().map(|f| (f.0.phase(), f.1.len())),
            Some((ButtonPhase::Pressed, 1))
        );
        assert_eq!(frames.get(1).map(|f| f.0.phase()), Some(ButtonPhase::Held));
        assert_eq!(gathering.take_frames().len(), 1);
    }
}
