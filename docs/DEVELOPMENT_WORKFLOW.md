# 開発ワークフロー — NeNe Docket

> Status: normative（規範）/ 2026-10-21 初版。出典は nene-ruler `docs/DEVELOPMENT_WORKFLOW.md`

ワークフローは制約系の一部である。**追跡可能な手順の外で作られた技術的に正しいパッチは、未完成である。**

---

## 1. 作業の単位

production コード・ビルド・依存・方針・利用者向けドキュメントの変更は、すべて 1 つの焦点を持つ Issue から始める。Issue には次を書く。

- 問題と根拠
- 意図する結果
- 影響する規則 ID とモジュール
- 受け入れ条件
- 検証の計画
- **やらないこと**

読み取りだけで、リポジトリを変更しない調査は Issue 無しで行ってよい。

---

## 2. 標準の流れ

1. 焦点を持つ Issue を 1 つ作る／選ぶ
2. 正本のドキュメントと関係する ADR を読む
3. **編集の前に**正典の実装経路を特定する
4. 最新の `main` からブランチを切る（`<type>/<N>-<kebab>`）
5. 設計そのものを変えるなら、**先に**方針文書か ADR を変える
6. 最小で完結する変更を実装する
7. テストを足す／更新する
8. 差分の挙動と直接の依存先・呼び出し元から、起こり得る退行を検出する最小限の検証を選んで回す（QLT-001）
9. PR を **draft** で出す
10. 影響した規則 ID を 1 つずつ自己レビューする
11. 検証結果・記録の所在・再利用の根拠を PR に書いて Ready にする。CI の必須 check（`make ci`）は Git 規約・記録の存在・空白だけを見る
12. 必須 check が通ってから squash merge する
13. ローカル枝を消す（remote 枝はマージで自動的に消える）

---

## 3. 検証の選び方

個別の target を使う（Issue #2 で置く）。

```bash
make fmt-check     # 整形
make lint          # clippy（全 target・--locked）
make lint-windows  # Windows の 2 クレート（Linux では cfg で外れて見えない）
make test          # cargo test（xtask の正例・反例を含む）
make conformance   # CNF-0xx
make deny          # 依存の許可制
make doc-check     # rustdoc
make build         # release
make prove         # ゲート発火の証明（ゲートを触ったときだけ）
```

全件は、限定した検証では影響を覆えない具体的な理由がある場合だけ、理由を添えて回す。

```bash
make check REASON='共通の lint 集合を変えたので全クレートに影響する'
```

実行していない結果を書かない。本件が原因の失敗は直し、無関係な既存失敗は根拠とともに別 Issue へ記録して本件を続ける。
成功するまでの再試行や無関係な修正・全件再実行は禁止。正本は QLT-001 / QLT-012 と [ADR 0003](adr/0003-diff-scoped-verification-and-result-reuse.md)。

---

## 4. 決定の記録（ADR）

### ADR が要るもの

- 憲章規則・依存方向の変更
- モジュール境界・状態の所有者・隔離区画の変更
- ジャーナルと設定ファイルの形式・版・併合の規則（ADR 0004 の改訂）
- 合成／DI の方針
- 並行性・スレッドモデル
- lint の集合・閾値・`forbid` からの降格・免除の追加（QLT-010）
- 対応プラットフォームの変更
- 実行時依存の追加（ARC-013）
- `unsafe` を要する経路（RS-010）
- 文字入力（IME）の方式・署名の方式

### ADR が要らないもの

- 承認済みの境界の内側に閉じた実装の詳細
- 文書化された振る舞いを回復するだけの不具合修正
- 方針を変えないテストとドキュメント
- **ゲートを強める変更**（弱める変更だけが ADR 事項）

---

## 5. 規則を変える手順

MUST / MUST NOT を変えるには次がそろう必要がある。

1. 矛盾と根拠を書いた Issue
2. 受理された ADR
3. 正本ドキュメントの更新
4. 強制（ゲート）の更新と `make prove` の再実行
5. コードの移行
6. **旧経路が拒否されることを示す回帰テスト**

`V2` / `Legacy` / `New` / `Alternative` という名の並行実装を作らない。

---

## 6. 依存の方針

新しい実行時依存を受け入れるのは、次がすべて満たされるときだけである。

- その機能が具体的にいま必要である
- 自前で書くほうが危険か、保守しにくい（暗号はこれに当たる。自前で書かない）
- ライセンスと保守状況が受け入れられる（`make deny`）
- 正しい境界の裏に隔離できる（中核には入らない）
- 版が固定でき再現可能である（QLT-011）
- ADR がある

**便利だから、は理由にならない。**

---

## 7. PR に書くこと

```text
Closes #N
目的:
使った正典経路:
規則 ID:
振る舞い・スキーマの変更:
確認する退行:
検証（実行したコマンドと結果・記録の所在）:
再利用（元の記録と不変性の根拠、または初回検証）:
抑制（#[expect]）: none | 一覧
残るリスク:
```

レビューは次を拒否する。

- 既存の意味に対する第 2 の経路
- 所有者の書かれていない状態
- 弱められた／飛ばされたゲート
- `#[expect]` の理由に規則 ID が無いもの
- アダプタ固有の業務ロジック・UI に置かれた併合や状態遷移
- 「将来使うかもしれない」公開 API
- 機能変更に紛れ込んだ無関係な整理
- ジャーナルを書き換える経路（追記だけ・ADR 0004）

---

## 8. ドキュメントの方針

重要な決定を、チャット・Issue のコメント・コミットメッセージ・AI との会話ログだけに残さない。

- 長生きする不変条件 → 正本ドキュメント
- 判断とトレードオフ → ADR
- いまのタスク状態 → Issue / PR（`docs/todo/current.md` は要約）
- コードのコメント → その場の非自明な理由（方針そのものは書かない）

設定ファイルとドキュメントが食い違ったら、**ドキュメントが決定の記録**である。ただし両方がそろうまで merge は止める。
**事故が起きたら、直すだけでなく再発を機械で塞ぐ規則を足し、Issue 番号と実測を規則本文に残す。**

---

## 9. 開発環境

### クラウドのコンテナ（Claude Code on the web）

コンテナは Ubuntu 24.04 の Linux で、rustup・Xvfb・cairo のヘッダが入っている。足りないものは `make doctor` が名指しする（Issue #2 で置く）。
環境の setup script（環境の設定で編集する）に次を置くと、毎回のセッションで揃う。版は `rust-toolchain.toml` が正で、ここに書く数字はそれを写しただけ。

```bash
rustup toolchain install 1.98.0 --profile minimal -c rustfmt -c clippy -c llvm-tools-preview
cargo install cargo-deny --locked
cargo install cargo-llvm-cov --locked
apt-get install -y --no-install-recommends libcairo2-dev pkg-config xvfb imagemagick x11-apps x11-utils xdotool xcompmgr mingw-w64
```

`.claude/settings.json` の SessionStart フックが `make doctor` を走らせ、欠けている道具を最初に知らせる（Issue #2）。
表示の確認は `make screenshot`（Xvfb の上で窓を出し PNG を `out/` に撮る）で行い、PNG を読む（QLT-013）。
操作は `xdotool` で駆動して確かめる。人が触る手応えと Windows の見えは施主の実機で確かめる。
ネットワーク共有の上での同時編集は施主の環境（Windows の共有）で確かめる。コンテナでは一時ディレクトリでの併合までしか見られない。

### WSLg（施主の実機）

```bash
sudo apt-get install -y libcairo2-dev pkg-config
git config --local core.hooksPath .githooks
make doctor
cargo run --release
```

---

## 10. リリース

製品の版入力は `Cargo.toml` の `[workspace.package] version` だけとする。画面の表示はここから導出する。
配布は nene-ruler の ADR 0025（配る物は 1 回だけ作り、その成果物をそのまま出す）を継承する予定。手順・署名・Store は別の焦点 Issue とする。
