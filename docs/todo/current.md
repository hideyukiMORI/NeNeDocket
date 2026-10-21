# いまのタスク — NeNe Docket

> GitHub Issue と PR が進行状態の正本。この文書はその要約（2026-10-21 時点・Issue #66 まで）。

## 状態

| | |
| --- | --- |
| 言語と土台 | Rust / x11rb ＋ cairo-rs / Win32 ＋ Direct2D（[ADR 0002](../adr/0002-no-toolkit-x11rb-cairo-win32-direct2d.md)）。施主決定 2026-10-21 |
| データの形 | アカウントごとの追記専用ジャーナルの併合（[ADR 0004](../adr/0004-append-only-journal-per-account.md)）。施主決定 2026-10-21 |
| 規範文書・ADR 0001〜0004 | ✅ Issue #1（文書だけ） |
| 機械（Cargo / lint の表 / clippy.toml / deny.toml / xtask / Makefile / CI / フック） | ✅ Issue #2・[ADR 0005](../adr/0005-rust-foundation-and-measured-limits.md)。nene-ruler から移植 |
| ゲート発火の証明 `make prove`（26 件）と `cargo test -p xtask`（80 件） | ✅ Issue #2・[gate-proofs](../quality/gate-proofs.md) 第 4〜8 節。測れた規則は active |
| 部品の核（配置・判定・フォーカス・編集状態・描画命令）を `crates/core` に | ✅ Issue #4・[ADR 0006](../adr/0006-widget-kernel-in-core.md)。テスト 49 件・region 99.71%。ARC-003 と QLT-009 を active に |
| X11 の窓（application / ui-x11 / app・x11rb ＋ cairo-rs） | ✅ Issue #5・[ADR 0007](../adr/0007-japanese-font-family-chosen-at-startup.md)。Xvfb で表・検索・絞り込み・チェック・追加・スクロールを xdotool で駆動（gate-proofs 第 11 節）。1 万件の 1 フレーム 0.9 ms |
| core の課題・操作・論理時計・併合（ADR 0004） | ✅ Issue #7。同じ操作の集合を別の順で読んでも同じ台帳になるテスト。region 99.58% |
| store のジャーナル版 1（1 行 1 操作・CRC-32・追記と同期・読み直し・`docket.v1.txt`） | ✅ Issue #9。一時ディレクトリで 20 件のテスト。SMB の上は施主の共有で |
| 画面とジャーナルの配線（`Ledger`・`Ports`・`FileJournal`・追加・題名・状態・削除・Ctrl+Z / Ctrl+Y・3 秒ごとの読み直し） | ✅ Issue #11・[ADR 0008](../adr/0008-account-file-and-refresh-tick.md)。Xvfb で 2 プロセスが同じフォルダを編集し、互いの変更が届いた（gate-proofs 第 14 節）。併合した台帳で 1 万件の 1 フレーム 0.87 ms ＋ 0.70 ms |
| adapters-x11（時計・アカウント・読み直しの刻み） | ✅ Issue #11 |
| 担当と期限を画面から変える（FR-001） | ✅ Issue #14。期限の形が違えば追記せず窓の下で示す（gate-proofs 第 15 節） |
| 設定ファイル（FR-012）: 最近の台帳と読み直しの間隔 | ✅ Issue #16。引数なしで最近の台帳を開く。読めない設定は起動を止めて示す（gate-proofs 第 16 節）。見えは Issue #27、列の幅は Issue #41（版 3・第 28 節）、列の順は Issue #43（版 4・第 29 節）、段の高さは Issue #55（版 5・第 35 節） |
| ラベルを画面から変える（FR-001） | ✅ Issue #20。`,` 区切り。検索はラベルにも効く（gate-proofs 第 18 節） |
| 表示言語 6 つと配色 125（FR-019 / FR-012 の見え） | ✅ Issue #27・[ADR 0011](../adr/0011-ui-language-is-a-versioned-setting.md) / [ADR 0012](../adr/0012-theme-model-and-builtin-colorschemes.md)。設定は版 2。Xvfb で日本語・简体中文・한국어・Español と dracula / solarized-light（gate-proofs 第 21 節）。翻訳と配色の見た目は施主の目を通していない |
| 本文（複数行の欄・FR-001） | ✅ Issue #31。core の `text_area`、道具の段の「本文」、表の下の段（履歴と同時なら 1/4 ずつ）。検索は本文にも。Xvfb で 2 プロセス（gate-proofs 第 23 節）。スクロールバーは Issue #36（第 26 節） |
| 参加者の段と画面からの権限（FR-009 / FR-010） | ✅ Issue #33。名前・ID・役。持ち主は「権限」のチェックで有効・無効を切り替え、行を押して選ぶと編集行（「編集者」のチェック・グループの欄 `,` 区切り）が出る（Issue #53）。グループは役の欄にも出る（Issue #47）（gate-proofs 第 24・30・33 節） |
| 履歴から選び直す・上書きの通知（FR-008） | ✅ Issue #35。履歴の `Set` の行を押すとその値を新しい `Set` で追記。欄を読み込んだ後に他人が変えた値を上書きすると通知（gate-proofs 第 25 節） |
| 過去の時点の再現（FR-006 後半） | ✅ Issue #39。履歴の行の上にポインタを置くと、表の選んだ行と本文の段がその時点の値になる（gate-proofs 第 27 節） |
| 課題の履歴の段（FR-006） | ✅ Issue #18。いつ（UTC）・誰が（ID の先頭 4 桁）・何を・何から何へ。Undo の行は控えめな色（gate-proofs 第 17 節） |
| adapters-windows / ui-win32（Windows の窓・ADR 0009） | 🟡 Issue #22。`x86_64-pc-windows-gnu` の clippy 0 件・exe 1.9 MB。**Windows では未実行**（施主の Windows で確かめる・gate-proofs 第 19 節） |
| IME の方式（FR-015） | 🟡 Issue #80・[ADR 0020](../adr/0020-japanese-input-ime-paths.md) 受理。Windows は IMM32（`WM_IME_COMPOSITION`・候補の窓は欄の下・**実機で未確認**）、X11 は当面 Ctrl+V の貼り付け（Xvfb で確認・gate-proofs 第 46 節）。XIM の自前実装は施主の WSLg で測ってから |
| 署名と権限（FR-010） | ✅ Issue #29・[ADR 0010](../adr/0010-signatures-and-permissions.md) 受理。core に `ed25519-dalek`、`account.v2.txt` に秘密鍵、台帳の `keys/` と `permissions.v1.txt`、ジャーナル版 2（署名付き）。持ち主はコマンドライン（`--permissions on` / `--allow` / `--group`）。Xvfb で 3 アカウント（gate-proofs 第 22 節） |
| 鍵の交代と持ち主の交代（ADR 0013） | ✅ Issue #51・[ADR 0013](../adr/0013-key-rotation-and-one-key-per-machine.md) 受理。`--succeed NEW OLD` で引き継ぎ（古い ID の履歴は新しい名前・参加者の段から消える）、`--owner ACCOUNT` で持ち主の交代（前の持ち主の署名 ＋ `handover`・新しい持ち主が開くと署名し直す）。鍵を失くした持ち主は `docket.v1.txt` を手で直す（gate-proofs 第 32 節）。引き継ぎは参加者の段の編集行の「引き継ぎ先」からも（Issue #54・第 34 節）。新しい ID は古い ID の編集者・グループを受け継ぐ |
| 監査（FR-020） | ✅ Issue #56・[ADR 0014](../adr/0014-administration-is-journaled-for-audit.md) 受理。権限の変更も `admin` の行としてジャーナルに追記（署名付き）。履歴の段は課題を選んでいなければ台帳全体の時系列、`--audit` でテキスト。読んだ記録は残さない（gate-proofs 第 36 節） |
| FR-017 の実測（1 万件・1 年分の併合） | ✅ Issue #24。30 万操作・45 MB を 0.6〜0.7 秒で併合（gate-proofs 第 20 節） |
| 大容量でも軽い（FR-017 を 10 万件・300 万操作に）| ✅ Issue #61・[ADR 0015](../adr/0015-large-dockets-snapshot-and-incremental-merge.md) 受理。段階 1（表の一覧のキャッシュ・増分の併合）で 1 フレーム 22 ms → 1.4 ms。段階 2（自分のスナップショット `snapshot/<account>.v1.txt` ＋ 索引 `.idx`・操作は記憶に持たず履歴はジャーナルから引く）で 300 万操作の 2 回目の起動 8.5 秒 → 0.9 秒・RSS 703 → 178 MB（目標 150 MB は未達・gate-proofs 第 37・38 節） |
| 添付（FR-021） | ✅ Issue #62・[ADR 0016](../adr/0016-attachments-copied-into-the-docket.md) 受理。台帳フォルダの `attachments/<課題>/<連番>-<名前>` に写し、`attach`（名前・大きさ・SHA-256 は core に自前）/ `detach` を追記。添付の段（道具の段の「添付」・パスの欄・「添付する」・「外す」）と `--attach`。写しは消さず、改変は `⚠` で示す。Xvfb で 3 アカウント（gate-proofs 第 39 節） |
| 添付の中身（FR-022） | 🟡 Issue #63・[ADR 0017](../adr/0017-attachment-viewers-and-external-open.md) 受理。行を選ぶと段の下に、文（読み取り専用の欄）・CSV / TSV（表・`Shift_JIS` も・store に `encoding_rs`）・それ以外はパスと「開く」（`xdg-open` / `ShellExecuteW`）。256 KiB で先頭だけ（gate-proofs 第 40 節）。PNG / JPEG は Issue #64・[ADR 0018](../adr/0018-image-viewer-png-jpeg.md) 受理: `crates/media`（`png` / `jpeg-decoder`）で復号し 2,048 画素に縮めて持ち、`DrawCommand::Image` で UI が描く（ホイールで拡大・ドラッグで移動・1 フレーム 1 ms・gate-proofs 第 41 節）。ドラッグ＆ドロップは Issue #65（X11 は XDND・Windows は `WM_DROPFILES`・`FrameInput::with_drop`・表の行ならその課題、段なら選んでいる課題・自前の送り手で Xvfb・gate-proofs 第 42 節）。PDF は Issue #66 で `hayro` 0.8 を試し、[ADR 0019](../adr/0019-pdf-viewer-hayro-rejected-for-now.md) で**今は却下**（A4 1 ページ 5〜15 ms で正しく描けるが、75 クレート・`syn` 2 / 3 などの重複で `deny` が落ちる・クリーンビルド 104 秒。埋め込みなしの CJK はフォントを渡す必要）。OS の既定のビューワーのまま（gate-proofs 第 43 節） |
| CI の `check`（`make ci`）と commit-msg フック | ✅ PR #3 で CI が動き、枝名で落ちた（gate-proofs 第 10 節）。QLT-005・GIT-001・GIT-003 を active に |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |

## 次に行うこと

施主 2026-10-21「SMB 遅延、NAS の癖、ファイル破損、タイムスタンプ差、Windows ロック、日本語入力、OS フリーズなども考慮予定にして」→ Issue #75〜#81 に起こした（棚卸しで見つけた添付のフォルダ名の `:` は Issue #74 で直した）。

- 🟡 #75 SMB の遅延: 他人の読み直しとスナップショットの書きをスレッドへ（`scanner`・`refresh` は待たない）、`--probe-share` で共有を測る道具。残り: 施主の NAS での実測・追記 ＋ sync の 2 秒まとめ（gate-proofs 第 45 節）
- #76 NAS の癖: 属性キャッシュ・rename の原子性・大文字小文字・名前と経路の制約の一覧と検査
- ✅ #77 壊れたファイルへの耐性: 壊し方 × 形の表（docs/design）を store の `damage` で試す。切れた行に改行を足してから追記・止まった書きかけを数える・スナップショットに検査和（gate-proofs 第 44 節）
- #78 壁時計のずれ: 地方時の表示・ずれの印・期限の「今日」・mtime 不使用の検査
- #79 Windows のファイルロック: 共有モード・上書き rename の再試行・ウイルス対策
- 🟡 #80 日本語入力: ADR 0020。Windows は IMM32（確定前の文字列を欄に・候補の窓は欄の下）、X11 は Ctrl+V の貼り付け、確定前は下線。残り: 施主の Windows での確認・WSLg で XIM が届くかの実測（gate-proofs 第 46 節）
- #81 OS のフリーズ・停電・強制終了・スリープ復帰: 止まり方の表と戻ったときの約束

1. **ruleset（main の保護・squash-only・`check` を必須）を施主が GitHub で設定する。** 設定したら gate-proofs に記録し、GIT-002 / GIT-004 を active に。PR は harness の枝 `claude/cloud-session-qus6l5` から squash で merge している（件名は PR のタイトル・GIT-003 の形）
2. PDF のビューワーは保留（ADR 0019 の再提案の条件: 施主が依存の重さと `deny.toml` の緩和を受け入れる・`hayro` の重複が消える・実運用で OS のビューワーでは足りないと分かる）
3. 持ち主の交代はコマンドラインのまま（施主の決定）。引き継ぎを取り消す経路は無い（要るなら Issue）
3. **XIM の実測は施主の WSLg で**: fcitx5 を入れ、`XMODIFIERS=@im=fcitx` で XIM サーバ（`XIM_SERVERS` と `_XIM_XCONNECT`）が x11rb の窓から届くかを見る。届けば XIM の自前実装の Issue（ADR 0020 の 3）
5. 環境の setup script（環境の設定）に DEVELOPMENT_WORKFLOW 第 9 節の行を置く。リポジトリ側のフック（`.claude/hooks/session-start.sh`）には `mingw-w64` を足した（Issue #24）
6. **施主の Windows で exe を動かす**（Issue #22 の受け入れ）。見えた欠陥は Issue にする

## 残る穴

- 本文の同時編集は項目単位で勝敗が付く（ADR 0004）。文字単位の併合はしない
- スナップショットは自分のアカウントのためだけの派生ファイル（消しても正しさは変わらず、全部読み直す）。初回の起動は操作数に比例（300 万操作で 13 秒）。メモリは 10 万件・300 万操作で 178 MB（目標 150 MB）
- 同じアカウントを 2 台で同時に開くことは守らない（`KeyMismatch` で示すだけ）。2 台目は別の ID で参加し、鍵の交代は持ち主の `--succeed`、持ち主の鍵の紛失は `docket.v1.txt` を手で直す（[ADR 0013](../adr/0013-key-rotation-and-one-key-per-machine.md)・Issue #51）。署名の外の経路なので、フォルダを書ける人は持ち主を名乗れる（ADR 0010 の守る相手と同じ）
- 権限を有効にしなければ、なりすましは防げない（ファイルを直接編集すれば）。有効にすると、それより前の版 1 の履歴は捨てられて数に出る（ADR 0010）。秘密鍵は設定フォルダの平文。権限の記録（ADR 0014）は書いた人の言い分で、`permissions.v1.txt` と突き合わせない。読んだ記録は残さない
- SMB の上で本当に壊れないかは、施主の共有で確かめるまで「動く」と書かない（QLT-013）
- アクセシビリティは無い（ADR 0002）
- Windows 版は組み立てただけで、Windows で動かしていない（ADR 0009・gate-proofs 第 19.3 節）。IME の確定前の文字列（`WM_IME_COMPOSITION`・Issue #80）も組み立てただけ。起動後の dpi の変化はまだ
- アカウントと設定の読み書きは adapters-x11 と adapters-windows に 2 度ある（adapters は store に依存できない。ADR 0009）
- `make lint-windows` は X11 の 2 クレートだけを外す（`app` は Windows の合成ルートで組む）
- `xdotool windowclose` は WM の無い Xvfb では `WM_DELETE_WINDOW` を送らず窓を消す。閉じる要求の経路は WSLg で確かめる
- `Add` は `create` だけを追記し、担当は空のまま（編集行で変える）。アカウントの名前は `USER` で、画面から変えられない（ADR 0008）
- 読み直しの刻みは X の `ClientMessage` で、Windows では別の経路になる（ADR 0008）。間隔は設定の `refresh_seconds`（Issue #16）
- 設定の列の幅と順は Issue #41 / #43（ヘッダの境と見出しのドラッグ）、段の高さは Issue #55（表との境のドラッグ・出している段は全部同じ高さ）、見えは Issue #27 で入った
- X11 の OS の明暗は GTK の慣習（`GTK_THEME` と `gtk-3.0/settings.ini`）から読む（Issue #48）。GNOME の `color-scheme`（dconf）は読まない。利用者のテーマファイル（nene-nib の `.v1.theme`）はまだ
- 翻訳（6 言語 × 65 語）と導出した配色の見た目は施主の確認待ち
- 共有（SMB / NAS）の遅延・癖・ロック・破損・時計のずれ・フリーズは Issue #75〜#81 で考慮中。読み直しはスレッドだが追記 ＋ sync はフレームの中（#75 の残り）。施主の実機と NAS で測るまで「動く」と書かない（QLT-013）
- 添付の写しは消えない（外しても残る・ADR 0016）。元のファイルは丸ごと記憶に読む。ドラッグ＆ドロップは `file://` の URI だけ（他のアプリからの画像のデータは受けない）。Windows の `WM_DROPFILES` は Windows で動かしていない。中身の文は折り返さず、表は横にスクロールしない。文字コードは UTF-8・UTF-16（BOM）・`Shift_JIS` だけ（ADR 0017）。画像は 2,048 画素に縮めた画素まで（拡大しても元には戻らない）・復号中は画面が止まる（4000 × 3000 の JPEG で 0.3 秒）・GIF / WebP / BMP と PDF は「開く」のまま（ADR 0018 / 0019）。UI の言語と違う文字はフォントの家族の都合で □ になる（ADR 0007）
- 履歴の「いつ」は UTC（時差を知るアダプタが無い）、「誰が」は `keys/` の名前（無ければ ID の先頭 4 桁）。過去の時点の再現はポインタを置いている間の表示だけ（Issue #39）。台帳をその時点へ戻すのは履歴の行を押して 1 項目ずつ（Issue #35）
