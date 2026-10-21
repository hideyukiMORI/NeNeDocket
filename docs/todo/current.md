# いまのタスク — NeNe Docket

> GitHub Issue と PR が進行状態の正本。この文書はその要約（2026-10-21 時点・Issue #36 まで）。

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
| 設定ファイル（FR-012）: 最近の台帳と読み直しの間隔 | ✅ Issue #16。引数なしで最近の台帳を開く。読めない設定は起動を止めて示す（gate-proofs 第 16 節）。列の幅と順・見えはまだ |
| ラベルを画面から変える（FR-001） | ✅ Issue #20。`,` 区切り。検索はラベルにも効く（gate-proofs 第 18 節） |
| 表示言語 6 つと配色 125（FR-019 / FR-012 の見え） | ✅ Issue #27・[ADR 0011](../adr/0011-ui-language-is-a-versioned-setting.md) / [ADR 0012](../adr/0012-theme-model-and-builtin-colorschemes.md)。設定は版 2。Xvfb で日本語・简体中文・한국어・Español と dracula / solarized-light（gate-proofs 第 21 節）。翻訳と配色の見た目は施主の目を通していない |
| 本文（複数行の欄・FR-001） | ✅ Issue #31。core の `text_area`、道具の段の「本文」、表の下の段（履歴と同時なら 1/4 ずつ）。検索は本文にも。Xvfb で 2 プロセス（gate-proofs 第 23 節）。スクロールバーは Issue #36（第 26 節） |
| 参加者の段と画面からの権限（FR-009 / FR-010） | ✅ Issue #33。名前・ID・役。持ち主は「権限」のチェックと行の押下で編集者を切り替える。グループはコマンドラインだけ（gate-proofs 第 24 節） |
| 履歴から選び直す・上書きの通知（FR-008） | ✅ Issue #35。履歴の `Set` の行を押すとその値を新しい `Set` で追記。欄を読み込んだ後に他人が変えた値を上書きすると通知（gate-proofs 第 25 節） |
| 課題の履歴の段（FR-006） | ✅ Issue #18。いつ（UTC）・誰が（ID の先頭 4 桁）・何を・何から何へ。Undo の行は控えめな色（gate-proofs 第 17 節） |
| adapters-windows / ui-win32（Windows の窓・ADR 0009） | 🟡 Issue #22。`x86_64-pc-windows-gnu` の clippy 0 件・exe 1.9 MB。**Windows では未実行**（施主の Windows で確かめる・gate-proofs 第 19 節） |
| IME の方式（FR-015） | ❌ ADR 未。コンテナには IM サーバ（fcitx / ibus）が無く XIM を測れない。施主の WSLg で測る |
| 署名と権限（FR-010） | ✅ Issue #29・[ADR 0010](../adr/0010-signatures-and-permissions.md) 受理。core に `ed25519-dalek`、`account.v2.txt` に秘密鍵、台帳の `keys/` と `permissions.v1.txt`、ジャーナル版 2（署名付き）。持ち主はコマンドライン（`--permissions on` / `--allow` / `--group`）。Xvfb で 3 アカウント（gate-proofs 第 22 節） |
| FR-017 の実測（1 万件・1 年分の併合） | ✅ Issue #24。30 万操作・45 MB を 0.6〜0.7 秒で併合（gate-proofs 第 20 節）。スナップショットはまだ要らない |
| CI の `check`（`make ci`）と commit-msg フック | ✅ PR #3 で CI が動き、枝名で落ちた（gate-proofs 第 10 節）。QLT-005・GIT-001・GIT-003 を active に |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |

## 次に行うこと

1. **PR #3（Issue #1 / #2 / #4）を merge する。** 枝は harness の `claude/cloud-session-qus6l5` で、CI の `make ci` は GIT-002 で落ちている（gate-proofs 第 10 節）。ruleset が無いので merge はできる。施主が squash の件名を PR のタイトル（GIT-003 の形）で付けて merge する。ruleset（main の保護・squash-only・`check` を必須）を設定したら gate-proofs に記録し、GIT-002 / GIT-004 を active に
2. 鍵の失効・交代の手続き（ADR 0010 の残る穴）。グループを画面から。段の高さの調整。過去の時点の再現（FR-006 後半）
3. **IME の実測は施主の WSLg で**: fcitx5 か ibus を入れ、`XMODIFIERS=@im=fcitx` で XIM の preedit が x11rb の窓に届く経路（XIM プロトコルの自前実装か、IM サーバの D-Bus か）を決める ADR
4. スナップショットの ADR は、1 年分の併合が 1 秒を超えてから（いまは 0.7 秒）
5. 環境の setup script（環境の設定）に DEVELOPMENT_WORKFLOW 第 9 節の行を置く。リポジトリ側のフック（`.claude/hooks/session-start.sh`）には `mingw-w64` を足した（Issue #24）
6. **施主の Windows で exe を動かす**（Issue #22 の受け入れ）。見えた欠陥は Issue にする

## 残る穴

- 本文の同時編集は項目単位で勝敗が付く（ADR 0004）。文字単位の併合はしない
- 同じアカウントを 2 台で同時に開くことは守らない（機械 ID は別 Issue）
- 権限を有効にしなければ、なりすましは防げない（ファイルを直接編集すれば）。有効にすると、それより前の版 1 の履歴は捨てられて数に出る（ADR 0010）。秘密鍵は設定フォルダの平文
- SMB の上で本当に壊れないかは、施主の共有で確かめるまで「動く」と書かない（QLT-013）
- アクセシビリティは無い（ADR 0002）
- Windows 版は組み立てただけで、Windows で動かしていない（ADR 0009・gate-proofs 第 19.3 節）。IME の確定前の文字列（`WM_IME_COMPOSITION`）と起動後の dpi の変化はまだ
- アカウントと設定の読み書きは adapters-x11 と adapters-windows に 2 度ある（adapters は store に依存できない。ADR 0009）
- `make lint-windows` は X11 の 2 クレートだけを外す（`app` は Windows の合成ルートで組む）
- `xdotool windowclose` は WM の無い Xvfb では `WM_DELETE_WINDOW` を送らず窓を消す。閉じる要求の経路は WSLg で確かめる
- `Add` は `create` だけを追記し、担当は空のまま（編集行で変える）。アカウントの名前は `USER` で、画面から変えられない（ADR 0008）
- 読み直しの刻みは X の `ClientMessage` で、Windows では別の経路になる（ADR 0008）。間隔は設定の `refresh_seconds`（Issue #16）
- 設定の「列の幅と順」はまだ無い（列幅を変える経路ができてから）。見えは Issue #27 で入った
- X11 では OS の明暗を観測できない（`system` はライト）。GNOME の `color-scheme` などを読む経路は別の Issue。利用者のテーマファイル（nene-nib の `.v1.theme`）もまだ
- 翻訳（6 言語 × 31 語）と導出した配色の見た目は施主の確認待ち
- 履歴の「いつ」は UTC（時差を知るアダプタが無い）、「誰が」は `keys/` の名前（無ければ ID の先頭 4 桁）。過去の時点の状態の再現（FR-006 後半）はまだ（履歴の行を押して 1 項目ずつ選び直せるだけ・Issue #35）
