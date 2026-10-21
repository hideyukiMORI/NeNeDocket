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

## これから書くもの（Issue が立ったら）

- 文字入力（IME）の方式（FR-015）
- 署名と鍵の方式・依存（FR-010）
- スナップショット（ジャーナルが長くなったとき・FR-017）
