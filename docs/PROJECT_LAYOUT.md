# モジュール構成と依存規則 — NeNe Docket

> Status: normative（規範）/ 2026-10-21 初版
> workspace: `nene-docket`（crate 名の接頭辞は `nene-docket-`）。出典は nene-ruler `docs/PROJECT_LAYOUT.md`

モジュールグラフはアーキテクチャの一部である。**モジュールの命名規約だけでは依存の境界にならない。**
Rust では層を**クレート**で表し、各クレートの `[dependencies]` が import 可能なものの全リストになる（ARC-002）。
承認された向きの表は `xtask` の CNF-013 が持ち、マニフェストと照合する。

🔴 クレートは具体的な責務とソースがある時だけ作る。**空の将来用モジュールは作らない。** Issue #22 の時点で、第 1 節のクレートはすべて在る（Windows の 2 クレートは Issue #22・ADR 0009）。

---

## 1. 承認されたモジュール

```text
crates/app（nene-docket・bin）
    合成ルート。ポートに実装を結び、UI を起動する。端末出力・引数・終了コードを持つ唯一の場所

crates/ui-x11（nene-docket-ui-x11）
    Linux の画面。x11rb で窓を持ち、cairo の ImageSurface に部品を描いて転送する。状態を描き、意図を発行する

crates/ui-win32（nene-docket-ui-win32）
    Windows の画面。Win32 の窓に Direct2D / DirectWrite で部品を描く。unsafe を書ける 2 クレートの 1 つ（ADR 0002）

crates/adapters-x11（nene-docket-adapters-x11）
    Linux のアダプタ。時計・乱数（アカウント ID）・クリップボード・画面の大きさと dpi・IME（XIM）

crates/adapters-windows（nene-docket-adapters-windows）
    Windows のアダプタ。同じポートを Win32 で実装する。unsafe を書ける 2 クレートの 1 つ（ADR 0002）

crates/store（nene-docket-store）
    ジャーナル（版 1・2）の形と置き場。`docket.v1.txt`・`journal/<account>.v1.log` / `.v2.log`・1 行 1 操作と CRC-32・追記と同期・他人の伸びた分の読み直し・`keys/`・`permissions.v1.txt`・自分のスナップショット `snapshot/<account>.v1.txt` と索引 `.v1.idx`（ADR 0015）・添付の写し `attachments/<課題>/<連番>-<名前>`（ADR 0016）。設定ファイルとアカウントのファイルは台帳でなくマシンのものなので、形は application、読み書きは adapters（ADR 0008）。標準ライブラリだけ・外部クレート 0

    アダプタ 2 つと store だけが、現在時刻・乱数・環境・ファイルを読める製品モジュール（ARC-007 / CNF-012）

crates/application（nene-docket-application）
    ポートの宣言、台帳の状態の所有者、Undo の束、編集状態、表示フレームの生成、結果型。#![no_std]

crates/core（nene-docket-core）
    値・不変条件・閉じた選択肢・操作（op）・併合の規則・部品の核（配置・当たり判定・フォーカス・編集状態・描画命令。ADR 0006）・整形・署名と検証（ADR 0010）。外部クレートは ed25519-dalek だけ・#![no_std]

xtask
    規約検査・Git 規約・ゲート発火の証明。成果物ではない（publish = false）。依存 0
```

---

## 2. 許可された依存グラフ

```text
core
    -> なし（alloc のみ）

application
    -> core

store
    -> application, core（外部なし）

adapters-x11
    -> application, core（外部: x11rb）

adapters-windows
    -> application, core（外部: windows。Windows でだけ）

ui-x11
    -> application, core（外部: x11rb, cairo-rs）

ui-win32
    -> application, core（外部: windows, windows-numerics。Windows でだけ）

app
    -> すべて（明示的な合成のためだけに）。X11 のクレートは Windows 以外、Windows のクレートは Windows でだけ（target の節）
```

ここに無い依存はすべて禁止である。とくに:

- core -> 外部 crate: **禁止**（CNF-013 が拒む）
- core -> application: 禁止
- application -> store / アダプタ / UI: 禁止（ファイルの形を application が知らないこと）
- ui-x11 -> adapters-x11 / store: 禁止（X の接続を共有したくなったら、接続を application のポートに畳んで ADR）
- store -> アダプタ: 禁止（時計が要るなら値として受け取る）
- 何か -> 合成ルート: 禁止

---

## 3. 各モジュールの責務

### `core`

意味の正本を持つ。課題・項目・状態の閉じた選択肢・操作（op）とその逆・論理時計・併合（純関数）・表の配置と当たり判定・表示文字列の整形。
UI 状態・直列化・永続化・現在時刻を持たない。`#![no_std]`。

### `application`

振る舞いの調整を持つ。ポート（`trait`）・台帳の状態の所有者・Undo の束・編集状態・表示フレームの生成・結果型。
ファイル・時刻・OS を知らない。`#![no_std]`。

### `store`

ジャーナルと設定ファイルの**形**と**置き場**。版と移行（ARC-009）、直列化の型、追記と同期（fsync）、他人のジャーナルの読み直し。
application の `JournalPort` の実装（`FileJournal`）もここ。
**決定性の禁止を適用しない 3 区画の 1 つ**（ARC-007）。CNF-012 の除外に明示する。外部クレート 0。

### `adapters-x11` / `adapters-windows`

時計・乱数・クリップボード・画面・IME のポートを OS の API で実装する。取得値をその場で core の型へ変換する（ARC-008）。
`adapters-x11` は壁時計（`SystemClock`）・アカウントの保存（`saved_account`・`/dev/urandom` の ID）・設定の読み書き（`saved_settings`・FR-012）・読み直しの刻み（`RefreshTicker`・スレッドが X の窓へ `ClientMessage` を送る。ADR 0008）を持つ。
`adapters-windows` は同じ壁時計・アカウント（`%APPDATA%`・`BCryptGenRandom`）・設定・デスクトップの dpi（`GetDpiForSystem`）を持つ。刻みは Windows では画面のタイマー（ADR 0009）。
両方が OS の言語（`locale`）と明暗（`appearance`）を起動時に観測して環境に添える（ADR 0011 / 0012）。設定は版 2（`settings.v2.txt`）で、版 1 しか無ければ読んで版 2 に書く。

### `ui-x11` / `ui-win32`

application が作った値を描き、操作を意図として渡す（ARC-011）。部品（表・セレクトボックス・入力欄・ボタン・スクロールバー）の**配置と当たり判定は core** が物理画素で計算し、UI はそれを描くだけ。可変性の隔離区画（ARC-005）。
`ui-x11` は cairo の `ImageSurface` に描いて `PutImage` で転送し、`ui-win32` は通常の窓に Direct2D の `HwndRenderTarget` で描く（ADR 0009）。色はフレームが運ぶ配色（core の `Palette`）から、文言は core の `wording` から（ARC-001 / ARC-011）。X11 は表示言語の字を描ける家族を選び直し、Windows は地域名を DirectWrite に渡す（ADR 0011）。鍵の読み替えとポインタの追跡は OS ごとの純関数で、Linux でもテストする。

### `app`

依存を結ぶ。端末出力・`std::env::args`・終了コードを扱ってよい唯一の場所（ARC-006）。

---

## 4. ソースの置き場

| 種類 | 置き場 |
| --- | --- |
| production | `crates/core` / `crates/application` / `crates/store` / `crates/adapters-*` / `crates/ui-*` / `crates/app` |
| 単体テスト | 各クレートの `#[cfg(test)]`（ファイル末尾）と `tests/`。OS 資源と表示は使わない。store のテストは一時ディレクトリを使ってよい |
| 検査器と証明 | `xtask/`（`cargo test -p xtask` が正例・反例） |
| 配布の素材と手順 | `packaging/`（Issue で足す。nene-ruler ADR 0025 / 0026 を継承する予定） |
| 検査設定 | `Cargo.toml` の `[workspace.lints]` / `clippy.toml` / `deny.toml` / `rust-toolchain.toml` / `Makefile` |
| 生成物 | `target/`（cargo・`make prove` の fixture と `results.json`）/ `out/`（`make screenshot` の PNG）/ `dist/` |

表示を伴う確認（Xvfb の上で窓を出し PNG を撮る）は環境検証であって単体テストではない（QLT-013）。
記録は `docs/quality/screenshots/` と gate-proofs。
