# ADR 0001 — 厳格さは機械で強制する

- 状態: 受理
- 日付: 2026-10-21
- Issue: #1
- 影響する規則: すべて

## 文脈

本リポジトリの規約は、AYANE 厳格規約ポリシーの 5 回目の適用である。先行は NeNe Loupe（C++23 / MSVC）、
NeNe Folio（C23 / clang-cl）、NeNe Nib（C++23 / clang-cl）、Rust では xi-tools と nene-ruler。共通する中核は 1 つに要約できる。

> **一つのことを実現する方法を 1 つに固定し、そのことを人の記憶ではなく機械に守らせる。**

規約は、破ったときに何も起きなければ、時間とともに必ず腐る。

### 施主の条件と、言語の選定

施主が示した条件は「軽くて速くて安定している」「X11 と Windows に出す」「ライブラリやコンポーネントを極力使わない（Electron は使わない）」「表やセレクトボックスの部品は自作する」「Java は避ける」（2026-10-21・クラウドセッション）。
言語は、同じ施主の nene-ruler がこのポリシーのもとで Rust の列だけが網羅性・不変性・`null`・抑制の封じ込めのすべてで「ある」と実測し、
2 OS でツールキット無しの自前描画を 1.1.0 まで運んだ実績があることから、**Rust・同じ土台**に決めた（[ADR 0002](0002-no-toolkit-x11rb-cairo-win32-direct2d.md)）。

nene-ruler が rustc 1.98.0 で測った出力（[nene-ruler の gate-proofs](https://github.com/hideyukiMORI/nene-ruler/blob/main/docs/quality/gate-proofs.md)）:

```text
P-05  match の variant 漏れ        → error[E0004]: non-exhaustive patterns
P-15  非公開 newtype の直接構築     → error[E0603]: tuple struct constructor `Id` is private
P-16  未初期化の束縛（Go のゼロ値）  → error[E0381]: used binding `z` isn't initialized
P-17  構造体の部分構築              → error[E0063]: missing field `b`
P-01  forbid した lint を #[expect] → error[E0453]: incompatible with previous forbid
P-02  発火しない #[expect]          → error: this lint expectation is unfulfilled
P-23  クレートの循環                → error: cyclic package dependency
```

C++ の先行事例が字句検査と waiver 台帳（人手）で補っていたものの大半を、**Rust ではコンパイラが執行している。**
`forbid` 水準の lint は、抑制しようとする行為そのものがコンパイルエラーになる。

**そして、いま実装が空である。** 規則を後から被せる場合に必要になる baseline が、今は要らない。違反ゼロから始められる時点は今しかない。

### NeNe Docket に固有の事情

nene-ruler と違い、NeNe Docket は**データを持つ**。ジャーナルの併合が決定的でなければ、同じフォルダを開いた 2 人が違う台帳を見る（FR-008）。
決定性（ARC-007）は美点ではなく正しさそのものであり、中核を `#![no_std]` にして時刻と乱数に構文的に到達できなくする価値は nene-ruler より大きい。

## 決定

**すべての規範規則に「機械強制の状態」を持たせ、その状態を文書の側で機械検査する。**

1. 規則の正本を `docs/ARCHITECTURE_CONSTITUTION.md` / `docs/CODING_RULES.md` / `docs/QUALITY_GATES.md` / `docs/COMMIT_CONVENTIONS.md` に置く。
   すべての規則に ID（ARC / RS / QLT / CNF / GIT）を与え、状態を **active / planned / 不能 / 不採用** で明示する。**planned を active と書かない**
2. 🔴 **抑制を二段構えにする。** `forbid` 層は例外が取れない（E0453）。`deny` 層は `#[expect(lint, reason = "<規則 ID>: …")]` だけで抑制でき、
   発火しなくなれば `unfulfilled_lint_expectations` が落とす。`reason` の規則 ID が規範文書に実在することは CNF-006 が見る
3. `make` の個別 target と `xtask` を判定の唯一の置き場とする。CI は `make ci` を呼ぶだけにし、CI 側にしか無い検査を作らない
4. baseline を持たない。クレート単位の `#![allow]` は `clippy::allow_attributes` が forbid なので書けない。設定側の無力化は CNF-011 が拒む
5. ゲートを弱める変更は ADR を要する（QLT-010）
6. 道具の版は `rust-toolchain.toml` だけが決める。2 か所に書かない
7. **ゲートは意図的な違反で発火することを証明してから active と書く。** `make prove` が本物の道具に違反を食わせ、記録を `docs/quality/gate-proofs.md` に残す。
   active の規則に証明行が無ければ CNF-009 が落とす
8. 規約検査（`xtask`）は依存ゼロで書く。Python を持ち込まない（Rust 1 言語）
9. **文書を先に置き、機械を次に置く。** Issue #1 は文書だけで、すべての規則を planned と書く。Issue #2 で nene-ruler の機械（lint の表・`clippy.toml`・`deny.toml`・`xtask`・`Makefile`・CI）を移植し、このリポジトリで発火を測り直してから active にする

## 強制

- **planned**: CNF-009（文書整合）・CNF-011（baseline 禁止）・CNF-006（抑制の規則 ID）。Issue #2 で `xtask` を置き、`cargo test -p xtask` と `make conformance` で実測してから active にする
- **不能**: 「`planned` を勝手に `active` と書き換えないこと」そのもの。マトリクスの行と実装の対応は、証明行の有無までしか機械は見ない

## 結果

得られるもの:

- 規約が自分について嘘をつく経路が塞がる
- 新しい貢献者（人でも AI でも）が「何がいま守られているか」を 1 か所で読める
- waiver 台帳という人手の制度を持たずに済む（Rust の forbid / expect が代わりに執行する）
- nene-ruler の `xtask` と設定を移植できるので、Issue #2 の大半は写しと再測定で済む

失うもの:

- 文書のメンテナンスコスト。規則を足すたびに 3 か所（本文・マトリクス・実装）が同期を要求される
- 書き味の制約: `as` 禁止・数値リテラルの型注釈・`v[i]` 禁止は、表の配置と論理時計の計算で最初に摩擦になる
- `forbid` は後から緩めにくい。降格には ADR が要る（それが狙いだが、コストである）
- **Issue #1 と #2 の間、規則は紙の上にしか無い。** その間に書かれたコードは無い（製品コードは #2 の後）ので、実害は無い

**正直に記録しておくこと:**

- 🔴 本リポジトリでは**まだ何も測っていない**。上の実出力は nene-ruler のものであり、本リポジトリの道具で再現するまで「本リポジトリで効いている」とは書かない
- 🔴 「不正な状態を表現不能にする」は Rust でも完全ではない。`#[derive(Default)]` でゼロ値を作れる（CNF-001 で塞ぐ）。
  同じモジュールに同居する型はファクトリを迂回できる（CNF-005 は planned）
- 🔴 ARC-007 の字句検査（CNF-012）はマクロ・別名・間接呼び出しを見ない。中核は `#![no_std]` で構造的に塞ぐが、UI は `std` を使うので字句検査だけが残る
- 🔴 併合の決定性（FR-008）は、中核が時刻を読めないことで**入力の決定性**までしか守れない。併合の規則そのものが正しいかは、同じ入力を別の順で読むテスト（QLT-008）で見る
- **本 ADR の厳格さは、実装がまだ空である今だからこそ無傷で導入できた。** 緑であることは、規則が良いことの証明ではない。
  実装で規則が邪魔になったとき、緩めるのではなく **ADR で判断を残すこと**が本 ADR の眼目である

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| 散文の規約だけを置く | 守られているかを確かめる手段が無く、時間とともに必ず腐る。**本 ADR の出発点そのもの** |
| 既製 lint の既定セットだけで済ませる | 既製ルールは「Rust として危ういこと」を見るが、「NeNe Docket として守るべきこと」（併合を UI に置かない・ジャーナルを書き換えない）は見られない |
| `clippy::restriction` / `nursery` の一括有効化 | 互いに矛盾する規則の在庫。群で入れると規則ではなく道具の機嫌に従うことになる |
| baseline を作って既存違反を後回しにする | 新規リポジトリなので既存違反が無い。ここで baseline を許すと、以後の違反も同じ入口から入ってくる |
| 実装が入ってから導入する | 既存違反を凍結する baseline が必要になる。今なら違反ゼロで始められる |
| 文書と機械を同じ Issue で入れる | 施主が「文書だけ先に」と決めた。文書だけの間は全部 planned なので、ADR 0001 の趣旨（嘘をつかない）は保てる。機械を置く Issue #2 が文書の誤りを実測で直す機会にもなる |
| nene-ruler で active だった規則をそのまま active と書く | 道具の版も workspace も無いこのリポジトリで発火することを、まだ誰も見ていない。**planned を active と書かない**の最初の適用例 |
| waiver 台帳（先行事例の `WVR-NNNN`）を移植する | Rust では forbid 層に例外の申請窓口が無く、deny 層の `#[expect]` は理由が必須で不要になれば失敗する。forbid を deny に降ろす判断をしたら ADR で導入する |
| Python で検査器を書く | Rust のリポジトリに第 2 の言語を持ち込むと道具の版が 2 系統になる。`xtask` に統一する |
| 規約を `CLAUDE.md` にだけ書く | 人間と AI の遵守に依存する。先行事例で実証済みの失敗 |
| 言語を C++23 / C23 にする | 先行 3 件の実績はあるが、検査器が Python になり、網羅性・不変性・抑制の封じ込めが「不十分」のまま。施主の憲章の比較表がそう記録している |
| 言語を Go にする | ゼロ値の構造体が RS-003（作り置きの値を用意しない）と正面衝突する。GC の停止があり、X11 と Win32 は cgo か syscall の直叩きになる |
| 言語を Zig にする | 安定版が無く、道具の版を固定する QLT-011 と相性が悪い。ポリシーの実測が無い |

## 関連

- 先行: NeNe Loupe ADR 0001、NeNe Nib ADR 0001、xi-tools ADR 0001、nene-ruler ADR 0001
- ポリシー: 施主の private ワークスペース（本リポジトリには置かない）
