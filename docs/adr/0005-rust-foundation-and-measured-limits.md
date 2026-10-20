# ADR 0005 — Rust の検査基盤と実測できた限界を固定する

- 状態: 受理
- 日付: 2026-10-21
- Issue: #2
- 影響する規則: RS-001〜RS-019 / ARC-002 / ARC-007 / ARC-013 / QLT-002 / QLT-003 / QLT-004 / QLT-011 / CNF-001〜CNF-014

## 文脈

[ADR 0001](0001-strictness-is-mechanically-enforced.md) は「文書を先に置き、機械を次に置く」と決め、Issue #1 ではすべての規則を planned と書いた。
Issue #2 で nene-ruler の機械を移植し、**このリポジトリの道具で**発火を測った。nene-ruler の出力を引用するのではなく、同じ版（rustc 1.98.0）で同じ違反を食わせて同じ出力が出ることを見た。

移植したもの: `Cargo.toml` の `[workspace.lints]`（forbid 層と deny 層の表）・`clippy.toml`・`deny.toml`・`rust-toolchain.toml`・`xtask`（CNF-001〜014・GIT 規約・`prove`）・`Makefile`・`.github/workflows/check.yml`・`.githooks`・`.claude`。
変えたのは: クレート名の接頭辞（`nene-docket-`）、CNF-008 の例外（nene-ruler のアイコンの `build.rs` は持ち込まない）、CNF-012 の除外（`settings-file` → `store`）、CNF-013 の許可表（[PROJECT_LAYOUT](../PROJECT_LAYOUT.md) 第 2 節）、`Makefile` の配布の target（無し）、`deny.toml` の `allow`（`cargo deny list` の実測で MIT だけ）。

実測（[gate-proofs](../quality/gate-proofs.md)）:

- `make prove` 26 件すべて、意図した診断で落ち、戻せば通る
- `cargo test -p xtask` 80 件すべて成功（nene-ruler の 69 件 ＋ nene-ruler が後から足した CNF-014 と target 依存のテスト ＋ 本リポジトリの差分）
- 実ファイルで CNF-009 / CNF-010 / CNF-011 / CNF-013 を発火させ、戻して 0 件
- `make validate-git` は harness の枝名（`claude/…`）で GIT-002 に落ちる。検証器が効いている証拠であり、枝名の扱いは [COMMIT_CONVENTIONS](../COMMIT_CONVENTIONS.md) に書いた

## 決定

**Issue #2 で測れた規則を active とし、測れなかった規則は planned のまま残す。**

1. active: RS-001〜RS-019・ARC-002・ARC-007・ARC-013・QLT-002・QLT-003・QLT-004・QLT-011・CNF-001〜CNF-004・CNF-006〜CNF-014
2. planned のまま: ARC-003（中核クレートの実ファイルで CNF-007 / CNF-013 を発火させてから）・QLT-009（中核が生まれてから `make coverage`）・GIT-001〜004 と QLT-005（最初の PR で CI の実行証拠を得てから）・CNF-005（判定方法が未定）・レビュー事項の ARC / QLT
3. workspace のメンバーは `xtask` だけ。製品クレートは具体的な責務とソースができたときに足す（[PROJECT_LAYOUT](../PROJECT_LAYOUT.md) 第 1 節）
4. `deny.toml` の `allow` は実測の `["MIT"]`。x11rb / cairo-rs / windows を足すときに `cargo deny list` で測り直す（ADR 0002）

## 強制

- **active**: 上の 1 の規則。実測は `make prove`・`cargo test -p xtask`・`make conformance`
- **planned**: 上の 2。それぞれ条件を書いた

## 結果

得られるもの:

- nene-ruler と同じ 26 件の negative proof がこのリポジトリで再現した。規約の本文に引く P-NN は、今はこのリポジトリの実測である
- 違反ゼロの時点で機械が入った。baseline は要らない（ADR 0001）
- `xtask` は 1 日で写せた。ポリシーの雛形（施主の private ワークスペース）に還流する点: CNF-008 の例外と CNF-012 の除外とCNF-013 の表は、リポジトリごとに変える「設定」であって検査器の本体ではない。定数で持つより、表を 1 か所に寄せる余地がある（本 ADR では変えない。還流点として報告する）

失うもの・残る穴（**正直に**）:

- 🔴 **製品クレートの実ファイルでは、まだ何も発火させていない。** CNF-007（`#![no_std]`）・CNF-012（決定性）・CNF-013（向き）・CNF-014（Windows の lint の表）は正例・反例のテストでしか見ていない。最初の縦切りで nene-ruler Issue #2 と同じ手順（`#![no_std]` を外す・ui に adapters を足す・ui で `std::env` を読む）を踏んで記録する
- 🔴 `make lint-windows` は今は `xtask` しか見ない。Windows のコードが生まれるまで、Windows でだけ緩い書き方は検査されない
- 🔴 `make validate-git` と CI の `make ci` は、harness が割り当てた枝名では GIT-002 で落ちる。クラウドセッションで作業するときは、PR 本文にその事実を書き、施主が squash merge の件名を GIT-003 の形で付ける。枝名を変えられないのは harness の制約であり、規則を緩めない
- `cargo-llvm-cov`・`xdotool`・`mingw-w64` はこのコンテナに無い。環境の setup script に足す行は [DEVELOPMENT_WORKFLOW](../DEVELOPMENT_WORKFLOW.md) 第 9 節
- 緑であることは規則が良いことの証明ではない。実装で邪魔になったら ADR で判断を残す

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| nene-ruler の実測を引用して active と書く | 道具の版が同じでも、workspace の設定を写し間違えれば発火しない。「設定が在ることは検査が効いている証明ではない」（QLT-007）。測ってから書く |
| 製品クレートの空の骨格（core / application）を先に置いて ARC-003 も active にする | 空の将来用クレートは作らない（PROJECT_LAYOUT 第 1 節）。空の `lib.rs` に `#![no_std]` を書いて発火させても、それは本物の実ファイルではない |
| CNF-008 のアイコンの例外を持ち込む | NeNe Docket にまだアイコンは無い。例外は要るときに ADR で足す（nene-ruler ADR 0023 の判断は、要るときに同じ形で下せる） |
| `xtask` を依存付きで書き直す（toml / serde） | 依存 0（ARC-013）。CNF-0xx は全て構文で判定できることを nene-ruler が示した |
| GIT-002 を harness の枝名に合わせて緩める | 規則を緩めるのは ADR 事項（QLT-010）だが、緩める理由が「道具の都合」でしかない。枝名の扱いを文書に書き、squash の件名で形を守る |

## 関連

- nene-ruler ADR 0003（Rust の検査基盤と実測できた限界）・gate-proofs
- 本リポジトリ ADR 0001・0002
