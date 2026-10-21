//! 台帳の窓（Windows）。窓を作り、メッセージを状態へ渡し、閉じられるまで回す（ADR 0009）。
//!
//! 窓の手続きは `GWLP_USERDATA` に結んだ `RefCell<WindowsApp>` を借りる。借りている最中に同じスレッドへ
//! 同期的に届いたメッセージは、借りられないので既定の手続きへ流れる。
//! 読み直しの刻みは `SetTimer` / `WM_TIMER`（X11 版はアダプタのスレッドが合図を送る。ADR 0008 / 0009）。

use core::cell::RefCell;
use core::ffi::c_void;
use core::ptr;

use nene_docket_application::docket_screen::DocketScreen;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::{BeginPaint, EndPaint, PAINTSTRUCT, ScreenToClient};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, ReleaseCapture, SetCapture, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent,
    VK_CONTROL, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, GWLP_USERDATA,
    GetMessageW, GetWindowLongPtrW, IDC_ARROW, KillTimer, LoadCursorW, MSG, PostQuitMessage,
    RegisterClassExW, SW_SHOW, SetTimer, SetWindowLongPtrW, ShowWindow, TranslateMessage,
    WINDOW_EX_STYLE, WM_CHAR, WM_DESTROY, WM_ERASEBKGND, WM_KEYDOWN, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_NCCREATE, WM_PAINT, WM_SIZE, WM_TIMER, WNDCLASSEXW,
    WS_OVERLAPPEDWINDOW,
};
use windows::core::{PCWSTR, w};

use crate::key_input::{self, Modifiers};
use crate::win32_failure::Win32Failure;
use crate::windows_app::WindowsApp;

/// 窓の類の名と題。
const CLASS: PCWSTR = w!("NeNeDocket");
const TITLE: PCWSTR = w!("NeNe Docket");
/// 最初の窓の大きさ（X11 版と同じ）。
const INITIAL_WIDTH: i32 = 960;
const INITIAL_HEIGHT: i32 = 600;
/// `WM_MOUSELEAVE`。windows クレートでは Controls の機能にあるので、値だけをここに置く（winuser.h の 0x02A3）。
const WM_MOUSELEAVE: u32 = 0x02A3;
/// 読み直しのタイマーの番号。
const REFRESH_TIMER: usize = 1;
/// 1 秒のミリ秒。
const MILLIS_PER_SECOND: u64 = 1_000;

/// 窓の手続きが借りる状態。
type AppCell = RefCell<WindowsApp>;

/// 窓を作る前に、プロセスを per-monitor v2 の dpi 対応にする。Windows に拡大させず、物理画素で描く（FR-016）。
/// デスクトップの dpi を読む前に呼ぶ。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の SetProcessDpiAwarenessContext を呼ぶ（ADR 0002 の 4）"
)]
pub fn declare_dpi_awareness() {
    // SAFETY: 値だけを渡す。既に決まっていれば失敗するが、そのときは既定の対応のまま続ける。
    let _declared: Result<(), _> =
        unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
}

/// 窓を作り、閉じられるまでメッセージを回す。`refresh_seconds` ごとに他人のジャーナルを読み直す。
///
/// # Errors
///
/// 窓を作れない。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の窓の作成・タイマー・メッセージループ（ADR 0002 の 4）"
)]
pub fn run(screen: DocketScreen, refresh_seconds: u64) -> Result<(), Win32Failure> {
    register_class()?;
    let cell: AppCell = RefCell::new(WindowsApp::new(screen, HWND(ptr::null_mut())));
    let created = create(&raw const cell)?;
    cell.borrow_mut().attach(created);
    let interval = u32::try_from(refresh_seconds.saturating_mul(MILLIS_PER_SECOND))
        .map_err(|_error| Win32Failure::GeometryOutOfRange)?;
    // SAFETY: 自分の窓にタイマーを結ぶ。手続きは無く、WM_TIMER で届く。
    let _timer = unsafe { SetTimer(Some(created), REFRESH_TIMER, interval, None) };
    // SAFETY: 作った窓を見せる。借りていないので、入れ子のメッセージも状態に届く。
    let _was_visible = unsafe { ShowWindow(created, SW_SHOW) };
    let mut message = MSG::default();
    // SAFETY: このスレッドのメッセージを、WM_QUIT（WM_DESTROY で積む）まで取り出して配る。
    // cell はこの関数の終わりまで生き、窓はループが終わる前に壊れるので、ポインタは生存中だけ辿られる。
    while unsafe { GetMessageW(&raw mut message, None, 0, 0) }.as_bool() {
        // SAFETY: 取り出したばかりのメッセージを訳して配る。
        unsafe {
            let _translated = TranslateMessage(&raw const message);
            DispatchMessageW(&raw const message);
        }
    }
    Ok(())
}

/// 窓の類を登録する。2 度目の登録の失敗は無視する（同じ類）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の RegisterClassExW / LoadCursorW を呼ぶ（ADR 0002 の 4）"
)]
fn register_class() -> Result<(), Win32Failure> {
    // SAFETY: 自分のモジュールのハンドルと、系統の共有の矢印を読むだけ。
    let (instance, arrow) = unsafe { (GetModuleHandleW(None)?, LoadCursorW(None, IDC_ARROW)?) };
    let size = u32::try_from(size_of::<WNDCLASSEXW>())
        .map_err(|_error| Win32Failure::GeometryOutOfRange)?;
    let class = WNDCLASSEXW {
        cbSize: size,
        lpfnWndProc: Some(procedure),
        hInstance: instance.into(),
        lpszClassName: CLASS,
        hCursor: arrow,
        ..WNDCLASSEXW::default()
    };
    // SAFETY: class はこの関数の局所で、登録の間だけ読まれる。
    let _registered: u16 = unsafe { RegisterClassExW(&raw const class) };
    Ok(())
}

/// 通常の窓を作る。作成の引数に状態のポインタを渡し、`WM_NCCREATE` で窓に結ぶ。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の CreateWindowExW を呼ぶ（ADR 0002 の 4）"
)]
fn create(cell: *const AppCell) -> Result<HWND, Win32Failure> {
    // SAFETY: 自分のモジュールのハンドルを読むだけ。
    let instance = unsafe { GetModuleHandleW(None) }?;
    // SAFETY: cell は run の局所の状態で、窓より長く生きる。
    let created = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS,
            TITLE,
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            INITIAL_WIDTH,
            INITIAL_HEIGHT,
            None,
            None,
            Some(instance.into()),
            Some(cell.cast::<c_void>()),
        )
    }?;
    Ok(created)
}

/// 窓の手続き。状態を借りられれば処理し、借りられない（入れ子）か扱わない種類なら既定の手続きへ渡す。
#[expect(
    unsafe_code,
    reason = "RS-010: 窓の手続きは Win32 から呼ばれ、窓に結んだポインタを辿る（ADR 0002 の 4）"
)]
unsafe extern "system" fn procedure(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        // SAFETY: WM_NCCREATE の lparam は CREATESTRUCTW を指し、lpCreateParams は run が渡した状態。
        unsafe {
            let create = ptr::with_exposed_provenance::<CREATESTRUCTW>(lparam.0.cast_unsigned());
            if let Some(create) = create.as_ref() {
                let address = create.lpCreateParams.expose_provenance().cast_signed();
                let _previous = SetWindowLongPtrW(window, GWLP_USERDATA, address);
            }
        }
    }
    // SAFETY: GWLP_USERDATA は WM_NCCREATE で結んだ状態か 0。状態は run の終わりまで生き、
    // 窓はそれより先に壊れる。共有の参照だけを作り、書き換えは RefCell の借用を通す。
    let cell = unsafe {
        ptr::with_exposed_provenance::<AppCell>(
            GetWindowLongPtrW(window, GWLP_USERDATA).cast_unsigned(),
        )
        .as_ref()
    };
    let handled = cell.and_then(|cell| {
        let mut app = cell.try_borrow_mut().ok()?;
        dispatch(&mut app, window, (message, wparam, lparam))
    });
    match handled {
        Some(result) => result,
        // SAFETY: 既定の手続きにそのまま渡す。借用は上で手放してある。
        None => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

/// メッセージを状態へ。`None` は既定の手続きへ渡す印。
fn dispatch(
    app: &mut WindowsApp,
    window: HWND,
    (message, wparam, lparam): (u32, WPARAM, LPARAM),
) -> Option<LRESULT> {
    match message {
        WM_SIZE => {
            let (width, height) = packed_words(lparam);
            app.resized(width, height);
            Some(LRESULT(0))
        }
        WM_PAINT => {
            paint(app, window);
            Some(LRESULT(0))
        }
        // 全部を自分で描くので、背景は消さない（ちらつきを防ぐ）。
        WM_ERASEBKGND => Some(LRESULT(1)),
        WM_LBUTTONDOWN | WM_LBUTTONUP | WM_MOUSEMOVE | WM_MOUSELEAVE | WM_MOUSEWHEEL => {
            pointer_message(app, window, (message, wparam, lparam))
        }
        WM_KEYDOWN => {
            let key = u16::try_from(wparam.0).ok()?;
            app.key(key_input::intent_of_key(key, modifiers()));
            Some(LRESULT(0))
        }
        WM_CHAR => {
            let unit = u16::try_from(wparam.0).ok()?;
            if let Some(character) = app.char_unit(unit) {
                app.key(key_input::intent_of_char(character));
            }
            Some(LRESULT(0))
        }
        WM_TIMER if wparam.0 == REFRESH_TIMER => {
            app.refresh();
            Some(LRESULT(0))
        }
        WM_DESTROY => {
            stop_timer(window);
            post_quit();
            Some(LRESULT(0))
        }
        _other => None,
    }
}

/// ポインタのメッセージ。押下と離しは取り込みを伴う。
fn pointer_message(
    app: &mut WindowsApp,
    window: HWND,
    (message, wparam, lparam): (u32, WPARAM, LPARAM),
) -> Option<LRESULT> {
    match message {
        WM_LBUTTONDOWN => {
            capture(window);
            let (x, y) = signed_words(lparam);
            let input = app.pointer_mut().pressed(x, y);
            app.pointer_input(input);
            Some(LRESULT(0))
        }
        WM_LBUTTONUP => {
            release_capture();
            let (x, y) = signed_words(lparam);
            let input = app.pointer_mut().released(x, y);
            app.pointer_input(input);
            Some(LRESULT(0))
        }
        WM_MOUSEMOVE => {
            track_leave(window);
            let (x, y) = signed_words(lparam);
            let input = app.pointer_mut().moved(x, y);
            app.pointer_input(input);
            Some(LRESULT(0))
        }
        WM_MOUSELEAVE => {
            let input = app.pointer_mut().left();
            app.pointer_input(input);
            Some(LRESULT(0))
        }
        WM_MOUSEWHEEL => {
            let delta = wheel_delta(wparam);
            let client = client_point(window, lparam)?;
            let input = app.pointer_mut().wheeled(client, delta);
            app.pointer_input(input);
            Some(LRESULT(0))
        }
        _other => None,
    }
}

/// `WM_PAINT`: 描き直しの要求を受け取り（`BeginPaint` / `EndPaint`）、最後のフレームを Direct2D で描く。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の BeginPaint / EndPaint を呼ぶ（ADR 0002 の 4）"
)]
fn paint(app: &mut WindowsApp, window: HWND) {
    let mut info = PAINTSTRUCT::default();
    // SAFETY: info はこの関数の局所。BeginPaint と EndPaint は対になる。描くのは Direct2D なので DC は使わない。
    unsafe {
        let _dc = BeginPaint(window, &raw mut info);
        app.paint();
        let _ended = EndPaint(window, &raw const info);
    }
}

/// いま押されている修飾鍵。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の GetKeyState を呼ぶ（ADR 0002 の 4）"
)]
fn modifiers() -> Modifiers {
    // SAFETY: このスレッドの鍵の状態を読むだけ。負なら押されている。
    let (shift, control) = unsafe {
        (
            GetKeyState(i32::from(VK_SHIFT.0)) < 0,
            GetKeyState(i32::from(VK_CONTROL.0)) < 0,
        )
    };
    Modifiers { shift, control }
}

/// ポインタが窓から出たら `WM_MOUSELEAVE` を受け取る。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の TrackMouseEvent を呼ぶ（ADR 0002 の 4）"
)]
fn track_leave(window: HWND) {
    let Ok(size) = u32::try_from(size_of::<TRACKMOUSEEVENT>()) else {
        return;
    };
    let mut request = TRACKMOUSEEVENT {
        cbSize: size,
        dwFlags: TME_LEAVE,
        hwndTrack: window,
        dwHoverTime: 0,
    };
    // SAFETY: request はこの関数の局所で、cbSize を正しく入れてある。
    let _tracked: Result<(), _> = unsafe { TrackMouseEvent(&raw mut request) };
}

/// 押したまま窓の外へ引いても、ポインタの動きと離しを受け取る（スクロールバーの掴み）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の SetCapture を呼ぶ（ADR 0002 の 4）"
)]
fn capture(window: HWND) {
    // SAFETY: 自分の窓にポインタを取り込む。
    let _previous = unsafe { SetCapture(window) };
}

/// 取り込みを外す。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の ReleaseCapture を呼ぶ（ADR 0002 の 4）"
)]
fn release_capture() {
    // SAFETY: このスレッドの取り込みを外すだけ。
    let _released: Result<(), _> = unsafe { ReleaseCapture() };
}

/// 読み直しのタイマーを止める。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の KillTimer を呼ぶ（ADR 0002 の 4）"
)]
fn stop_timer(window: HWND) {
    // SAFETY: 自分の窓のタイマーを番号で止めるだけ。
    let _stopped: Result<(), _> = unsafe { KillTimer(Some(window), REFRESH_TIMER) };
}

/// このスレッドのループに `WM_QUIT` を積む。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の PostQuitMessage を呼ぶ（ADR 0002 の 4）"
)]
fn post_quit() {
    // SAFETY: 値だけを渡す。
    unsafe {
        PostQuitMessage(0);
    }
}

/// 画面の座標（lparam）を窓の中の座標へ（`WM_MOUSEWHEEL` は画面の座標で届く）。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の ScreenToClient を呼ぶ（ADR 0002 の 4）"
)]
fn client_point(window: HWND, lparam: LPARAM) -> Option<(i32, i32)> {
    let (x, y) = signed_words(lparam);
    let mut point = POINT { x, y };
    // SAFETY: point はこの関数の局所。
    unsafe { ScreenToClient(window, &raw mut point) }
        .as_bool()
        .then_some((point.x, point.y))
}

/// `WM_MOUSEWHEEL` の wparam の上位語（符号付きの刻み）。
fn wheel_delta(wparam: WPARAM) -> i16 {
    let bits = wparam.0;
    u16::try_from((bits >> 16_u32) & 0xFFFF).map_or(0, u16::cast_signed)
}

/// lparam の下位語と上位語（符号なし）。
fn packed_words(lparam: LPARAM) -> (u16, u16) {
    let bits = lparam.0.cast_unsigned();
    let low = u16::try_from(bits & 0xFFFF).unwrap_or(0);
    let high = u16::try_from((bits >> 16_u32) & 0xFFFF).unwrap_or(0);
    (low, high)
}

/// lparam の下位語と上位語（符号付き・座標）。窓の外は負になりうる。
fn signed_words(lparam: LPARAM) -> (i32, i32) {
    let (low, high) = packed_words(lparam);
    (i32::from(low.cast_signed()), i32::from(high.cast_signed()))
}
