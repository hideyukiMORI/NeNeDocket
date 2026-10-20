//! ホバー・押下・クリック・フォーカスの判定。フレームをまたぐ状態の持ち主（ADR 0006）。
//!
//! 使い方は即時モード: `begin` に入力を渡し、部品ごとに `probe` を呼び、最後に `end` を呼ぶ。
//! `probe` の順が部品の順で、Tab の移動はその順に従う。

use alloc::vec::Vec;

use crate::button_phase::ButtonPhase;
use crate::pixel_rect::PixelRect;
use crate::pointer_input::PointerInput;
use crate::response::Response;
use crate::response_phase::ResponsePhase;
use crate::widget_id::WidgetId;

/// 押している部品・フォーカスの持ち主と、このフレームの入力。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interaction {
    pointer: PointerInput,
    held: Option<WidgetId>,
    focus: Option<WidgetId>,
    order: Vec<WidgetId>,
    press_taken: bool,
}

impl Interaction {
    /// 何も押しておらず、フォーカスも無い状態。
    #[must_use]
    pub const fn idle() -> Self {
        Self {
            pointer: PointerInput::absent(),
            held: None,
            focus: None,
            order: Vec::new(),
            press_taken: false,
        }
    }

    /// フレームの始まり。このフレームの入力を受け取り、部品の順を空にする。
    pub fn begin(&mut self, pointer: PointerInput) {
        self.pointer = pointer;
        self.order.clear();
        self.press_taken = false;
    }

    /// 部品 1 つを問い合わせる。`focusable` な部品はフォーカスを取れ、Tab の順に入る。
    pub fn probe(&mut self, id: WidgetId, rect: PixelRect, focusable: bool) -> Response {
        if focusable {
            self.order.push(id);
        }
        let hovered = self
            .pointer
            .position()
            .is_some_and(|point| rect.contains(point));
        let mut clicked = false;
        match self.pointer.phase() {
            ButtonPhase::Pressed if hovered => {
                self.held = Some(id);
                self.press_taken = true;
                self.focus = focusable.then_some(id);
            }
            ButtonPhase::Released if hovered && self.held == Some(id) => clicked = true,
            ButtonPhase::Pressed
            | ButtonPhase::Released
            | ButtonPhase::Held
            | ButtonPhase::Idle => {}
        }
        let phase = if self.held == Some(id) {
            ResponsePhase::Held
        } else if hovered {
            ResponsePhase::Hovered
        } else {
            ResponsePhase::Idle
        };
        Response::new(phase, clicked, self.focus == Some(id))
    }

    /// フレームの終わり。何も無い所で押せばフォーカスが外れ、離せば押している部品が無くなる。
    pub fn end(&mut self) {
        match self.pointer.phase() {
            ButtonPhase::Pressed if !self.press_taken => self.focus = None,
            ButtonPhase::Released => self.held = None,
            ButtonPhase::Pressed | ButtonPhase::Held | ButtonPhase::Idle => {}
        }
    }

    /// Tab。直前のフレームの順で次の部品へ。フォーカスが無ければ最初へ。末尾なら最初へ戻る。
    pub fn focus_next(&mut self) {
        self.focus = match self.focus.and_then(|id| self.index_of(id)) {
            Some(index) => self.order.get(index.saturating_add(1)).copied(),
            None => None,
        }
        .or_else(|| self.order.first().copied());
    }

    /// Shift+Tab。前の部品へ。先頭なら末尾へ戻る。
    pub fn focus_previous(&mut self) {
        self.focus = match self.focus.and_then(|id| self.index_of(id)) {
            Some(index) => index
                .checked_sub(1)
                .and_then(|i| self.order.get(i).copied()),
            None => None,
        }
        .or_else(|| self.order.last().copied());
    }

    /// フォーカスを外す（Esc・モーダルを閉じた）。
    pub fn blur(&mut self) {
        self.focus = None;
    }

    /// フォーカスの持ち主。
    #[must_use]
    pub const fn focus(&self) -> Option<WidgetId> {
        self.focus
    }

    /// 押したまま離していない部品。
    #[must_use]
    pub const fn held(&self) -> Option<WidgetId> {
        self.held
    }

    fn index_of(&self, id: WidgetId) -> Option<usize> {
        self.order.iter().position(|&candidate| candidate == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pixel_point::PixelPoint;

    const A: WidgetId = WidgetId::new(1);
    const B: WidgetId = WidgetId::new(2);
    const RECT_A: PixelRect = PixelRect::new(0, 0, 10, 10);
    const RECT_B: PixelRect = PixelRect::new(20, 0, 10, 10);

    fn at(x: u16, y: u16, phase: ButtonPhase) -> PointerInput {
        PointerInput::new(Some(PixelPoint::new(x, y)), phase, 0)
    }

    fn frame(interaction: &mut Interaction, pointer: PointerInput) -> (Response, Response) {
        interaction.begin(pointer);
        let a = interaction.probe(A, RECT_A, true);
        let b = interaction.probe(B, RECT_B, true);
        interaction.end();
        (a, b)
    }

    #[test]
    fn click_is_press_and_release_on_the_same_widget() {
        let mut interaction = Interaction::idle();
        let (pressed, _) = frame(&mut interaction, at(5, 5, ButtonPhase::Pressed));
        assert_eq!(pressed.phase(), ResponsePhase::Held);
        assert!(!pressed.clicked());
        assert!(pressed.focused());
        let (released, other) = frame(&mut interaction, at(5, 5, ButtonPhase::Released));
        assert!(released.clicked());
        assert!(!other.clicked());
        assert_eq!(interaction.held(), None);
        let (idle, _) = frame(&mut interaction, at(5, 5, ButtonPhase::Idle));
        assert_eq!(idle.phase(), ResponsePhase::Hovered);
        assert!(!idle.clicked());
    }

    #[test]
    fn releasing_elsewhere_is_not_a_click_but_keeps_focus() {
        let mut interaction = Interaction::idle();
        frame(&mut interaction, at(5, 5, ButtonPhase::Pressed));
        let (held, _) = frame(&mut interaction, at(25, 5, ButtonPhase::Held));
        assert_eq!(held.phase(), ResponsePhase::Held);
        let (released, other) = frame(&mut interaction, at(25, 5, ButtonPhase::Released));
        assert!(!released.clicked());
        assert!(!other.clicked());
        assert_eq!(interaction.focus(), Some(A));
    }

    #[test]
    fn pressing_on_nothing_drops_focus() {
        let mut interaction = Interaction::idle();
        frame(&mut interaction, at(5, 5, ButtonPhase::Pressed));
        frame(&mut interaction, at(5, 5, ButtonPhase::Released));
        frame(&mut interaction, at(15, 5, ButtonPhase::Pressed));
        assert_eq!(interaction.focus(), None);
        frame(&mut interaction, PointerInput::absent());
        assert_eq!(interaction.focus(), None);
    }

    #[test]
    fn unfocusable_widgets_take_the_press_without_focus() {
        let mut interaction = Interaction::idle();
        interaction.begin(at(5, 5, ButtonPhase::Pressed));
        let response = interaction.probe(A, RECT_A, false);
        interaction.end();
        assert_eq!(response.phase(), ResponsePhase::Held);
        assert!(!response.focused());
        assert_eq!(interaction.focus(), None);
    }

    #[test]
    fn tab_walks_the_probe_order_and_wraps() {
        let mut interaction = Interaction::idle();
        frame(&mut interaction, PointerInput::absent());
        interaction.focus_next();
        assert_eq!(interaction.focus(), Some(A));
        interaction.focus_next();
        assert_eq!(interaction.focus(), Some(B));
        interaction.focus_next();
        assert_eq!(interaction.focus(), Some(A));
        interaction.focus_previous();
        assert_eq!(interaction.focus(), Some(B));
        interaction.focus_previous();
        assert_eq!(interaction.focus(), Some(A));
        interaction.blur();
        assert_eq!(interaction.focus(), None);
        interaction.focus_previous();
        assert_eq!(interaction.focus(), Some(B));
    }

    #[test]
    fn tab_with_no_widgets_keeps_nothing() {
        let mut interaction = Interaction::idle();
        interaction.begin(PointerInput::absent());
        interaction.end();
        interaction.focus_next();
        assert_eq!(interaction.focus(), None);
    }
}
