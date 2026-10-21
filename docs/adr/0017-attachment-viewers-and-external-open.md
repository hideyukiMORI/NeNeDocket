# ADR 0017 — 添付の中身: 文と表は段の中に、それ以外は OS の既定のアプリで開く。store に `encoding_rs` を許す

- 状態: **受理**（施主 2026-10-21「csv やテキストもビューワーが欲しくて、Excel、ワードなどはパスへリンクか起動」→「全部推しで」・Issue #63）
- 日付: 2026-10-21
- Issue: #63。画像（PNG / JPEG）は #64、ドラッグ＆ドロップは #65、PDF は #66
- 影響する規則: FR-022 / ARC-007（プロセスの起動）/ ARC-013（store の外部クレート）/ 規約 第 5 節の表 / CNF-013

## 文脈

添付（ADR 0016）の中身を見たい。テキストと CSV は窓の中で読みたく、Excel の CSV は `Shift_JIS` で来る。Excel や Word のファイルは自前で描けないので OS の既定のアプリに渡す。中核は決定的（ARC-007）で、ファイルの読みとプロセスの起動はアダプタと store だけ。文字コードの変換は自前で書く量ではない（`Shift_JIS` の表は 7,000 対）。

## 決定

1. **種類は拡張子で決める**（core `AttachmentKind`・大文字小文字を問わない）: 文（`txt` / `md` / `log` / `json` / `toml` / `yaml` / `yml` / `xml` / `ini` / `cfg` / `rs` / `py` / `sh` / `html` / `htm` / `svg`）・表（`csv` は `,`・`tsv` は `\t`）・画像（`png` / `jpg` / `jpeg`・#64 まではそれ以外と同じ）・それ以外
2. **読むのは store。** `JournalPort::read_attachment` が写しを文にして返す。BOM があればそれ（UTF-8・UTF-16 LE / BE）、無ければ UTF-8、通らなければ `Shift_JIS`。読めない並びは U+FFFD。上限は 256 KiB で、超えれば先頭だけにして途中で切れた最後の行を落とし、「先頭だけを出しています」と示す（大容量でも軽い・FR-017）
3. **`encoding_rs` を store に許す**（Mozilla・`(Apache-2.0 OR MIT) AND BSD-3-Clause`・依存は `cfg-if` だけ・`default-features = false` ＋ `alloc`）。ARC-013 の「store は依存 0」はこれだけ緩める。application・xtask は依存 0 のまま。`deny.toml` の `allow` は変えない（3 つとも既にある）。CNF-013 の表に足す
4. **段の中の見せ方**（application `AttachmentView` / `AttachmentsPane`）: 行を選ぶと一覧は見出し ＋ 3 行までに縮み、下に中身が出る。文は core の `text_area` の読み取り専用（行の単位でスクロール）。表は core の `csv::parse`（`"` の囲み・`""`・`\r\n`）で行と欄にし、最初の行を見出し、列の幅は欄の字数から（8 px × 字数・60〜320 px）。それ以外はパスと「開く」のボタン。読むのは鍵（台帳の世代・課題・連番）が変わったときだけ（`AttachmentViewCache`）
5. **OS の既定のアプリで開く**は `OpenerPort`（application）。X11 は `xdg-open` を起動して別のスレッドで終わりを待つ（ゾンビを残さない）、Windows は `ShellExecuteW(open)`（戻りが 32 より大きければ成功）。失敗は「OS のアプリで開けません」と示す（FR-014）。**行を押しただけでは起動しない**（Issue の文は「行を押すと開く」だが、行を押すのは外すための選択でもあるので、起動は「開く」のボタンに分けた）
6. **`Ports` に 4 つ目**（`opener`）。合成ルートが結ぶ。テストは偽の起動（開いたパスを覚える）

## 強制

- active: 拡張子の判定（core `attachment_kind`）、CSV の読み（core `csv`）、種類ごとの見せ方（application `attachment_view`）、段の中身・「開く」・失敗の知らせ・先頭だけの印（application `docket_screen`）、文字コードの判定と上限（store `attachment_store`: UTF-8・BOM・`Shift_JIS`・256 KiB）、`xdg-open` の起動と無い命令の失敗（adapters-x11 `external_opener`）。CNF-013（store の表に `encoding_rs`）。Issue #63・gate-proofs 第 40 節
- 不能: Windows の `ShellExecuteW` は施主の Windows で PDF を既定のアプリで開いた（施主の報告・gate-proofs 第 51.5 節）。`xdg-open` の先のアプリが開けたかは分からない（`xdg-open` の戻りだけ）

## 結果

- 得られるもの: テキストと CSV（Excel の `Shift_JIS` も）が窓の中で読める。他は OS に渡せる。中核は決定的のまま
- 失うもの: store に外部クレートが 1 つ（`encoding_rs`）。列の幅は字数の近似（core はフォントを知らない）
- 残る穴: 文の 1 行の幅は折り返さない（長い行は切れる）。表は横にスクロールしない。文字コードは 3 つだけ（EUC-JP・GB は U+FFFD になる）。画像は #64、PDF は #66。開く先のアプリは OS の設定次第
