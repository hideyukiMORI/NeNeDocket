# いまのタスク — NeNe Docket

> GitHub Issue と PR が進行状態の正本。この文書はその要約（2026-10-21 時点）。

## 状態

| | |
| --- | --- |
| 言語と土台 | Rust / x11rb ＋ cairo-rs / Win32 ＋ Direct2D（[ADR 0002](../adr/0002-no-toolkit-x11rb-cairo-win32-direct2d.md)）。施主決定 2026-10-21 |
| データの形 | アカウントごとの追記専用ジャーナルの併合（[ADR 0004](../adr/0004-append-only-journal-per-account.md)）。施主決定 2026-10-21 |
| 規範文書・ADR 0001〜0004 | ✅ Issue #1（文書だけ） |
| 機械（Cargo / lint の表 / clippy.toml / deny.toml / xtask / Makefile / CI / フック） | ❌ 未着手。Issue #2 |
| ゲート発火の証明 `make prove` | ❌ 何も測っていない（[gate-proofs](../quality/gate-proofs.md)） |
| クレート | ❌ 1 つも無い |
| IME の方式（FR-015） | ❌ ADR 未。最初の縦切りの前に決める |
| 署名と鍵の方式（FR-010） | ❌ ADR 未。権限の Issue で決める |
| CI の必須 check（`make ci`）と commit-msg フック | 未配線 |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |

## 次に行うこと

1. **Issue #2 — 機械の移植。** nene-ruler の `Cargo.toml`（`[workspace.lints]`）・`clippy.toml`・`deny.toml`・`rust-toolchain.toml`・`xtask`・`Makefile`・`.github/workflows/check.yml`・`.githooks`・`.claude` を写し、クレート名（`nene-docket-`）と CNF-012 / CNF-013 の表（`store` を除外・許可表）を NeNe Docket の [PROJECT_LAYOUT](../PROJECT_LAYOUT.md) に合わせる。製品コード 0 の workspace（core / application / xtask だけ）で `make prove` と `cargo test -p xtask` を回し、[gate-proofs](../quality/gate-proofs.md) に実出力を置いて、発火したものから active に書き換える。実測の記録は ADR（nene-ruler ADR 0003 相当）
2. **IME の ADR**（XIM と TSF / IMM32 の実測）。最初の縦切りで文字入力が要るので、その前
3. **最初の縦切り**（Issue #3 相当）: core の `Item` / `Op` / `merge`、store のジャーナル版 1、application の所有者、ui-x11 の表 1 枚。Xvfb で撮る。同じ操作の集合を別の順で読むテスト
4. 署名の ADR と権限（FR-010）。スナップショットの ADR（FR-017 を実測してから）

## 残る穴

- 本文の同時編集は項目単位で勝敗が付く（ADR 0004）。文字単位の併合はしない
- 同じアカウントを 2 台で同時に開くことは守らない（機械 ID は別 Issue）
- 権限を有効にしなければ、なりすましは防げない（ファイルを直接編集すれば）
- SMB の上で本当に壊れないかは、施主の共有で確かめるまで「動く」と書かない（QLT-013）
- アクセシビリティは無い（ADR 0002）
- 文書はすべて nene-ruler からの写しと翻案であり、このリポジトリの道具で何も測っていない
