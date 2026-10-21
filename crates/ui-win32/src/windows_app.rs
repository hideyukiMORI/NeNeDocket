//! 窓の手続きが借りる状態（ADR 0009）。画面の状態の持ち主は application（ARC-004）で、ここは入力を渡して描くだけ。
//!
//! 入力のメッセージが届くたびに 1 フレーム進め、描き直しを求める（`InvalidateRect`）。描くのは `WM_PAINT`。

use nene_docket_application::docket_frame::DocketFrame;
use nene_docket_application::docket_screen::DocketScreen;
use nene_docket_application::frame_input::FrameInput;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::pasted_text;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::pointer_input::PointerInput;
use nene_docket_core::preedit::Preedit;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::InvalidateRect;

use crate::char_assembler::CharAssembler;
use crate::pen::Pen;
use crate::pointer_state::PointerState;
use crate::win32_failure::Win32Failure;

/// 画面・描く道具（失敗したら作り直す）・ポインタ・サロゲートの待ち・IME の確定前の文字列・窓の大きさ・最後のフレーム。
#[derive(Debug)]
pub(crate) struct WindowsApp {
    screen: DocketScreen,
    window: HWND,
    pen: Option<Pen>,
    pointer: PointerState,
    chars: CharAssembler,
    preedit: Option<Preedit>,
    bounds: PixelRect,
    frame: Option<DocketFrame>,
}

impl WindowsApp {
    /// 画面と窓から。最初の大きさは `WM_SIZE` が届ける。
    pub(crate) const fn new(screen: DocketScreen, window: HWND) -> Self {
        Self {
            screen,
            window,
            pen: None,
            pointer: PointerState::outside(),
            chars: CharAssembler::empty(),
            preedit: None,
            bounds: PixelRect::new(0, 0, 0, 0),
            frame: None,
        }
    }

    /// 作った窓を結ぶ。
    pub(crate) const fn attach(&mut self, window: HWND) {
        self.window = window;
    }

    /// 窓の中の大きさが変わった（`WM_SIZE`）。
    pub(crate) fn resized(&mut self, width: u16, height: u16) {
        self.bounds = PixelRect::new(0, 0, width, height);
        if let Some(pen) = &self.pen
            && pen.resize(self.bounds).is_err()
        {
            self.pen = None;
        }
        self.step(self.pointer.current(), Vec::new(), false);
    }

    /// ポインタの追跡。
    pub(crate) const fn pointer_mut(&mut self) -> &mut PointerState {
        &mut self.pointer
    }

    /// ポインタの入力を 1 フレームにする。
    pub(crate) fn pointer_input(&mut self, input: PointerInput) {
        self.step(input, Vec::new(), false);
    }

    /// 鍵を 1 フレームにする。
    pub(crate) fn key(&mut self, key: Option<KeyIntent>) {
        if let Some(key) = key {
            self.step(self.pointer.current(), vec![key], false);
        }
    }

    /// `WM_CHAR` の UTF-16 の単位。字が完成すれば鍵にする。
    pub(crate) fn char_unit(&mut self, unit: u16) -> Option<char> {
        self.chars.push(unit)
    }

    /// IME の組み立て（FR-015）。確定した文字列があれば 1 字ずつ鍵にし、確定前の文字列を次のフレームから欄に出す（`None` で消す）。
    pub(crate) fn composition(&mut self, result: Option<&str>, preedit: Option<Preedit>) {
        self.preedit = preedit;
        let keys = result.map(pasted_text::key_intents).unwrap_or_default();
        self.step(self.pointer.current(), keys, false);
    }

    /// フォーカスのある入力欄の矩形（最後のフレームでカーソルを描いた文字の命令）。候補の窓を置く位置。
    pub(crate) fn caret_rect(&self) -> Option<PixelRect> {
        self.frame.as_ref()?.commands().iter().find_map(|command| {
            if let DrawCommand::Text {
                rect,
                caret: Some(_),
                ..
            } = command
            {
                Some(*rect)
            } else {
                None
            }
        })
    }

    /// 落とされたファイル（`WM_DROPFILES`・FR-021 の第 2 段）を 1 フレームにする。
    pub(crate) fn dropped(&mut self, point: PixelPoint, paths: Vec<String>) {
        let input = FrameInput::new(self.bounds, self.pointer.current())
            .with_preedit(self.preedit.clone())
            .with_drop(point, paths);
        self.frame = Some(self.screen.step(&input));
        self.invalidate();
    }

    /// 読み直しの合図（`WM_TIMER`）。
    /// 窓を閉じた後。入力中の欄を確定し、スナップショットを書く（ADR 0015）。
    pub(crate) fn close(&mut self) {
        self.screen.close();
    }

    pub(crate) fn refresh(&mut self) {
        self.step(self.pointer.current(), Vec::new(), true);
    }

    /// 1 フレーム進め、描き直しを求める。
    fn step(&mut self, pointer: PointerInput, keys: Vec<KeyIntent>, refresh: bool) {
        let mut input = FrameInput::new(self.bounds, pointer)
            .with_keys(keys)
            .with_preedit(self.preedit.clone());
        if refresh {
            input = input.with_refresh();
        }
        self.frame = Some(self.screen.step(&input));
        self.invalidate();
    }

    /// 最後のフレームを描く（`WM_PAINT`）。道具が無ければ作り、描けなければ捨てて次に任せる（ADR 0009）。
    pub(crate) fn paint(&mut self) {
        let Some(frame) = &self.frame else {
            return;
        };
        if self.bounds.is_empty() {
            return;
        }
        if self.pen.is_none() {
            self.pen = Pen::new(self.window, self.bounds).ok();
        }
        let painted: Result<(), Win32Failure> =
            self.pen.as_ref().map_or(Ok(()), |pen| pen.paint(frame));
        if painted.is_err() {
            self.pen = None;
        }
    }

    /// 窓全体の描き直しを求める。
    #[expect(
        unsafe_code,
        reason = "RS-010: Win32 の InvalidateRect を呼ぶ（ADR 0002 の 4）"
    )]
    fn invalidate(&self) {
        // SAFETY: 自分の窓に、矩形を渡さず（全体）、背景を消さずに描き直しを求めるだけ。
        let _queued = unsafe { InvalidateRect(Some(self.window), None, false) };
    }
}
