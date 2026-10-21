# いまのタスク — NeNe Docket

> GitHub Issue と PR が進行状態の正本。この文書はその要約（2026-10-21 時点・Issue #20 まで）。

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
| 課題の履歴の段（FR-006） | ✅ Issue #18。いつ（UTC）・誰が（ID の先頭 4 桁）・何を・何から何へ。Undo の行は控えめな色（gate-proofs 第 17 節） |
| adapters-windows / ui-win32 | ❌ 無い |
| IME の方式（FR-015） | ❌ ADR 未。コンテナには IM サーバ（fcitx / ibus）が無く XIM を測れない。施主の WSLg で測る |
| 署名と鍵の方式（FR-010） | ❌ ADR 未。権限の Issue で決める |
| CI の `check`（`make ci`）と commit-msg フック | ✅ PR #3 で CI が動き、枝名で落ちた（gate-proofs 第 10 節）。QLT-005・GIT-001・GIT-003 を active に |
| ruleset（main の保護・squash-only） | 未設定。施主が GitHub で設定したら gate-proofs に記録する |

## 次に行うこと

1. **PR #3（Issue #1 / #2 / #4）を merge する。** 枝は harness の `claude/cloud-session-qus6l5` で、CI の `make ci` は GIT-002 で落ちている（gate-proofs 第 10 節）。ruleset が無いので merge はできる。施主が squash の件名を PR のタイトル（GIT-003 の形）で付けて merge する。ruleset（main の保護・squash-only・`check` を必須）を設定したら gate-proofs に記録し、GIT-002 / GIT-004 を active に
2. 署名の ADR と権限（FR-010）。本文（複数行の欄・FR-001 の残り）。アカウントの名前の一覧（FR-009・ジャーナルに名前を置く版付きの変更）
3. **IME の実測は施主の WSLg で**: fcitx5 か ibus を入れ、`XMODIFIERS=@im=fcitx` で XIM の preedit が x11rb の窓に届く経路（XIM プロトコルの自前実装か、IM サーバの D-Bus か）を決める ADR
4. スナップショットの ADR（FR-017 を実測してから）
5. 環境の setup script に `cargo-llvm-cov`・`xdotool`・`mingw-w64` を足す（DEVELOPMENT_WORKFLOW 第 9 節）

## 残る穴

- 本文の同時編集は項目単位で勝敗が付く（ADR 0004）。文字単位の併合はしない
- 同じアカウントを 2 台で同時に開くことは守らない（機械 ID は別 Issue）
- 権限を有効にしなければ、なりすましは防げない（ファイルを直接編集すれば）
- SMB の上で本当に壊れないかは、施主の共有で確かめるまで「動く」と書かない（QLT-013）
- アクセシビリティは無い（ADR 0002）
- CNF-014（Windows の 2 クレートの lint の表）は実ファイルでまだ発火させていない。クレートが生まれてから
- `make lint-windows` は `xtask`・`core`・`application` を見る。`ui-x11` と `app` は Windows では組めないので外している
- `xdotool windowclose` は WM の無い Xvfb では `WM_DELETE_WINDOW` を送らず窓を消す。閉じる要求の経路は WSLg で確かめる
- `Add` は `create` だけを追記し、担当は空のまま（編集行で変える）。アカウントの名前は `USER` で、画面から変えられない（ADR 0008）
- 読み直しの刻みは X の `ClientMessage` で、Windows では別の経路になる（ADR 0008）。間隔は設定の `refresh_seconds`（Issue #16）
- 設定の「列の幅と順・見え」はまだ無い（列幅を変える経路ができてから）
- 履歴の「いつ」は UTC（時差を知るアダプタが無い）、「誰が」は ID の先頭 4 桁（名前はジャーナルに無い）。過去の時点の状態の再現（FR-006 後半）はまだ
