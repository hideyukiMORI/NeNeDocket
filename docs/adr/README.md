# アーキテクチャ決定記録（ADR）

トレードオフのある判断を、**理由ごと**残す場所。正本ドキュメントが「いま何が規範か」を書くのに対し、
ADR は「なぜそう決めたか」「何を却下したか」を書く。

## 書き方

- 連番 4 桁 ＋ kebab の題名: `NNNN-short-kebab-title.md`
- `0000-template.md` を複製して書く
- 状態は `提案` / `受理` / `却下` / `置換（→ NNNN）`
- **却下した選択肢を必ず書く。** 再提案は、その理由への反論から始めること
- 受理された ADR を後から書き換えない。変えるときは新しい ADR で置き換える
- 文脈は可能な限り実データ（コード・履歴・計測・コンパイラの出力）で裏を取る

## 一覧

| ADR | 題名 | 状態 |
| --- | --- | --- |
| [0001](0001-strictness-is-mechanically-enforced.md) | 厳格さは機械で強制する | 受理 |
| [0002](0002-no-toolkit-x11rb-cairo-win32-direct2d.md) | X11 は x11rb ＋ cairo、Windows は Win32 ＋ Direct2D。UI ツールキットを使わず、部品は自前で描く | 受理 |
| [0003](0003-diff-scoped-verification-and-result-reuse.md) | 差分に必要な検証と成功結果の再利用 | 受理 |
| [0004](0004-append-only-journal-per-account.md) | 台帳はアカウントごとの追記専用ジャーナルの併合。サーバもロックも無い | 受理 |
| [0005](0005-rust-foundation-and-measured-limits.md) | Rust の検査基盤と実測できた限界を固定する | 受理 |
| [0006](0006-widget-kernel-in-core.md) | 部品の核は即時モードで core に置く。既製の部品ライブラリは入れない | 受理 |
| [0007](0007-japanese-font-family-chosen-at-startup.md) | 日本語のフォントは、起動時に「日」の字形を持つ家族を探して 1 つ選ぶ | 受理（ADR 0002 の決定 6 を改める） |
| [0008](0008-account-file-and-refresh-tick.md) | アカウントは設定フォルダの乱数の ID。他人のジャーナルは X の合図で読み直す | 受理 |
| [0009](0009-windows-port-plain-window-and-timer.md) | Windows 版は通常の窓に Direct2D で描き、読み直しの刻みは窓のタイマー。アダプタの読み書きは OS ごとに持つ | 受理 |
| [0010](0010-signatures-and-permissions.md) | 権限を有効にした台帳では、操作の行に Ed25519 の署名を付け、読み手が公開鍵で検証する | 受理（Issue #29） |
| [0011](0011-ui-language-is-a-versioned-setting.md) | 表示言語は 6 言語の閉じた集合で、文言は core の 1 表から出し、希望は版付きの設定に残して OS の観測値で解決する | 受理 |
| [0012](0012-theme-model-and-builtin-colorschemes.md) | 配色は nene-nib の組み込み 125 テーマを写し、台帳の 8 色は本文の 3 色から整数の補間で導く。希望は設定に残し `system` は OS の明暗に従う | 受理 |
| [0013](0013-key-rotation-and-one-key-per-machine.md) | 鍵は 1 台に 1 つ。同じ人の 2 台目は別のアカウントで、鍵の交代は持ち主が署名した引き継ぎで表す | 受理（Issue #51） |
| [0014](0014-administration-is-journaled-for-audit.md) | 権限の変更もジャーナルに追記し、台帳全体の時系列を監査として出す（`OpKind::Admin`・履歴の段・`--audit`） | 受理（Issue #56） |
| [0015](0015-large-dockets-snapshot-and-incremental-merge.md) | 大容量でも軽い: 派生値のキャッシュ・増分の併合・アカウントごとのスナップショットと索引 | 受理（Issue #61・段階 1・2 実装済み） |

## これから書くもの（Issue が立ったら）

- 文字入力（IME）の方式（FR-015）
- 利用者のテーマファイル（nene-nib の `.v1.theme`・ADR 0012 の 7）
