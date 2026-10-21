# 品質ゲート — NeNe Docket

> Status: normative（規範）/ 2026-10-21 初版
> 本書は「いま何が機械で守られているか」の**正本**である。
> 規範の本文は [ARCHITECTURE_CONSTITUTION.md](ARCHITECTURE_CONSTITUTION.md) と
> [CODING_RULES.md](CODING_RULES.md)、機械側の実体はこの文書に対応する。出典は nene-ruler `docs/QUALITY_GATES.md`。

🔴 Issue #2 で nene-ruler の機械（lint の表・`clippy.toml`・`deny.toml`・`xtask`・`Makefile`・CI・フック）を移植し、
`make prove`（26 件）と `cargo test -p xtask`（80 件）と実ファイルでの発火でこのリポジトリの道具で測った規則だけを active にした（[quality/gate-proofs.md](quality/gate-proofs.md)・[ADR 0005](adr/0005-rust-foundation-and-measured-limits.md)）。
Issue #4 で `crates/core` が生まれ、中核の実ファイルで CNF-007 / CNF-012 / CNF-013 を発火させて ARC-003 を、`make coverage` の下限で QLT-009 を active にした。PR #3 で CI が `make ci` を走らせ、QLT-005 と GIT-001 / GIT-003 を active にした。ruleset が要る GIT-002 / GIT-004 は planned のまま。

完了条件は、差分の挙動と直接の依存・呼び出し元に必要な最小限の検証が成功し、その根拠・対象・結果・再利用条件が記録されていること。
既存の個別 target（`make lint` / `make test` / …）と対象指定を使う。全件（`make check REASON=…`）は既定ではない
（[ADR 0003](adr/0003-diff-scoped-verification-and-result-reuse.md)）。

---

## 1. ゲートの整合性規則（QLT-0xx）

### QLT-001 — 検証は差分と退行の根拠から選ぶ

変更した挙動と直接影響を受ける依存先・呼び出し元を確認し、その退行を検出できる最小限の検証を選ぶ。
「今回何が壊れる可能性を確認するか」を説明できない検証は実行しない。`make` の個別 target・`cargo test -p <crate> <name>` を優先し、大規模な自動選定基盤を作らない。
文書・コメント・ルール変更にアプリの動作テストは不要。フック・開発ツール変更はその道具の必要な正例・反例を短く確認する。

全件は共通基盤（lint の集合・`xtask`・workspace の設定）など、限定した検証では影響を覆えない具体的な理由がある場合のみ。
`make check REASON=<理由>` を使う。指定なし・空の理由は実行前に拒否する。「念のため」「習慣」は理由にならない。

- 機械強制: **planned**（`make check` は `REASON` を要求し、CI は検証記録の存在を検査する。対象と理由の妥当性はレビュー事項）

### QLT-002 — 警告は失敗する

`rustfmt` の差分・clippy の指摘・rustdoc の警告はすべて失敗にする。重大度の引き下げは禁止。

- 機械強制: **active**（clippy `-D warnings`・`cargo fmt --check`・`RUSTDOCFLAGS=-D warnings`。実測は `make prove` の P-01〜P-20）

### QLT-003 — baseline を作らない

既存の違反を「無かったこと」にする経路を作らない。クレート単位の `#![allow]` は `clippy::allow_attributes` が forbid なので**そもそも書けない**。
lint を `allow` に落とす設定・`continue-on-error`・`--cap-lints`・`rustfmt.toml`・`baseline` と名の付くファイルは CNF-011 が拒む。
**例外機構は `#[expect]` だけで、それは理由と規則 ID を要求し、腐れば落ちる（RS-019）。**

- 機械強制: **active**（`allow_attributes` の forbid（P-03）＋ CNF-011）

### QLT-004 — 整形は検査であって修復ではない

整形は rustfmt の既定から決定的に決まる。CI がソースを書き換えて成功させることはしない。`rustfmt.toml` を置かない。

- 機械強制: **active**（`cargo fmt --all --check`。実測 P-20。`rustfmt.toml` の禁止＝CNF-011）

### QLT-005 — ローカルと CI は同一

CI で必要な判定は、すべてローカルで実行できる。CI の workflow は `make ci` を呼び、PR のメタデータを環境変数で渡すだけ。
判定のロジックを workflow に書かない。検査を足したくなったら、まず `Makefile` と `xtask` に足す。

- 機械強制: **active**（`check.yml` は `make ci` を呼ぶだけ。PR #3 の実行 39517454232 で、手元と同じ `GIT-002: invalid branch name` が CI でも出た・[gate-proofs](quality/gate-proofs.md) 第 10 節）

### QLT-006 — アーキテクチャは実行可能

モジュール境界・依存の向き・状態の経路は機械が検査する。散文だけのアーキテクチャ規則は、**「不能」と明記されない限り未完成**として扱う。

- 機械強制: **planned**（向きは CNF-013、循環は cargo。状態の所有者と隔離区画はレビュー事項のまま）

### QLT-007 — カスタムゲートには negative proof が要る

自作の検査は、「わざと違反させた入力で、意図した規則によって落ちること」と「正しい入力では通ること」の両方を示すまで信用しない。
`xtask` の各 CNF は正例・反例の単体テストを持ち `make test` で常に走る。lint・コンパイラ・cargo-deny・rustfmt は `make prove` が本物の道具に違反を食わせる。
証拠は [quality/gate-proofs.md](quality/gate-proofs.md) に記録する。ゲートを変えたら証拠も同じ変更で更新する。

🔴 **設定が在ることは、検査が効いていることの証明ではない。** xi-tools では `cognitive_complexity` が閾値だけ設定されて lint が無効という死んだ状態だった。
nene-ruler では `indexing_slicing` が固定長配列の定数添字を「安全」と判断して発火せず、slice に直して初めて落ちた。

- 機械強制: **planned**（`make prove` と `xtask` のテストは機械だが、証明の存在と網羅は人手。CNF-009 が active の証明行の欠落だけを拒む）

### QLT-008 — 振る舞いの変更はテストを伴う

振る舞いを変えたら、いちばん狭い安定した境界にテストを足す。不具合の修正は、可能なら**先に落ちる回帰テスト**を書いてから直す。
併合（FR-008）の変更は、**同じジャーナルの集合を別の順で読んでも同じ状態になる**テストを必ず伴う。

- 機械強制: **planned**（レビュー事項。カバレッジ下限＝QLT-009 が部分的に代替する）

### QLT-009 — カバレッジは下げられない

中核（core / application）の region カバレッジ下限を 90% とする。閾値は上げてよいが下げてはならない。下げるには ADR が要る。
**置いていない層は「置いていない」と書く。** 分岐は stable の rustc では測れないので、単位は region（nene-ruler ADR 0010 の継承）。

- 機械強制: **active**（`make coverage` の `--fail-under-regions 90`。実測値と発火は [gate-proofs](quality/gate-proofs.md) 第 9 節。application ができたら `-p` に足す）

### QLT-010 — ゲートの弱体化はアーキテクチャ変更

閾値・`forbid` からの降格・lint の削除・`clippy.toml` の免除追加・モジュール境界・必須 CI ジョブ・`deny.toml` の緩和を変えるには明示的な根拠が要る。
MUST 規則を弱める変更には ADR が要る。**無関係な作業を通すために検査を切ることは禁止。**

- 機械強制: **不能**（判断そのものが対象。PR の手続きで担保する）

### QLT-011 — 依存は再現可能

道具の版は `rust-toolchain.toml` にだけ書く（`Makefile` にも CI にも書かない）。依存は `Cargo.lock` で固定し、ゲートは `--locked` で走る。
lock が更新される状態でのゲート通過を拒む。

- 機械強制: **active**（`--locked`。実測 P-22。cargo-deny の取得元の制限＝ARC-013）

### QLT-012 — 検証の頻度は変更に従う

検証済みの実装・テスト・関連依存と必要な環境条件が変わらなければ、成功済み結果を push・レビュー・merge で再利用する。
担当・工程・文書追記・コミット ID の変更だけでは再実行しない。関連する変更・失敗・具体的な未確認事項がある場合だけ、その範囲を再検証する。
PR に「確認する退行」「対象・依存」「検証結果（コマンド・結果・記録の所在）」「再利用（元の記録と不変性の根拠、または初回検証）」を残す。
CI の必須 `check` は非 draft の ready_for_review / synchronize / edited / reopened で Git 規約・記録の存在・空白だけを確認する。

- 機械強制: **planned**（`make ci` は Git・記録の欄・空白を検査する。結果の真偽・再利用条件・失敗との因果はレビュー事項）

### QLT-013 — 環境依存の主張は正直に名付ける

単体テストは X サーバ・Windows・実機・実 OS 資源・ネットワーク共有を必要としない。単体テストが通ったことを「実機で動く」「共有の上で壊れない」の証拠として扱わない。
Xvfb の上での表示確認は自動でも「仮想の X サーバでの確認」であり、WSLg や Windows の実機での確認とは別に記録する。
**SMB の上での同時編集は、実際の共有（施主の環境）で確かめるまで「動く」と書かない。** 一時ディレクトリでの併合テストはそれの代わりにならない。

- 機械強制: **planned**

---

## 2. 規約検査の規則（CNF-0xx）— `xtask`

lint が見ないもの、つまり**このリポジトリ固有の規約**を検査する。依存ゼロで書き、`make conformance` から常に呼ぶ。
各規則には正例・反例の単体テストを付け、`make test` に結線する（QLT-007）。nene-ruler の `xtask` を写し、クレート名・CNF-008 の例外（無し）・CNF-012 の除外・CNF-013 の許可表だけを変えた（Issue #2）。

🔴 **`.rs` は、桁 0 の `#[cfg(test)]` の次の行が `mod` で始まる所より後を見ない**（テストは意図的な違反を書く場所）。`#[cfg(test)] extern crate std;` のような 1 項目だけの印では止まらない（Issue #4 で実ファイルで見つけた穴）。
🔴 **検出語は `concat!` で分割して書く**（検査器が自分自身を違反として報告しないため）。

### CNF-001 — `Default` の禁止

`#[derive(Default)]`・`impl Default for`・`..Default::default()` を拒む。生成経路は唯一のファクトリに限る。

- 対応する規則: RS-003
- 機械強制: **active**（`xtask`）

### CNF-002 — 禁止構文

`dyn Any`・`OnceLock`・`LazyLock`・`lazy_static`・`proc-macro = true`・`XlibSurface`・`XCBSurface` を拒む。

- 対応する規則: RS-006 / RS-009 / RS-010 / ADR 0002
- 機械強制: **active**（`xtask`）

### CNF-003 — 1 ファイル 1 主要宣言

桁 0 の `struct` / `enum` / `trait` / `union` を数える。2 つ目を拒む。関数内のローカル型は数えない。

- 対応する規則: RS-012
- 機械強制: **active**（`xtask`）

### CNF-004 — 役割を語らない名前

型名の語尾 `Manager` / `Helper` / `Util` / `Utils` / `Common` と、モジュール名 `utils` / `helpers` / `managers` / `misc` / `common` を拒む。
判断が要る語（`Processor` 等）は拒まない。

- 対応する規則: RS-013
- 機械強制: **active**（`xtask`）

### CNF-005 — 不変条件を持つ型は単独のモジュールに置く

- 対応する規則: RS-003
- 機械強制: **planned**（「不変条件を持つ型」を構文で判定する方法が未定。中核の実装を見てから決める）

### CNF-006 — `#[expect]` の `reason` は実在する規則 ID を引く

`reason = "<規則 ID>: <理由>"` の形で、ID が規範文書の `### ID` 見出しに実在すること。複数行の `#[expect]` も 1 件として読む。

- 対応する規則: RS-019
- 機械強制: **active**（`xtask`）

### CNF-007 — 中核クレートは `#![no_std]` を宣言する

名前が `-core` / `-application` で終わるクレートの `src/lib.rs` の先頭の属性行群に `#![no_std]` があること。

- 対応する規則: ARC-003 / RS-015
- 機械強制: **active**（`xtask`）

### CNF-008 — `build.rs` を置かない

`build.rs` そのものと、マニフェストの `build = ` を拒む。例外は無い。アイコンを埋める必要が出たら ADR で 1 つだけ足し、検査で固定する。

- 対応する規則: RS-010
- 機械強制: **active**（`xtask`）

### CNF-009 — 文書整合

規則 ID の重複定義・機械強制の状態の欠落・本文とマトリクスの不一致・active の証明行の欠落・未定義 ID の参照・切れた相対リンク・未置換の雛形値を拒む。

- 対応する規則: QLT-007 / ADR 0001
- 機械強制: **active**（`xtask`。Issue #2 で実ファイルの発火を記録した。gate-proofs 第 8 節）

### CNF-010 — `TODO` / `FIXME` に Issue 番号

`.rs` / `.toml` / `.yml` / `Makefile` のタスクの印は同じ行に `#N` を持つ。

- 対応する規則: QLT-008
- 機械強制: **active**（`xtask`）

### CNF-011 — baseline とゲート無力化の禁止

`baseline` / `suppressions` / `rustfmt.toml` の名を持つファイル、`= "allow"`・`continue-on-error: true`・`--cap-lints`・`-A warnings` を拒む。

- 対応する規則: QLT-003 / QLT-004
- 機械強制: **active**（`xtask`）

### CNF-012 — 決定性の入力 API

`crates/` の `.rs` で `std::time` / `SystemTime` / `Instant` / `std::env` / `std::fs` / `std::thread` / `std::process::Command` / `std::net` / `rand::` を拒む。
`crates/adapters-x11/`・`crates/adapters-windows/`・`crates/store/` は除外。`crates/app/` は `std::env::args` だけを許す。テストにも適用する。

- 対応する規則: ARC-007 / RS-015
- 機械強制: **active**（`xtask`。マクロ・別名・間接呼び出しは字句では見えない）

### CNF-013 — クレート間の依存は許可された向きだけ

`crates/*/Cargo.toml` と `xtask/Cargo.toml` の `[dependencies]` と `[target.….dependencies]` を [PROJECT_LAYOUT.md](PROJECT_LAYOUT.md) 第 2 節の許可表と照合する。内部クレートは向き、中核・`store`・`xtask` は外部クレート 0。

- 対応する規則: ARC-002 / ARC-013
- 機械強制: **active**（`xtask`）

### CNF-014 — `unsafe` を許す 2 クレートの lint の表は workspace と同じ

`crates/adapters-windows/Cargo.toml` と `crates/ui-win32/Cargo.toml` の `[lints.rust]` / `[lints.clippy]` が、workspace の `[workspace.lints.rust]` / `[workspace.lints.clippy]` と同じであることを見る。違ってよいのは `unsafe_code = "deny"` だけ（Cargo は継承した表の 1 項目だけを変えられないので、2 クレートは表を写す・ADR 0002）。

- 対応する規則: RS-010 / QLT-003 / QLT-010
- 機械強制: **active**（`xtask`）

### CNF-015 — ファイルの時刻を読まない・書かない

`crates/` の `.rs` の production コードで `.modified()` / `.accessed()` / `.created()` / `set_modified(` / `set_times(` / `FileTimes` を拒む。adapters と store も例外ではない（壁時計は機械ごとにずれ、NAS では粒度も違う。順序は論理時計、伸びは読み位置で決める・ADR 0004・Issue #76）。

- 対応する規則: ARC-004 / RS-015
- 機械強制: **active**（`xtask`。字句検査なので別名・間接呼び出しは見えない）

---

## 3. 強制マトリクス

規範の各規則が、いまどの層で守られているか。**この表が実装と食い違ったら merge を止める。**
CNF-009 が「本文に定義があるのにここに行が無い」「状態が本文と違う」「active に証明行が無い」を拒否する。

| 規則 | 状態 | 機械強制の実体 |
| --- | --- | --- |
| ARC-001 | planned | レビュー事項（CNF-004 と `pub_use` が温床を減らす） |
| ARC-002 | active | cargo の循環拒否（P-23）＋ CNF-013（実ファイル: gate-proofs 第 8 節） |
| ARC-003 | active | CNF-007（`#![no_std]`）＋ CNF-013（外部 crate の禁止）。実ファイル発火と `std` の名前解決エラー E0433 は gate-proofs 第 9 節 |
| ARC-004 | planned | CNF-015（ファイルの時刻を使わない）が一部を守る。状態の持ち主はレビュー事項 |
| ARC-005 | planned | |
| ARC-006 | planned | CNF-002 と RS-014 が一部を守る |
| ARC-007 | active | CNF-012 |
| ARC-008 | planned | レビュー事項 |
| ARC-009 | planned | |
| ARC-010 | planned | RS-005 が panic を塞ぐ |
| ARC-011 | planned | |
| ARC-012 | 不能 | QLT-010 の手続き |
| ARC-013 | active | `wildcard_dependencies`・`--locked`（P-22）・cargo-deny（P-24 / P-25）・CNF-013（実ファイル: gate-proofs 第 8 節） |
| RS-001 | active | E0603（P-15） |
| RS-002 | active | E0004（P-05）＋ `wildcard_enum_match_arm`（P-04） |
| RS-003 | active | E0381（P-16）・E0063（P-17）・CNF-001 |
| RS-004 | active | `unwrap_used` / `expect_used` の forbid（P-13） |
| RS-005 | active | forbid 群（P-13 / P-14）＋ `panic_in_result_fn` / `unwrap_in_result` |
| RS-006 | active | CNF-002 |
| RS-007 | active | `as_conversions`（P-06） |
| RS-008 | active | `missing_docs` 等（P-09） |
| RS-009 | active | `unsafe_code` の forbid ＋ CNF-002 |
| RS-010 | active | `unsafe_code`（P-18）・CNF-002・CNF-008。Windows の 2 クレートは `deny` と `#[expect]`・`undocumented_unsafe_blocks`・CNF-014（ADR 0002） |
| RS-011 | active | `cognitive_complexity`（P-11）・`too_many_lines`（P-12） |
| RS-012 | active | `mod_module_files`（P-10）＋ CNF-003 |
| RS-013 | active | `disallowed-names`（P-26）＋ CNF-004 |
| RS-014 | active | `print_stdout` / `print_stderr`（P-19） |
| RS-015 | active | CNF-012 |
| RS-016 | active | `disallowed-types`（P-07）＋ `iter_over_hash_type` |
| RS-017 | active | `overflow-checks`（P-21） |
| RS-018 | active | `default_numeric_fallback`（P-08） |
| RS-019 | active | forbid の抑制不能（P-01）・腐った抑制（P-02）・`#[allow]` の禁止（P-03）・CNF-006 |
| GIT-001 | active | PR テンプレート＋`make ci`（`Closes #N`）。CI の実行は gate-proofs 第 10 節 |
| GIT-002 | planned | `make ci`（head ブランチ名）は CI で落ちた。ruleset（main の保護）が未設定 |
| GIT-003 | active | `.githooks/commit-msg`＋`make ci`（全コミットと PR タイトル）。CI の実行は gate-proofs 第 10 節 |
| GIT-004 | planned | PR テンプレート＋`make ci`（欄）は CI で動いた。ruleset（squash-only）が未設定 |
| QLT-001 | planned | `make check` の `REASON` 要求 |
| QLT-002 | active | clippy `-D warnings`・rustfmt・rustdoc（P-01〜P-20） |
| QLT-003 | active | `allow_attributes` の forbid（P-03）＋ CNF-011 |
| QLT-004 | active | `cargo fmt --check`（P-20）＋ CNF-011 |
| QLT-005 | active | `check.yml` → `make ci` だけ。CI と手元が同じ出力（gate-proofs 第 10 節） |
| QLT-006 | planned | |
| QLT-007 | planned | `make prove` と `xtask` のテスト。証明の網羅は人手 |
| QLT-008 | planned | レビュー事項 |
| QLT-009 | active | `make coverage`（region 90%）。発火は gate-proofs 第 9 節 |
| QLT-010 | 不能 | PR の手続き |
| QLT-011 | active | `rust-toolchain.toml` ＋ `--locked`（P-22） |
| QLT-012 | planned | |
| QLT-013 | planned | |
| CNF-001 | active | xtask / `cargo test -p xtask` |
| CNF-002 | active | xtask / `cargo test -p xtask` |
| CNF-003 | active | xtask / `cargo test -p xtask` |
| CNF-004 | active | xtask / `cargo test -p xtask` |
| CNF-005 | planned | |
| CNF-006 | active | xtask / `cargo test -p xtask` |
| CNF-007 | active | xtask / `cargo test -p xtask` |
| CNF-008 | active | xtask / `cargo test -p xtask` |
| CNF-009 | active | xtask / `cargo test -p xtask`（実ファイル: gate-proofs 第 8 節） |
| CNF-010 | active | xtask / `cargo test -p xtask`（実ファイル: gate-proofs 第 8 節） |
| CNF-011 | active | xtask / `cargo test -p xtask`（実ファイル: gate-proofs 第 8 節） |
| CNF-012 | active | xtask / `cargo test -p xtask` |
| CNF-013 | active | xtask / `cargo test -p xtask`（実ファイル: gate-proofs 第 8 節） |
| CNF-014 | active | xtask / `cargo test -p xtask` |
| CNF-015 | active | `xtask`（正例・反例 3 件・gate-proofs 第 49 節） |

---

## 4. `make` の target に入っている層

| target | 目的 | 実体 |
| --- | --- | --- |
| `fmt-check` | 文字列としての正本 | rustfmt の既定 |
| `lint` | 型安全・網羅性・警告ゼロ・二段構えの抑制 | clippy ＋ `[workspace.lints]` ＋ `clippy.toml` |
| `lint-windows` | Windows の 2 クレートを Windows のターゲットで検査する | clippy `--target x86_64-pc-windows-gnu` |
| `test` | 振る舞い・検査器の正例と反例 | `cargo test --workspace` |
| `conformance` | NeNe Docket 固有（CNF-0xx） | `xtask conformance` |
| `deny` | ライセンス・脆弱性・重複版・取得元 | cargo-deny / `deny.toml` |
| `doc-check` | rustdoc の警告 | `RUSTDOCFLAGS=-D warnings` |
| `build` | release の 1 バイナリ | `cargo build --release --locked` |
| `prove` | ゲート発火の証明 | `xtask prove` → `target/proofs/results.json` |
| `ci` | Git 規約・PR の記録・空白 | `xtask validate-git` ＋ `git diff --check` |
| `coverage` | 中核の region カバレッジ 90%（QLT-009） | cargo-llvm-cov |
| `screenshot` | Xvfb の上で窓を PNG に撮る（環境検証・QLT-013） | Xvfb ＋ ImageMagick |

---

## 5. マージゲート

`main` は次を必須とする。

- Pull Request 経由であること
- `check`（`make ci`）が成功していること（CI の必須 check）
- PR に差分の検証記録があること
- 未解決のレビュー指摘が無いこと
- squash merge のみ

`main` への直接 push・force push・ブランチ削除は禁止する。

🔴 **リポジトリ設定（ruleset）を適用したら、その事実を [quality/gate-proofs.md](quality/gate-proofs.md) に記録する。**
**設定していないものを「必須になっている」と書かない。** ruleset は未設定。CI（`check.yml` → `make ci`）は PR #3 で動いた（gate-proofs 第 10 節）が、必須 check にするのは ruleset の仕事であり、設定して記録するまで GIT-002 / GIT-004 は planned。
