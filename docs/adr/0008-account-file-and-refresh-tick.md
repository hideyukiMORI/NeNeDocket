# ADR 0008 — アカウントは設定フォルダの乱数の ID。他人のジャーナルは X の合図で読み直す

- 状態: 受理
- 日付: 2026-10-21
- Issue: #11
- 影響する規則: ARC-004 / ARC-006 / ARC-007 / ARC-009 / RS-016 / CNF-012 / CNF-013

## 文脈

Issue #11 で画面をジャーナル（ADR 0004）に結んだ。決めずに進められなかったことが 2 つある。

1. **このプロセスのアカウント（FR-009）をどう作り、どこに置くか。** ID は衝突しない乱数で、再起動をまたいで同じでなければならない（自分のジャーナルに書き続けるため・Undo の束を組み直すため）
2. **他人のジャーナルを 3 秒ごとに読み直す（ADR 0004 の 8）時間を、どこから入れるか。** 画面のイベントループは `wait_for_event` で X のイベントを待つだけで、時間を知らない。`ui-x11` は時刻・スレッドに到達できない（ARC-007 / CNF-012）

実測: x11rb の `RustConnection` は `wait_for_event` に時限が無く、`poll_for_event` を回すのは CPU を焼くか遅れる。ファイル記述子を `poll(2)` するには `libc` と `unsafe` が要る（ADR 0002 で Linux には許していない）。

## 決定

**アカウントは `$XDG_CONFIG_HOME/nene-docket/account.v1.txt`（無ければ `$HOME/.config/nene-docket/`）に置く。無ければ `/dev/urandom` の 16 バイトを ID、`USER` を名前にして作る。形は application の `account_file`（`version = 1` / `id = <16 進 32 桁>` / `name = <名前>`）で、版が違えば黙って作り直さず起動を止める。**

**読み直しの刻みは `adapters-x11` のスレッドが、自分の X 接続から窓へ `_NENE_DOCKET_REFRESH` 型の `ClientMessage` を 3 秒ごと（決め打ちしない）に送る。`ui-x11` はその atom を見て次の `FrameInput` に読み直しを乗せ、`DocketScreen::step` が取り込む。**

加えて、**自分の操作を作る直前にも他人の伸びた分を読む**（ADR 0004 の 8 の後半）。自分の操作は見たすべての操作より後に並ぶ。

## 強制

- アカウントのファイルの形は application の純関数（`parse` / `encode`）で、往復と拒否をテストが守る（active）
- `std::fs` / `std::env` / `std::thread` / `/dev/urandom` に触れるのは `adapters-x11` だけで、CNF-012 が他の製品クレートで拒む（active・gate-proofs 第 14 節で `crates/app` の `std::time` が実際に落ちた）
- `adapters-x11` が依存してよいのは application・core と x11rb だけ。CNF-013（active）と deny.toml
- 3 秒の間隔は合成ルートの定数 1 つ（`REFRESH_INTERVAL_SECONDS`）。設定ファイル（FR-012）に移すのは別の Issue（planned）

## 結果

- 得たもの: `ui-x11` は時間を知らないまま、他人の変更が 3 秒以内に表に出る。Xvfb で 2 プロセスが同じフォルダを開き、片方の追加・状態・題名・削除がもう片方に届いた（gate-proofs 第 14 節）
- 得たもの: 再起動後も同じアカウントで自分のジャーナルに続きを書き、Undo の束がジャーナルから組み直される
- 失うもの: X の接続が 2 本になる（刻みのスレッドの分）。窓が消えれば送れなくなってスレッドは終わる
- 残る穴: 同じアカウントのファイルを 2 台のマシンが持つ（設定フォルダを共有する）ことは守らない（ADR 0004 の 2 と同じ）。名前は `USER` のままで、画面から変える経路はまだ無い
- 残る穴: Windows では刻みの経路が別になる（`SetTimer` か `WM_USER`）。adapters-windows ができるときに同じポートの向こうで決める

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| `poll_for_event` を短い `sleep` で回す | `ui-x11` に `std::thread::sleep` が要り（CNF-012）、遅れと CPU のどちらかを払う |
| x11rb の接続の fd を `poll(2)` で待つ | `libc` の `unsafe` が要る。Linux には `unsafe` を許していない（ADR 0002） |
| `ui-x11` がスレッドを持つ | 画面が時間とスレッドを知ることになり、ARC-007 の 3 区画の外に非決定性が入る |
| アカウントのファイルを台帳のフォルダに置く | 台帳は共有フォルダで、アカウントはマシンのもの。別の台帳を開いても同じ ID でなければならない |
| アカウントの形を store に置く | store は application と core にしか依存できず、adapters は store に依存できない（PROJECT_LAYOUT 第 2 節）。純関数は application に、I/O は adapters に |
| `uuid` クレート | 外部クレートを増やす理由が無い。16 バイトの乱数は `/dev/urandom` で足りる（ARC-013） |
