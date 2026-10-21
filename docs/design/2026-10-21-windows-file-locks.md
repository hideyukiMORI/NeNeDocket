# Windows のファイルロック: 共有モード・上書き rename・ウイルス対策への備え（Issue #79・2026-10-21）

Windows は開いているファイルの共有モードで他のプロセスを拒み、他のプロセスに開かれているファイルへの `rename` の上書きも失敗する。Linux の Xvfb では一切起きないので、設計で先に避け、施主の Windows で確かめる（QLT-013）。

## 1. 台帳の中のファイルと、開き方

Rust の `std::fs` は Windows で既定の共有モードを `FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE` にする（`std::os::windows::fs::OpenOptionsExt::share_mode` の文書）。つまり何も指定しなければ、他のプロセスは読め・書け・rename と削除もできる。

| ファイル | 誰が・いつ開く | 共有モード（Windows） | 他のプロセスに起こりうること | 備え |
| --- | --- | --- | --- | --- |
| `journal/<自分>.vN.log` | 自分のプロセスが起動から終了まで `append` で | **`READ \| WRITE`（`DELETE` を外した・Issue #79）** | 他の参加者は読める（読み手は `READ \| WRITE \| DELETE` で開く）。同期ツール・バックアップは rename / 削除できず共有違反を受ける（それでよい。開いたまま rename されると、以後の追記が別名のファイルへ消えるため） | `own_journal::open_append` |
| `journal/<他人>.vN.log` | 読み直しのたびに開いて閉じる（スレッド・ADR 0008 / Issue #75） | 既定（`READ \| WRITE \| DELETE`） | 書き手の追記を妨げない。読んでいる最中の rename / 削除も妨げない | ロックを持ち越さない |
| `snapshot/<自分>.v1.txt` / `.idx` | 自分が `checkpoint`（別スレッド）で一時名 → rename | 既定 | 他の参加者は読まない。Explorer のプレビュー・検索インデクサ・ウイルス対策が開いていると rename が失敗する | 再試行（下の 2） |
| `permissions.v1.txt` / `docket.v1.txt` / `keys/*.txt` | 持ち主が権限を変えるとき・初回に、一時名 → rename（Issue #79 で一時名にした。前は直接 `write` で途中の状態が読めた） | 既定 | 読み直し中の他の参加者が開いている（`DELETE` 共有なので rename は通る）。ウイルス対策が一時名のファイルを数百 ms 開く | 再試行 |
| `attachments/<課題>/<連番>-<名前>` | 添付するときに一時名 → rename | 既定 | ウイルス対策が作ったばかりの写しを開く。「開く」で OS の既定のアプリが開いたままでも、再度の写しは別の連番なので衝突しない | 再試行 |
| `account.v2.txt` / 設定 | adapters が起動時に読み、設定は閉じるときに書く | 既定 | 自分の機械のファイルで、台帳の共有には無い | なし |

## 2. 再試行（`store::retry`）

`fs::write` と `fs::rename`（`store::replace::replace_file` を通る全部）は、一時的な失敗なら 50 ms 待って試し直し、5 回で諦めて最後の失敗を返す（FR-014: 失敗は示す）。一時的とみなす失敗:

- `io::ErrorKind::PermissionDenied`（Windows の `ERROR_ACCESS_DENIED`。ウイルス対策が開いている間はこれになる）
- Windows の `ERROR_SHARING_VIOLATION`（32）と `ERROR_LOCK_VIOLATION`（33）

Linux では `EACCES` も `PermissionDenied` だが、5 回（250 ms）で諦めるだけで害は無い。`NotFound` などは 1 回で返す。

## 3. 決めたこと

1. 自分のジャーナルは `FILE_SHARE_DELETE` を**外す**。同期ツール（OneDrive・Dropbox・Google Drive）やバックアップが開いている最中に rename / 削除すると、以後の追記が消えるため。それらは共有違反を受けて後で試し直す（それらの仕事）。
2. 読み手は既定のまま（`READ | WRITE | DELETE`）。書き手も道具も妨げない。
3. ファイルを丸ごと置き換える経路は 1 つ（`replace_file`: 一時名に書いて rename・再試行）。`permissions.v1.txt` / `docket.v1.txt` / `keys/` も通す。
4. 書き込みと rename の失敗は再試行の後に示す。ジャーナルの追記（`write_all` ＋ `sync_data`）は再試行しない（自分のファイルで、他のプロセスは開いているだけ。失敗すれば示す）。

## 4. 施主の Windows で確かめる手順（gate-proofs 第 47 節に結果を書く）

**結果（2026-10-21・Issue #90）:** gate-proofs 第 51.8 節。3 手順とも失敗の表示なし・`.tmp` なし・写しは 1 つ。Defender の検査が写しの途中で走ったかは外から分からない。


1. **2 プロセスで同じフォルダ**: 2 つの `nene-docket.exe` で同じ台帳を開き、片方で追加・題名の変更、もう片方に 3 秒で届く。両方で続けて操作し、どちらも止まらない。
2. **Explorer でフォルダを開いたまま権限を変える**: Explorer で台帳のフォルダ（と `snapshot/`）を開き、プレビューウィンドウを出した状態で、持ち主が参加者の段から権限を切り替える（`permissions.v1.txt` の置き換え）。失敗の表示が出ないこと。
3. **ウイルス対策を有効にして添付する**: Defender を有効のまま、数 MB の PNG を添付の段に落とす。失敗の表示が出ず、`attachments/` に写しが 1 つだけでき `.tmp` が残らないこと。

いずれも Linux では起きないので、ここでは再試行の経路のテスト（偽の失敗を 3 回返して通る・5 回で諦める・恒久の失敗は 1 回）と `make build-windows` の通過だけ。
