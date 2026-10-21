# ADR 0018 — 画像のビューワー: PNG / JPEG を `crates/media` で復号し、UI が伸縮して描く

- 状態: **受理**（施主 2026-10-21「画像や pdf はビューワーも欲しい」→「全部推しで」・Issue #64。PDF は #66）
- 日付: 2026-10-21
- Issue: #64
- 影響する規則: FR-022 / FR-017（1 フレーム 16 ms）/ ARC-013（新しいクレート 2 つ）/ ADR 0006（描画命令）/ PROJECT_LAYOUT（新しいクレート）/ CNF-013

## 文脈

添付の PNG と JPEG を窓の中で見たい（ADR 0017 ではパスと「開く」だった）。core は `#![no_std]`・依存は署名だけ、application は依存 0。復号は純 Rust のクレートに任せたいが、置く場所が要る。描くのは cairo（X11）と Direct2D（Windows）で、画素の列を渡す形が違う。4000 × 3000 の JPEG（1,200 万画素・RGBA で 48 MB）を毎フレーム描けば 16 ms を超える。

## 決定

1. **新しいクレート `crates/media`**（`nene-docket-media`）: バイト列 → `Bitmap`（RGBA 8 ビット・core の値）。外部クレートは `png` 0.18 と `jpeg-decoder` 0.3（image-rs・MIT / Apache-2.0）だけ。ファイルも時計も読まない。依存は core だけで、store が使う（store → media → core）。`png` の `flate2` は `Cargo.lock` で 1.1.4 に留める（1.1.10 は `miniz_oxide` 0.9 を引いて 0.8 と 2 版になり、`deny.toml` の `multiple-versions = "deny"` が落とす）。`deny.toml` の `allow` は変えない（`(MIT OR Apache-2.0)`・`MIT OR Zlib OR Apache-2.0`・`0BSD OR MIT OR Apache-2.0` は MIT で通る）
2. **縮めて持つ。** 長い辺が 2,048 画素を超えれば、整数の倍率の箱の平均で縮める（`downscale`）。4000 × 3000 は 2000 × 1500（12 MB）になる。それ以上に拡大しても元の画素には戻らない（残る穴）
3. **描画命令 `DrawCommand::Image { dest, source, bitmap }`**（core・ADR 0006）。`bitmap` は `SharedBitmap`（`Arc<Bitmap>` ＋ ID）。命令は毎フレーム複製されるので画素は共有し、同じかどうかは ID で決める。ID は添付の SHA-256 の先頭 8 バイト（同じ中身なら同じ）。UI は ID で覚える（X11 は cairo の `ImageSurface`、Windows は `ID2D1Bitmap`・どちらも 8 枚まで、超えたら全部忘れる）。RGBA → α を掛けた BGRA の変換は core の `Bitmap::premultiplied_bgra`（両 UI で同じ）
4. **段の中の見せ方**（application）。行を選ぶと `JournalPort::read_attachment_image` で読み、収まる大きさを 100% として中央に置く（core `image_fit::place`・純関数）。ホイール 1 刻みで 10%（10〜1600%）、押したまま動かすとずれる。別の添付を選ぶと戻る。復号できなければ「画像を読めません」（FR-014）
5. **伸縮のフィルタは Bilinear。** cairo の既定（Good）は縮めるときに畳み込みで、2000 × 1500 を 1100 × 260 に描くのに 36 ms かかった。Bilinear は 1 ms（gate-proofs 第 41 節）。縮める倍率が大きいと粗くなるが、2 で縮めてから描くので 4 倍まで

## 強制

- active: PNG（α・RGB・灰・灰 ＋ α）と JPEG の復号、縮小、壊れた入力の拒否（media）、`place` の中央・拡大・ずれ（core `image_fit`）、`premultiplied_bgra`・`SharedBitmap` の同一性（core）、段の画像・ホイール・ドラッグ・復号の失敗（application `docket_screen`）、surface の覚えと上限（ui-x11 `surface_cache`）、CNF-013（media と store の表）。Issue #64・gate-proofs 第 41 節
- 不能: Windows の `CreateBitmap` / `DrawBitmap` は Windows で動かしていない（ADR 0009 と同じ）

## 結果

- 得られるもの: PNG / JPEG が段の中で見られ、拡大と移動ができる。1 フレーム 1 ms（目標 16 ms）。依存は復号の 2 つだけ（`image` クレートは入れない）
- 失うもの: クレートが 1 つ増え、store の依存に media が加わる。復号は選んだときに同期で走る（4000 × 3000 の JPEG で 0.3 秒・画面が止まる）
- 残る穴: 拡大しても 2,048 画素に縮めた画素まで。復号中は止まる（スレッドに逃がすなら別の Issue）。GIF / WebP / BMP は「開く」のまま。EXIF の向きは見ない
