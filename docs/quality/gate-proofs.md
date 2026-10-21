# ゲート発火の証明 — NeNe Docket

> 記録: 2026-10-21 / Issue #1（文書だけ・第 3 節）、Issue #2（機械の移植・第 2〜8 節）、Issue #4（部品の核・第 9 節）、PR #3（CI・第 10 節）、Issue #5（X11 の窓・第 11 節）。以下は実行した結果のみ。
> 関連: QLT-007 / QLT-011 / QLT-013 / ADR 0001 / ADR 0005 / ADR 0006 / ADR 0007

## 1. 環境と範囲

クラウドの Linux コンテナ（Ubuntu 24.04・x86_64）。rustc 1.98.0 / clippy 0.1.98 / rustfmt 1.9.0 / cargo-deny 0.20.2。
これらは実測時の版であり、今後使う版の正本は `rust-toolchain.toml`（道具は `make doctor`）。
Issue #2 の時点で製品コードは 0。workspace のメンバーは `xtask` だけ。表示を伴う確認は無い。

## 2. active の規則と、その証明

CNF-009 は active の規則ごとに `| ID |` で始まる行がここにあることを要求する。

| 規則 | 最小の違反 | 検証経路 | 実測 |
| --- | --- | --- | --- |
| RS-001 | 非公開 newtype をモジュール外で直接構築 | `make prove` P-15 | E0603 |
| RS-002 | `match` に `_`・variant の漏れ | P-04 / P-05 | `wildcard_enum_match_arm` / E0004 |
| RS-003 | 未初期化・部分構築・`Default` | P-16 / P-17 ＋ CNF-001 | E0381 / E0063 / xtask |
| RS-004 | `unwrap` | P-13 | `unwrap_used`（forbid） |
| RS-005 | `unwrap`・slice の添字 | P-13 / P-14 | `unwrap_used` / `indexing_slicing` |
| RS-006 | `dyn Any` | CNF-002 の反例テスト | xtask |
| RS-007 | `as` で数値変換 | P-06 | `as_conversions` |
| RS-008 | 未文書の公開関数 | P-09 | `missing_docs` |
| RS-009 | `OnceLock` | P-18 ＋ CNF-002 | `unsafe_code` / xtask |
| RS-010 | `unsafe` ブロック・`build.rs` | P-18 ＋ CNF-002 / CNF-008 | `unsafe_code` / xtask |
| RS-011 | 認知的複雑度 13・62 行の関数 | P-11 / P-12 | `cognitive_complexity` / `too_many_lines` |
| RS-012 | `mod.rs`・2 つ目の主要宣言 | P-10 ＋ CNF-003 | `mod_module_files` / xtask |
| RS-013 | 束縛名 `data`・型名 `RulerManager` | P-26 ＋ CNF-004 | `disallowed-names` / xtask |
| RS-014 | `println!` | P-19 | `print_stdout` |
| RS-015 | core の `std::time`・ui の `std::env` | CNF-012 の反例テスト | xtask |
| RS-016 | `HashMap` | P-07 | `disallowed-types` |
| RS-017 | release で `255_u8 + 1_u8` | P-21 | `attempt to add with overflow`・終了 101 |
| RS-018 | 型注釈の無いリテラル | P-08 | `default_numeric_fallback` |
| RS-019 | forbid を `#[expect]`・腐った `#[expect]`・`#[allow]` | P-01 / P-02 / P-03 ＋ CNF-006 | E0453 / unfulfilled / `allow_attributes` |
| ARC-002 | クレートの循環・xtask に依存を足す | P-23 ＋ CNF-013（実ファイル・第 8 節） | `cyclic package dependency` / xtask |
| ARC-003 | core の `#![no_std]` を外す・core で `std::time` を読む・core に `regex` を足す | CNF-007 / CNF-012 / CNF-013（実ファイル・第 9 節）＋ `cargo check` | xtask / E0433 |
| ARC-007 | adapters と store の外で `std::time` | CNF-012 の反例テスト ＋ 実ファイル（第 9 節） | xtask |
| ARC-013 | 許可外のライセンス・先回りの許可・lock 無し・xtask の依存 | P-24 / P-25 / P-22 ＋ CNF-013（実ファイル・第 8 節） | cargo-deny / `--locked` / xtask |
| QLT-002 | lint・整形・言語の違反 | P-01〜P-20 | すべて非 0。復帰は 0 |
| QLT-003 | `#[allow]`・`= "allow"`・`continue-on-error`・`rustfmt.toml` | P-03 ＋ CNF-011（実ファイル・第 8 節） | `allow_attributes`（forbid） / xtask |
| QLT-004 | 一行に詰めた `main`・`rustfmt.toml` | P-20 ＋ CNF-011 | `Diff in`・終了 1 / xtask |
| QLT-005 | harness の枝名の PR を Ready にする | CI の `check`（`make ci`）・第 10 節 | CI でも手元と同じ `GIT-002: invalid branch name`・終了 2 |
| GIT-001 | PR 本文に `Closes #N` が無い | xtask の反例 `pull_request_rejects_draft_missing_close_and_bad_title` ＋ CI の実行（第 10 節） | xtask / CI は `Closes #1` のある本文を通した |
| GIT-003 | 英語の説明・Issue 番号なし・不正な type | xtask の反例 ＋ commit-msg フック ＋ CI の実行（第 10 節） | xtask / 手元のフックが #2 と #4 で動いた / CI は 3 コミットと PR タイトルを通した |
| QLT-009 | 実測値（region 99.71%）より上の下限 100 を置く | `cargo llvm-cov … --fail-under-regions 100`（第 9 節） | 終了 1。下限 90 の `make coverage` は終了 0 |
| QLT-011 | `Cargo.lock` を消して `--locked`・lock が古い状態で `--locked` | P-22 ＋ 実ファイル（第 8 節） | 終了 101 |
| CNF-001 | `Default` の derive・impl・穴埋め | xtask の正例・反例 | 反例は CNF-001、正例は 0 |
| CNF-002 | `OnceLock`・proc-macro・`XlibSurface` / `XCBSurface` | 同上 | 反例は CNF-002 |
| CNF-003 | 2 つ目の主要宣言 | 同上 ＋ 実ファイル（第 9 節） | 反例は CNF-003。ローカル型は通る |
| CNF-004 | 禁止語尾・禁止モジュール名 | 同上 | 反例は CNF-004。`TickProcessor` は通る |
| CNF-006 | 無い規則 ID・ID の無い reason・複数行 | 同上 | 反例は CNF-006 |
| CNF-007 | `-core` / `-application` に `#![no_std]` が無い | 同上 ＋ 実ファイル（第 9 節） | 反例は CNF-007。bin は対象外 |
| CNF-008 | `build.rs`（`crates/app/build.rs` を含む）・マニフェストの `build = ` | 同上 | 反例は CNF-008。`rebuild.rs` は通る。例外は無い |
| CNF-009 | 状態の不一致・行の欠落・証明の欠落・未定義 ID・雛形値・切れたリンク | 同上 ＋ 実ファイル（第 8 節） | 反例は CNF-009。マトリクスの 1 行を active にすると 2 件を報告した |
| CNF-010 | Issue 番号の無い `FIXME` | 同上 ＋ 実ファイル（第 8 節） | 反例は CNF-010 |
| CNF-011 | `rustfmt.toml`・`= "allow"`・`continue-on-error` | 同上 ＋ 実ファイル（第 8 節） | 反例は CNF-011 |
| CNF-012 | core の `std::time`・app の `std::env::var`・`crates/app/build.rs` の `std::fs` | 同上 ＋ 実ファイル（第 9 節） | 反例は CNF-012。adapters・store・`std::env::args` は通る |
| CNF-013 | ui → adapters・core の外部 crate・xtask の依存 | 同上 ＋ 実ファイル（第 8 節・第 9 節） | 反例は CNF-013 |
| CNF-014 | Windows のクレートの lint の表の 1 項目を緩める・消す | xtask の正例・反例 | 反例は CNF-014 を 2 件（rust と clippy の表）。正例と他のクレートは通る |

## 3. Issue #1 で確かめたこと（文書だけ）

機械が無かったので、CNF-009 が見る予定のものを手元の grep で見た: 規則 ID の参照先の実在・相対リンク・マトリクスに active が無いこと・未置換の雛形値。
Issue #2 で同じことを xtask の CNF-009 が見て、違反なしだった（第 8 節）。

## 4. `make prove` の実出力（26 件・Issue #2）

`cargo run -p xtask -- prove` が `target/proofs/` に最小 workspace を作り、本物の道具に違反を食わせた。
各件は「落ちること」の後に元へ戻して「通ること」も確かめている（復帰の終了コードは 0）。nene-ruler の 26 件と同じ出力が出た。

| # | 規則 | 命令 | 終了 | 実出力 |
| --- | --- | --- | --- | --- |
| P-01 | RS-019 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0453]: expect(clippy::unwrap_used) incompatible with previous forbid` |
| P-02 | RS-019 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: this lint expectation is unfulfilled` |
| P-03 | QLT-003 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: #[allow] attribute found` |
| P-04 | RS-002 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: wildcard matches only a single variant and will also match any future added variants` |
| P-05 | RS-002 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0004]: non-exhaustive patterns: 'Mode::Mm' not covered` |
| P-06 | RS-007 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: using a potentially dangerous silent 'as' conversion` |
| P-07 | RS-016 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: use of a disallowed type 'std::collections::HashMap'` |
| P-08 | RS-018 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: default numeric fallback might occur` |
| P-09 | RS-008 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: missing documentation for a function` |
| P-10 | RS-012 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: 'mod.rs' files are not allowed, found 'probe/src/probe10/mod.rs'` |
| P-11 | RS-011 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: the function has a cognitive complexity of (13/10)` |
| P-12 | RS-011 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: this function has too many lines (62/60)` |
| P-13 | RS-005 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: used 'unwrap()' on 'Some' value` |
| P-14 | RS-005 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: indexing may panic` |
| P-15 | RS-001 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0603]: tuple struct constructor 'Id' is private` |
| P-16 | RS-003 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0381]: used binding 'z' isn't initialized` |
| P-17 | RS-003 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error[E0063]: missing field 'b' in initializer of 'Config'` |
| P-18 | RS-010 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: usage of an 'unsafe' block` |
| P-19 | RS-014 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: use of 'println!'` |
| P-26 | RS-013 | `cargo clippy --all-targets --quiet -- -D warnings` | 101 | `error: use of a disallowed/placeholder name 'data'` |
| P-20 | QLT-004 | `cargo fmt --all --check` | 1 | `Diff in target/proofs/workspace/probe/src/main.rs:1:` |
| P-21 | RS-017 | `cargo run --release -p probe` | 101 | `attempt to add with overflow` |
| P-22 | QLT-011 | `cargo check --locked（Cargo.lock を消した状態）` | 101 | `error: cannot create the lock file target/proofs/workspace/Cargo.lock because --locked was passed to prevent this` |
| P-23 | ARC-002 | `cargo check（a ⇄ b の循環）` | 101 | `error: cyclic package dependency: package 'a v0.0.0 (target/proofs/cycle/a)' depends on itself. Cycle:` |
| P-24 | ARC-013 | `cargo deny --locked check licenses --deny license-not-encountered（allow = []）` | 4 | `│            rejected: license is not explicitly allowed` |
| P-25 | ARC-013 | `cargo deny --locked check licenses --deny license-not-encountered（allow = ["MIT", "Apache-2.0"]）` | 4 | `error[license-not-encountered]: license was not encountered` |

## 5. `xtask` の正例・反例（`cargo test -p xtask`・81 件）

| 検査 | 件数 | テスト名 |
| --- | --- | --- |
| CNF-001 | 7 | cnf001_catches_derive_default, cnf001_catches_impl_default, cnf001_catches_struct_update_syntax, cnf001_ignores_clean_source, cnf001_ignores_markdown, cnf001_ignores_test_module, cnf001_sees_past_a_single_item_cfg_test_but_not_the_test_module（Issue #4） |
| CNF-002 | 4 | cnf002_catches_lazy_global, cnf002_catches_proc_macro_manifest, cnf002_catches_x_surfaces, cnf002_ignores_clean_source |
| CNF-003 | 3 | cnf003_allows_single_declaration, cnf003_catches_second_declaration, cnf003_ignores_nested_declaration |
| CNF-004 | 3 | cnf004_allows_context_dependent_name, cnf004_catches_forbidden_module_name, cnf004_catches_forbidden_suffix |
| CNF-006 | 4 | cnf006_accepts_expect_citing_known_rule, cnf006_catches_reason_without_rule_id, cnf006_catches_unknown_rule_id, cnf006_reads_multiline_expect |
| CNF-007 | 5 | cnf007_allows_core_crate_with_no_std, cnf007_catches_core_crate_without_no_std, cnf007_catches_no_std_after_real_code, cnf007_ignores_a_binary_crate, cnf007_requires_no_std_of_application_too |
| CNF-008 | 4 | cnf008_catches_a_build_script, cnf008_catches_a_manifest_pointing_at_one, cnf008_catches_every_build_script, cnf008_ignores_a_similar_file_name |
| CNF-009 | 11 | catches_active_without_proof, catches_broken_relative_link, catches_duplicate_definition, catches_missing_matrix_row, catches_missing_state, catches_state_mismatch, catches_template_placeholder, catches_undefined_rule_reference, consistent_documents_pass, first_state_line_wins_over_later_ones, ignores_external_and_anchor_links |
| CNF-010 | 3 | cnf010_accepts_marker_with_issue, cnf010_catches_marker_without_issue, cnf010_ignores_markdown |
| CNF-011 | 4 | cnf011_catches_continue_on_error, cnf011_catches_forbidden_filename, cnf011_catches_lint_set_to_allow, cnf011_ignores_clean_configuration |
| CNF-012 | 8 | cnf012_allows_adapters, cnf012_allows_arguments_in_composition_root, cnf012_allows_the_windows_adapters_and_the_store, cnf012_catches_env_in_ui, cnf012_catches_files_and_commands_in_every_product_crate, cnf012_catches_other_env_in_composition_root, cnf012_catches_time_in_core, cnf012_ignores_xtask |
| CNF-013 | 7 | cnf013_allows_application_depending_on_core_only, cnf013_allows_the_documented_direction, cnf013_catches_a_dependency_in_xtask, cnf013_catches_external_crate_in_core, cnf013_catches_ui_depending_on_adapters, cnf013_keeps_the_store_free_of_external_crates, cnf013_reads_target_specific_dependencies |
| CNF-014 | 3 | cnf014_accepts_a_faithful_copy_with_unsafe_denied, cnf014_catches_a_loosened_or_missing_lint, cnf014_ignores_other_crates |
| GIT-002 / GIT-003 / GIT-004 | 9 | accepts_the_documented_form, branch_names, breaking_commit_needs_footer_but_title_does_not, pull_request_passes_with_all_fields, pull_request_rejects_draft_missing_close_and_bad_title, rejects_english_description, rejects_missing_issue, rejects_unknown_type_and_bad_scope, rejects_zero_issue_and_long_subject |
| 規則 ID の字句・prove の補助 | 6 | accepts_each_prefix, extracts_ids_from_prose, rejects_wrong_shapes, license_allow_replaces_only_the_license_list, probe_members_replaces_a_multi_line_list, probe_members_replaces_a_single_line_list |

nene-ruler からの差分: CNF-008 のアイコンの例外を外した（`cnf008_catches_every_build_script`）、CNF-012 の除外を `store` に替えた（`cnf012_allows_the_windows_adapters_and_the_store`・`cnf012_catches_files_and_commands_in_every_product_crate`）、CNF-013 の許可表を NeNe Docket のクレート名にした（`cnf013_keeps_the_store_free_of_external_crates`）。

## 6. この手順で見つかったこと

- 移植した `check.rs` のテストを書き換えた直後、`make fmt-check` が落ちた（手で書いた `assert_eq!` の折り返し）。`make fmt` で直した。整形は検査であって修復ではない（QLT-004）ので、CI には直す経路が無い
- 第 8 節の CNF-013 の実ファイル検査で `--offline` の `cargo run` を使ったところ、cargo が `Cargo.lock` に `serde` を書き込み、戻した後の `make conformance` が `--locked` で落ちた（`cannot update the lock file`）。`cargo generate-lockfile` で戻した。**lock が古い状態でのゲート通過を `--locked` が拒む**ことの、意図しない実ファイルでの発火である（QLT-011）
- nene-ruler で見つかった欠陥（`indexing_slicing` が固定長配列の定数添字で発火しない）は、slice に直した P-14 を写したので再現しなかった
- 🔴 **Issue #4: `#[cfg(test)] extern crate std;` が検査を止めていた。** 検査器は「桁 0 の `#[cfg(test)]` より後を見ない」と決めていたが、`lib.rs` の先頭にある `#[cfg(test)] extern crate std;`（`no_std` のテストに要る）がそれに当たり、`lib.rs` の残り全部が CNF-001〜013 の対象外になっていた。`lib.rs` の末尾に `std::time` を書いても CNF-012 が鳴らず、実ファイルの発火で見つけた。nene-ruler の `xtask` にも同じ穴がある（還流点）。終わりを「`#[cfg(test)]` の次の行が `mod` で始まる所」に強め、正例・反例（`cnf001_sees_past_a_single_item_cfg_test_but_not_the_test_module`）を足し、`cnf001_ignores_test_module` の fixture を `mod tests` の形に直した

## 7. まだ証明していないこと

- `make lint-windows` は `xtask` と `core` を Windows のターゲットで検査して 0 件。Windows の 2 クレートの `#[expect(unsafe_code)]` と CNF-014 の実ファイルは、クレートが生まれてから
- ruleset（`main` の保護・squash-only）。未設定。GIT-002 / GIT-004 はこれで active になる
- ruleset（`main` の保護・squash-only）。未設定
- 併合の決定性・Xvfb での表示・Windows の実機・SMB の上での同時編集

## 8. Issue #2 — 機械の移植（2026-10-21）

同じコンテナ。製品コード 0・workspace は `xtask` だけ。

### 8.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` | 差分なし（`make fmt` の後） |
| `make lint` | 0 件 |
| `make lint-windows` | 0 件（`xtask` だけ） |
| `make test` | xtask 80 件、すべて成功 |
| `make conformance` | 違反なし（38 ファイル） |
| `make deny` | advisories / bans / licenses / sources すべて ok。`cargo deny list` は `MIT (1): xtask@0.1.0` |
| `make doc-check` | 警告なし |
| `make build` | release の `xtask` が組めた |
| `make prove` | 26 件成功（第 4 節） |
| `make validate-git` | **GIT-002 で落ちる**: `invalid branch name claude/cloud-session-qus6l5`。クラウドセッションの harness が割り当てた枝名で、GIT-002 の形ではない。コミットの件名（GIT-003）は通った。CI では PR の head で見るので、同じ理由で落ちる（COMMIT_CONVENTIONS の GIT-002 に扱いを書いた） |
| `make doctor` | rustc 1.98.0 / cargo-deny 0.20.2 / cairo 1.18.0 / Xvfb / ImageMagick。無いのは cargo-llvm-cov・xdotool・mingw-w64 |

### 8.2 実ファイルでの規約検査の発火

| 規則 | やったこと | `make conformance` の実出力 |
| --- | --- | --- |
| CNF-009 | `docs/QUALITY_GATES.md` の強制マトリクスで ARC-004 の行だけを `active` にした | `CNF-009: docs/QUALITY_GATES.md — ARC-004 の本文の状態とマトリクスの行が一致しない` と `CNF-009: docs/quality/gate-proofs.md — ARC-004 は active だが証明行が無い` の 2 件 |
| CNF-011 | `rustfmt.toml` を置いた | `CNF-011: rustfmt.toml — baseline・抑制一覧・整形の別設定は置かない（QLT-003 / QLT-004）` |
| CNF-010 | `Makefile` の末尾に Issue 番号の無いタスクの印を足した | `CNF-010: Makefile:115 — タスクの印には同じ行に Issue 番号（#N）が要る（QLT-008）` |
| CNF-013（ARC-002 / ARC-013） | `xtask/Cargo.toml` に `serde = "1"` を足した（`--locked` が先に落ちるので `cargo run --offline -p xtask -- conformance` で実行） | `CNF-013: xtask/Cargo.toml:18 — serde への依存は xtask/ に許されていない（ARC-002 / ARC-013）` |
| QLT-011（意図せず） | 上の `--offline` で `Cargo.lock` が更新された状態で `make conformance` | `error: cannot update the lock file … because --locked was passed to prevent this`・終了 101 |

元に戻した後の `make conformance`: `規約違反なし（検査 38 ファイル）`。

## 9. Issue #4 — 部品の核を core に置く（2026-10-21）

同じコンテナ。`crates/core`（`nene-docket-core`・`#![no_std]`・依存 0）が生まれた。rustc 1.98.0 / cargo-llvm-cov 0.9.1。

### 9.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` | 差分なし |
| `make lint` | 0 件（`#[expect]` は 0） |
| `make lint-windows` | 0 件 |
| `make test` | core 49・xtask 81、すべて成功 |
| `make conformance` | 違反なし（68 ファイル） |
| `make doc-check` | 警告なし |
| `make coverage` | core: regions 99.71%（2442 中 7 が未到達）・functions 100%・lines 100%。下限 90 で終了 0 |

### 9.2 実ファイルでの発火

| 規則 | やったこと | 実出力 |
| --- | --- | --- |
| CNF-007（ARC-003） | `crates/core/src/lib.rs` の `#![no_std]` を注釈に | `CNF-007: crates/core/src/lib.rs — nene-docket-core は中核クレートだが、先頭の属性に #![no_std] が無い（ARC-003）` |
| ARC-003（構造） | `lib.rs` の末尾に `std::time::SystemTime::now()` を呼ぶ関数を足して `cargo check -p nene-docket-core` | `error[E0433]: cannot find module or crate std in this scope`。`no_std` なので lint 以前に名前解決で落ちる |
| CNF-012（ARC-007） | 同じ関数で `make conformance` | 初回は**鳴らなかった**（第 6 節の穴）。検査器を直した後: `CNF-012: crates/core/src/lib.rs:46 — std::time はアダプタと store の外で読まない（ARC-007）` |
| CNF-003（RS-012） | `pixel_point.rs` のテストの前に 2 つ目の `pub struct Second;` | `CNF-003: crates/core/src/pixel_point.rs:37 — Second は 2 つ目の主要宣言（1 つ目は PixelPoint）。ファイルを分ける（RS-012）` |
| CNF-013（ARC-002 / ARC-013） | `crates/core/Cargo.toml` に `regex = "1"`（`--offline` で実行） | `CNF-013: crates/core/Cargo.toml:16 — regex への依存は crates/core/ に許されていない（ARC-002 / ARC-013）` |
| QLT-009 | `cargo llvm-cov -p nene-docket-core --locked --summary-only --fail-under-regions 100` | 表は同じ値（99.71%）で出し、**終了 1**。下限 90 の `make coverage` は終了 0 |

元に戻した後の `make conformance`: `規約違反なし（検査 68 ファイル）`。

### 9.3 まだ証明していないこと（Issue #4）

- 1 万件の表の配置が 16 ms に収まること（FR-002）。`TableLayout::visible_rows` が見えている行だけを返すことは単体テストで見たが、時間は窓を出してから測る
- 描画命令が 2 OS で同じ画素になること。UI クレートが無い

## 10. PR #3 — CI の `make ci` の実行証拠（2026-10-21）

PR #3 を draft から Ready にした。`check.yml` の `check`（ubuntu-24.04・actions/checkout）が実行 [39517454232](https://github.com/hideyukiMORI/NeNeDocket/actions/runs/39517454232) で `make ci` を呼んだ。

| 項目 | 実出力 |
| --- | --- |
| CI の環境変数 | `PR_HEAD_REF: claude/cloud-session-qus6l5`・`PR_BASE_SHA: 60350f9…`・`PR_DRAFT: false` |
| `cargo run --quiet -p xtask --locked -- validate-git` | `GIT-002: invalid branch name claude/cloud-session-qus6l5`・`make: *** [Makefile:60: validate-git] Error 1`・終了 2 |
| GIT-001 / GIT-003 / GIT-004 の欄 | 報告なし（`Closes #1`・全欄・GIT-003 の形のタイトルと 3 コミットは通った） |
| 手元の `make validate-git` | 同じ 1 行（第 8.1 節） |

CI と手元が同じ検証器で同じ結果を出した（QLT-005）。落ちた理由は harness の枝名で、PR 本文に書いた逸脱そのもの。ruleset が無いので、この失敗は merge を止めない（GIT-002 / GIT-004 が planned のまま、の理由）。

## 11. Issue #5 — X11 の窓（2026-10-21）

同じコンテナ。`crates/application`（`#![no_std]`）・`crates/ui-x11`（`x11rb` 0.14 ＋ `cairo-rs` 0.22）・`crates/app` が生まれた。Xvfb / xdotool / ImageMagick / IPA ゴシック。

### 11.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` | 差分なし |
| `make lint` | 0 件。`#[expect]` は `crates/app/src/main.rs` の `print_stderr`（RS-014）だけ |
| `make lint-windows` | 0 件（`xtask`・`core`・`application`。`ui-x11` と `app` は除外） |
| `make test` | core 51・application 11・ui-x11 16・xtask 81、すべて成功 |
| `make conformance` | 違反なし（98 ファイル） |
| `make deny` | ok。`cargo deny list` の実測で `allow` を MIT / Apache-2.0 / Apache-2.0 WITH LLVM-exception にした（`deny.toml` の注釈） |
| `make doc-check` | 警告なし |
| `make coverage` | core ＋ application: regions 99.55%・functions 99.62%・lines 99.69% |

### 11.2 実ファイルでの発火

| 規則 | やったこと | 実出力 |
| --- | --- | --- |
| CNF-007（ARC-003） | `crates/application/src/lib.rs` の `#![no_std]` を注釈に | `CNF-007: crates/application/src/lib.rs — nene-docket-application は中核クレートだが、先頭の属性に #![no_std] が無い（ARC-003）` |
| CNF-012（ARC-007） | `crates/ui-x11/src/flow.rs` に `std::env::var("HOME")` を足した | `CNF-012: crates/ui-x11/src/flow.rs:13 — std::env はアダプタと store の外で読まない（ARC-007）` |
| CNF-013 | `crates/ui-x11/Cargo.toml` に `nene-docket-store` を足した（`--offline`） | **鳴らなかった。** `store` はまだ無いので cargo が先に落ち、`cargo run --offline -p xtask` 自体が組めない。xtask のテスト `cnf013_catches_ui_depending_on_adapters` が同じ向きを見ている。実ファイルは store ができてから |

### 11.3 Xvfb での表示と操作（環境検証・単体テストではない・QLT-013）

Xvfb `:97`（1024×700×24・`-noreset`）、WM 無し。`target/debug/nene-docket` を起動し、`xdotool` で駆動し、`import -window root` で撮った。
窓は 960×600、`Xft.dpi` は無いので 1x。

| 手順 | `xdotool` | 見えたもの | PNG |
| --- | --- | --- | --- |
| 起動 | | 検索欄・絞り込み `All`・`Show done`・`Add`・ヘッダと 19 行。日本語の題名が IPA P ゴシックで出る | [01-initial](screenshots/2026-10-21-xvfb-01-initial.png) |
| 3 行目をクリック | `mousemove 200 150 click 1` | `#3` の行が選択の色 | （04 に同じ） |
| 検索欄をクリックして `Windows` を打つ | `mousemove 100 22 click 1; type Windows` | 縁が強調の色、文字とカーソル。表が `Windows` を含む行だけになる | [03-search](screenshots/2026-10-21-xvfb-03-search.png) |
| Shift+Left × 3 | `key shift+Left ×3` | `ows` が選択の色、カーソルは `d` の右 | [04-selection](screenshots/2026-10-21-xvfb-04-selection.png) |
| Esc → 絞り込みの箱をクリック | `key Escape; mousemove 660 22 click 1` | 候補 4 つが表の上に重なって出る。`All` が光る | [05-filter-open](screenshots/2026-10-21-xvfb-05-filter-open.png) |
| `Done` をクリック | `mousemove 660 134 click 1` | 箱が `Done`、表が `Done` の行だけ | [06-filter-done](screenshots/2026-10-21-xvfb-06-filter-done.png) |
| ホイール下 × 5 | `click 5 ×5` | 15 行分送られ、つまみが下がる | [07-scrolled](screenshots/2026-10-21-xvfb-07-scrolled.png) |
| `Show done` をクリック | `mousemove 850 22 click 1` | チェックが外れ、`Done` の行が消える | [08-hide-done](screenshots/2026-10-21-xvfb-08-hide-done.png) |
| `xdotool windowclose` | | WM が無いので `WM_DELETE_WINDOW` は来ず、窓が直接消える。`DestroyNotify` で戻るようにして終了コード 0 |

🔴 **見つかった欠陥:** 最初の起動では日本語が全部豆腐だった。cairo の toy text API は fallback しない（ADR 0007）。
「日」と U+E000 の `text_extents` を比べる判定は `sans-serif` が Inter に解決すると誤判定した。`text_to_glyphs` の字形番号 0 で判定して `IPAPGothic` が選ばれた。
絞り込みの印 `▾` も DejaVu に無く豆腐だったので、字形でなく矩形 4 段の三角にした（core の `select_box`）。
最初の試行では xdotool の x 座標（740）が絞り込みの箱（596〜736）と `Show done`（744〜）の隙間に当たり、候補が開かなかった。アプリの不具合ではない。

### 11.4 1 万件の表の 1 フレーム（FR-002 / FR-017）

リポジトリの外の計測用クレート（scratchpad・`std::time` は製品クレートでは禁止）が、`DocketScreen::step` ＋ `Canvas::render`（cairo・960×600・`IPAPGothic`）を 200 フレーム回した。release ビルド。

| 課題の数 | `step` | `render` | 描画命令の数 |
| --- | --- | --- | --- |
| 100 | 0.020 ms | 0.872 ms | 124 |
| 10,000 | 0.167 ms | 0.693 ms | 124 |
| 100,000 | 1.866 ms | 0.922 ms | 124 |

1 万件で 1 フレーム 0.9 ms。目標 16 ms の 1/17。10 万件でも 2.8 ms。`step` が件数に比例するのは絞り込みが全件を舐めるためで、配置と描画は見えている行だけ（命令の数が変わらない）。

### 11.5 まだ証明していないこと（Issue #5）

- **IME。** コンテナに IM サーバ（fcitx / ibus）が無く、XIM の preedit は測れない。施主の WSLg で測る（todo 第 3 項）
- WM のある環境での `WM_DELETE_WINDOW`・題・大きさの変更。Xvfb には WM が無い
- Windows の見え（Windows 版はまだ無い）
- `Xft.dpi` が 144 以上のときの 2x の見え（`UiScale::Double` は core のテストだけ）

## 12. Issue #7 — 課題・操作・論理時計・併合（2026-10-21）

同じコンテナ。core に 14 モジュールを足し、`ItemState` を application から core へ移した。表示は無い。

| target | 結果 |
| --- | --- |
| `make lint` / `fmt-check` / `doc-check` | 0 件・差分なし・警告なし |
| `make test` | core 68・application 10・ui-x11 16・xtask 81、すべて成功 |
| `make conformance` | 違反なし（111 ファイル） |
| `make coverage` | core ＋ application: regions 99.58%。`docket.rs` / `merge.rs` / `item.rs` は 100% |

順序に依らないことのテスト（QLT-008・ADR 0003 の 7）: `the_same_ops_in_any_order_give_the_same_docket` が、2 アカウント 7 操作の集合を、元の順・逆順・3 回転・アカウントごとに分けて B → A・ばらばらの添字順の 5 通りで併合し、`Docket` が等しいことを見る。
同時の操作（時計 3 の A と B）はアカウントの番号が大きい B が勝ち、負けた A の操作が `history` の 3 番目に残る。同じ操作を 2 度与えても 1 度だけ数える。宛先の無い操作は落とさず数える（2 件）。

## 13. Issue #9 — store のジャーナル版 1（2026-10-21）

同じコンテナ。`crates/store`（標準ライブラリだけ・外部クレート 0）。テストは一時ディレクトリ（`std::env::temp_dir()`。store は CNF-012 の除外なので読める）。

| target | 結果 |
| --- | --- |
| `make lint` / `fmt-check` / `doc-check` | 0 件・差分なし・警告なし |
| `make test` | store 20・core 68・application 10・ui-x11 16・xtask 81、すべて成功 |
| `make conformance` | 違反なし（124 ファイル） |
| `make deny` | ok（外部クレートは増えていない） |

実ファイルの発火: `crates/store/Cargo.toml` に `serde = "1"` を足すと `CNF-013: crates/store/Cargo.toml:17 — serde への依存は crates/store/ に許されていない（ARC-002 / ARC-013）`。戻して違反なし。

確かめたこと（単体テスト）:

- 全部の `OpKind` と `FieldValue`（タブ・改行・`\`・`,` を含む題名、空の本文、状態、担当、期限の有無、ラベルの有無、負の並び順、打ち消す操作）が 1 行を往復して等しい
- 1 文字変えた行は `ChecksumMismatch`。版・数・アカウント・種類・項目・値・欄の数のそれぞれに失敗の経路がある
- 改行の無い末尾は読まずに次回へ回し、途中の壊れた行は位置と理由を覚える。末尾が完成すれば次の読み直しで読める
- 追記 → `sync` → 開き直しで同じ操作が読める。落ちた直後を真似た断片は数えて捨てる
- 他人のジャーナルは伸びた分だけ返り、自分のジャーナルは返らない。ファイルから読んだ操作の併合が、メモリの操作の併合と等しい
- `docket.v1.txt` は無ければ作り、在れば名前を保ち、版が違えば `UnknownDocketVersion`。ジャーナルでない名前のファイルは失敗として返す

🔴 **まだ証明していないこと:** SMB の上での同時の追記と読み直し（QLT-013・施主の共有で）。2 プロセスが同じ機械で同じフォルダを同時に使うこと（一時ディレクトリのテストは 1 プロセスの中で 2 アカウント）。

## 14. Issue #11 — 画面とジャーナルの配線（2026-10-21）

同じコンテナ。application に `Ledger`・`UndoStack`・`Ports`・`JournalPort` / `ClockPort`・`Notice`・`account_file`、store に `FileJournal`、
新しい `crates/adapters-x11`（`SystemClock`・`saved_account`・`RefreshTicker`・ADR 0008）、`crates/app` に引数（`nene-docket [FOLDER]`）。見本の課題（`SampleItem`）は消した。

### 14.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` | 差分なし |
| `make lint` | 0 件。`#[expect]` は `crates/app/src/main.rs` の `print_stderr` だけ |
| `make lint-windows` | 0 件（`adapters-x11` も除外に足した。X が無い） |
| `make test` | core 69・application 29・store 22・adapters-x11 3・ui-x11 16・app 3・xtask 81、すべて成功 |
| `make conformance` | 違反なし（144 ファイル） |
| `make deny` | ok。`adapters-x11` の外部は x11rb だけで、新しいクレートは増えていない（`Cargo.lock` の差分は内部クレートの 3 行） |
| `make doc-check` | 警告なし |
| `make coverage` | core ＋ application: regions 99.54%・functions 99.58%・lines 99.73% |

### 14.2 実ファイルでの発火

| 規則 | やったこと | 実出力 |
| --- | --- | --- |
| CNF-012（ARC-007） | `crates/app/src/x11_root.rs` に `use std::time::Duration;` を書いた（刻みの間隔を `Duration` で渡そうとした） | `CNF-012: crates/app/src/x11_root.rs:3 — std::time はアダプタと store の外で読まない（ARC-007）`。間隔は秒の `u64` にして `Duration` はアダプタの中で作る |
| CNF-013（ARC-002） | `crates/ui-x11/Cargo.toml` に `nene-docket-store` を足した（Issue #5 では store が無く組めなかった） | `CNF-013: crates/ui-x11/Cargo.toml:20 — nene-docket-store への依存は crates/ui-x11/ に許されていない（ARC-002 / ARC-013）` |

### 14.3 Xvfb で 2 プロセスが同じフォルダを編集する（環境検証・QLT-013）

Xvfb `:97`（2000×700×24）、WM 無し。`target/debug/nene-docket <一時フォルダ>/shared` を、`XDG_CONFIG_HOME` と `USER` を変えて 2 つ起動した（A = alice、B = bob。アカウントは別々に作られる）。B の窓は `xdotool windowmove` で x = 1000 に寄せた。読み直しの刻みは 3 秒なので、相手の操作の後に 4 秒待って撮った。

| 手順 | 見えたもの | PNG |
| --- | --- | --- |
| A: `Add` → 題名の欄にフォーカス → `Written by A` と打って Enter | A の表に `9283-1 Written by A`。4 秒後、B の表にも同じ行 | [03-b-sees-a](screenshots/2026-10-21-two-03-b-sees-a.png) |
| B: 行を選び、状態の箱を開いて `Done` | B の行が `Done`。4 秒後、A の行も `Done`。A の編集行の箱も `Done` に追従 | [06-a-sees-done](screenshots/2026-10-21-two-06-a-sees-done.png) |
| A: 題名を `Renamed by A` にして Enter → Ctrl+Z → Ctrl+Y | 題名が `Written by A` に戻り、また `Renamed by A` に。4 秒後、B も `Renamed by A` | |
| A: `Delete` | A の表から行が消え、選択が外れる。B はまだ出ている（撮った時点で 0.3 秒） | [10-a-deleted](screenshots/2026-10-21-two-10-a-deleted.png) |
| A: Ctrl+Z | 行が戻る（`restore` の追記） | [11-a-restored](screenshots/2026-10-21-two-11-a-restored.png) |
| A を `kill -9` → 同じフォルダで起動し直す | 同じアカウント（`account.v1.txt`）で開き、表は同じ | |
| A（再起動後）: Ctrl+Z → Ctrl+Y | 束はジャーナルから組み直され、次に戻すのは素の操作 4（題名の変更）。題名が `Written by A` に戻り、Redo で `Renamed by A` に | [13-a-undo-after-restart](screenshots/2026-10-21-two-13-a-undo-after-restart.png) |
| 2 つの窓を `windowclose` | どちらも終了コード 0 | |

終わったあとの `journal/`（A のファイル。B は `3` の 1 行）:

```text
v1	1	9283…	…	-	create	9283…:1	…
v1	2	9283…	…	-	set	9283…:1	title	Written by A	…
v1	4	9283…	…	-	set	9283…:1	title	Renamed by A	…
v1	5	9283…	…	4:9283…	set	9283…:1	title	Written by A	…
v1	6	9283…	…	5:9283…	set	9283…:1	title	Renamed by A	…
v1	7	9283…	…	-	delete	9283…:1	…
v1	8	9283…	…	7:9283…	restore	9283…:1	…
v1	9	9283…	…	4:9283…	set	9283…:1	title	Written by A	…
v1	10	9283…	…	9:9283…	set	9283…:1	title	Renamed by A	…
```

B の操作は論理時計 `3`（A の 1・2 を見たあと）、A の次の操作は `4`（B の 3 を見たあと）で、全順序が 2 プロセスをまたいで繋がっている。
Undo は逆の操作の**追記**で、ジャーナルは一度も書き換えていない（ADR 0004）。`kill -9` の直前の操作まで残っている（追記ごとに `sync`）。

🔴 **見つかったこと:** 最初の試行では B の状態の箱の x 座標を間違え（960 幅の窓では 754〜864）、B の操作が 1 つも書かれなかった。アプリの不具合ではないが、「書かれなかった」ことがジャーナルを見るまで分からなかった。
`Add` は `create` 1 行だけを追記する（検索欄に字があれば `title` も）。担当は空のままで、画面から担当を変える経路はまだ無い。

### 14.4 併合した台帳で 1 万件の表の 1 フレーム（FR-002 / FR-017・PR #13）

第 11.4 節と同じ計測用クレート（scratchpad・release）を、見本の課題でなく**併合した `Docket`**（他人の `create` ＋ `title` ＋ `state` の操作を `JournalPort` から読む）に替えて 200 フレーム回した。

| 課題の数 | 開く（読んで併合） | `step` | `render` |
| --- | --- | --- | --- |
| 100 | 0.2 ms | 0.028 ms | 0.64 ms |
| 10,000 | 16 ms | 0.870 ms | 0.70 ms |
| 100,000 | 212 ms | 14.9 ms | 0.94 ms |

🔴 **見つかった欠陥（PR #13 で直した）:** 配線の直後は 1 万件で `step` が 2.75 ms、10 万件で 48.5 ms だった。`visible_items` を 1 フレームに 2 度（行の数と行の描画）数え、見えている行だけでなく**絞り込んだ全行を複製**していた。
1 フレームに 1 度だけ ID の列を数え、描く行だけを複製するようにして 0.87 ms。残りの大半は `Docket::live_items`（作られた順に `BTreeMap` を引く）で、1 万件では目標 16 ms の 1/10 に収まる。10 万件は目標の外（FR-002 は 1 万件）。

### 14.5 まだ証明していないこと（Issue #11）

- SMB の上（施主の共有で）。2 プロセスは同じマシンの同じディレクトリ
- IME（第 11.5 節と同じ）・WM のある環境・Windows

## 15. Issue #14 — 担当と期限を画面から変える（2026-10-21）

同じコンテナ。application に `EditSlot`（題名・担当・期限の欄と項目の行き来）、`Notice::NotADate`。編集行は題名・担当・期限・状態・Delete の 5 列、表に期限の列。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `doc-check` | 差分なし・0 件・警告なし |
| `make test` | 227 件（application 33、ほかは変わらず） |
| `make conformance` | 違反なし（145 ファイル） |
| `make coverage` | core ＋ application regions 99.53% |

Xvfb（`:97`・1 プロセス）: `Add` → 題名 → Tab → 担当 `rina` → Tab → 期限に `next week` → Enter。
窓の下に `due: not a date (YYYY-MM-DD), not saved` が出て、ジャーナルには `title` と `assignee` だけが追記された（[01-bad-date](screenshots/2026-10-21-fields-01-bad-date.png)）。
Ctrl+A → `2026-11-30` → Enter で文言が消え、表の `Due` に出て、`due` が追記された（[02-fields](screenshots/2026-10-21-fields-02-fields.png)）。

🔴 `xdotool type` はこのコンテナのロケールで日本語を打てない（`Invalid multi-byte sequence`）。日本語の表示は第 11.3 節で確かめてあり、今回は ASCII の題名で駆動した。

## 16. Issue #16 — 設定ファイル（2026-10-21）

同じコンテナ。application に `Settings`（最近の台帳 10 件・読み直しの間隔）と `settings_file`（`version = 1` / `refresh_seconds` / `recent`）、
adapters-x11 に `config_directory`・`saved_settings`（読む・開いた台帳を絶対の道で覚えて丸ごと書く）。app は引数が無ければ最近の台帳、それも無ければ `./docket`。刻みの間隔は設定から。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 233 件（application 37・adapters-x11 5） |
| `make conformance` | 違反なし（149 ファイル） |
| `make coverage` | core ＋ application regions 99.54% |

Xvfb（`:97`・`XDG_CONFIG_HOME` を一時フォルダに）:

1. `nene-docket <一時フォルダ>/team` で開き、課題を 1 つ足して閉じる → `settings.v1.txt` に `recent = /…/recent/team`（絶対の道）
2. 別の作業フォルダから引数なしで `nene-docket` → 同じ台帳が開き、`remembered` の行が出る（[01-reopened](screenshots/2026-10-21-settings-01-reopened.png)）。作業フォルダに `./docket` は作られない
3. 設定に知らない鍵 `theme = dark` を書いて起動 → `NeNe Docket 0.1.0: settings file is not readable: /…/settings.v1.txt`、終了コード 1。既定値へ落ちない

## 17. Issue #18 — 課題の履歴の段（2026-10-21）

同じコンテナ。core の `WallClock::text`（UTC の `YYYY-MM-DD HH:MM`）、application の `HistoryLine`（いつ・誰が・何を・何から何へ。新しいものが先）と `HistoryPane`（表の下 2/5・自分のスクロール）、道具の段の `History` のチェック。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 240 件（core 70・application 43） |
| `make conformance` | 違反なし（151 ファイル）。途中で CNF-003 が `history_line.rs` の 2 つ目の宣言（私用の trait）を拒み、関数にした |
| `make coverage` | core ＋ application regions 99.22%（`history_pane.rs` は 91%: スクロールバーの掴みが未テスト） |

Xvfb（`:97`）: `Add` → 題名 → 状態を `Done` → 題名を変えて Enter → Ctrl+Z → `History` を入れる。
表の下に 5 行（`undo: title : Snapshot ADR -> Decide the snapshot format` / `title : … -> Snapshot ADR` / `state : Open -> Done` / `title :  -> …` / `created`）。Undo の行は控えめな色（[01-pane](screenshots/2026-10-21-history-01-pane.png)）。

🔴 **見つかった欠陥（この PR で直した）:** 最初のスクリーンショットには 6 行目 `title : Decide … -> Snapshot ADR` が余計に出ていた。題名の欄にフォーカスがあるまま Ctrl+Z すると、欄には戻す前の文字が残り（入力中の欄は読み直しで触らない規則）、
フォーカスが外れた瞬間に「欄と課題が違う」と判定して戻す前の値を追記していた。Undo / Redo / Delete の後は入力中の欄も強制的に合わせるようにした（`reload_editor(true)`）。テスト `undo_while_a_field_is_focused_reloads_it_so_nothing_is_re_appended`。

## 18. Issue #20 — ラベルを画面から変える（2026-10-21）

同じコンテナ。`EditSlot::Labels`（`,` 区切り・前後の空白を落とす・空は無し）、編集行の 4 つ目の欄、表の `Labels` の列、検索欄は題名かラベルに含まれば出す。確定した欄は正規の形（`a, b`）に揃える。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 241 件（application 44） |
| `make conformance` | 違反なし（151 ファイル） |
| `make coverage` | regions 99.22% |

Xvfb（`:97`）: 課題を 2 つ足し、1 つ目にラベル `ui,table , later`、2 つ目に `adr`。ジャーナルには `labels	ui,table,later` と `labels	adr`（`,` は `\,` に逃がして 1 行）。
検索欄に `table` で 1 つ目だけが出る（[01-labels-search](screenshots/2026-10-21-labels-01-search.png)）。

## 19. Issue #22 — Windows の窓（2026-10-21）

同じコンテナ（Linux）。`apt-get install mingw-w64` でリンカを入れた。新しい `crates/adapters-windows`（`SystemClock`・`saved_account`・`saved_settings`・`desktop_dpi`・`random_id`）と
`crates/ui-win32`（`docket_window`・`windows_app`・`pen`・`text_line`・`key_input`・`char_assembler`・`pointer_state`・`win32_failure`）、`crates/app` の `windows_root`。`palette` と `Rgb` を core へ移した（ADR 0009）。

### 19.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `doc-check` | 差分なし・0 件・警告なし |
| `make lint-windows` | 0 件（`xtask`・`core`・`application`・`store`・`adapters-windows`・`ui-win32`・`app` を `x86_64-pc-windows-gnu` で。`#[expect(unsafe_code, reason = "RS-010: …")]` は 2 クレートに 30 か所、すべて発火） |
| `make build-windows` | `target/x86_64-pc-windows-gnu/release/nene-docket.exe`（1,919,311 バイト） |
| `make test` | 255 件（ui-win32 8・adapters-windows 4 は Linux で走る純粋な部分） |
| `make conformance` | 違反なし（173 ファイル）。途中で CNF-001（`CharAssembler` の `derive(Default)`）と CNF-003（`pen.rs` の 2 つ目の型）が落ち、直した |
| `make deny` | `windows` の手続きマクロが引く `unicode-ident` の `Unicode-3.0` で `licenses FAILED` → 実測して `deny.toml` の `allow` に足した（ADR 0009 の 7）。足した後 ok |
| `make coverage` | core ＋ application regions 99.19% |

### 19.2 実ファイルでの発火

| 規則 | やったこと | 実出力 |
| --- | --- | --- |
| CNF-014（ADR 0002 の 4） | `crates/ui-win32/Cargo.toml` の `unwrap_used = "forbid"` を `"deny"` にした | `CNF-014: crates/ui-win32/Cargo.toml:1 — [lints.clippy] が workspace の表と違う。違ってよいのは unsafe_code = "deny" だけ（ADR 0002）` |
| CNF-013（ARC-002） | `app` の `[target.'cfg(windows)'.dependencies]` に 2 クレートを足した | 通る（許可表にある）。`ui-x11` に `store` を足す反例は第 14.2 節 |

### 19.3 証明していないこと（Issue #22・QLT-013）

🔴 **Windows では一度も動かしていない。** 組み立て（clippy 0 件・exe）までが証拠で、窓が出ること・表と入力欄が描けること・Segoe UI と日本語の fallback・`WM_CHAR` の日本語・2 プロセスの 3 秒ごとの読み直し・`BCryptGenRandom` のアカウント・`%APPDATA%` の設定は、施主の Windows で確かめるまで「動く」と書かない。
確かめ方: `make build-windows` の exe を Windows に持ち込み、`nene-docket.exe <フォルダ>` で開く。2 つ目は別の `APPDATA`（`set APPDATA=...`）で起動すると別のアカウントになる。

## 20. Issue #24 — FR-017 の実測: 1 万件・1 年分のジャーナルの併合（2026-10-21）

リポジトリの外の計測用クレート（scratchpad・release・このコンテナ）。3 アカウント・1 万件・1 件あたり 30 操作（`create` ＋ 題名・状態・本文の `set`。1 年に 1 件 30 回触る想定）＝ **30 万操作・45.2 MB** を一時フォルダに書き、
`FileJournal::open`（自分のジャーナルを読む）→ `refresh`（他人の分を読む）→ `Ledger::open`（併合）の時間を測った。2 回目は OS のキャッシュが効いている。

| 段 | 1 回目 | 2 回目 |
| --- | --- | --- |
| 自分のジャーナルを読む（10 万行） | 0.167 s | 0.143 s |
| 他人の 2 本を読む（20 万行） | 0.538 s（累計） | 0.467 s |
| 併合して 1 万件 | **0.677 s（累計）** | 0.597 s |

目標 1 秒（FR-017）の中だが余裕は 1/3 で、SMB の上の 1 回目はこれより遅い。スナップショットの ADR はまだ起こさない（ADR 0004 の 71 行目の条件に達していない）。
読むのは起動時だけで、以後の読み直しは伸びた分だけ（ADR 0004 の 8）。署名を付ける（ADR 0010 の提案）と行が 130 バイト増え、約 85 MB になる。

## 21. Issue #27 — 表示言語（6 言語）と配色（2026-10-21）

同じコンテナ。core に `Language` / `LanguageChoice` / `Word` / `wording`（6 × 31 語）、`Appearance` / `Palette` / `ThemeSeed` / `theme_catalog`（nene-nib の 125 配色）/ `theme_derivation` / `Theme` / `ThemeChoice`。
application に設定の版 2・`SettingsPort`・`Environment` の言語と明暗・`DocketFrame` の言語と配色・道具の段の言語の箱と配色の欄。adapters に `locale` / `appearance` / 版 2 の読み書きと版 1 からの移行。UI はフレームの配色と言語で描く（ADR 0011 / 0012）。

### 21.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make build-windows` | exe（Windows では未実行のまま） |
| `make test` | 274 件（core 82・application 51・adapters-x11 6・adapters-windows 5・ui-x11 16） |
| `make conformance` | 違反なし（194 ファイル） |
| `make deny` | ok（外部クレートは増えていない） |
| `make coverage` | core ＋ application regions 99.12% |

テストが要求すること: 125 配色すべてで字 / 背景のコントラスト 4.5 以上・字 / 面 4.0 以上、名前の一意性と `_` の別綴り、6 言語 × 31 語が空でない、設定の版 2 の往復と版 1 の読み込み（`language` を含む版 1 は拒む）、OS の言語の解決（`zh_HK` → 繁体字・`C` → `None`）。

### 21.2 Xvfb（`:97`・`LANG=ja_JP.UTF-8`・QLT-013）

| 手順 | 見えたもの | PNG |
| --- | --- | --- |
| 起動（設定は無し＝ `system`） | OS の言語を日本語に解決。道具の段・列の見出し・状態・履歴の語が日本語。字は IPA P ゴシック | [01-ja-system](screenshots/2026-10-21-i18n-01-ja-system.png) |
| 言語の箱で 简体中文 | 全部の語が簡体字。字形は文泉驛正黑（IPA には無い字をコンテナの fontconfig が持っていた） | [03-zh-hans](screenshots/2026-10-21-i18n-03-zh-hans.png) |
| 한국어 | ハングル。同じく文泉驛正黑 | [05-ko](screenshots/2026-10-21-i18n-05-ko.png) |
| Español → 配色の欄に `dracula` と Enter | 紫の強調のダーク。面・縁・選択・控えめが導出された色 | [07-dracula](screenshots/2026-10-21-i18n-07-dracula.png) |
| 欄で Down | `one-dark` に巡る | |
| 欄に `solarized_light` と Enter | `_` を `-` と同じに受け、ライトの配色 | [09-solarized-light](screenshots/2026-10-21-i18n-09-solarized-light.png) |
| 閉じる | `settings.v2.txt` に `language = es` と `theme = solarized-light`、終了コード 0 | |

繁體中文と English も撮った（02 / 04 / 06・docs には置かない）。`xdotool type` は ASCII だけなので題名は英語。

🔴 **正直に:** 翻訳 6 言語 × 31 語は自分で書き、母語話者の目を通していない。導出した 8 色の見た目は仮（ADR 0012 の 2）。既定のライトが `neutral-light`（橙の強調）になり、Issue #5〜#20 の青い強調から変わった。

## 22. Issue #29 — 署名と権限（ADR 0010 受理・2026-10-21）

同じコンテナ。core に `ed25519-dalek`（唯一の外部クレート）と `hex` / `SigningKey` / `PublicKey` / `Signature` / `Permissions`。application に `account_file` 版 2・`key_record`・`permissions_file`・`AdmissionReport`・履歴の「誰が」の名前。store にジャーナル版 2（署名付き）・`keys/`・`permissions.v1.txt`・`verifier`・`administration`。adapters に鍵の乱数と `account.v2.txt`（版 1 から移行）。app に持ち主の旗（`--permissions` / `--allow` / `--disallow` / `--group` / `--ungroup`）。

### 22.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make build-windows` | exe 2.4 MB（2.0 MB から・Windows では未実行のまま） |
| `make test` | 306 件（core 89・application 56・store 39・adapters-x11 8・adapters-windows 5・xtask 81） |
| `make conformance` | 違反なし（210 ファイル）。CNF-013 は core の `ed25519-dalek` を通し、他のクレートの外部クレートは今まで通り拒む（xtask のテスト） |
| `make deny` | ok。`BSD-3-Clause` を足す前は `licenses FAILED`（ed25519-dalek / curve25519-dalek / subtle）で落ちることを実測してから足した |
| `make coverage` | core ＋ application regions 99.03% |

テストが要求すること: 鍵の 16 進の往復と `Debug` が秘密を出さないこと、署名が持ち主の公開鍵でだけ通ること（1 ビット変えれば落ちる）、版 2 の行の往復・署名の無い版 2 の行は読めない（`BadSignature`）・値を書き換えて検査和だけ直した行は署名で落ちる、編集者の名で別の鍵が署名した行（なりすまし）・編集者でない人の行・鍵の無い人の行・版 1 の行がそれぞれの理由で捨てられ数えられること、`permissions.v1.txt` の署名が持ち主の鍵でだけ本物になること（持ち主でない署名・持ち主の欄の改変・鍵の無い持ち主は起動を止める）、持ち主でない人の `--permissions` は何も書かないこと、`account.v1.txt` が鍵を足して版 2 になり版 1 が残ること。

### 22.2 Xvfb（`:96`・`LANG=en_US.UTF-8`・3 アカウント・QLT-013）

| 手順 | 見えたもの | PNG |
| --- | --- | --- |
| alice が台帳を作り 1 件書く（権限なし） | `journal/<alice>.v1.log` に 2 行 | |
| bob と carol が開いて閉じる | `keys/` に 3 人の公開鍵と名前 | |
| `nene-docket shared --permissions on --allow <bob>`（alice） | `docket.v1.txt` に `owner` と `permissions = on`、`permissions.v1.txt` に `editor = <bob>` と署名。終了コード 0 | |
| 同じ命令を carol が | 終了コード 1・`only the docket owner may change permissions`。ファイルは変わらない | |
| 3 人が開き、それぞれ 1 件足す | alice と bob は互いの課題を見て carol の課題は見ない。窓の下に `lines failing signature or permission: 4`（alice の版 1 の 2 行 ＋ carol の 2 行）。carol は自分の課題を見るが `not an editor of this docket; changes are ignored` | [03-after-edits](screenshots/2026-10-21-permissions-03-after-edits.png) |
| alice が bob の行を選び History | 「誰が」が `keys/` の名前 `bob` | [04-alice-history](screenshots/2026-10-21-permissions-04-alice-history.png) |
| ジャーナル | 3 人とも `.v2.log` に署名 128 桁付きの行。alice の `.v1.log` は残る | |

🔴 **正直に:** 権限を有効にする前の履歴（alice の 2 行）は捨てられて数に出る（ADR 0010 の 4）。権限を変える画面は無い。秘密鍵は平文。SMB の上と Windows は施主の実機で。

## 23. Issue #31 — 本文（複数行の欄・2026-10-21）

同じコンテナ。core に `text_area`（行ごとの `Text` 命令・カーソルの行を追う・行をまたぐ選択は棒を 1 本だけ `TextCaret::without_bar`）と `TextEdit` の行（Home / End は行の中・`applied_in_area` で Enter は改行、上下は隣の行）。application に `EditSlot::Body`・`BodyPane`・道具の段の「本文」・表の下の段（履歴と両方なら 1/4 ずつ）・本文の検索・履歴の `↵`。UI は棒の無いカーソルを描かない。

### 23.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 317 件 |
| `make conformance` | 違反なし（212 ファイル） |
| `make coverage` | core ＋ application regions 99.08% |

テストが要求すること: 行と桁の往復、Home / End が行の中で止まること、上下が端で先頭・末尾に落ちること、Enter が改行を挿すこと、行ごとの命令でカーソルの行にだけ棒が出て選択は行ごとに切れること、下にはみ出すカーソルの行まで上を省くこと、段の Esc で `Set body` が 1 つ追記されること、別の行を選ぶと欄が入れ替わり前の値が追記されること、検索が本文に効くこと、履歴が改行を `↵` で 1 行にすること、Tab の順（本文のチェックは履歴の次）。

### 23.2 Xvfb（`:95`・2 アカウント・QLT-013）

| 手順 | 見えたもの | PNG |
| --- | --- | --- |
| alice が追加し、「本文」を押す | 表の下に空の段 | |
| 段を押して `first line` Enter `second line` | 2 行。Up Home Shift+End で 1 行目が選択され、棒は 1 本 | [03-a-selection](screenshots/2026-10-21-body-03-a-selection.png) |
| Esc | ジャーナルに `set … body first line\nsecond line` が 1 行（改行は逃がし） | |
| bob が行を選び「本文」と「履歴」を押す | 段に 2 行、履歴に `Body :  -> first line↵second line`、表は残る | [05-b-body-and-history](screenshots/2026-10-21-body-05-b-body-and-history.png) |
| bob が検索に `second` | 1 件（本文に効く） | |

🔴 **正直に:** 本文の段にスクロールバーは無い（カーソルの行を追うだけ）。段の高さは変えられない。`xdotool` の 1 回目はクリックが段の 1 画素上に当たり、何も起こらなかった（手順の誤り・2 回目で記録）。

## 24. Issue #33 — 参加者の段と画面からの権限（2026-10-21）

同じコンテナ。application に `AdminCommand`（store から移動）・`Member` / `MemberRole`・`MembersPane`（名前・ID・役、持ち主には 4 列目に「権限」のチェック、行の押下で編集者の切り替え）・`JournalPort::owner / permissions / administer`・`JournalFailure::PermissionsUnwritable`。store の `FileJournal::administer` は書いた後に `docket.v1.txt`・鍵・権限を読み直し、有効・無効が変われば自分のジャーナルを版を変えて開き直す。道具の段が 9 列になったので窓の初期の幅を 960 → 1120 に（X11 / Win32）。文言 9 語 × 6 言語。

### 24.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 322 件 |
| `make conformance` | 違反なし（216 ファイル） |
| `make coverage` | core ＋ application regions 98.97% |

テストが要求すること: 役が権限から決まり一覧が ID の順であること、段の「権限」で持ち主と閲覧のみに変わり行の押下で編集者と閲覧のみを行き来し持ち主の行は変わらないこと、書けなければ通知が出ること、ポートの `administer` が持ち主以外で失敗し有効にすると版 2 で書き無効に戻せること。

### 24.2 Xvfb（`:94`・3 アカウント・QLT-013）

| 手順 | 見えたもの | PNG |
| --- | --- | --- |
| alice が台帳を作り、bob と carol が参加して閉じる | `keys/` に 3 人 | |
| alice が「参加者」を押す | 名前・ID・役（誰でも編集）と、持ち主なので「権限」のチェック | |
| 「権限」を押す | alice が持ち主、他は閲覧のみ。`docket.v1.txt` に `permissions = on` | |
| bob の行を押す | bob が編集者。`permissions.v1.txt` に `editor = <bob>` と署名 | |
| bob と carol が開いて 1 件ずつ足す | alice と bob は bob の課題を見て carol の課題は見ず、carol には「not an editor」 | [04-three](screenshots/2026-10-21-members-04-three.png) |
| alice が「権限」を外す | `permissions = off`。開き直した人から誰でも編集 | |

🔴 **正直に:** 権限の変更は、既に読み進めた行には開き直すまで効かない（ADR 0010 の 4）。グループは画面から触れない（コマンドラインだけ）。段にスクロールは無い。1 回目の実行は bob が先に台帳を作って持ち主になり（手順の誤り）、2 回目は通知の行で段が上がってチェックの代わりに carol の行を押した（手順の誤り・3 回目で記録）。

## 25. Issue #35 — 履歴から選び直す・上書きの通知（FR-008・2026-10-21）

同じコンテナ。application: `HistoryLine::change`、`HistoryPane` の行（ID 4000〜）を押すと `Set` の行の値を新しい `Set` として追記（いまと同じ値・作成・削除・復元の行は何もしない）。編集欄を読み込んだ時点の課題（`base`）を持ち、確定のとき他人がその後に変えていれば `Notice::Overwrote`（`Word::Overwrote`・6 言語）。Undo / Redo のテスト（他人の操作が間に入っても自分の分だけ戻す・本文とラベルも戻る・新しい操作で Redo が消える）。

### 25.1 ゲート

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 327 件 |
| `make conformance` | 違反なし（216 ファイル） |
| `make coverage` | core ＋ application regions 99.00% |

### 25.2 Xvfb（`:93`・2 アカウント・QLT-013）

| 手順 | 見えたもの | PNG |
| --- | --- | --- |
| alice が `Original` を作る。bob が題名の欄に `By bob` を打ちかける（確定しない） | | |
| alice が `By alice` にして確定し Esc | bob の欄は打ちかけのまま | |
| bob が Enter | 題名は `By bob`（最後の書き手）。bob に `overwrote a change by someone else; pick again from the history` | |
| alice が履歴を出し `Title :  -> Original` の行を押す | 新しい `Set`（時計 5）が追記され題名は `Original`。bob にも届く | [04-alice-repicked](screenshots/2026-10-21-conflict-04-alice-repicked.png) |

🔴 **正直に:** 1 回目は alice の欄がフォーカスを持ったままで、履歴のチェックを押した瞬間に欄の確定が走り（Issue #11 からの振る舞い）、alice 側にも上書きの通知が出た。FR-008 の規則どおりだが、選び直しは確かめられず、Esc を足した 2 回目で記録した。FR-006 の「過去のどの時点の状態も再現」はまだ（履歴の行を押して 1 項目ずつ戻すだけ）。

## 26. Issue #36 — 本文と参加者の段のスクロールバー（2026-10-21）

同じコンテナ。core の `text_area` は最初に見せる行を受け取るだけになり（`visible_lines` を公開）、追従は application へ。`BodyPane`（ID 20 の溝）はフォーカスがあればカーソルの行が見えるまで送り、ホイールは段の上でだけ 3 行ずつ、つまみのドラッグも受ける。`MembersPane`（ID 21 の溝）は履歴と同じ仕組み。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 328 件 |
| `make conformance` | 違反なし（216 ファイル） |
| `make coverage` | core ＋ application regions 98.66% |
| Xvfb（`:92`） | 15 行の本文を打つと段は 7〜15 行目を見せ、右に溝とつまみ（[01-body](screenshots/2026-10-21-scroll-01-body.png)）。Esc の後のホイールで段だけが動き、表は動かない |

テストが要求すること: 6 行の本文で 3 行の段がホイール 1 刻みで 4〜6 行目になること、段の外のホイールは効かないこと、フォーカスを持って上へ動かすとカーソルの行まで戻ること。

## 27. Issue #39 — 履歴の行の上で過去の時点を再現する（FR-006 後半・2026-10-21）

同じコンテナ。`HistoryLine` が操作の直後の課題（`snapshot`）を持ち、`HistoryPane::line_at` がポインタの下の行を返す。画面は表の下の段を分けた直後にその行の課題を `preview` に取り、表の選んだ行をその時点の値（控えめな色・番号に `←`）で、本文の段を出していればその時点の本文で描く。ジャーナルも欄も変えない。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 330 件 |
| `make conformance` | 違反なし（216 ファイル） |
| `make coverage` | core ＋ application regions 98.67% |
| Xvfb（`:91`・`LANG=ja_JP.UTF-8`） | 題名を 2 度変えた課題で、履歴の「作成」の行にポインタを置くと表の行が `← f4a8-1`・題名なし・未着手になり、離すと戻る（[02-hover](screenshots/2026-10-21-preview-02-hover.png)） |

テストが要求すること: 履歴の 2 行目（In progress -> Done）の上のフレームで表が `Done` と `←` を出し、離したフレームで `In progress` に戻り、`preview` が消えること。各行の `snapshot` が直後の課題（題名・状態・削除）であること。

🔴 **正直に:** 再現はポインタを置いている間だけで、その時点へ台帳を戻す操作ではない（戻すのは Issue #35 の選び直しで 1 項目ずつ）。手順の状態の変更は箱の位置を誤って効かず、題名の変更だけで記録した。

## 28. Issue #41 — 表の列の幅をドラッグで変え、設定の版 3 に残す（FR-012・2026-10-21）

同じコンテナ。application: `TableColumn` / `ColumnWidths`（番号・状態・担当・期限・ラベルの論理画素・40 以上）、`Settings::columns`、設定ファイルの版 3（`columns = 72,110,90,90,120`。版 2 と版 1 は無い鍵を既定で補って読み、前の版に `columns` があれば拒む）。表のヘッダの列の境（ID 5000〜・6 画素）をつまむと境の左の列、題名の右の境なら右の列（状態）の幅が変わり、離したときに設定を書く。adapters: `settings.v3.txt`、無ければ版 2 → 版 1。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 334 件 |
| `make conformance` | 違反なし（218 ファイル） |
| `make coverage` | core ＋ application regions 98.72% |
| Xvfb（`:90`） | 番号の境を 79 → 160、題名の右の境を 689 → 629 にドラッグして閉じると `settings.v3.txt` に `columns = 153,170,90,90,120`。開き直すと同じ幅（[04-reopened](screenshots/2026-10-21-columns-04-reopened.png)） |

テストが要求すること: 境を押して動かすと幅が変わり離すと設定が 1 度書かれること、題名の右の境は状態の幅を変えること、40 未満にならないこと、版 3 の往復・版 2 の読み込み（`columns` 付きは拒む）・版の連鎖（3 → 2 → 1 → 初期値）。

🔴 **正直に:** 列の順（FR-012 の「順」）はまだ。境の見た目は無く（カーソルも変わらない）、つまめる幅は 6 画素。

## 29. Issue #43 — 表の列の順をヘッダのドラッグで変え、設定の版 4 に残す（FR-012 完了・2026-10-21）

同じコンテナ。application: `ColumnOrder`（6 列の並び・`moved_to`）、`TableColumn::keyword / from_keyword`、`Settings::order`、設定ファイルの版 4（`order = …`。版 3・2・1 は既定の順で読み、前の版に `order` があれば拒む）。表のヘッダの見出し（ID 6000〜）をつまんで横に動かし、離した位置の列の前後に入れる（離したフレームで入れる。境のつまみが押下を取っていれば動かない）。見出し・セル・幅はすべて順に従う。adapters: `settings.v4.txt`、無ければ版 3 → 2 → 1。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 338 件 |
| `make conformance` | 違反なし（219 ファイル） |
| `make coverage` | core ＋ application regions 98.74% |
| Xvfb（`:89`） | 「State」の見出しを「#」の上で離して閉じると `order = state,number,title,assignee,due,labels`。開き直すと状態の列が左端（[02-reopened](screenshots/2026-10-21-order-02-reopened.png)） |

テストが要求すること: 見出しを別の列の上で離すと順が変わり設定が 1 度書かれ、同じ場所で離しても書かないこと、見出しとセルの順が変わること、版 4 の往復・版 3 の読み込み（`order` 付きは拒む）・6 列が 1 度ずつ無い `order` の拒否。

## 30. Issue #47 — 参加者の段にグループを出す（2026-10-21）

同じコンテナ。`Member` が入っているグループの名前（名前の順）を持ち、参加者の段の役の欄に `Editor (qa)` のように添える。グループの編集はコマンドラインのまま。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` | 差分なし・0 件・0 件 |
| `make test` | 338 件（`Member::list` のグループ・段の文字列） |
| `make conformance` | 違反なし（220 ファイル） |

coverage は回していない（application の 2 ファイルの小さな差分で、第 29 節の 98.74% から下がる根拠が無い）。

## 31. Issue #48 — X11 で OS の明暗を GTK の慣習から読む（2026-10-21）

同じコンテナ。adapters-x11 の `appearance::read` は `GTK_THEME` が `:dark` / `:light` で終わればそれ、無ければ `$XDG_CONFIG_HOME/gtk-3.0/settings.ini`（無ければ `~/.config/gtk-3.0/settings.ini`）の `gtk-application-prefer-dark-theme`（`1` / `true` / `0` / `false`）、どちらも無ければ `None`（今まで通りライト）。判定は純関数 `from_gtk_theme` / `from_settings_ini`。`config_directory::base` を共通の根にした。GNOME の `color-scheme`（dconf）は読まない。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `doc-check` | 差分なし・0 件・警告なし |
| `make test` | 339 件 |
| `make conformance` | 違反なし（220 ファイル・`env` / `fs` は adapters-x11 の中） |
| Xvfb（`:88`） | `GTK_THEME=Adwaita:dark` の alice と、`settings.ini` に `gtk-application-prefer-dark-theme=1` を置いた bob が、どちらも既定のダーク（ubuntu-aubergine）で開く（[01-dark-both](screenshots/2026-10-21-dark-01-both.png)） |

Windows の lint と coverage は回していない（adapters-x11 だけの差分）。

## 32. Issue #51 — 鍵の交代と持ち主の交代（ADR 0013 受理・2026-10-21）

同じコンテナ。`permissions.v1.txt`（版は 1 のまま）に `succeeds = <新 ID>: <旧 ID>`（0 行以上）と `handover = <前の持ち主>`（交代の直後だけ）が入った。core の `Permissions` は `successions` を持ち、`resolved` が引き継ぎを辿り（輪があっても引き継ぎの数で止まる）、`may_edit` は引き継いだ ID の役を古い ID にも与える。`handed_to` は持ち主を替えて前の持ち主を編集者に残す。store の `administration::apply` は `Succeed` / `Owner` を当て、持ち主が替わったら前の持ち主の鍵で署名して `handover` を書く。`administration::adopt` は台帳を開いた持ち主が、自分の署名でない `permissions.v1.txt`（交代の直後・鍵を失くして `docket.v1.txt` を手で直した後）を受け入れて署名し直す。`Verifier::load` は `handover` があればその鍵で確かめる。`FileJournal::names` は引き継がれた ID に引き継いだ ID の名前を返し、`Member::list` は引き継がれた ID を出さない。コマンドラインは `--succeed NEW OLD` と `--owner ACCOUNT`。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 348 件（core 99・application 82・store 45・app 15） |
| `make conformance` | 違反なし（221 ファイル） |
| `make coverage` | core ＋ application の region 98.72%（閾値 90） |
| `make deny` | 回していない（依存の変更なし） |

Xvfb（`:97`）で 1 度流した（`scratchpad/handover.sh`）:

1. alice が台帳を作って 1 件書き、bob を編集者にして権限を有効にした。bob が 1 件書いた（署名付き）
2. bob が鍵を失くした想定で、別の設定フォルダから同じ名前 `bob` の新しい ID（`4590…`）で参加した
3. 新しい bob の `--owner` は `only the docket owner may change permissions` で止まった（exit 1）
4. alice の `--succeed 4590… 17a6… --owner 4590…` で `docket.v1.txt` の `owner` が `4590…` になり、`permissions.v1.txt` は `editor = 17a6…`（古い bob）・`editor = 30be…`（alice）・`succeeds = 4590…: 17a6…`・`handover = 30be…` を持ち、alice の鍵で署名されていた
5. alice の `--permissions off` は持ち主でないので止まった（exit 1）
6. 新しい bob が開くと `permissions.v1.txt` が bob の鍵で署名し直され、`handover` の行が消えた。参加者の段は alice（Editor）と bob `4590`（Owner）の 2 人で、古い bob `17a6` は出ない（[01-members](screenshots/2026-10-21-handover-01-members.png)）。古い bob の課題の履歴の「誰が」は `bob`（[02-history](screenshots/2026-10-21-handover-02-history.png)）。alice も編集者として開けた

権限を有効にする前の alice の版 1 の 2 行は、これまで通り捨てて数える（`lines failing signature or permission: 2`・ADR 0010）。

## 33. Issue #53 — 参加者の段からグループを変える（2026-10-21）

同じコンテナ。参加者の段に「選んだ行」が入った。持ち主が権限を有効にしていて、持ち主でない人の行を押すと、段の上に編集行（「編集者」のチェックとグループの欄・`,` 区切り）が出る。行の押下は Issue #33 の「編集者の切り替え」から「選ぶ」に変わり、編集者の切り替えは編集行のチェックになった（押すたびに編集者が切り替わる行では、グループを変えるためだけに選べない）。チェックは編集者の一覧にいるかを示す（グループだけで書ける人は空で、役は「編集者 (qa)」のまま）。欄は Enter か、フォーカスを失ったときに確定し、いまのグループとの差分を `Group` / `Ungroup` にする。`:` を含む名前は書かずに示す（`BadGroupName`）。書きかけのまま別の行を押すと先に確定する。権限を無効にすると選びは外れる。部品の ID は 22（編集者）・23（グループ）。文言は 6 言語に `Groups` / `BadGroupName` を足した。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 352 件（application 85） |
| `make conformance` | 違反なし（221 ファイル） |
| `make coverage` | core ＋ application の region 98.77%（閾値 90） |
| `make deny` | 回していない（依存の変更なし） |

Xvfb（`:98`・`scratchpad/groups.sh`）: alice が台帳を作り bob が参加、alice が参加者の段で「権限」を押し、bob の行を押すと編集行が出た。グループ欄に `qa, dev` と打って Enter → `permissions.v1.txt` に `group = dev: …` / `group = qa: …`、役は `Editor (dev, qa)`、欄は正規の形 `dev, qa` に揃う（[01-typed](screenshots/2026-10-21-groups-01-typed.png)）。「編集者」のチェックを押すと `editor = …` の行が足された（[02-editor](screenshots/2026-10-21-groups-02-editor.png)）。行は ID の順なので、手順は 2 つの ID を比べて行を選ぶ。

## 34. Issue #54 — 参加者の段から引き継ぎを書く（2026-10-21）

同じコンテナ。編集行（Issue #53）の右に「引き継ぎ先」の箱（ID 24・候補は 7000〜）が付いた。候補は選んだ人の他の参加者（`名前 (ID の先頭 4 桁)`）で、先頭は「引き継ぎ先なし」。候補を押すか、開いて上下で光らせ Enter で決めると `AdminCommand::Succeed(新, 旧)` を書く。閉じた箱の上下では選ばない（引き継ぎは戻せない）。候補の列は行の上に重なるので、行より後に問い合わせ、最後に描き、開いているあいだ表と行は押せない。core の `Permissions::succeeding` は、古い ID が編集者なら新しい ID を編集者に、古い ID の入っていたグループに新しい ID を入れ、古い ID を一覧から外すようになった（ADR 0013 の 3 に追記）。引き継がれた ID は参加者の段から消える（Issue #51）。文言は 6 言語に `NoSuccessor` を足した。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 355 件（core 100・application 87） |
| `make conformance` | 違反なし（221 ファイル） |
| `make coverage` | core ＋ application の region 98.83%（閾値 90） |
| `make deny` | 回していない（依存の変更なし） |

Xvfb（`:99`・`scratchpad/successor.sh`）: alice が台帳を作り bob を編集者に、鍵を失くした想定の bob が別の設定フォルダから同じ名前で参加（3 人）。alice が古い bob の行を押して編集行、「引き継ぎ先」を押すと候補 `No successor` / `alice (…)` / `bob (…)` が行の上に出た（[01-open](screenshots/2026-10-21-successor-01-open.png)）。新しい bob を押すと `permissions.v1.txt` に `succeeds = <新> : <旧>`、`editor =` は新しい bob に移り、段は alice（Owner）と新しい bob（Editor）の 2 人になった（[02-succeeded](screenshots/2026-10-21-successor-02-succeeded.png)）。行と候補の番号は ID の順なので、手順は 3 つの ID を並べて選ぶ。

## 35. 再現

```bash
make prove
cargo test -p xtask
make conformance
make coverage
```
