# 用語集 — NeNe Docket

> Status: normative（規範）/ 2026-10-21 初版
> ここに載っている語は、コード・ドキュメント・Issue で**同じ意味**で使う。同義語を発明しない。
> 型／場所の欄は、core に在るものは実名、まだ無いものは置く予定の名前である。

| 語 | 意味 | 型／場所 |
| --- | --- | --- |
| 台帳（docket） | 1 つのフォルダ。課題の集合と、それを作ったジャーナルの全部。併合した状態も同じ名で呼ぶ | `Docket`（core・課題の集合と作られた順と宛先の無い操作の数）・フォルダ |
| 画面の状態の所有者 | 検索欄・絞り込み・完了の表示・表の選択とスクロール・判定を 1 つ持ち、1 フレームの入力で次の状態へ置き換える | `DocketScreen`（application）・ADR 0006 |
| 本文の段（body pane） | 選んだ課題の本文を出す複数行の欄。表の下。道具の段の「本文」で出す。フォーカスが外れたら `Set` を追記 | `BodyPane`（application）・`text_area`（core） |
| 課題（item） | 台帳の 1 行。題名・本文・状態・担当・期限・ラベル・並び順と、消されているか。併合の派生値 | `Item` / `ItemId`（作ったアカウントと連番）（core） |
| 項目（field） | 課題の 1 つの属性。操作は項目単位で勝敗が付く | `Field` / `FieldValue`（core・閉じた `enum`） |
| 状態（state） | 課題の進み具合。閉じた集合 | `ItemState`（core） |
| 操作（op） | 台帳を変える最小の単位。ID（論理時計とアカウント）・壁時計・打ち消す操作の ID・種類（作る・置く・消す・戻す）。不変 | `Op` / `OpId` / `OpKind`（core） |
| 逆の操作 | ある操作を打ち消す操作。Undo はこれを**追記**し、`undoes` に元の ID を持つ。Redo は Undo を打ち消す | `Op::undoing`（core） |
| ジャーナル（journal） | 1 つのアカウントが書いた操作の列。追記だけ。版を持つ。1 行 1 操作で末尾に CRC-32。権限を有効にした台帳では版 2（署名付き） | `journal/<account 16 進>.v1.log` / `.v2.log`・`journal_record`（store）・ADR 0004 / 0010 |
| 自分のジャーナル | このプロセスのアカウントが書くジャーナル。書くのはこれだけ。起動時に壊れた末尾を数えて捨てる | `OwnJournal`（store） |
| 読み直し（refresh） | 他人のジャーナルの伸びた分を読んで取り込むこと。読んだ位置をアカウントごとに覚える | `OtherJournals::refresh`（store）・`JournalPort`（application・予定） |
| 同期（sync） | 自分のジャーナルの追記を記憶装置に確実に書くこと（fsync）。数秒以内 | `OwnJournal::sync`（store） |
| 併合（merge） | 全アカウントのジャーナルから、いまの台帳の状態を作る純関数。同じ入力から同じ出力。同じ ID の操作は 1 度だけ数える | `merge::merge` → `Docket`（core）・ADR 0004 |
| 論理時計（clock） | 操作の全順序を決める数。Lamport 時計。壁時計は表示にだけ使う | `LogicalClock`（core） |
| 壁時計（wall clock） | 人に見せる時刻。順序には使わない | `WallClock`（core・`ClockPort` から値として） |
| 勝者（winner） | 同じ項目への操作のうち、全順序で最後のもの。負けた値は履歴に残る | `merge` の規則 |
| 履歴（history） | 課題ごとの操作の列。勝者も敗者も含む | `merge::history`（core） |
| Undo の束 | 自分の操作のうち、まだ戻していないものと戻したもの | `UndoStack`（application） |
| アカウント（account） | 台帳に書く人。ID・名前・署名する鍵 | `AccountId`（core）・`Account`（application）・`account.v2.txt`（adapters） |
| 鍵（key） | アカウントの Ed25519 の鍵対。秘密鍵は設定フォルダ、公開鍵は台帳の `keys/<account>.v1.pub`（名前も） | `SigningKey` / `PublicKey`（core）・`KeyRecord`（application）・`key_file`（store）・ADR 0010 |
| 署名（signature） | 版 2 の行と `permissions.v1.txt` に付く 64 バイト。読み手が公開鍵で検証し、通らない行は捨てて数える | `Signature`（core）・`verifier`（store）・ADR 0010 |
| 持ち主（owner） | 台帳を作ったアカウント。権限を有効にし、編集者とグループを決め、`permissions.v1.txt` に署名する | `docket.v1.txt` の `owner`・`administration`（store） |
| グループ（group） | アカウントの集合。中の全員が書ける | `Permissions::groups`（core）・FR-010 |
| 権限（permission） | 台帳に「書ける」かどうか（持ち主・編集者・グループの中）。閲覧は制限しない | `Permissions`（core）・`permissions.v1.txt`（store）・`nene-docket --permissions on` |
| 設定（settings） | 最近開いた台帳・読み直しの間隔・表示言語の希望・配色の希望（列の幅と順はまだ）。版付きの `key = value`（版 2）。形は application、読み書きは adapters | `Settings` / `settings_file`（application）・`saved_settings`（adapters-x11 / adapters-windows） |
| 表示言語（language） | 画面の語の言語。6 つの閉じた集合。希望（`system` か決まった言語）は設定に、表示する言語は OS の観測値と合わせて解決する | `Language` / `LanguageChoice`（core）・ADR 0011・FR-019 |
| 語（word）と文言の表 | 画面に出す語の名前と、言語ごとの文字列の表。UI は文字列を書かない | `Word` / `wording::text`（core） |
| 配色（theme） | 名前・明暗・台帳の 8 色。組み込み 125 の元（`ThemeSeed`: 本文の 3 色と出典）から整数の補間で導く。希望（`system` か名前）は設定に | `Theme` / `ThemeSeed` / `theme_catalog` / `theme_derivation` / `ThemeChoice`（core）・ADR 0012 |
| 明暗（appearance） | ライトかダークか。OS の観測値と配色の表が持つ | `Appearance`（core） |
| 結果（outcome） | 期待される成功・失敗を表す閉じた型 | `Result<T, *Failure>`（core / application） |
| 拒否理由（failure） | 結果に添える閉じた理由の集合 | `*Failure`（`enum`） |
| ポート（port） | 中核がプラットフォームを使うための型のある窓口 | `*Port`（application の `trait`） |
| アダプタ（adapter） | ポートの実装。時計・乱数・クリップボード・画面・IME を中核へ渡す | adapters-x11 / adapters-windows |
| ポートの束 | 合成ルートが中核へ渡すポートの組 | `Ports`（application） |
| ジャーナルのポート | 自分のジャーナルへの追記・同期と、まだ渡していない操作の読み直し | `JournalPort`（application）・`FileJournal`（store） |
| 台帳の状態 | 見た操作の列・併合した台帳・論理時計・Undo の束の持ち主。自分の操作を作り、他人の操作を取り込む | `Ledger`（application） |
| Undo の束 | 自分の素の操作のうち生きているものと、戻されたもの。ジャーナルの順に `record` すれば組み直せる | `UndoStack`（application）・ADR 0004 の 4 |
| 編集の欄（slot） | 編集行の文字の欄（題名・担当・期限・ラベル）。欄の文字列と課題の項目を行き来する。期限は `YYYY-MM-DD`、ラベルは `,` 区切り | `EditSlot`（application） |
| 履歴の行 | 課題 1 件に触れた操作を全順序で追い、いつ・誰が・何を・何から何へにした 1 行。Undo の追記は `undo:` | `HistoryLine`（application）・`merge::history`（core）・FR-006 |
| 履歴の段 | 選んだ課題の履歴を表の下に出す段。自分のスクロールを持つ | `HistoryPane`（application） |
| 降格の文言（notice） | 書けない・読めない・宛先の無い操作・読めなかった行を、窓の下 1 行で示す | `Notice`（application）・FR-014 |
| アカウント | このプロセスの ID と名前。設定フォルダの `account.v1.txt` | `Account` / `account_file`（application）・`saved_account`（adapters-x11）・ADR 0008 |
| 読み直しの刻み | 他人のジャーナルを読み直す合図を、一定の間隔で窓へ送るスレッド | `RefreshTicker`（adapters-x11）・`atoms::REFRESH_NAME`（ui-x11）・ADR 0008 |
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
| 反映（render） | 描画命令を画素へ写す操作。UI 状態を変えてよい唯一の場所 | `Canvas::render`（ui-x11）・`Pen::paint`（ui-win32） |
| 見えの色（palette） | 色の意味（`Paint`）から 8 ビットの色へ。配色ごとに 1 つで、フレームが運び、X11 と Windows が同じ表を使う | `Palette` / `Rgb`（core） |
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
