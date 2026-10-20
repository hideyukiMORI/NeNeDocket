# ADR 0002 — X11 は x11rb ＋ cairo、Windows は Win32 ＋ Direct2D。UI ツールキットを使わず、部品は自前で描く

- 状態: 受理
- 日付: 2026-10-21
- Issue: #1
- 影響する規則: ARC-003 / ARC-005 / ARC-013 / RS-010 / CNF-002 / CNF-012 / CNF-013 / CNF-014

## 文脈

施主の条件は「X11 と Windows に出す」「ライブラリやコンポーネントは極力使わない。Electron は使わない」「表やセレクトボックスの部品は自作する」「軽くて速くて安定」。
nene-ruler は同じ条件で、Linux は `x11rb`（純 Rust の X プロトコル）＋ `cairo-rs`（`ImageSurface` に描いて `put_image` で転送）、
Windows は `windows` クレートで Win32 の窓に Direct2D / DirectWrite で描く形を 1.1.0 まで運んだ（nene-ruler ADR 0002 / 0021）。
枠・モーダル・色見本・入力欄・ピッカー・ボタンをすべて自前で描き、配置と当たり判定は core が物理画素で計算し UI は描くだけ、という分担が成り立っている。

NeNe Docket の部品は nene-ruler より多い（表・ヘッダ・セレクトボックス・複数行の入力欄・スクロールバー・ダイアログ）。
しかし**種類が増えるだけで、分担は同じ**である。部品の配置と当たり判定は core の純関数、UI は画素への反映と意図の発行。

nene-ruler から 2 つの違いがある。

1. **窓は OS の通常の窓**（FR-018）。飾りなし・透明・入力 shape は要らない。nene-ruler ADR 0005 / 0008 の大半は継承しない
2. **文字入力が本物**（FR-015）。nene-ruler の入力欄は 16 進 7 文字だった。日本語の入力には IME が要り、X11 は XIM、Windows は TSF / IMM32 を自分で扱うことになる。ツールキットが隠してくれていた最大の部分であり、**別の ADR** で方式を決める

## 決定

**UI ツールキットを使わない。Linux は `x11rb` ＋ `cairo-rs`、Windows は `windows`（Win32 ＋ Direct2D ＋ DirectWrite）だけで、すべての部品を自前で描く。**

1. クレートは nene-ruler と同じ分け方: `ui-x11` / `ui-win32` が画面、`adapters-x11` / `adapters-windows` が OS 資源、`store` がファイル。core と application は `#![no_std]`
2. 部品の配置・当たり判定・表示文字列は core が物理画素で計算する。UI はそれを描き、クリックと鍵を意図に変える。**UI に業務の判断を置かない**（ARC-011）
3. 描画は画面外の面（cairo の `ImageSurface` / Direct2D のビットマップ）に完成させてから窓へ転送する。画素の列は OS 無しで単体テストする
4. `unsafe` は Windows の 2 クレートにだけ `deny` で許し、Win32 を呼ぶ小さな関数ごとに `#[expect(unsafe_code, reason = "RS-010: …")]` と `// SAFETY:` を置く。2 クレートの lint の表は workspace の写しで、CNF-014 が同じであることを見る（nene-ruler ADR 0021 の継承）
5. X11 の `XlibSurface` / `XCBSurface` は `unsafe` が要るので使わない（CNF-002）
6. 文字の描画は cairo の toy text API と DirectWrite に任せ、フォントの fallback も OS に任せる。シェーピングのライブラリ（HarfBuzz / pango）は入れない。日本語の表示で足りなくなったら ADR
7. 文字入力（IME）の方式は、最初の縦切りの前に別の ADR で決める。本 ADR は「ツールキットに任せない」ことだけを決める

## 強制

- **planned**: CNF-013（ui / adapters の外部 crate は `x11rb` / `cairo-rs` / `windows` / `windows-numerics` だけ）・CNF-002（`XlibSurface` / `XCBSurface`）・CNF-014（Windows の 2 クレートの lint の表）・`deny.toml` の `allow`。Issue #2 で置き、最初の縦切りで実ファイルの発火を記録してから active にする
- **不能**: 「UI に業務の判断を置いていないか」はレビュー事項（ARC-011）

## 結果

得られるもの:

- 依存が 4 つで止まる。ビルドが速く、配る物が 1 つの exe / 1 つのバイナリになる（nene-ruler の実績: Linux は `libcairo2` だけ、Windows は何も要らない）
- 部品の見た目と振る舞いが 2 OS で同じになる。配置が core にあるので、OS ごとの違いは画素への反映だけ
- 配置と当たり判定が純関数なので、1 万件の表の配置を OS 無しで測れる（FR-002）

失うもの・残る穴:

- 🔴 **IME は重い。** XIM は古く、`x11rb` に高水準の束ねは無い。Windows の TSF は COM の塊で、IMM32 は非推奨だが単純。どちらを取るかで `unsafe` の量が変わる。方式の ADR で実測してから決める
- 🔴 **アクセシビリティは無い。** スクリーンリーダは自前の部品を読めない。nene-ruler も同じ。必要になったら UI Automation / AT-SPI の ADR
- 🔴 **文字の整形は OS 任せ。** 絵文字・結合文字・右から左の文字は正しく出ないかもしれない。日本語と英数字だけを「動く」と書く
- スクロール・ホバー・フォーカスの手応えを自分で作る。ツールキットが無料でくれていた細部（ダブルクリックの間隔・キーリピート・スクロールの慣性）はすべて自前
- Wayland ネイティブと macOS は対象外のまま（SPECIFICATION 第 3 節）

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| Electron / Tauri | 施主の条件で除外。Chromium か WebView を抱え、軽くも速くもならない |
| GTK / Qt | ツールキット。部品を自作する意味が無く、依存が数十に増え、Windows での配布が重い |
| egui / iced / slint 等の Rust のツールキット | 「ライブラリを極力使わない」に反する。描画のバックエンド（wgpu 等）が依存の木を広げる。部品の見た目と当たり判定を自分で持てなくなる |
| winit ＋ 自前描画 | 窓だけを任せる折衷。しかし winit は Wayland / macOS / Web を抱えて依存が多く、X11 と Win32 だけなら `x11rb` と `windows` の直叩きのほうが小さい。nene-ruler で実証済み |
| Linux も OpenGL / Vulkan で描く | 透明と部品の描画に GPU は要らない。cairo の `ImageSurface` は OS 無しでテストできる |
| Windows も cairo で描く | Windows に cairo の DLL を配ることになる。Direct2D は OS にある。nene-ruler ADR 0021 と同じ判断 |
| IME をツールキットに任せるために GTK だけ入れる | 「1 つのために全部入れる」。IME のためだけに依存の木が数十増える。自前で扱う重さは認めた上で、ADR で方式を決める |

## 関連

- nene-ruler ADR 0002（x11rb ＋ cairo）・ADR 0021（Windows 版と confined unsafe）・ADR 0011 / 0027 / 0028（倍率と論理画素）
