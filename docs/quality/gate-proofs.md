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
| CNF-015 | store の `.modified()`・adapters の `.accessed()`・core の `.created()`・`set_times` | xtask の正例・反例 | 反例は CNF-015。`.len()` と `file_type()`・テストの中・xtask は通る |

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
| CNF-015 | 3 | cnf015_allows_sizes_and_other_metadata, cnf015_catches_file_times_in_every_crate, cnf015_ignores_tests_and_xtask |
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

テストが要求すること: 125 配色すべてで字 / 背景のコントラスト 4.5 以上・字 / 面 4.0 以上、名前の一意性と `_` の別綴り、6 言語 × 31 語（当時）が空でない、設定の版 2 の往復と版 1 の読み込み（`language` を含む版 1 は拒む）、OS の言語の解決（`zh_HK` → 繁体字・`C` → `None`）。

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

🔴 **正直に:** 翻訳 6 言語 × 31 語（当時）は自分で書き、母語話者の目を通していない。導出した 8 色の見た目は仮（ADR 0012 の 2）。既定のライトが `neutral-light`（橙の強調）になり、Issue #5〜#20 の青い強調から変わった。

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

## 35. Issue #55 — 表と下の段の境をドラッグで動かし、設定の版 5 に残す（2026-10-21）

同じコンテナ。表と一番上の段の隙間（8 px）に 6 px のつまみ（ID 25）を置き、縦にドラッグすると出している段すべてが同じ高さになる。高さは application の `PaneHeight`（`auto` か 56 以上の論理画素）で、設定の版 5 に `pane_px = auto|<高さ>` で残る（版 4 以前は読んで `auto` で補い、`pane_px` を持つ版 4 以前は拒む）。範囲は行 2 つ分（56）以上、表に行 2 つ分が残る高さ以下で、ドラッグ中も読んだ値も同じ `clamped_pane_height` で収める。離したときに設定を書く。列の境のドラッグと同じ `resizing` を `DragTarget`（列か段か）で共有し、どちらかをドラッグ中は見出しを動かさない。adapters は `settings.v5.txt` を書き、版 4→3→2→1 の順に読む。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 358 件（application 90） |
| `make conformance` | 違反なし（223 ファイル） |
| `make coverage` | core ＋ application の region 98.86%（閾値 90） |
| `make deny` | 回していない（依存の変更なし） |

Xvfb（`:100`・`scratchpad/panes.sh`）: 参加者の段を出す（自動の高さ・[01-auto](screenshots/2026-10-21-panes-01-auto.png)）→ 表との隙間（y≈382）を押して y=250 まで動かして離す → `settings.v5.txt` に `pane_px = 342` → 閉じて開き直すと段は同じ高さ（[02-reopened](screenshots/2026-10-21-panes-02-reopened.png)）。

## 36. Issue #56 — 権限の変更をジャーナルに追記し、台帳全体の時系列を監査として出す（ADR 0014 受理・2026-10-21）

同じコンテナ。`AdminCommand` を core に移し、`OpKind::Admin(AdminCommand)` を足した（`item()` は `Option`）。`Docket::applied` は何もせず、`inverse_of` は `None`、Undo の束は積まない。store は種類 `admin`・課題 `-`・命令と引数の行を読み書きする（グループ名は逃がす）。application の `administration_log::record` が、画面の参加者の段と合成ルートの両方から、権限を書いた後に命令を操作として追記して同期する。`HistoryLine::all`（`merge::chronicle`）が台帳全体の時系列を作り、履歴の段は課題を選んでいなければそれを出す（再現は無く、押しても選び直さない）。`audit_report::text` が `--audit` の文字列（古い順・タブ区切り）。文言は 6 言語に 8 語（権限の記録）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 368 件（core 101・application 96・store 47・app 16） |
| `make conformance` | 違反なし（226 ファイル） |
| `make coverage` | core ＋ application の region 98.82%（閾値 90） |
| `make deny` | 回していない（依存の変更なし） |

Xvfb（`:101`・`scratchpad/audit.sh`）: alice が台帳を作り、bob が参加。alice の `--permissions on --allow <bob> --group qa <bob>` で `permissions.v1.txt` が書かれ、alice のジャーナル（版 2）に `admin - permissions on` / `admin - allow <bob>` / `admin - group qa <bob>` の署名付き 3 行が追記された。`--audit` は English で `turned permissions on` / `added an editor: bob` / `added to a group: qa: bob`、bob の `LANG=ja_JP.UTF-8` では `権限を有効にした` / `編集者に加えた: bob` / `グループに入れた: qa: bob`（いつ・誰が・何を、タブ区切り・古い順）。画面で履歴の段を出す（課題を選ばない）と同じ 3 行が新しい順に出た（[01-history-all](screenshots/2026-10-21-audit-01-history-all.png)）。参加者の段で bob の「編集者」を外すと `removed an editor: bob` が加わり、`--audit` にも出た（[02-after-disallow](screenshots/2026-10-21-audit-02-after-disallow.png)）。権限を有効にする前の alice の版 1 の 2 行は、これまで通り捨てて数える（`lines failing signature or permission: 2`）。

## 37. Issue #61 — 大容量の実測と段階 1（表の一覧のキャッシュ・増分の併合・2026-10-21）

リポジトリの外の計測用クレート（scratchpad の `bench/src/year.rs`・release・このコンテナ）。3 アカウントで `<件数> <1 件あたりの操作数>` のジャーナル（版 1）を一時フォルダに書き、`FileJournal::open` → `refresh` → `Ledger::open`、続けて `DocketScreen::open` と 1 フレーム（何もしない・検索 1 字・読み直しで何も届かない）を測り、RSS を `/proc/self/status` から読んだ。2 回目は OS のキャッシュが効いている。

| 規模 | 起動（読む＋併合） | 何もしない 1 フレーム | 検索 1 字 | 読み直し（何も無い） | RSS |
| --- | --- | --- | --- | --- | --- |
| 1 万件・30 万操作・45 MB | 0.87〜0.92 s | 1.0 ms | 1.6 ms | 1.4 ms | 127 MB |
| 10 万件・100 万操作・151 MB（段階 1 の前） | 2.7〜3.0 s | **21.6〜23.4 ms** | 26〜30 ms | **25.6〜26.7 ms** | 276 MB |
| 10 万件・100 万操作（段階 1 の後） | 3.0 s | **1.4 ms** | 27.7 ms | **0.27 ms** | 275 MB |
| 10 万件・300 万操作・460 MB（段階 1 の後） | **8.5〜10.0 s** | 1.4〜1.7 ms | 29〜31 ms | 0.2 ms | **703 MB** |

段階 1: `DocketScreen` が（台帳の世代・検索・絞り込み・完了の表示）を鍵に表の一覧を持ち、`Ledger::absorb` は後の操作だけなら併合し直さずに当てる（`generation` / `last_applied`）。1 フレームと読み直しは目標の中。起動 3 秒とメモリは段階 2（スナップショットと索引・ADR 0015）で。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 369 件（`absorb` が後の操作を当て、前の操作が届けば併合し直して同じ台帳） |
| `make conformance` | 違反なし（227 ファイル） |
| `make coverage` | core ＋ application の region 98.83%（閾値 90） |

## 38. Issue #61 の段階 2 — アカウントごとのスナップショットと索引（2026-10-21）

同じコンテナ。`Ledger` は操作の列を持たず、台帳・時計・Undo の束（最新 1,000 回）・Undo の相手の課題だけを持つ。履歴・Undo の逆の値・選び直しは `JournalPort::ops_of`（課題ごとの索引からジャーナルの行を 1 本ずつ読む）、台帳全体の時系列は `recent_ops`（各ジャーナルの末尾から・新しい 1,000 件）、監査は `all_ops`。全順序で前の操作が後から届けば、その課題だけを全操作から作り直す（`Docket::replaced`）。store は自分の `snapshot/<account>.v1.txt`（状態・読み位置・`permissions.v1.txt` の署名の印・Undo の束）と `.v1.idx`（32 バイト固定長の索引・記憶の上では 8 バイトに詰める）を、読み直し 100 回ごとと窓を閉じたときに一時名 → rename で書く。権限が変わった・ジャーナルが縮んだ・形が読めないスナップショットは捨てて全部読む。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 375 件（application 100・store 51） |
| `make conformance` | 違反なし（234 ファイル・CNF-003 で `Resumed` と `Reading` を別ファイルに分けた） |
| `make coverage` | core ＋ application の region 98.15%（閾値 90） |

bench（scratchpad の `year.rs`・release・3 アカウント・別プロセスで 2 回目を測る）:

| 規模 | 初回（全部読む・索引を作る）| 2 回目（スナップショットから） | 履歴 1 件 | 新しい 1,000 件 | 1 フレーム | 検索 1 字 | RSS（2 回目） | スナップショット |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 10 万件・100 万操作 | 4.1 s | 0.5〜0.6 s | 0.1〜0.2 ms | 8〜11 ms | 2.0 ms | 35 ms | 146〜150 MB | 18 MB ＋ 32 MB（0.9 s） |
| 10 万件・300 万操作 | 13.4 s | **0.8〜0.9 s** | 0.2〜0.3 ms | 7〜8 ms | 1.6 ms | 24〜30 ms | **174〜178 MB** | 18 MB ＋ 96 MB（0.5 s） |

目標（起動 3 秒・1 フレーム 16 ms・検索 100 ms）は 2 回目から中。メモリ 150 MB は 178 MB で未達（ADR 0015 の残る穴）。段階 1 の前（第 37 節）は 300 万操作で起動 8.5〜10 秒・703 MB だった。

Xvfb（`:102`・`scratchpad/snapshot.sh`）: 読み直し 1 秒の設定で alice が 2 件足し、100 回目の読み直しで `snapshot/<alice>.v1.txt`（810 バイト・`clock = 4`・`sequence = 3`）と `.idx`（128 バイト・4 件）が書かれた。その後 3 件目を足してプロセスを殺し（窓を閉じる経路は WM の無い Xvfb では通らない）、開き直すと 3 件とも出て（3 件目はスナップショットの後の自分の行から）、履歴の段は索引から引いた 2 行（[01-resumed](screenshots/2026-10-21-snapshot-01-resumed.png)）、Ctrl+Z は 3 件目の題名を戻した（前の回の自分の操作が Undo の束に積まれている・[02-undo](screenshots/2026-10-21-snapshot-02-undo.png)）。

## 39. Issue #62 — 添付: 台帳フォルダに写し、`attach` / `detach` を追記（ADR 0016 受理・2026-10-21）

同じコンテナ。core に `Attachment`（連番・名前・バイト数・SHA-256）と自前の `sha256`（FIPS 180-4・外部クレートを足さない）、`OpKind::Attach` / `Detach`、`Item::attachments` と逆の操作。store に `attachment_store`（`attachments/<作ったアカウント>-<連番>/<添付の連番>-<名前>` に一時名 → rename で写す・`is_intact`。当初は `:` 区切りで、Windows / SMB で作れないと Issue #74 で直した）、`attach`（11 欄）/ `detach`（8 欄）の行、スナップショットの課題の行に添付の列。application に添付の段 `AttachmentsPane`（パスの欄・「添付する」・「外す」・名前・大きさ・SHA-256 の先頭 8 桁）、履歴の「添付した」「添付を外した」、知らせ（読めない・置けない）。app に `--attach <課題> <パス>`。文言は `Word::Attachment(AttachmentWord)` で 6 言語 × 10 語。道具の段の「添付」は 110 px（90 では英語が切れた）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` / `deny` | 差分なし・0 件・0 件・警告なし・ok |
| `make test` | 384 件（core 104・application 103・store 54・app 5・xtask 81） |
| `make conformance` | 違反なし（240 ファイル） |
| `make coverage` | core ＋ application の行 97.75%・region 97.56%（閾値 90） |

Xvfb（`:101` / `:102`・`scratchpad/attach.sh` / `attach2.sh` / `attach3.sh`・1200×700・英語）:

1. alice が課題を足し、「添付」を押して段を出し、パスの欄に `note.txt`（17 バイト）を打って Enter → 行に名前・17・`2852d726`（`sha256sum` と一致）、`attachments/<alice>:1/1-note.txt` が置かれ、alice のジャーナルに `attach` の行（[01-attached](screenshots/2026-10-21-attach-01-attached.png)）
2. bob が同じフォルダを開くと 4 秒後に同じ行が見えた。carol が `--attach <alice>:1 photo.png`（5,000 バイト・画面は開かない）→ 両方の窓に 2 行（[02-two-processes](screenshots/2026-10-21-attach-02-two-processes.png)）
3. bob が `note.txt` の行を選んで「外す」→ 行が消え、bob のジャーナルに `detach`。写しは残る。写しに 1 バイト足してから Ctrl+Z → 行が戻り（`attach` の行が `undoes` 付きで追記）、名前に `⚠ note.txt: contents differ from the record`（[03-altered](screenshots/2026-10-21-attach-03-altered.png)）
4. 履歴の段: `undo: attached: note.txt` / `detached: note.txt` / `attached: photo.png` / `attached: note.txt` が誰が付きで並ぶ（[04-history](screenshots/2026-10-21-attach-04-history.png)）。`--audit` も同じ 6 行

🔴 **正直に:** 元のファイルは丸ごと記憶に読む（大きなファイルの上限は無い）。写しを消す経路は無い。ビューワーはまだ（#63 / #64 / #66）。

## 40. Issue #63 — 添付の中身: 文と表は段の中、それ以外は OS の既定のアプリ（ADR 0017 受理・2026-10-21）

同じコンテナ。core に `AttachmentKind`（拡張子）と `csv`（`"` の囲み・`""`・`\r\n`）、`AttachmentWord` に 3 語（開く・先頭だけ・開けない）。application に `AttachmentView`（文 / 表 / パス）・`AttachmentViewCache`・`AttachmentsContext`・`OpenerPort` / `OpenFailure`・`Ports` の 4 つ目・添付の段の中身（行を選ぶと一覧は見出し ＋ 3 行に縮み、下に `text_area` の読み取り専用か表かパスと「開く」）。store に `encoding_rs`（唯一の外部クレート・CNF-013 の表）と `attachment_store::read_text`（BOM → UTF-8 → `Shift_JIS`・256 KiB で先頭だけ・最後の行は落とす）。adapters-x11 に `ExternalOpener`（`xdg-open`・終わりは別スレッドで待つ）、adapters-windows に `ShellExecuteW` 版（Windows では未実行）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` / `deny` | 差分なし・0 件・0 件・警告なし・ok（`encoding_rs` は `(Apache-2.0 OR MIT) AND BSD-3-Clause`・`allow` は変えていない） |
| `make test` | 398 件（core 108・application 110・store 56・adapters-x11 10・xtask 81） |
| `make conformance` | 違反なし（251 ファイル） |
| `make coverage` | 行 97.60%・region 97.14%（閾値 90） |

Xvfb（`:103` 英語 / `:104` 日本語・`scratchpad/view.sh` / `view2.sh`・偽の `xdg-open` を `PATH` に置き、呼ばれたパスを記録）: 課題に `memo.txt`（UTF-8・日本語 5 行）・`sales.csv`（`Shift_JIS`・`\r\n`・`"みかん, 小"` の囲み）・`book.xlsx`（乱数 200 バイト）を添付し、段を高くして行を選ぶ。

1. `memo.txt` → 5 行が読み取り専用の欄に出る（[01-text](screenshots/2026-10-21-view-01-text.png)）
2. `sales.csv` → 見出し「品名・数量・単価」と 2 行の表（`"みかん, 小"` は 1 欄）。`Shift_JIS` が読めている（[02-csv](screenshots/2026-10-21-view-02-csv.png)）
3. `book.xlsx` → 写しのパスと「開く」。押すと `opened.log` に `xdg-open <写しのパス>` が 1 行（[03-path](screenshots/2026-10-21-view-03-path.png)）

🔴 **正直に:** 英語の UI では日本語の字が □ になる（起動時に言語でフォントの家族を選ぶ・ADR 0007 の既知の制約で、本文の欄も同じ）。`xdg-open` の先は記録だけ（コンテナに既定のアプリは無い）。Windows の `ShellExecuteW` は lint だけ。

## 41. Issue #64 — 画像のビューワー: PNG / JPEG を `crates/media` で復号し、UI が伸縮して描く（ADR 0018 受理・2026-10-21）

同じコンテナ。新しいクレート `crates/media`（`png` 0.18 ＋ `jpeg-decoder` 0.3・依存は core だけ・store が使う）。`flate2` は `Cargo.lock` で 1.1.4（1.1.10 は `miniz_oxide` が 2 版になり `make deny` の `multiple-versions` が落ちる）。core に `Bitmap`（RGBA・`premultiplied_bgra`）・`SharedBitmap`（`Arc` ＋ ID・ID は添付の SHA-256 の先頭 8 バイト）・`DrawCommand::Image { dest, source, bitmap }`・`image_fit::place`（収まる大きさ 100%・ホイール 10%・ずれ）。application に `AttachmentView::Image` と段の画像（ホイールで拡大・押したまま動かすと移動・行を選び直すと戻る）。ui-x11 に `SurfaceCache`（ID → cairo の `ImageSurface`・8 枚まで）と Bilinear のパターン、ui-win32 に `ID2D1Bitmap` の覚え（lint だけ）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` / `deny` | 差分なし・0 件・0 件・警告なし・ok（`allow` は変えていない） |
| `make test` | 410 件（core 113・application 112・store 56・media 4・ui-x11 16・adapters-x11 10・xtask 81） |
| `make conformance` | 違反なし（262 ファイル・CNF-013 に media の表） |
| `make coverage` | 行 97.57%・region 97.16%（閾値 90） |

bench（scratchpad の `bench-img`・release・`Canvas::render` を 20 回）:

| 画像 | 復号（縮小込み） | 初回（surface の作成 ＋ 描画） | 2 回目から | 拡大の部分 |
| --- | --- | --- | --- | --- |
| JPEG 4000 × 3000（1.6 MB → 2000 × 1500） | 297〜324 ms | 20.5 ms | **1.0 ms** | 0.7 ms |
| PNG 600 × 400（α 付き） | 3.0 ms | 4.2 ms | 1.1 ms | 0.8 ms |

cairo の既定のフィルタ（Good）では 2 回目からが 36 ms だった（縮めるときの畳み込み）。Bilinear に替えて 1 ms。目標 16 ms の中。復号は選んだときに 1 度（0.3 秒・画面は止まる）。

Xvfb（`:105`・release・`scratchpad/image.sh`・日本語 UI）: `logo-alpha.png`（600 × 400・α 付き）と `big-4000x3000.jpg`（ImageMagick の `plasma:fractal`）を添付し、段を高くして行を選ぶ。

1. PNG が収まる大きさで中央に出る（透明は段の面の色・[01-png](screenshots/2026-10-21-image-01-png.png)）
2. JPEG を選ぶと 2000 × 1500 に縮めた画素が収まる大きさで出る（[02-jpeg](screenshots/2026-10-21-image-02-jpeg.png)）
3. ホイールを上に 5 刻み → 150%、押したまま左上へ動かす → ずれる（[03-zoomed-panned](screenshots/2026-10-21-image-03-zoomed-panned.png)）

🔴 **正直に:** debug ビルドでは 4000 × 3000 の復号に数秒かかり、画面が止まる（release は 0.3 秒）。同じフレームで前の選択の色が残る 1 フレームの欠陥を見つけ、行を全部問い合わせてから描く形に直した。Windows の `CreateBitmap` / `DrawBitmap` は lint だけ。EXIF の向きは見ない。

## 42. Issue #65 — 添付のドラッグ＆ドロップ（X11 は XDND・Windows は `WM_DROPFILES`・2026-10-21）

同じコンテナ。application の `FrameInput::with_drop(位置, パスの列)` を `DocketScreen::step` が受け、位置が表の行ならその課題、添付の段なら選んでいる課題に `attach_file`（写してから `Attach` を追記・Issue #62 と同じ経路）、その行を選んで段を出す。他の場所は何もしない。ui-x11 に XDND 版 5 の受け手（`XdndAware`・`XdndEnter` / `XdndPosition` → `XdndStatus`・`XdndDrop` → `ConvertSelection(text/uri-list)` → `SelectionNotify` → property を読んで `XdndFinished`・`uri_list` で `file://` を戻す）。ui-win32 に `DragAcceptFiles` と `WM_DROPFILES`（`DragQueryPoint` / `DragQueryFileW` / `DragFinish`・lint だけ）。依存は増えない。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 413 件（application 114・ui-x11 17） |
| `make conformance` | 違反なし（264 ファイル） |
| `make coverage` | 行 97.59%・region 97.18%（閾値 90） |

Xvfb（`:106`・`scratchpad/dnd.sh`・送り手は scratchpad の `xdnd-send`（x11rb 70 行: `XdndSelection` の持ち主になり `XdndEnter` / `XdndPosition` を送り、`XdndStatus` を待って `XdndDrop`、`SelectionRequest` に `text/uri-list` を書き、`XdndFinished` を待つ）:

1. 2 件の課題がある窓の 1 行目（y = 120）に `file:///…/logo-alpha.png` と `file:///…/memo.txt` を落とす → `XdndStatus` accept=1、`XdndFinished` accepted=1、`attachments/<課題>/1-logo-alpha.png` と `2-memo.txt` が置かれ、1 行目が選ばれて添付の段が出る（[01-row](screenshots/2026-10-21-dnd-01-row.png)）
2. 添付の段（y = 450）に `sales.csv` を落とす → 選んでいる 1 件目に `3-sales.csv`（[02-pane](screenshots/2026-10-21-dnd-02-pane.png)）。ジャーナルに `attach` が 3 行

🔴 **正直に:** 落とす相手が GTK / Qt のアプリでないので、`XdndTypeList`（型が 3 つを超える）と `XdndLeave` の経路はテストの偽の送り手でしか通していない。`WM_DROPFILES` は Windows で動かしていない。

## 43. Issue #66 — PDF の `hayro` を試した（ADR 0019・却下・2026-10-21）

同じコンテナ・`scratchpad/pdftry`（`hayro = "0.8"` だけ・release）。PDF は reportlab で 3 つ作った（ASCII の Helvetica・IPAゴシック埋め込み・平成角ゴ / 平成明朝の CID 埋め込みなし）。`pdftoppm` の参照と見比べた。

| 観点 | 実測 |
| --- | --- |
| 描画 | ASCII・図形・埋め込み日本語は正しい（[01-embedded](screenshots/2026-10-21-hayro-01-embedded.png)）。埋め込みなしの CID は白紙。`font_resolver` で IPAゴシックを渡すと出る（[02-cid-fallback](screenshots/2026-10-21-hayro-02-cid-fallback.png)・明朝もゴシックになる） |
| 速さ | A4 1 ページ 1 倍 2〜5 ms・2 倍 6〜15 ms・読み込み 0.1 ms |
| 依存 | 75 クレート（`cargo tree` 136 行）。重複: `syn` 2 / 3・`fearless_simd` 0.7 / 1.1・`miniz_oxide` 0.8 / 0.9。`Zlib` だけのクレート 2 つ |
| ビルド | 依存だけのクリーンビルド 104 秒・`target/release` 283 MB・rlib 72 本 |

この台帳のゲート（`multiple-versions = "deny"`・依存は許可制）を弱めないと入らないので、ADR 0019 で今は却下。製品のコードは変えていない（`make deny` などのゲートはそのまま）。

🔴 **正直に:** 試した PDF は 3 つだけ。画像・透明・暗号化・大きなページは測っていない。却下の理由は hayro の出来ではなく依存の重さ。

## 44. Issue #77 — 壊れたファイルへの耐性: 壊し方 × ファイルの形を機械で試す（2026-10-21）

同じコンテナ。store に `damage`（`#[cfg(test)]`・切り詰め・ビット反転・0 埋め・空・種で決まる線形合同法）。変えた振る舞い: (1) 自分のジャーナルを開くとき最後の行が改行で終わらなければ改行を足す（切れた行を 1 つの壊れに閉じ込め、次の追記が繋がらない）、(2) 他人の書きかけの末尾が読み直し 4 回伸びなければ `RecordFailure::Unfinished` として壊れに数え、伸びたら数えるのをやめる、(3) スナップショットの最後の行に `crc = <状態> <索引>` を置き、合わなければ捨てて全部読む（#61 のスナップショットは 1 度だけ捨てられる）。表は [docs/design/2026-10-21-corruption-tolerance.md](../design/2026-10-21-corruption-tolerance.md)。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 422 件（store 64: 切り詰め 40 通り・反転 60 通り・0 埋め・空・止まった末尾・スナップショットと索引の反転・`docket.v1.txt` と `keys/` の壊れ） |
| `make conformance` | 違反なし |

見つけて直した穴: 自分のジャーナルの切れた行の後に追記すると 2 行とも壊れていた（(1) で直した）。スナップショットのビット反転は形が通れば黙って違う状態になっていた（(3) で直した）。

🔴 **正直に:** ファイルシステムそのものの壊れ（`EIO`）・`.tmp` の残骸・空き容量ゼロは試していない（#81）。壊れた行の位置を人に見せる手段はまだ無い。

## 45. Issue #75 — SMB の遅延: 他人の読み直しとスナップショットの書きをフレームから外す・共有の実測の道具（2026-10-21）

同じコンテナ。store に `scanner`（スレッド・頼みは 1 つに溜める）と `scan`（結果）。`FileJournal::refresh` は「届いていれば受け取り、次を頼み、50 ms だけ待つ」になった（ローカルの台帳なら同じ刻みで届く・遅い共有では次の刻みへ回る・最初の 1 回だけは結果を待ち、起動の画面に他人の課題が揃う）。最初の試作は待たない形で、Xvfb で相手の変更が次の刻みまで眠って最大 6 秒かかったので 50 ms の待ちを足した。鍵と `permissions.v1.txt` の読み直しもスレッドの側。スナップショットの書き（300 万操作で 0.5〜0.9 秒）は別スレッドで、次の書きと `drop` で待つ。テストは `refresh_settled`（頼んで待つ）。`--probe-share <フォルダ>` で `stat`・末尾の読み・追記 ＋ `sync_data`・一時名からの rename の 1 回あたりを測る（`store::share_probe`）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 423 件（store 65: `share_probe` と、`refresh_settled` に直した読み直しのテスト） |
| `make conformance` | 違反なし（CNF-012: `std::time` は store の中だけ・CNF-003: `Scan` と `Scanner` を分けた） |

`--probe-share`（このコンテナのローカルディスク・20 回の平均）: stat 0.00 ms / read tail 0.07 ms / append+sync 0.89 ms / rename 0.37 ms。

Xvfb（2300×700・2 プロセスを横に並べる・読み直しの刻み 3 秒）: alice が追加して 4 秒後に bob の窓に `From A`（[screenshots/2026-10-21-share-01-b-sees-a.png](screenshots/2026-10-21-share-01-b-sees-a.png)）、bob が追加して 4 秒後に alice の窓に `From B`（[screenshots/2026-10-21-share-02-a-sees-b.png](screenshots/2026-10-21-share-02-a-sees-b.png)）。待たない試作では 2 枚目が届いていなかった。

🔴 **正直に:** 施主の共有（NAS）ではまだ測っていない（QLT-013）。読み直しの間隔の自動調整は入れていない（頼みを 1 つに溜めるので、遅い共有では合図が重なっても 1 回の読みにまとまる）。追記 ＋ `sync_data` は今もフレームの中（操作ごと・ローカルで 0.9 ms・SMB では 10〜50 ms になりうる。SPECIFICATION 第 2 節の「2 秒にまとめる」は未実装）。

## 46. Issue #80 — 日本語入力: Windows は IMM32、X11 は貼り付けの逃げ道、確定前の下線（2026-10-21）

同じコンテナ。ADR 0020 を受理。core に `pasted_text`（貼り付け・確定の文字列を 1 字ずつ `KeyIntent::Char` に・改行を揃え制御文字を捨てる）と `TextCaret::underline`（確定前の範囲）。`TextEdit` は改行の字を 1 行の欄では空白、複数行の欄では改行にする。ui-x11 に `clipboard`（Ctrl+V → `ConvertSelection(CLIPBOARD, UTF8_STRING)` → `SelectionNotify`）と cairo の下線。ui-win32 に `ime`（`WM_IME_SETCONTEXT` で組み立ての窓を消す・`WM_IME_COMPOSITION` の `GCS_COMPSTR` / `GCS_CURSORPOS` / `GCS_RESULTSTR`・`ImmSetCandidateWindow` を欄の下）、`composition`（UTF-16 のカーソルを文字の添字に・Linux でテスト）、Direct2D の下線。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` | 差分なし・0 件・0 件・警告なし |
| `make test` | 430 件（core 117: `pasted_text`・下線・改行の字 / ui-x11 19: Ctrl+V の判定・下線の画素 / ui-win32 10: UTF-16 のカーソル） |
| `make conformance` | 違反なし |

Xvfb（1120×600・`LANG=ja_JP.UTF-8`・送り手は scratchpad の `clip-own`: `CLIPBOARD` を持ち `UTF8_STRING` を答える x11rb の 40 行）: 検索欄に Ctrl+V で `日本語の検索 テスト`、「追加」の題名の欄に同じく貼って Enter（[screenshots/2026-10-21-paste-01-title.png](screenshots/2026-10-21-paste-01-title.png)・ジャーナルに `set … title 日本語の検索 テスト`）、本文の欄に `一行目\r\n二行目\tタブ` を貼ると 2 行になりタブは空白（[screenshots/2026-10-21-paste-02-body.png](screenshots/2026-10-21-paste-02-body.png)）。`LANG=en_US.UTF-8` では字が豆腐になる（起動時に選ぶ家族に日本語が無い・ADR 0007 のとおり）。

🔴 **正直に:** Windows の IME は組み立てただけ（`make lint-windows` 0 件）で、確定前の文字列が欄に出ること・候補の窓の位置・`WM_IME_CHAR` を流さないことの実機の確認は施主の Windows（QLT-013）。X11 の IME 本体（XIM）は無く、Ctrl+V の貼り付けだけ。`INCR`（256 KiB 超）と `PRIMARY` は受けない。確定前の下線は Xvfb で出す手段が無い（IM サーバが無い）ので、cairo の画素のテストだけ。

## 47. Issue #79 — Windows のファイルロック: 共有モード・置き換えの再試行（2026-10-21）

同じコンテナ。設計は [docs/design/2026-10-21-windows-file-locks.md](../design/2026-10-21-windows-file-locks.md)。store に `retry`（`PermissionDenied` と Windows の共有違反 32 / 33 に 50 ms × 5 回）と `replace`（一時名に書いて rename・両方を再試行）。`snapshot_file` / `attachment_store` / `permissions_store` / `docket_file` / `key_file` の書きは全部 `replace_file` を通る（`permissions.v1.txt` / `docket.v1.txt` / `keys/` は前は直接書いていた）。自分のジャーナルは Windows で `FILE_SHARE_DELETE` を外して開く（`own_journal::open_append`・`cfg(windows)`）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `doc-check` | 差分なし・0 件・警告なし |
| `make build-windows` | exe ができた（store の `cfg(windows)` の `share_mode` が組み立つ） |
| `make test` | 434 件（store 69: `retry` 3 件 — 偽の失敗 3 回で通る・5 回で諦める・恒久の失敗は 1 回 / `replace` 1 件 — 置き換えて `.tmp` が残らない） |
| `make conformance` | 違反なし |

🔴 **正直に:** Linux ではロックが起きないので、再試行は偽の失敗でしか試していない。施主の Windows での 3 つの手順（2 プロセス・Explorer のプレビューを出したまま権限を変える・Defender を有効にして添付）は**未実施**（設計文書の第 4 節）。結果はこの節に足す。Rust の std の既定の共有モード（`READ | WRITE | DELETE`）は文書から取った値で、実機で `handle.exe` 等で見てはいない。

## 48. Issue #81 — OS のフリーズ・停電・強制終了・スリープ復帰: 止まり方の表と戻ったときの約束（2026-10-21）

同じコンテナ。表は [docs/design/2026-10-21-interruptions.md](../design/2026-10-21-interruptions.md)（8 行・各行にテストか実機の手順）。store に: 追記に失敗した操作の列（`FileJournal::unwritten`・次の追記・同期で古い順に書き直す・残っている間は `checkpoint` を `StoreFailure::Unwritten` で拒む）、自分のジャーナルの開き直し（`OwnJournal::reopen`・追記の失敗で 1 度だけ）、`leftovers`（自分の分と持ち主の分の `.tmp` を次に開いたときに捨てる）、`key_file::read_all` は `.tmp` を読まない。テストの失敗の仕込みは `OwnJournal::fail_next`（`cfg(test)`・`StorageFull`）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `doc-check` | 差分なし・0 件・警告なし |
| `make test` | 440 件（store 75: 失敗を 4 回仕込んで 2 つの操作が順に残る・`checkpoint` が拒む / 残骸は自分の分だけ消え他人と添付の分は残る / 2 世代の状態と索引の組は捨てて全部読む / 消えたファイルの後の開き直し / `.tmp` の鍵は飛ばす） |
| `make conformance` | 違反なし |

🔴 **正直に:** 空き容量ゼロは仕込んだ失敗でしか試していない。施主の実機での 3 手順（共有を切って戻す・スリープして戻す・編集中に電源を切る）は未実施。添付の `.tmp` は消さない（誰のものか分からず、時刻を使わない）と決めた — Issue の「掃除する」から変えた。

## 49. Issue #76 — NAS の癖: 一覧と守り、CNF-015（ファイルの時刻を使わない）、名前は小文字の ASCII（2026-10-21）

同じコンテナ。表は [docs/design/2026-10-21-share-quirks.md](../design/2026-10-21-share-quirks.md)（8 行・各行に 避けた / 検出して示す / 許容と明記）。読み直しが大きさを見ない（読み位置から EOF）のは既にそうで、表に結んだ。新しいのは CNF-015（xtask・どのクレートでも `.modified()` / `.accessed()` / `.created()` / `set_times` / `FileTimes` を拒む・実ファイルは 0 件）と、store が作る名前が全部小文字の ASCII であることのテスト（台帳・鍵・権限・ジャーナル・スナップショット・添付のフォルダと連番）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `doc-check` | 差分なし・0 件・警告なし |
| `make test` | 444 件（xtask ＋3: `cnf015_*` / store ＋1: `every_name_the_store_makes_is_lowercase_ascii`） |
| `make conformance` | 違反なし（CNF-015 を含む） |

🔴 **正直に:** 属性キャッシュ・一覧のキャッシュ・oplock の遅れ・`fsync` の無視は、このコンテナ（ローカルディスク）では出ない。施主の NAS での確認（設計文書の第 2 節: 機種と SMB の版・`--probe-share`・2 台の同時編集・3 台目が見えるまでの秒数）は**未実施**。

## 50. Issue #78 — 壁時計のずれ: 地方時の表示・ずれの印・期限の「今日」の約束（2026-10-21）

同じコンテナ。順序は論理時計のままで（`merge` の壁時計をずらしたテストを足した）、変えたのは表示だけ。application に `ClockDisplay`（時差といま。5 分以上先の壁時計には `⚠`）と `ClockChoice`（設定の `clock = local|utc`・版 6・adapters は `settings.v6.txt` を書き版 5→1 の順に読む）。`ClockPort::offset_minutes` は X11 が `date +%z`、Windows が `GetTimeZoneInformation`（`cfg(windows)`・`make lint-windows`）。履歴の段と `--audit` は `HistoryLine::columns(language, display)` を通る。期限の「今日」は SPECIFICATION 第 2 節に約束だけ。mtime は CNF-015（第 49 節）。

| target | 結果 |
| --- | --- |
| `make fmt-check` / `lint` / `lint-windows` / `doc-check` / `deny` | 差分なし・0 件・0 件・警告なし・ok |
| `make test` | 448 件（core: `text_at`・壁時計をずらしても同じ台帳 / application: `ClockDisplay`・`ClockChoice`・設定の版 6 / adapters-x11: `+0900` の読み） |
| `make conformance` | 違反なし |

Xvfb（`TZ=Asia/Tokyo`・`scratchpad/skew78.sh`・送り手は `scratchpad/skew-append`: 別アカウントが壁時計を 3,600 秒進めて課題を 1 つ作る）: 履歴が地方時（UTC 10:09 → 19:09）で出て（[01-local-time](screenshots/2026-10-21-clock-01-local-time.png)）、進んだ時計の課題の行には `⚠ 20:09`（[02-skew-mark](screenshots/2026-10-21-clock-02-skew-mark.png)）。`--audit` も同じ行で、`clock = utc` にすると `10:09` / `⚠ 11:09`。

🔴 **正直に:** 施主の 2 台の時計を意図的にずらした実機の確認は未実施。遅れている時計には印が付かない（自分より先の時計しか分からない）。X11 の時差は `date` に頼り、無い環境では UTC になる（示さない）。期限の色分けは無い。

## 51. 施主の実機での確認（2026-10-21・Issue #90）

施主が Windows と NAS の前で 1 手順ずつ操作し、見えたものを報告した。ここには報告をそのまま書く。推測は「推測」と書く（QLT-013）。

### 51.0 環境とビルド

| 項目 | 観測 |
| --- | --- |
| ビルドした機械 | 施主の WSL2（`6.18.54.1-microsoft-standard-WSL2`）・Windows `10.0.26300.9711` |
| 道具 | `make doctor`: rustc / cargo / clippy 1.98.0・cargo-deny 0.20.2・cargo-llvm-cov 0.9.0・cairo 1.16.0・Xvfb・ImageMagick・xdotool・mingw-w64。欠けなし |
| `make build-windows`（`2f3ad4a`・Windows の target を消してから） | 32.1 秒 |
| `nene-docket.exe` | 4,316,531 バイト・SHA-256 `733cf6efe533309f2e6056efe7a6a3aa8b5581617d6b36c54996d05038cfcedc` |
| 取り込む DLL（`objdump -p`） | KERNEL32・USERENV・WS2_32・advapi32・bcrypt・bcryptprimitives・combase・d2d1・dwrite・imm32・msvcrt・ntdll・oleaut32・rpcrt4・shell32・user32・api-ms-win-core-synch-l1-2-0・api-ms-win-core-winrt-error-l1-1-0。すべて Windows に入っているもので、mingw の DLL は要らない |

### 51.1 Windows 版の起動と最初の表示

施主の PC: 拡大縮小 125%・3840×2160。exe は `C:\Users\info\WORKS\NeNeDocket\`、台帳は空の `ledger-test\`。

- **施主の報告:** 窓が出る・題名・大きさ・日本語の表示・文字化けや豆腐やにじみの有無・部品の有無・配色・PowerShell の出力の 7 点とも「全部オッケー」。
- **スクリーンショット:** [real-01-windows-start](screenshots/2026-10-21-real-01-windows-start.png)。1,103×592 画素。暗い配色・日本語の見出し（# 題名 状態 担当 期限 ラベル）・上の段（検索・すべて・完了も表示・履歴・本文・参加者・添付・OS に従う・system・追加）・下の段（編集の欄・未着手・削除）が出ている。
- **起動で作られたファイル:** 台帳に `docket.v1.txt`・`journal/`・`keys/`。`%APPDATA%\nene-docket\` に `account.v2.txt`・`settings.v6.txt`（`version = 6`・`language = system`・`theme = system`・`clock = local`・`recent = \\?\C:\...`）。

🔴 **正直に:** 125% は dpi 120 で、設計（FR-016・`UiScale` は 144 未満を 1 倍）どおり 1 倍で描く。窓の初期の大きさ 1,120×600 も物理画素のままなので、125% の画面では他のアプリより約 2 割小さく出る。これは設計の帰結であって不具合とは書かない。150% 以上（2 倍）の見えは未確認。`recent` は `\\?\` の付いた形で保存されている（読み直しで困るかは未確認）。

### 51.2 追加・題名・状態・削除・Ctrl+Z / Ctrl+Y

手順: 「追加」を 3 回 → 1 行目の題名 → 1 行目を「着手中」 → 2 行目を削除 → Ctrl+Z → Ctrl+Y → Ctrl+Z を 2 回。

- **施主の報告:** 「全部想定通り」。
- **スクリーンショット:** [real-02-windows-edit-undo](screenshots/2026-10-21-real-02-windows-edit-undo.png)。最後の状態で 3 行（`9fcb-1` 題名「テスト１」・`9fcb-2`・`9fcb-3`）、すべて「未着手」。
- **ジャーナル（WSL から `journal/9fcb….v1.log` を読んだ）:** 10 行。`create` × 3 → `set title テスト１` → `set state in_progress` → `delete :2` → `restore :2`（取り消し）→ `delete :2`（やり直し）→ `restore :2` と `set state open`（2 回の取り消し）。どの行も版 1 の形で CRC が付いている。題名は全角の「１」で入っており、施主が IME で打った確定の字が入った。

🔴 **正直に:** 反応の速さは施主の体感（「想定通り」）だけで、測っていない。IME の確定前の表示はこの手順では見ていない（51 節の IME の項で見る）。

### 51.3 検索・絞り込み・列の幅と順・段の境・起動し直し

手順: 3 行目の題名を「りんご」→ 検索「テスト」→ 空に戻す → 「着手中」で絞る → 「すべて」→「題名」と「状態」の境を左へ約 200 画素 →「期限」を「題名」の左へ →「履歴」を出して境を上下 → 閉じて起動し直す。

- **施主の報告:** 「全部その通りになってる」（検索で 1 行・空で 3 行・「着手中」で 0 行・幅・順・段の境・起動し直して残る）。
- **設定（WSL から `settings.v6.txt` を読んだ）:** `columns = 72,110,90,90,120` → `72,377,90,90,120`（題名は残りの幅を取る列で、境を左へ動かすと「状態」が広がる）。`order = number,due,title,state,assignee,labels`。`pane_px = auto` → `246`。
- **ジャーナル:** 11 行目に `set :3 title りんご`。検索・絞り込み・列・段は台帳を変えない（追記なし）。

- **スクリーンショット（起動し直した後）:** [real-03-windows-columns-restart](screenshots/2026-10-21-real-03-windows-columns-restart.png)。見出しが「# 期限 題名 状態 担当 ラベル」の順、「状態」が広い。

🔴 **正直に:** 境の上でポインタの形が変わるかは報告に無い。

### 51.4 同じ PC の 2 プロセスで同じ台帳（届くまで・同時の書き換え）

A は既定の `%APPDATA%`、B は `APPDATA=C:\Users\info\WORKS\NeNeDocket\appdata-b`（別アカウント `9783…`）。台帳はローカルのフォルダ（NAS ではない）。

**施主の手での操作:** 「1 秒くらい」「追加は速かった」。スクリーンショット [real-04-windows-two-processes](screenshots/2026-10-21-real-04-windows-two-processes.png) で A と B の表が同じ 11 行。この間に、A の打っていない題名の欄が B の「Bから」を空で戻した（ジャーナルの 12〜18 行目）。不具合として **#91** に起こした。

**同時の書き換えは施主の求めで自動化した:** 施主が手でできないため、WSL から Windows の PowerShell を呼んで操作した（`SetProcessDPIAware`・`SendInput`・前面の窓が目的の窓でなければ止める・`PrintWindow` で撮る）。スクリプトは施主の PC の `C:\Users\info\WORKS\NeNeDocket\auto\` にあり、リポジトリには入れていない。

| 試し | 結果 |
| --- | --- |
| 届くまで（A で追加と削除を交互に 10 回、B の表に出る・消えるまでを 100 ms ごとに撮る・間をランダムに 0.3〜3.3 秒） | 10 回とも届いた。ms: 2,926 / 1,236 / 2,416 / 1,819 / 1,859 / 1,333 / 2,669 / 2,927 / 280 / 1,167。**最大 2,927 ms** |
| 同じ（間を一定にした最初の 10 回） | 35 / 1,992 / 1,990 / 2,041 / 2,018 / 1,980 / 2,003 / 2,065 / 1,895 / 2,120 ms。試し 1 回が約 3 秒で読み直しの刻みと同じため、位相がそろったと見る（推測）。上の表を正とする |
| 同じ行の題名を A と B で違う値に（確定の間 638 ms） | A が時計 28、B が時計 29 で書き、両方の窓が B の値にそろった。B に「他の人の変更を上書きしました。履歴で選び直せます」（[real-05](screenshots/2026-10-21-real-05-windows-overwrite-notice.png)）。A には何も出ない |
| 同じ行の題名を A と B で違う値に（確定の間 24 ms・B へは `PostMessage`） | A が時計 50、B が時計 51 で書き、両方の窓が B の値にそろった（[real-06](screenshots/2026-10-21-real-06-windows-near-simultaneous.png)）。**このときはどちらの窓にも上書きの知らせが出なかった** |
| 壊れた行 | どちらの窓にも「読めなかった行」は出ていない。ジャーナルは 2 本・51 行 |

🔴 **正直に:**
- 等しい論理時計（真の同時）は作れなかった。24 ms の差でも B は A の 50 を見た後の 51 で書いた。B が 51 になった経緯はジャーナルからは決められない（推測: 前面に無い窓は次の読み直しの刻みまでフレームを回さない）。同じ時計の順はアカウント ID で決まり、その収束は core の単体テスト（QLT-008）だけが根拠。
- 24 ms の試しで知らせが出なかった理由は未解明。B の欄には IME の確定前の「ー」が残っていた（自動で送った `tie-B` が `tieBー` になった）ので、IME の経路が絡む可能性がある（推測）。IME の項で人の手で確かめ直す。
- 上書きされた側（A）には知らせが出ない。FR-008 は上書きした側に示す設計で、これは設計どおり。
- 最初の自動の試しはスクリプトの文字コードを誤り（BOM なしの UTF-8 を Windows PowerShell 5.1 が CP932 で読んだ）、題名が化けた（`A縺ｮ蜷梧凾` 等）。アプリの不具合ではない。
- 2 台の PC・NAS の上ではまだ測っていない（51 節の NAS の項）。

### 51.5 ドラッグ＆ドロップと添付の段（文・CSV / TSV・画像・「開く」）

見本（WSL で作って `samples\` に置いた）: `memo.txt`（日本語）・`list-utf8.csv`・`list-sjis.csv`（`iconv` で Shift_JIS）・`members.tsv`・`photo.png`（1600×1000）・`photo.jpg`（4000×3000・1,600,243 バイト）・`doc.pdf`（手で組んだ 1 ページ）。

手順: エクスプローラーから `memo.txt` を表の 9fcb-1 の行へ → 「添付」の段を出す → 残りの 6 つをまとめて段へ → 1 つずつ選ぶ（CSV・TSV は表、PNG・JPEG はホイールで拡大・ドラッグで移動、PDF は「開く」）→ B でも 9fcb-1 の添付を見る。

- **施主の報告:** 「全部想定通り」。
- **ジャーナル:** A の `attach` が 7 行（52〜58 行目・連番 1〜7・大きさと SHA-256 と名前）。まとめて落とした 6 つはエクスプローラーの渡した順（`members.tsv`・`photo.jpg`・`photo.png`・`doc.pdf`・`list-sjis.csv`・`list-utf8.csv`）。
- **写し:** `attachments\9fcb…-1\1-memo.txt` 〜 `7-list-utf8.csv`。7 つとも SHA-256 が元のファイルと一致した（WSL で `sha256sum`）。
- **スクリーンショット（施主の操作の後に WSL から `PrintWindow` で撮った）:** A は JPEG を段の中に出している（[real-07](screenshots/2026-10-21-real-07-windows-attachment-jpeg.png)）。B の添付の段にも 7 つが同じ大きさ・SHA-256 で並ぶ（[real-08](screenshots/2026-10-21-real-08-windows-attachments-in-b.png)）。

🔴 **正直に:** Shift_JIS の CSV を出した画面と拡大した JPEG の画面は撮っていない（施主の「想定通り」だけ）。JPEG が出るまでの秒数は測っていない。PDF が既定のアプリで開いたことは施主の報告だけが根拠。

### 51.6 dpi（150% の 2 倍・実行中の切り替え・起動し直し）

画面は 3840×2160。撮影は施主の「撮って」で WSL から `PrintWindow`。

| 段 | 観測 |
| --- | --- |
| 150%（dpi 144）で起動 | 文字・行・欄の高さは 2 倍。窓は 1,120×600 の物理画素のまま（中身 1,098×544）で、検索の欄と題名の編集欄が幅 0 につぶれ、「追加」など右端が窓の外、題名の列もつぶれて題名が「状態」の列に重なる（[real-09](screenshots/2026-10-21-real-09-windows-150-initial.png)） |
| 同じ窓を最大化（3840×2054） | 部品がすべて並ぶ。チェックボックスの四角と選択箱の「▾」は 1 倍のままで文字に比べて小さい（[real-10](screenshots/2026-10-21-real-10-windows-150-maximized.png)） |
| 開いたまま 125% に戻す | 施主「変わらない気がする」。撮った画面も 2 倍のまま、崩れなし（[real-11](screenshots/2026-10-21-real-11-windows-125-while-running.png)）。ADR 0009 の planned（`WM_DPICHANGED`）どおり |
| 125% で起動し直す | 施主「なった気がする」。1 倍で 1,102×553（[real-12](screenshots/2026-10-21-real-12-windows-125-restart.png)） |

起こした不具合: **#93**（2 倍で窓の初期の大きさに倍率を掛けていない・X11 も同じ定数）、**#94**（チェックボックスと選択箱の寸法が倍率を通らない）、**#95**（表のセルの文字が列で切り取られない・1 倍でも同じ経路のはず・1 倍では未確認）。

🔴 **正直に:** 2 倍の見えを目で確かめたのはこれが初めてで、FR-016「HiDPI で崩れない」は 2 倍では満たしていなかった。2 枚のモニタの間の移動・200% は試していない。

### 51.7 IME（ADR 0020・#80）: 題名の欄と本文の欄

Microsoft IME・125%・1 倍。手順: 9fcb-8 の題名の欄で「にほんご」（確定前を見る）→ スペースで変換（候補の窓の位置）→ Enter で確定 → 末尾に「てすと」を打って Esc → 「本文」の段で変換・確定・改行・2 行目。

- **施主の報告:** 「想定通り」（確定前が欄の中に下線付きで出る・変換で欄の中が変わる・確定した字が入る・Esc で確定前だけ消える・本文の欄でも同じ）。**候補の窓は「入力中テキストの左下」**。
- **ジャーナル（WSL で読んだ・アカウント `9783…`）:** 59 行目 21:50:25 `set :8 title にほんご` → 60 行目 21:50:32 `set :8 title （空）` → 61 行目 21:51:34 `set :8 title 日本語`。59・60 行目は施主が「にほんご」をかなのまま確定して消した操作（施主に確かめた）。確定前の文字が書かれたのではない。
- **手順の後の画面（WSL から撮った）:** 9fcb-8 の題名「日本語」。本文の欄に 2 行（「かいぎょう」「にぎょうめ」）が入り、欄はフォーカスを持ったまま（[real-13](screenshots/2026-10-21-real-13-windows-ime-after.png)）。本文はフォーカスが外れたときに確定する作りなので、この時点ではジャーナルに本文の行はまだ無い。施主が表の空いている所を押すと、62 行目に `set :8 body かいぎょう\nにぎょうめ`（改行は `\n` で 1 行に）が書かれた。

🔴 **正直に:** スクリーンショットは無い（変換中は施主が撮影を頼めない。WSL からの `PrintWindow` では別の窓である候補の窓は写らない）。根拠は施主の目視の報告だけ。IME は Microsoft IME だけで、ATOK 等は試していない。51.4 で自動の打鍵が IME の確定前に残った件（`tieBー`）は、人の手の打鍵では起きていない（施主の報告）。

### 51.8 Windows のロック（docs/design/2026-10-21-windows-file-locks.md 第 4 節）

| 手順 | 結果 |
| --- | --- |
| 1. 2 プロセスで同じフォルダ | 51.4 のとおり。追加・題名・削除を両方で続けても止まらず、10 回とも 3 秒以内に届いた |
| 2. Explorer で台帳と `snapshot\` を開きプレビューを出したまま、持ち主（A・`9fcb…`）が参加者の段の「権限」を 3 回入れて外す | 施主: 「失敗の知らせは出ない。チェック押した通りに入ったり外れたりする」。ジャーナル 63〜68 行目に `admin permissions on / off` が交互に 6 行（on の行は署名付きの版 2、off の行は版 1）。最後の `permissions.v1.txt` は `owner` と `signature` だけ（無効）。台帳のどこにも `.tmp` は残っていない |
| 3. Defender のリアルタイム保護をオンのまま、15,138,659 バイトの PNG（`big-noise.png`・1800×1400 のノイズ）を 9fcb-9 の添付の段へ落とす | 施主: リアルタイム保護はオン、失敗の知らせなし、画像が出るまで「0.5 秒以下」。69 行目に `attach :9 1 15138659 8e74079f…`。`attachments\9fcb…-9\` に `1-big-noise.png` が 1 つだけで SHA-256 は元と一致。`.tmp` は残っていない |

🔴 **正直に:** 秒数は施主の体感。Defender の検査が実際に写しの途中で走ったか（ロックが起きて再試行の経路を通ったか）は外からは分からない。再試行の回数を数える仕組みは無いので、「失敗が見えなかった」までが観測した事実。SMB の上でのロックは NAS の項で見る。

### 51.9 NAS（docs/design/2026-10-21-share-quirks.md「施主の NAS で確かめること」）

| 項目 | 観測 |
| --- | --- |
| 機種 | Buffalo LinkStation LS710D（施主の報告）。共有は `\\LinkStation\share`（`net view` で唯一の共有）。台帳は `\\LinkStation\share\nene-docket-test`（このために作った） |
| SMB の版 | **未取得**。`Get-SmbConnection` は WSL からは権限で拒まれ、施主の PowerShell 7 では空（接続を開く前だった・推測） |
| Windows の SMB クライアントの設定（`Get-SmbClientConfiguration`） | `DirectoryCacheLifetime` 10 秒・`FileInfoCacheLifetime` 10 秒・`FileNotFoundCacheLifetime` 5 秒 |
| `--probe-share`（20 回の平均・3 回） | stat 2.75 / 0.09 / 0.08 ms・read tail 3.27 / 3.47 / 3.80 ms・**append+sync 51.28 / 53.38 / 47.11 ms**・rename 16.86 / 18.12 / 18.84 ms |
| 同じ（ローカルの `C:\` の台帳・比べるため） | stat 0.03〜0.04 ms・read tail 0.91〜0.96 ms・append+sync 0.65〜1.04 ms・rename 0.57〜0.75 ms |

**2 プロセスの同時編集（同じ PC から NAS へ・自動化・51.4 と同じ道具）:** A は既定の `%APPDATA%`、B は `appdata-b`。

| 試し | 結果 |
| --- | --- |
| A で追加と削除を交互に 10 回・B に見えるまで | ms: 2,384 / 2,056 / 726 / 1,346 / 1,352 / 1,945 / 2,158 / 2,616 / 2,695 / 56。**最大 2,695 ms** |
| B → A の 10 回 | ms: 322 / 281 / 2,539 / 560 / 150 / 1,813 / 1,063 / 1,188 / 1,296 / 1,090。**最大 2,539 ms** |
| 両方で続けて 1 分（A と B が追加・追加・削除・削除を繰り返す） | 192 操作。その後の A の表は 0 行（[real-15](screenshots/2026-10-21-real-15-nas-after-minute.png)）、知らせなし |
| 壊れた行 | ジャーナル 8 本・235 行のすべてが `v1 <時計> <32 桁> <秒> … <8 桁の CRC>` の形（PowerShell の正規表現で数えて 0 件）。窓に「読めなかった行」は出ない |
| 3 台目（新しいアカウント `appdata-c4〜c6`・起動して約 3 秒後に追加 1 回） | 追加から A に見えるまで 830 / 2,287 / 2,315 ms、B はその後 +1,092 / +1,127 / +23 ms（[real-14](screenshots/2026-10-21-real-14-nas-third-accounts.png)） |

🔴 **正直に:**
- **2 台の PC ではない。** 施主の手元に 2 台目が無いので、同じ PC の 2 プロセス。SMB のクライアントのキャッシュ（属性・一覧・oplock）は 1 台の中で共有されるので、別の PC の間の遅れはこの数字に出ていない。
- 3 台目のジャーナルは**起動の時に**（空で）作られ、最初の操作はその約 3 秒後。`DirectoryCacheLifetime` 10 秒が効く「一覧に新しいファイルが見えない」窓を、この順序ではほとんど踏んでいない（推測）。別の PC から起動直後に操作した場合の秒数は未測定。
- 自動化の最初の試し（`nas.ps1`）は、測る関数の名前が PowerShell の `Measure-Object` の別名に負けて秒数を取れず、3 台目の追加は窓の外を押して空振りした（`appdata-c1〜c3` のジャーナルは 0 バイトで残っている）。数字は 2 回目（`nas2.ps1`）だけを使う。
- append+sync が 1 回 50 ms 前後。1 操作ごとに同期するので、NAS では 1 秒に 20 操作ほどが上限（推測・画面の操作では気にならない量）。

## 52. 再現

```bash
make prove
cargo test -p xtask
make conformance
make coverage
```
