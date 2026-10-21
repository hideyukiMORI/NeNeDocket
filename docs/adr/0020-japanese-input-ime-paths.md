# ADR 0020 — 日本語入力（IME）: Windows は IMM32 で確定前の文字列を欄に描く。X11 は当面クリップボードから貼り、XIM の自前実装は施主の WSLg で届く経路を測ってから

- 状態: **受理**（Issue #80・施主「日本語入力 … も考慮予定にして」2026-10-21・FR-015）
- 日付: 2026-10-21
- Issue: #80
- 影響する規則: FR-015 / ARC-002・ARC-013（依存は許可制）/ ARC-011（UI は描くだけ）/ ADR 0002（Windows は `windows` クレートだけ）/ ADR 0019（依存の重さの判断）

## 文脈

core には確定前の文字列（`Preedit`）と欄への差し込み（`TextEdit::display`）が Issue #5 からあり、application は `FrameInput::with_preedit` でフォーカスのある欄に渡す。無いのは OS から確定前の文字列を受け取る経路だった。X11 版は keysym を字にするだけなので、**IME が無いと日本語をまったく打てない**。Windows 版は IME の確定が `WM_CHAR` で届くので打てるが、確定前の文字列が見えず、IME の組み立ての窓が欄の外に出る。

このコンテナには IM サーバ（fcitx5 / ibus）が無く、X11 の IME は測れない（Issue #7 以来の制約）。

## 選択肢（X11）

| 案 | 中身 | 依存 | 大きさ | 判断 |
| --- | --- | --- | --- | --- |
| (a) XIM の自前実装 | XIM をクライアントとして x11rb の上に書く（`XIM_XCONNECT` / `XIM_OPEN` / `XIM_CREATE_IC` / `XIM_COMMIT` / `XIM_PREEDIT_DRAW`）。プロトコルは文書化されていて、fcitx5 も ibus も XIM サーバを持つ | 増えない | 1,000 行規模 | **第一候補**。着手は施主の WSLg で XIM サーバが届くと分かってから |
| (b) IM サーバの D-Bus | fcitx5 / ibus のそれぞれの D-Bus の口を `zbus` 一式で叩く | `zbus` と依存（数十クレート） | IM ごとに別 | 却下。ADR 0019 と同じ理由（依存の重さ・版の重複の恐れ）。IM ごとに違うのも悪い |
| (c) ツールキット | GTK / Qt の入力メソッドを使う | 大 | — | ADR 0002 で却下済み |
| (d) クリップボードからの貼り付け | Ctrl+V で `CLIPBOARD` の `UTF8_STRING` を `ConvertSelection` で求め、`SelectionNotify` で読む。XDND の受け手（Issue #65）と同じ仕組み | 増えない | 100 行 | **当面の逃げ道として入れる**。他のアプリで打って貼る |

## 決定

1. **Windows は IMM32**（`windows` クレートの `Win32_UI_Input_Ime`・依存は増えない）。`WM_IME_SETCONTEXT` で `ISC_SHOWUICOMPOSITIONWINDOW` を消して IME の組み立ての窓を出させず、`WM_IME_STARTCOMPOSITION` / `WM_IME_COMPOSITION` / `WM_IME_ENDCOMPOSITION` を `FrameInput::with_preedit` に写す（`GCS_COMPSTR` と `GCS_CURSORPOS`・UTF-16 の単位のカーソルは文字の添字に直す）。確定（`GCS_RESULTSTR`）はこちらで読んで 1 字ずつ `KeyIntent::Char` にする（`WM_IME_CHAR` / `WM_CHAR` には流さない）。候補の窓は `ImmSetCandidateWindow` でフォーカスのある欄の左下に置く。TSF（`ITextStoreACP`）は IMM32 より大きく、IMM32 は TSF の上で今も動くので選ばない。
2. **X11 は当面 (d)**。Ctrl+V は意図（`KeyIntent`）にはならず、UI がクリップボードの変換を求める。`INCR`（分割の転送・256 KiB 超）は受けない。`PRIMARY`（中ボタン）は扱わない。
3. **X11 の IME 本体は (a) を第一候補にし、施主の WSLg で測ってから着手する。** 測ること: fcitx5 を入れ `XMODIFIERS=@im=fcitx` で `XIM_SERVERS` の property と `_XIM_XCONNECT` が x11rb の窓から届くか。届けば (a) の Issue を切る。届かなければ再度 ADR。
4. **確定前の文字列の見え**は両 UI 共通: `TextCaret::underline` で範囲を渡し、UI が 1 画素の下線を引く。
5. **改行の字**（貼り付け・確定で届く `\n`）は core が扱う: 1 行の欄では空白、複数行の欄では改行（`TextEdit::applied` / `applied_in_area`）。`\r\n` と `\r` は `\n` に揃え、タブは空白、ほかの制御文字は捨てる（`core::pasted_text`）。

## 帰結

- 依存は増えない。`unsafe` は ui-win32 の `ime`（IMM32 の 5 関数）に閉じる。
- 候補の窓の位置は欄の矩形の左下で、カーソルの画素の位置ではない（UI がカーソルの位置を application に返す経路が無い。要るなら `Pen` が描いた位置を覚える）。
- Windows は施主の実機で未確認（ADR 0009 と同じ）。X11 で日本語を打つには、当面は他のアプリで打って貼る。
- Windows の Ctrl+V の貼り付けは入れていない（IME で打てる。要るなら `OpenClipboard` / `CF_UNICODETEXT`・Issue を切る）。

## 検証

gate-proofs 第 46 節（Xvfb で検索・題名・本文に貼る。Windows は `make lint-windows` だけ）。
