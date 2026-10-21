# ADR 0019 — PDF のビューワー: `hayro` は描けるが、依存の重さと版の重複のため今は入れない（却下・保留）

- 状態: **却下（保留）**（Issue #66「純 Rust の `hayro` を試して ADR」・施主の「全部推しで」の PDF の分。足りなければ却下の理由を書く、と Issue が定めた）
- 日付: 2026-10-21
- Issue: #66
- 影響する規則: FR-022（PDF は OS の既定のビューワーのまま・ADR 0017 の 5）/ ARC-013 / QLT-010（`deny.toml` を弱めない）

## 文脈

施主は PDF も窓の中で見たい。純 Rust の `hayro` 0.8（2025 年・MIT / Apache-2.0・vello の CPU 描画）を scratchpad で試した（`scratchpad/pdftry`・release・このコンテナ）。試した PDF は reportlab で作った 3 つ: ASCII（Helvetica・埋め込みなし）、日本語（IPAゴシックの TrueType を埋め込み）、日本語（平成角ゴシック W5 / 平成明朝 W3 の CID フォント・埋め込みなし）。

## 実測

| 観点 | 結果 |
| --- | --- |
| 描画の正しさ | ASCII・図形・埋め込みの日本語は正しい（[01-embedded](../quality/screenshots/2026-10-21-hayro-01-embedded.png)）。埋め込みなしの CID フォントは**何も出ない**（白紙）。`font_resolver` で IPAゴシックのバイト列を渡せば出る（[02-cid-fallback](../quality/screenshots/2026-10-21-hayro-02-cid-fallback.png)）。明朝の指定もゴシックになる |
| 速さ | A4 1 ページ: 1 倍（595 × 841）で 2〜5 ms、2 倍（1190 × 1683）で 6〜15 ms。読み込みは 0.1 ms。目標の 16 ms に入る |
| 依存 | **75 クレート**（いまのワークスペースは外部 8 ＋ Windows）。vello_cpu・peniko・kurbo・skrifa・read-fonts・image・moxcms・hashbrown・phf・yoke・syn 3 など、描画の土台を丸ごと引く |
| 版の重複 | `syn` 2 と 3、`fearless_simd` 0.7 と 1.1、`miniz_oxide` 0.8 と 0.9。`deny.toml` の `multiple-versions = "deny"` が落ちる。`syn` は `ed25519-dalek` 側（2）と `yoke-derive` 側（3）で、lock では揃えられない |
| ライセンス | MIT / Apache-2.0 が大半。`Zlib` だけのクレートが 2 つ（`allow` に `Zlib` を足す必要）。`Unicode-3.0`・`BSD-3-Clause`・`0BSD` は既に通る |
| ビルド | 依存だけのクリーンビルド（release）**104 秒**、`target/release` 283 MB、rlib 72 本。rust-version 1.92（1.98 で可） |
| フォント | 埋め込みなしの CJK は、こちらがフォントのファイルを渡す必要がある。core は読めないので adapters が OS のフォントを探す経路（ADR 0007 の家族の選び方）を広げるか、フォントを同梱する |

## 決定

**今は入れない。PDF は OS の既定のビューワーで開く（ADR 0017 の 5）のまま。**

理由は 1 つに尽きる: 依存は許可制で、ゲートを弱めて通さない（ARC-013 / QLT-010）。`hayro` を入れるには `multiple-versions` を `warn` にするか 3 つを `skip` に書き、`Zlib` を許し、外部クレートを 8 から 70 余りにし、ビルドを 100 秒伸ばす。添付の PDF を窓の中で見る便利さに対して、この台帳の「小さく・決定的・機械で守る」を崩しすぎる。描画の品質と速さは十分で、却下の理由は hayro の出来ではない。

## 再提案の条件

次のどれかが満たされたら、この ADR を差し替える。

1. 施主が「依存の重さと `deny.toml` の緩和を受け入れる」と決める（そのときは `multiple-versions` の `skip` を 3 つに限り、`Zlib` を `allow` に足し、規約 第 5 節の表に 70 行でなく「`hayro` 一式」として 1 行で書く）
2. `hayro` の依存が整理され、`syn` 3 などの重複が消える
3. PDF の中身を見る頻度が高く、OS のビューワーでは足りないと分かる（実運用の声）

## 結果

- 得られるもの: ワークスペースの依存は今のまま（core 1・store 1・media 2・UI 3）。`make deny` はそのまま通る
- 失うもの: PDF は「開く」のボタンで OS に渡す。窓の中では見えない
- 残る穴: 試した PDF は 3 つで、画像・透明・暗号化・大きなページは測っていない。却下は「今」の判断で、hayro 自体は有望
