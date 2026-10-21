# ADR 0009 — Windows 版は通常の窓に Direct2D で描き、読み直しの刻みは窓のタイマー。アダプタの読み書きは OS ごとに持つ

- 状態: 受理
- 日付: 2026-10-21
- Issue: #22
- 影響する規則: ARC-002 / ARC-006 / ARC-007 / ARC-013 / RS-010 / CNF-012 / CNF-013 / CNF-014 / QLT-013。ADR 0002 の Windows 側を実装し、ADR 0008 の「Windows では刻みの経路が別になる」を解く

## 文脈

ADR 0002 は Windows を Win32 ＋ Direct2D / DirectWrite と決め、`unsafe` を 2 クレートにだけ許した。Issue #11〜#20 で X11 版が台帳を編集できるようになったので、同じ中核（core / application / store）で Windows の窓を出す。

nene-ruler の Windows 版（ADR 0021）は `WS_EX_LAYERED` の透明な窓に DC レンダーターゲットで描き `UpdateLayeredWindow` で出す。定規は透明でなければならないが、課題管理の窓は不透明でよく、大きさも利用者が変える。

X11 版の読み直しの刻みは、アダプタのスレッドが X の窓へ `ClientMessage` を送る（ADR 0008）。Windows には窓に結ぶタイマー（`SetTimer` / `WM_TIMER`）があり、スレッドも 2 本目の接続も要らない。

実測（2026-10-21・このコンテナ）: `windows` 0.62.2 と `windows-numerics` 0.3.1 で `x86_64-pc-windows-gnu` の clippy が 0 件、mingw-w64 で 1.9 MB の単体の exe ができた。**Windows そのものでは動かしていない**（コンテナに Windows が無い）。

## 決定

**Windows 版は通常の窓（`WS_OVERLAPPEDWINDOW`）に Direct2D の `HwndRenderTarget` で core の描画命令を描く。読み直しの刻みは `SetTimer` / `WM_TIMER`。アカウントと設定の読み書きは adapters-windows が X11 版と同じ手続きで持つ。**

1. `crates/ui-win32`: 窓の類を登録し、`GWLP_USERDATA` に `RefCell<WindowsApp>` を結ぶ。`WM_SIZE` / `WM_PAINT` / 左ボタン / `WM_MOUSEMOVE` / `WM_MOUSELEAVE` / `WM_MOUSEWHEEL` / `WM_KEYDOWN` / `WM_CHAR` / `WM_TIMER` を `FrameInput` にして `DocketScreen::step` へ渡し、`InvalidateRect` で描き直しを求める。描くのは `WM_PAINT`（`BeginPaint` / `EndPaint` の間に Direct2D）。`WM_ERASEBKGND` は消さない（ちらつき防止）
2. 描画は X11 版の `canvas.rs` と同じ順・同じ意味（`Fill` / `Stroke`（半画素ずらし）/ `Text`（縦中央・寄せ・選択・カーソル）/ `Clip` / `Unclip`）。矩形は `D2D1_ANTIALIAS_MODE_ALIASED`、字は灰色の AA。色は core の `palette`（X11 版もここから読むように移した。ARC-001）
3. 字は DirectWrite の `Segoe UI`、地域 `ja-jp`。日本語は DirectWrite の fallback に任せる（ADR 0002 の 6）。カーソルと選択の位置は `HitTestTextPosition`（UTF-16 の位置なので、core の文字の添字を直して渡す）
4. 鍵: `WM_KEYDOWN` の仮想鍵を `key_input::intent_of_key`（Esc / Enter / BS / Del / Tab / Shift+Tab / 矢印 / Home / End / Ctrl+A / Ctrl+Z / Ctrl+Y）、`WM_CHAR` の字を `intent_of_char`（制御文字は捨てる。サロゲートの対は `CharAssembler` が組み立てる）。純粋な部分は Linux でもテストする
5. `crates/adapters-windows`: `SystemClock`（`SystemTime`）、`saved_account`（`%APPDATA%\nene-docket\account.v1.txt`・`BCryptGenRandom` の 16 バイト・`USERNAME` の名前）、`saved_settings`（同じ置き場の `settings.v1.txt`）、`desktop_dpi`（`GetDpiForSystem`）。ファイルの手続きは adapters-x11 と同じで、違うのは置き場と乱数と名前の環境変数だけ
6. レンダーターゲットの失敗（`EndDraw` の `D2DERR_RECREATE_TARGET` を含む）は道具を捨てて次の `WM_PAINT` で作り直す。窓の手続きは失敗を返せない（nene-ruler ADR 0021 の 11 と同じ）
7. 依存: `windows`（Microsoft・MIT OR Apache-2.0）と `windows-numerics`。Windows のターゲットでだけ引く。`windows` の手続きマクロが引く `unicode-ident` の `Unicode-3.0` を `deny.toml` に足す（実測）
8. `make lint-windows` は X11 の 2 クレートだけを外し、合成ルートも Windows のターゲットで検査する。`make build-windows` が exe を作る

## 強制

- active: `unsafe` は 2 クレートだけ `deny`（他は `forbid`・E0453）。関数ごとの `#[expect(unsafe_code, reason = "RS-010: …")]` と `// SAFETY:`（`undocumented_unsafe_blocks`）。lint の表の写しは CNF-014（実ファイルで発火・gate-proofs 第 19 節）
- active: CNF-013（2 クレートは application・core・windows-rs だけ）、CNF-012（時刻・乱数・環境は adapters-windows だけ）
- active: `make lint-windows`（Windows のターゲットの clippy・0 件）と `make build-windows`（exe）
- planned: Windows での実行の確認（QLT-013）。施主の Windows で、窓・表・入力・日本語・2 プロセスの読み直しを見るまで「動く」と書かない
- planned: `WM_IME_COMPOSITION`（確定前の文字列）と `WM_DPICHANGED`（起動後の dpi の変化）

## 結果

- 得たもの: 同じ中核で Windows の exe が組める。描く手続きだけが 2 本（cairo と Direct2D）
- 失うもの: アカウントと設定の読み書きが 2 つのアダプタに 2 度書かれる（adapters は store に依存できない。PROJECT_LAYOUT 第 2 節）。3 つ目の OS が来たら、標準ライブラリだけの読み書きを store へ寄せる ADR を書く
- 残る穴: Windows で一度も動かしていない。字の大きさ・行の高さの見え方は Segoe UI と IPA ゴシックで違う。IME の確定前の文字列は出ない（確定した字は `WM_CHAR` で入る）。起動後に dpi が変わっても倍率は変わらない

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| nene-ruler と同じ `WS_EX_LAYERED` ＋ `UpdateLayeredWindow` | 透明は要らない。通常の窓のほうが伸縮・最大化・タスクバーが Windows の既定で揃う |
| 読み直しをアダプタのスレッドが `PostMessage` で送る | タイマーで足りる。スレッドと同期の分だけ `unsafe` が増える |
| アカウント・設定の読み書きを store に移して両アダプタから使う | adapters は store に依存できない（CNF-013）。許すなら依存の向きの ADR が要る。2 度書く量は 100 行程度で、今は小さい |
| GDI（`TextOutW`）で描く | ADR 0002 が Direct2D / DirectWrite と決めた。GDI は日本語の fallback と AA が弱い |
| `windows-sys`（薄い束縛）に替えて手続きマクロを避ける | COM（Direct2D）の呼び出しが生のポインタになり、`unsafe` が広がる。`Unicode-3.0` 1 つを許すほうが小さい |

## 補足（Issue #79・2026-10-21）

Windows のファイルロックへの備えは store に置いた（adapters-windows ではない。ARC-007 の区画のまま）: 置き換えは一時名 → rename で、`PermissionDenied` と共有違反に 50 ms × 5 回の再試行。自分のジャーナルは `FILE_SHARE_DELETE` を外して開く。表と施主の確認の手順は [docs/design/2026-10-21-windows-file-locks.md](../design/2026-10-21-windows-file-locks.md)。
