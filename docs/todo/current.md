# いまのタスク — NeNe Docket

> GitHub Issue と PR が進行状態の正本。この文書はその要約（2026-10-21 時点）。

## 状態

| | |
| --- | --- |
| 言語と土台 | Rust / x11rb ＋ cairo-rs / Win32 ＋ Direct2D（[ADR 0002](../adr/0002-no-toolkit-x11rb-cairo-win32-direct2d.md)）。施主決定 2026-10-21 |
| データの形 | アカウントごとの追記専用ジャーナルの併合（[ADR 0004](../adr/0004-append-only-journal-per-account.md)）。施主決定 2026-10-21 |
| 規範文書・ADR 0001〜0004 | ✅ Issue #1（文書だけ） |
| 機械（Cargo / lint の表 / clippy.toml / deny.toml / xtask / Makefile / CI / フック） | ✅ Issue #2・[ADR 0005](../adr/0005-rust-foundation-and-measured-limits.md)。nene-ruler から移植 |
| ゲート発火の証明 `make prove`（26 件）と `cargo test -p xtask`（80 件） | ✅ Issue #2・[gate-proofs](../quality/gate-proofs.md) 第 4〜8 節。測れた規則は active |
| 部品の核（配置・判定・フォーカス・編集状態・描画命令）を `crates/core` に | ✅ Issue #4・[ADR 0006](../adr/0006-widget-kernel-in-core.md)。テスト 49 件・region 99.71%。ARC-003 と QLT-009 を active に |
| ui-x11 / adapters-x11 / store / application / app | ❌ 無い |
| IME の方式（FR-015） | ❌ ADR 未。X11 の窓を出して XIM を実測してから決める |
| 署名と鍵の方式（FR-010） | ❌ ADR 未。権限の Issue で決める |
| CI の `check`（`make ci`）と commit-msg フック | ✅ PR #3 で CI が動き、枝名で落ちた（gate-proofs 第 10 節）。QLT-005・GIT-001・GIT-003 を active に |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |

## 次に行うこと

1. **PR #3（Issue #1 / #2 / #4）を merge する。** 枝は harness の `claude/cloud-session-qus6l5` で、CI の `make ci` は GIT-002 で落ちている（gate-proofs 第 10 節）。ruleset が無いので merge はできる。施主が squash の件名を PR のタイトル（GIT-003 の形）で付けて merge する。ruleset（main の保護・squash-only・`check` を必須）を設定したら gate-proofs に記録し、GIT-002 / GIT-004 を active に
2. **X11 の窓**（ui-x11 ＋ adapters-x11 ＋ app）: x11rb ＋ cairo で窓を出し、core の `DrawCommand` を描き、表 1 枚と入力欄を動かす。Xvfb で撮る。XIM を実測して IME の ADR。依存を足す ADR（x11rb / cairo-rs）と deny.toml の実測
3. core の `Item` / `Op` / `merge`（ADR 0004）と、同じ操作の集合を別の順で読むテスト。store のジャーナル版 1
4. 署名の ADR と権限（FR-010）。スナップショットの ADR（FR-017 を実測してから）
5. 環境の setup script に `cargo-llvm-cov`・`xdotool`・`mingw-w64` を足す（DEVELOPMENT_WORKFLOW 第 9 節）

## 残る穴

- 本文の同時編集は項目単位で勝敗が付く（ADR 0004）。文字単位の併合はしない
- 同じアカウントを 2 台で同時に開くことは守らない（機械 ID は別 Issue）
- 権限を有効にしなければ、なりすましは防げない（ファイルを直接編集すれば）
- SMB の上で本当に壊れないかは、施主の共有で確かめるまで「動く」と書かない（QLT-013）
- アクセシビリティは無い（ADR 0002）
- CNF-014（Windows の 2 クレートの lint の表）は実ファイルでまだ発火させていない。クレートが生まれてから
- `make lint-windows` は `xtask` と `core` しか見ない。Windows のコードが生まれるまで
