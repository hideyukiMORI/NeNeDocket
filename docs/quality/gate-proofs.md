# ゲート発火の証明 — NeNe Docket

> 記録: 2026-10-21 / Issue #1。以下は実行した結果のみ。
> 関連: QLT-007 / QLT-011 / QLT-013 / ADR 0001

## 1. 環境と範囲

Issue #1 は文書だけである。**このリポジトリの道具で測ったゲートは 1 つも無い。** `Cargo.toml`・`xtask`・`Makefile` が無いので `make prove` は走らない。

## 2. active の規則と、その証明

CNF-009 は active の規則ごとに `| ID |` で始まる行がここにあることを要求する。
**active の規則は無いので、行は無い。** Issue #2 で `make prove` を置き、発火した規則から行を足す。

| 規則 | 最小の違反 | 検証経路 | 実測 |
| --- | --- | --- | --- |

## 3. Issue #1 で確かめたこと

文書だけなので、確かめたのは文書の整合だけ。CNF-009 が機械で見る予定のものを手元の grep で見た。

| 対象 | 方法 | 結果 |
| --- | --- | --- |
| 規則 ID の参照先が実在する | 全 `.md` の `ARC-NNN` / `RS-NNN` / `QLT-NNN` / `CNF-NNN` / `GIT-NNN` を集め、`### ID —` の見出しと突き合わせ | Issue #1 の PR に記録 |
| 相対リンクが切れていない | 全 `.md` の Markdown リンクのうち `:` を含まないものを、その文書からの相対で解決 | 同上 |
| 強制マトリクスに active が無い | `docs/QUALITY_GATES.md` 第 3 節を grep | 同上 |
| 未置換の雛形値が無い | 二重の波括弧で囲んだ値を grep | 同上 |

## 4. nene-ruler からの引用（本リポジトリの実測ではない）

本リポジトリの規範文書が引く P-01〜P-26 の実出力は、nene-ruler が rustc 1.98.0 で測ったものである
（[nene-ruler の gate-proofs](https://github.com/hideyukiMORI/nene-ruler/blob/main/docs/quality/gate-proofs.md)）。
同じ道具を同じ版で置けば同じ出力が出る見込みだが、**見込みは証明ではない。** Issue #2 でここに実出力を置くまで、どの規則も active と書かない。

## 5. まだ証明していないこと

- すべて。lint の二段構え・`xtask` の CNF・cargo-deny・rustfmt・rustdoc・`--locked`・クレートの循環拒否
- CI の `make ci` と commit-msg フックの実行証拠（最初の PR で得る）
- ruleset（`main` の保護・squash-only）。未設定
- 併合の決定性（同じ操作の集合を別の順で読んでも同じ状態になること）。core が生まれてから
- Xvfb での表示・Windows の実機・SMB の上での同時編集

## 6. 再現（Issue #2 以降）

```bash
make prove
cargo test -p xtask
make conformance
```
