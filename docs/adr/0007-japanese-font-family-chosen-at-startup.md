# ADR 0007 — 日本語のフォントは、起動時に「日」の字形を持つ家族を探して 1 つ選ぶ

- 状態: 受理
- 日付: 2026-10-21
- Issue: #5
- 影響する規則: ARC-007 / ARC-011 / ARC-013 / QLT-013。ADR 0002 の決定 6（「fallback は OS に任せる」）を改める

## 文脈

ADR 0002 は「文字の描画は cairo の toy text API に任せ、フォントの fallback も OS に任せる」と書いた。
Issue #5 で X11 の窓を出して実測すると、`sans-serif` で日本語が全部豆腐（`.notdef` の箱）になった（gate-proofs 第 11 節の 1 枚目の PNG）。

原因: cairo の toy text API は家族名を 1 つ fontconfig に渡すだけで、無い字形を別のフォントで補わない。
fallback をするのは pango（＋ HarfBuzz）であって cairo ではない。このコンテナの `sans-serif` は DejaVu Sans（後に Inter）に解決し、どちらにも CJK は無い。

試した判定法:

1. 「日」と私用領域 U+E000 の `text_extents` が同じなら字形が無い → **外れた**。`sans-serif` が Inter に解決すると、U+E000 と「日」で箱の幅が違い、「ある」と誤判定した
2. `ScaledFont::text_to_glyphs` で字形の番号を見て、0（`.notdef`）なら無い → **当たった**。`IPAPGothic` が選ばれ、日本語が出た

## 決定

**UI は起動時に 1 度だけ、候補の家族を順に `text_to_glyphs` で試し、「日」の字形番号が 0 でない最初の家族を選ぶ。無ければ `sans-serif` のまま。**

1. 候補の順: `sans-serif` → `Noto Sans CJK JP` → `IPAPGothic` → `IPAGothic` → `VL PGothic` → `TakaoPGothic` → `WenQuanYi Zen Hei`。`sans-serif` が最初なのは、fontconfig の既定が CJK を持つ環境（日本語ロケールの多くの Linux）ではそれを尊重するため
2. 選んだ家族は `Display` が持ち、`Canvas::render` に渡す。core は知らない（文字の幅と同じく UI の仕事・ADR 0006）
3. 全部の文字をその 1 家族で描く。英数字も日本語フォントの字形になる（IPA P ゴシックの英数字は読める）
4. 選べなかったときは豆腐を認める。黙って別の経路（pango）を足さない

## 強制

- **active**: 選ぶ関数の正例・反例（`cargo test -p nene-docket-ui-x11 font`）。`DejaVu Sans` で「日」が無く `IPAGothic` で有ることを見る。このコンテナのフォントに依るので、環境検証として gate-proofs に書く（QLT-013）
- **不能**: 施主の WSLg（Ubuntu 22.04）に候補の家族が在るかは、施主の実機で見るまで分からない

## 結果

得られるもの:

- 依存を足さずに日本語が出る。`cairo-rs` と `x11rb` のまま（ARC-013）
- 判定が決定的（フォントの集合が同じなら同じ家族）

失うもの・残る穴（**正直に**）:

- 🔴 **字形単位の fallback は無い。** 日本語フォントに無い字（絵文字・一部の記号）は豆腐のまま。ADR 0002 の「日本語と英数字だけを動くと書く」はそのまま
- 🔴 候補の名前は決め打ち。Windows 版（DirectWrite）は fallback を OS が持つので別の経路になる。「2 OS で同じ見た目」は字形の形までは及ばない
- 英数字の見た目が日本語フォント次第で変わる。`sans-serif` が CJK を持つ環境と持たない環境で字が違う
- 文字の整形（結合・右から左）は相変わらず無い

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| pango を入れる | fallback と整形を正しくやる唯一の道だが、依存が glib / gobject / pango / harfbuzz と広がり、ARC-013 の許可制に対して大きすぎる。絵文字や多言語が要件になったら再検討する |
| fontconfig の C API を FFI で直接引く | `unsafe` と libfontconfig の束ねが要る。`text_to_glyphs` で同じことが safe にできた |
| `fc-match` を子プロセスで呼ぶ | `std::process::Command` は ui-x11 で禁止（CNF-012）。アダプタに置いても、フォントの選択は描画の都合であって OS 資源のポートではない |
| 家族を設定ファイルで指定させる | 設定ファイルはまだ無い（FR-012 は次の Issue）。入ったら「候補の先頭に設定の家族を足す」で両立する |
| 豆腐のまま IME の ADR まで待つ | 画面確認のたびに題名が読めない。1 関数で直るものを放置する理由が無い |

## 関連

- ADR 0002（toy text API）・ADR 0006（文字の幅は UI）・gate-proofs 第 11 節
