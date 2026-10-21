# 用語集 — NeNe Docket

> Status: normative（規範）/ 2026-10-21 初版
> ここに載っている語は、コード・ドキュメント・Issue で**同じ意味**で使う。同義語を発明しない。
> 型／場所の欄は、core に在るものは実名、まだ無いものは置く予定の名前である。

| 語 | 意味 | 型／場所 |
| --- | --- | --- |
| 台帳（docket） | 1 つのフォルダ。課題の集合と、それを作ったジャーナルの全部。ローカルでもネットワーク共有でもよい | `Docket`（application が所有）・フォルダ |
| 画面の状態の所有者 | 検索欄・絞り込み・完了の表示・表の選択とスクロール・判定を 1 つ持ち、1 フレームの入力で次の状態へ置き換える | `DocketScreen`（application）・ADR 0006 |
| 課題（item） | 台帳の 1 行。題名・本文・状態・担当・期限・ラベル・並び順を持つ | `Item`（core） |
| 項目（field） | 課題の 1 つの属性。操作は項目単位で勝敗が付く | `Field`（core・閉じた `enum`） |
| 状態（state） | 課題の進み具合。閉じた集合 | `ItemState`（core） |
| 操作（op） | 台帳を変える最小の単位。誰が・いつ（論理時計と壁時計）・どの課題の・どの項目を・何から何へ。不変 | `Op`（core） |
| 逆の操作 | ある操作を打ち消す操作。Undo はこれを**追記**する | `Op::inverse`（core） |
| ジャーナル（journal） | 1 つのアカウントが書いた操作の列。追記だけ。版を持つ | `journal/<account-id>.v1.log`（store）・ADR 0004 |
| 自分のジャーナル | このプロセスのアカウントが書くジャーナル。書くのはこれだけ | `OwnJournal`（store） |
| 読み直し（refresh） | 他人のジャーナルの伸びた分を読んで取り込むこと | `JournalPort::refresh`（application） |
| 同期（sync） | 自分のジャーナルの追記を記憶装置に確実に書くこと（fsync）。数秒以内 | `JournalPort::sync` |
| 併合（merge） | 全アカウントのジャーナルから、いまの台帳の状態を作る純関数。同じ入力から同じ出力 | `merge`（core）・ADR 0004 |
| 論理時計（clock） | 操作の全順序を決める数。Lamport 時計。壁時計は表示にだけ使う | `LogicalClock`（core） |
| 壁時計（wall clock） | 人に見せる時刻。順序には使わない | `WallClock`（core・`ClockPort` から値として） |
| 勝者（winner） | 同じ項目への操作のうち、全順序で最後のもの。負けた値は履歴に残る | `merge` の規則 |
| 履歴（history） | 課題ごとの操作の列。勝者も敗者も含む | `ItemHistory`（application が併合から作る） |
| Undo の束 | 自分の操作のうち、まだ戻していないものと戻したもの | `UndoStack`（application） |
| アカウント（account） | 台帳に書く人。ID・名前・（権限が有効なら）鍵 | `Account` / `AccountId`（core） |
| 鍵（key） | アカウントの署名鍵。権限を有効にした台帳でだけ使う | 別の ADR |
| グループ（group） | アカウントの集合。権限の単位 | `Group`（core）・FR-010 |
| 権限（permission） | 台帳・項目に対して「書ける」かどうか。閲覧は制限しない | `Permission`（core） |
| 設定（settings） | 最近開いた台帳・列の幅と順・見え。版付きの `key = value` | `SettingsFileV1`（store） |
| 結果（outcome） | 期待される成功・失敗を表す閉じた型 | `Result<T, *Failure>`（core / application） |
| 拒否理由（failure） | 結果に添える閉じた理由の集合 | `*Failure`（`enum`） |
| ポート（port） | 中核がプラットフォームを使うための型のある窓口 | `*Port`（application の `trait`） |
| アダプタ（adapter） | ポートの実装。時計・乱数・クリップボード・画面・IME を中核へ渡す | adapters-x11 / adapters-windows |
| ポートの束 | 合成ルートが中核へ渡すポートの組 | `Ports`（application） |
| 合成ルート | 依存を結ぶ唯一の場所。端末と終了コードを持つ | `crates/app`（`main`） |
| フレームの入力 | 窓の大きさ・ポインタ・押された鍵・確定前の文字列。UI が OS のイベントから作る | `FrameInput`（application） |
| 表示フレーム | 最新の状態から生成する、描くための値の組（描画命令の列と倍率） | `DocketFrame`（application） |
| 部品（widget） | 自前で描く UI の要素。矩形と状態と判定から描画命令を返す純関数 | `button` / `checkbox` / `text_field` / `select_box` / `scroll_bar`（core）・ADR 0006 |
| 配置（layout） | 段を上から積み列に割る。表は見えている行だけを計算する | `Layout` / `TableLayout`（core） |
| 判定（interaction） | フレームをまたぐ状態の持ち主。押している部品とフォーカス。部品ごとに矩形と ID を問い合わせ、ホバー・押下・クリック・フォーカスを返す | `Interaction` / `Response`（core）・ADR 0006 |
| 意図（intent） | 利用者の操作を application に渡す閉じた型。即時モードではポインタと鍵の入力そのもの（ADR 0006） | `PointerInput` / `KeyIntent`（core）・`FrameInput`（application） |
| 編集状態 | 入力欄の文字列・カーソル・選択・確定前の文字列。位置は文字の添字 | `TextEdit`（core）。直前の拒否は `EditSession`（application） |
| 確定前の文字列（preedit） | IME が組み立て中の文字列。確定で操作になる | `Preedit`（core）・`TextEdit::with_preedit` |
| 描画命令 | core が出し UI が描く列。塗り・縁・文字（カーソルは文字の添字）・切り抜き。色は意味（`Paint`）で渡す | `DrawCommand` / `Paint`（core） |
| 反映（render） | 描画命令を画素へ写す操作。UI 状態を変えてよい唯一の場所 | `render*`（ui-x11 / ui-win32） |
| 隔離区画 | 可変性を許した唯一の場所 | ARC-005 の表 |
| 規約検査（conformance） | NeNe Docket 固有の自作ゲート | `xtask`（CNF-NNN） |
| 抑制 | `#[expect(lint, reason = "<規則 ID>: …")]`。理由付きで、発火しなくなれば落ちる | RS-019 |
| 機械強制の状態 | active / planned / 不能 / 不採用 | `docs/QUALITY_GATES.md` の強制マトリクス |
| negative proof | ゲートが意図した規則で落ちることの実測 | `make prove` / `docs/quality/gate-proofs.md` |
| 論理画素 | 物理画素 ÷（dpi / 96）。文字の大きさはこれで定める | `DevicePixelRatio`（core）・nene-ruler ADR 0027 / 0028 の継承 |
| 環境 | UI の倍率と論理画素の比。合成ルートが起動時に渡し、`DpiChanged` で更新する | `Environment`（application） |

## 使ってはいけない語

`Manager` / `Helper` / `Util` / `Utils` / `Common` を型名の語尾に使わない（RS-013）。
役割を語る名前が思いつかないときは、その型が 2 つの責務を持っている可能性が高い。
「保存（save）」を操作の追記の意味で使わない。上書きの含みがある。**追記**と**同期**を使う。
