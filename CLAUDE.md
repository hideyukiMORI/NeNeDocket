# CLAUDE.md — NeNe Docket

Claude Code / AI エージェントがこのリポジトリで作業するための**中核ハンドブック**。
簡潔な英語版の入口は [AGENTS.md](AGENTS.md)。詳細の正本は `docs/` にあり、ここには複製しない。

---

## 0. まず読むもの（production コードに触れる前に必ず）

0. [SPECIFICATION.md](SPECIFICATION.md) — 何を作るか（FR-NNN）
1. [docs/ARCHITECTURE_CONSTITUTION.md](docs/ARCHITECTURE_CONSTITUTION.md) — 憲章（ARC-NNN）
2. [docs/PROJECT_LAYOUT.md](docs/PROJECT_LAYOUT.md) — クレートと依存方向
3. [docs/CODING_RULES.md](docs/CODING_RULES.md) — Rust 規約（RS-NNN）
4. [docs/QUALITY_GATES.md](docs/QUALITY_GATES.md) — **いま何が機械で守られているか**（QLT-NNN / CNF-NNN）
5. [docs/DEVELOPMENT_WORKFLOW.md](docs/DEVELOPMENT_WORKFLOW.md) — 手順
6. [docs/COMMIT_CONVENTIONS.md](docs/COMMIT_CONVENTIONS.md) — Issue・ブランチ・コミット・PR（GIT-NNN）
7. [docs/GLOSSARY.md](docs/GLOSSARY.md) — 用語
8. 該当する ADR（`docs/adr/`）。とくに [ADR 0004](docs/adr/0004-append-only-journal-per-account.md)（ジャーナルと併合）

---

## 1. このリポジトリの統治原則

> **一つのことを実現する方法を 1 つに固定し、そのことを人の記憶ではなく機械に守らせる。**

その帰結として、次の 3 つを常に守る。

1. **正典の経路を先に特定してから編集する。** 「ここで書いたほうが早いから」で第 2 の経路を作らない（ARC-001 / ARC-012）
2. **ゲートを弱めて通さない。** 検査が落ちたらコードを直す。閾値・`forbid` の降格・免除・`deny.toml` を触るのは ADR 相当の判断（QLT-010）
3. **`planned` を `active` と書かない。** 未実装の強制を実装済みに見せるのは、この規約体系で唯一「壊す」行為（[ADR 0001](docs/adr/0001-strictness-is-mechanically-enforced.md)）

🔴 ruleset が要る規則（GIT-002 / GIT-004）は planned のまま。Windows の 2 クレートはまだ無い。

---

## 2. このプロジェクトで間違えやすい所

### ジャーナルは追記だけ。Undo は逆の操作を足す

ジャーナルを書き換える・縮める・並べ替える経路を作らない。削除も「削除した」の追記。各プロセスが書くのは**自分のアカウントのジャーナル 1 本だけ**（[ADR 0004](docs/adr/0004-append-only-journal-per-account.md)）。

### いまの状態は併合の派生値。正本ではない

台帳の状態は core の `merge`（純関数）が全ジャーナルから作る。UI が持つ表示用の複製を正本にしない。同じ操作の集合を別の順で読んでも同じ状態になるテストを、併合に触れたら必ず回す（QLT-008 / ADR 0003）。

### 順序は論理時計。壁時計は表示にだけ

併合の全順序は（Lamport 時計・アカウント ID）。壁時計は機械ごとにずれるので順序に使わない。

### 抑制は二段構え。`#[allow]` は書けない

`forbid` の lint（`unwrap` / `as` / `_` / `unsafe` / `#[allow]` そのもの）は抑制できない（E0453）。抑制したくなった時点で設計の問題。
例外は Windows の 2 クレート（`adapters-windows` / `ui-win32`）の `unsafe` だけで、`deny` に下げる。lint の表は workspace の写しで、ずれると CNF-014 が落とす（ADR 0002）。
`deny` の lint は `#[expect(lint, reason = "<規則 ID>: <理由>")]` だけで抑制でき、発火しなくなれば落ちる。規則 ID は規範文書に実在しないと CNF-006 が落とす（RS-019）。

### 現在時刻・乱数・環境・ファイル・スレッドを持てる場所は決まっている

読んでよいのは **`crates/adapters-x11`・`crates/adapters-windows`・`crates/store`** だけ（ARC-007）。合成ルートは `std::env::args` だけ。中核は `#![no_std]` で構文的に到達できない。
字句検査は CNF-012。**テストが実時刻を読むことも決定性の破壊である。**

### 網羅性検査を殺す分岐を書かない

閉じた `enum` の `match` に `_` を書かない。選択肢が増えたらコンパイルが落ちるのが正しい状態（RS-002）。
X のイベント（`x11rb::protocol::Event`）だけは開いた集合なので `if let` の連なりで書く（規約 第 4 節）。

### `Default` を書かない

`#[derive(Default)]` / `impl Default` / `..Default::default()` は唯一のファクトリを迂回する門。CNF-001 が拒む（RS-003）。

### 数値は型を明示し、`as` を書かない

`400_u32` / `1.5_f64`。変換は `From` / `TryFrom`。cairo / Direct2D の浮動小数との間も同じ（RS-007 / RS-018）。

### 依存は許可制。core・application・store・xtask は依存 0

足すときは ADR ＋ `deny.toml` の `allow` を `cargo deny list` の実測で更新 ＋ 規約 第 5 節の表（ARC-013）。X11 と描画は `x11rb` と `cairo-rs`、Windows は `windows` だけ（ADR 0002）。

### 部品の配置と当たり判定は core。UI は描くだけ

即時モード（ADR 0006）。UI は毎フレーム `Interaction::begin(入力)` → 部品ごとに `probe(ID, 矩形)` → `end()` と回し、`button` / `text_field` / `select_box` などが返す `DrawCommand` の列を描く。文字の幅は core が知らないので、カーソルは文字の添字で渡り、UI がフォントで位置に直す。UI に業務の判断（並び・選択肢・状態遷移）を置かない（ARC-011）。

### 1 ファイル 1 主要宣言・`mod.rs` を作らない

`foo.rs` ＋ `foo/` の形だけ（RS-012）。禁止名（`Manager` / `Helper` / `utils` 等）は CNF-004 が拒む。

### 検査器は自分自身も検査対象

`xtask` に検出語を書くときは `concat!` で割る。`#[cfg(test)]` の次が `mod` の所より後は見ない（`#[cfg(test)] extern crate std;` では止まらない）。

---

## 3. 検証コマンド

差分の挙動・直接の依存先と呼び出し元から、起こり得る退行を検出する最小限の検証を選ぶ。
何が壊れる可能性を確認するか説明できない検証は実行しない。`make` の個別 target と `cargo test -p <crate> <name>` を使う。
文書・コメント・規約変更にはアプリの動作テストは不要。フック・開発ツールは変更した道具だけを短く確認する。

```bash
make lint          # clippy（全 target・--locked）
make lint-windows  # Windows の 2 クレート
make test          # cargo test（xtask の正例・反例を含む）
make conformance   # CNF-0xx
make deny          # 依存の許可制
make doc-check     # rustdoc
make prove         # ゲート発火の証明（ゲートを触ったときだけ）
```

全件は限定した検証では覆えない具体的な理由がある場合だけ、理由を添えて明示実行する。

```bash
make check REASON='限定した検証では影響を確認できない具体的な理由'
```

実行していない結果を書かない。本件が原因の失敗は直し、無関係な既存失敗は根拠とともに別 Issue へ記録して本件を続ける。
成功するまでの再試行や無関係な修正・全件再実行は禁止。PR には対象・退行の根拠・結果と所在・再利用の根拠を記録する。
正本は QLT-001 / QLT-012 と [ADR 0003](docs/adr/0003-diff-scoped-verification-and-result-reuse.md)。

---

## 4. 変更の進め方

[docs/DEVELOPMENT_WORKFLOW.md](docs/DEVELOPMENT_WORKFLOW.md) が正本。要約すると:

Issue → 正典経路の特定 → ブランチ（`<type>/<N>-<kebab>`）→（設計を変えるなら先に ADR）→ 最小の実装 →
必要な検証（成功結果を再利用）→ 規則 ID ごとの自己レビュー → PR（draft）→ 検証記録を整えて Ready → 必須 check（`make ci`）→ squash merge。

コミットは Conventional Commits（`type` と `scope` は英語、説明は日本語、末尾に `(#N)`）。形の正本は GIT-003。

---

## 5. 完了報告の形

作業を終えたら必ず次を報告する。

```text
Issue / 規則 ID:
変更したファイルと振る舞い:
実行した検証コマンドと結果:
ドキュメント・スキーマの変更:
抑制（#[expect]）: none | 一覧
残るリスク:
```

調査だけを頼まれたときは、編集・コミット・push・PR 作成・外部状態の変更を行わない。

---

## 6. いまの状況

現在のタスクは [docs/todo/current.md](docs/todo/current.md)。GitHub Issue が正で、そこは要約。

2026-10-21: Issue #1 で規範文書と ADR 0001〜0004 の初版を置いた。Issue #2 で nene-ruler の機械（lint の表・clippy.toml・deny.toml・xtask・Makefile・CI・フック）を移植し、`make prove` 26 件・`cargo test -p xtask` 80 件・実ファイルの発火を記録して、測れた規則を active にした（ADR 0005）。
Issue #4 で部品の核（即時モード・ADR 0006）を `crates/core` に置き、ARC-003 と QLT-009 を active にした。Issue #5 で X11 の窓（application / ui-x11 / app・x11rb ＋ cairo-rs）を出し、表・検索・絞り込み・スクロールが Xvfb で動いた。日本語は起動時に家族を選んで出す（ADR 0007）。1 万件の 1 フレームは 1 ms 未満（gate-proofs 第 11 節）。
Issue #7 で ADR 0004 の中核（`Item` / `Op` / `OpId` / `merge` / `history`）を core に置いた。IME は IM サーバの無いコンテナでは測れない。Issue #9 で store にジャーナル版 1（1 行 1 操作・CRC-32・追記と同期・読み直し）を置いた。
Issue #11 で画面をジャーナルに結んだ: application の `Ledger`（併合した台帳・論理時計・Undo の束）と `Ports`、store の `FileJournal`、`crates/adapters-x11`（時計・アカウント・読み直しの刻み・ADR 0008）。`nene-docket <フォルダ>` で起動し、追加・題名・状態・削除・Ctrl+Z / Ctrl+Y が追記され、Xvfb で 2 プロセスが同じフォルダを編集して互いの変更が 3 秒以内に届いた（gate-proofs 第 14 節）。Issue #14 で担当と期限も画面から変えられるようにした（形が違う期限は追記せず示す）。次は署名と権限の ADR（FR-010）、本文とラベル、設定ファイル（FR-012）、Windows。

---

## 7. 開発環境

クラウドのコンテナ（Claude Code on the web）が主戦場。Linux で、実ディスプレイは無い。
表示は `make screenshot`、操作は Xvfb の上で `xdotool`（手順は gate-proofs 第 11 節）。人が触る手応え・Windows の見え・IME・ネットワーク共有の上での同時編集は施主の実機で確かめる。
道具の有無は `make doctor`。環境の setup script に置く行は [docs/DEVELOPMENT_WORKFLOW.md](docs/DEVELOPMENT_WORKFLOW.md) 第 9 節。

---

## 8. 上位のポリシーと雛形の所在

このリポジトリの規約は **AYANE 厳格規約ポリシー**の 5 回目の適用例である（1 回目 nene-loupe・C++23 / MSVC、2 回目 nene-folio・C23 / clang-cl、3 回目 nene-nib・C++23 / clang-cl、4 回目 nene-ruler・Rust。Rust では xi-tools に続く 3 回目）。
文書と機械の直接の出典は [nene-ruler](https://github.com/hideyukiMORI/nene-ruler)。
ポリシー本文・初期化手順・雛形は施主の private ワークスペースにあり、このリポジトリには置かない。
雛形で足りなかったものを見つけたら、このリポを直すと同時に施主へ還流点として報告する。
