# 課題管理表の調査 — 日本・欧米・アジア（Issue #106・2026-10-22）

> 2026-10-21〜22 に web で調べた 3 本をそのまま残す（調査役のエージェントが書いた・出典は各主張に付く）。方針は [カタログ先行の課題管理表](2026-10-22-issue-list-catalog-first.md)、決定は [ADR 0021](../adr/0021-issue-list-catalog-first.md)。
> 🔴 調査は二次資料を含む。PMBOK・PRINCE2 の列構成は本体で確かめていない。画像の参考 URL は取得した PNG を目で読んだ。

## 1. 日本 — 日本の課題管理表：列構成・使われ方・Excelの限界（調査メモ）

### 0. 施主の参考URL 3つの読み取り結果
WebFetch では画像の中身が取れなかったため、取得した PNG をこちらで直接見て読み取った。

- **kaizen-penguin の画像**（https://kaizen-penguin.com/wp-content/uploads/2021/07/issue-list-image-1-1024x539.png）
  - 表の上に「総務部 業務改善プロジェクト 課題管理表」という題と日付（2021/07/01）がある。
  - 列は **no. / 起票日 / 課題 / 課題の詳細 / ステータス / 解決策 / 備考**。
  - ステータスの値は「未対応・対応中・継続対応」。
  - 「課題の詳細」と「解決策」は複数行の長い文章で、表の幅の大半を占める。空行（6〜20）があらかじめ用意されている。
  - 推測：題・日付・枠線付きの空行は、印刷や PDF 配布を前提にした帳票の作り方。
- **wondershare の画像**（https://images.wondershare.jp/edraw/articles/edrawmind/2026/issue-tracking-list-template1.png、Excel の「基本課題管理表.xlsx」）
  - 列は **No. / 課題名 / 担当者 / 期限 / ステータス / 対応策 / 備考**。
  - ステータスは「完了・対応中・未着手」。完了した行は灰色の背景で、行を消さずに残している。
- **wondershare の記事**（https://edraw.wondershare.jp/task-management/issue-tracking-list-templates.html）は 8 つの型を挙げている。
  - ①基本（課題名・担当者・期限・ステータス）
  - ②PM 強化（課題ID・発生日・課題分類・優先度・対応方針・担当部署・進捗率・完了日）
  - ③ガント連動
  - ④リスク統合（影響度・発生確率）
  - ⑤議事録連動（決定事項）
  - ⑥KPI 連動
  - ⑦カンバン
  - ⑧原因分析（発生原因・真因・再発防止策）
  - 最低限の項目は「課題内容・担当者・期限・優先度・ステータス」とされている。

### 1. 典型的な列構成と、どれくらい共通か

| 列 | 共通度 | 出典・備考 |
|---|---|---|
| No.（管理番号） | 必須級 | 両画像、[CREX](https://crexgroup.com/ja/development/project/issue-tracking-sheet-template/)、[@SOHO](https://atsoho.com/blog/issue-tracking-sheet-templates) |
| 件名（課題名） | 必須級 | 全出典 |
| 内容・詳細 | 必須級 | kaizen、CREX、@SOHO |
| 担当者 | 必須級 | wondershare、CREX、@SOHO |
| 期限（完了希望日・回答期限） | 必須級 | wondershare、CREX（完了希望日）、@SOHO |
| ステータス | 必須級 | 全出典 |
| 対応策・解決策・対応内容 | 必須級 | 両画像、@SOHO |
| 起票日・発生日・登録日 | よくある | kaizen、CREX、@SOHO |
| 備考 | よくある | 両画像、CREX |
| 優先度（高・中・低） | よくある | CREX、@SOHO（追加項目）、wondershare 記事 |
| 分類・カテゴリ | よくある | CREX、@SOHO（追加項目） |
| 完了日 | よくある | CREX、@SOHO（追加項目） |
| 起票者 | よくある | @SOHO（必須項目） |
| 影響範囲・影響度・発生確率 | 一部 | @SOHO、wondershare ④ |
| 担当部署 | 一部 | wondershare ② |
| 進捗率（%） | 一部 | wondershare ② |
| 確認者・確認日 | 一部 | 列を明記した出典は見つからなかった。CREX は「確認中」の状態を、@SOHO は「依頼者が確認してから完了」とする運用を書いている。推測：SIer 系の様式では列として置かれることが多い |
| 原因・真因・再発防止策 | 一部（障害管理表） | wondershare ⑧、[gihyo（問題管理）](https://gihyo.jp/dev/serial/01/teamdev-toolstrategy/0012) |
| 再現手順・添付 | 一部（障害管理表） | [ONES](https://ones.com/ja/blog/?p=191074) |
| 質問者・回答者・回答内容（QA 管理表） | 一部（QA 管理表） | 日本語のテンプレートを出典として確認できなかった。推測：No・質問・質問者・質問日・回答者・回答・回答期限・回答日・確認・状態、という形が一般的 |
| 関連資料 | 一部 | @SOHO |

- 補足：Redmine では「1 チケット = 管理表の 1 行」とみなし、トラッカー（課題・障害・QA など）ごとに使うカスタムフィールドとステータスの遷移を決められる（[redmine.jp 入門 PDF](https://redmine.jp/get-start/hajimeru-redmine-2021.pdf)）。課題・障害・QA という「種別ごとに列のセットを変える」構造は、ツール側でもこの形で実現されている。

### 2. 使われ方

- **定例会議での読み合わせ**
  - 週 1 回、「完了以外」に絞った一覧を全員で確認する（[@SOHO](https://atsoho.com/blog/issue-tracking-sheet-templates)）。
  - ステータス会議は課題管理表を中心に進める（[ONES](https://ones.com/ja/blog/?p=17255)）。
- **ステータスの段階と承認の流れ**
  - 段階は「未着手 → 対応中 → 確認中（レビュー中） → 完了」に「保留」を加えた 5 つ。
  - 「確認中」で起票者や関係者が対応結果を確かめ、完了を起票者とマネージャーのどちらが判断するかをルールで決める。
  - 「完了日」が対応を終えた日なのか確認が取れた日なのかを、チームで統一する必要がある（[CREX](https://crexgroup.com/ja/development/project/issue-tracking-sheet-template/)）。
  - 推測：起票 → 対応 → 確認 → クローズの流れで、「確認者」が承認者を兼ねるのが日本でよく見る形。
- **優先度・重要度・影響度**
  - 「高・中・低」の 3 段階で付け、緊急度 × 重要度の 2 軸で判断する（CREX、[ONES](https://ones.com/blog/ja/risk-issue-management-differences)）。
  - リスク型の表では影響度 × 発生確率を使う（wondershare ④）。
- **課題・リスク・QA の使い分け**
  - リスクはまだ起きていない将来の可能性、課題はいま起きている問題。リスクが現実になったら課題管理表へ移す（[ONES](https://ones.com/blog/ja/risk-issue-management-differences)）。
  - QA は「質問と回答」を管理し、回答期限が中心になる。推測：QA の扱いを明記した出典は見つからなかった。
- **色付け**
  - 優先度「高」の行は薄い赤、「中」は薄い黄、期限を過ぎて未完了のセルは赤（CREX）。
  - 期限切れで未完了の行は背景を薄い赤、完了した行は文字を灰色にして消さずに残す（@SOHO、wondershare の画像）。
- **入力・フィルタ・並べ替え**
  - カテゴリ・担当者・優先度・状態はドロップダウンで入力を揃える（CREX）。
  - 見出し行にフィルタを付け、担当者別・状態別に絞り込む（@SOHO）。
- **顧客との共有・印刷**
  - 印刷や PDF の手順を書いた出典は見つからなかった。
  - 推測：kaizen の様式（題・部署・日付・空行）は、紙・PDF・Excel で送ることを前提にしている。

### 3. Excel 管理の不満と、ツールへ移るときに失われがちなもの

**Excel の不満**
- 同時に編集できず、どれが最新版か分からなくなる。版の管理が煩雑で、ファイルが壊れることがある。手入力による表記揺れや更新漏れ、作った人への依存も起きる（[CREX](https://crexgroup.com/ja/development/project/issue-tracking-sheet-template/)）。
- ファイルが重くなって開けなくなる。どこを更新したかをメールで知らせ直す手間がかかる（[ヌーラボ資料](https://nulab.com/app/assets/pdf/document/ja/excel_mail.pdf)）。
- 表で回らなくなるサイン（@SOHO）
  - 100 件を超えると、絞り込みや並べ替えを毎回やり直すことになる
  - 議論がチャットやメールに流れ、表への書き写しが負担になる
  - 担当者への負荷の偏りが見えにくい
- メンバーが増えると限界が来る。変更履歴の管理が大変（[gihyo](https://gihyo.jp/dev/serial/01/project_manager_tool/0009)）。

**ツールへ移るときに失われがちなもの**
- 定着の失敗：ツール導入の失敗は「Excel からの移行の挫折」「期日管理の形骸化」「現場の反発」などに分類され、シンプルに始めて段階的に広げることが勧められている（[SalesNow](https://salesnow.jp/db/knowledge/92)）。
- 自由度：Backlog の利用者からは「孫課題が作れない」「1 枚の課題を複数人が順に確認する運用に向かない（担当者を付け替えて回すしかない）」という声がある（[ITreview](https://www.itreview.jp/products/backlog/reviews/85184)）。また「200 件を超えると分類と優先度の機能が足りない」という声もある（[ITreview](https://www.itreview.jp/products/backlog/reviews/93369)）。
  - Redmine は「細かく直すにはコーディングが要る」とされ、表をまとめて書くのが手作業では難しい（[ITreview](https://www.itreview.jp/products/redmine/reviews)、redmine.jp）。
- 一覧性と印刷：ツールでは一覧性が落ちる・印刷できないと直接書いた出典は見つからなかった。推測は次のとおり。
  - Excel にある「長い文章の折り返しを含めて 1 画面（1 枚）で全行を見渡す」ことと、「題・日付付きの帳票として PDF にして顧客へ送る」ことが、チケット型のツールでは弱くなりやすい。
  - その結果、ツールから CSV を書き出して Excel で整形し直す二重運用が起きる。
- Docket への示唆（推測）
  - 「長い文章を折り返して表示する列（対応策・課題の詳細）」「完了行を灰色で残す」「期限切れを赤くする」「確認中・確認者」「印刷用の題と日付」の 5 つは、日本の利用者がツールに移っても手放したがらない要素だと考えられる。

## 2. 欧米 — 欧米の「課題管理表」に当たるもの：調査報告

### 0. 前提（日本側の基準）
- kaizen-penguin の画像は WebFetch では中身を読めず、PNG をローカルに保存して目で確認した。列は「no. / 起票日 / 課題 / 課題の詳細 / ステータス（未対応・対応中・継続対応）/ 解決策 / 備考」の7つ。ヘッダには部署名、プロジェクト名、作成日がある。出典: https://kaizen-penguin.com/wp-content/uploads/2021/07/issue-list-image-1-1024x539.png
- Wondershare の記事にはテンプレートが8種類ある：基本型、強化型（ID・発生日・分類・優先度・対応方針・担当部署・担当者・期限・進捗率・完了日）、ガント連動型、リスク統合型（影響度・発生確率）、議事録連動型（決定事項）、KPI連動型、カンバン型、原因分析型（発生原因・真因・再発防止策）。最低限の列として挙がっているのは「課題内容・担当者・期限・優先度・ステータス」。出典: https://edraw.wondershare.jp/task-management/issue-tracking-list-templates.html
- 2枚目の画像（edraw）は読んでいない。列は上の記事本文から拾った。

### 1. 欧米の対応物と列構成

| 名称 | 何を記録するか | 典型列 |
|---|---|---|
| Issue Log（PMBOK） | 利害関係者の間で議論中や係争中の事項。誰がいつまでに解決するか | 起票者と日付、説明、種別、優先度、担当、目標解決日、状態、最終解決策 |
| Issue Register（PRINCE2） | 正式に扱う課題 | ID、種別（変更要求 / 仕様外 / 問題・懸念）、起票日、起票者、優先度、重大度、状態、影響分析、推奨対応、決定、完了日 |
| RAID log | Risks / Assumptions（または Actions）/ Issues / Dependencies の4つを分けた台帳 | 定まった列はない。安定した ID・担当・日付付きの根拠・リスクから課題への連結が推奨されている |
| Decision log | 決定事項。RAID の D を Decisions にする流儀もある | 決定、決定者、理由 |
| Punch list（建設） | 引き渡し前の手直し項目 | 項目、場所、責任業者、優先度、期限、状態（open / in progress / complete と検収）、写真 |
| Bug tracker | 不具合 | ツールごとに違う（2章） |

- PMBOK の定義と目的: https://www.simplilearn.com/issue-log-concepts-article 。列の一覧は二次資料の要約から取った: https://www.scribd.com/document/435740485/Project-Documents 。PMBOK 本体は確認していないので、列が規定されているとは言い切れない。
- Asana の Issue Log テンプレートは「目標解決日」と「実際の解決日」を分けている: https://asana.com/templates/issue-log.md
- PRINCE2 の課題種別は3つ（資料によっては5つ）。列は https://www.techno-pm.com/pages/how-to-use-the-issue-log-template-for-prince2-and-agile と https://open-exam-prep.com/study-guides/prince2-practitioner/practices-issues/issues-basics から取った。いずれも試験対策サイト。
- RAID log: https://lucid.co/templates/raid-log 、https://playcode.io/blog/raid-log-template 、https://www.craft.do/templates/raid-log
- Punch list: https://www.mastt.com/resources/punch-list-template 、https://www.smartsheet.com/free-punch-list-templates

### 日本の課題管理表との違い（推測を含む）
- **一つの表にまとめるか、分けるか。** 日本の表はリスク・課題・決定事項・アクションを1枚に並べがちで、リスク統合型や議事録連動型がその例。欧米は RAID のように分類ごとに台帳を分け、相互に連結する。（推測。テンプレートの構成から読み取った）
- **原因と対策の列。** 日本は「原因・真因・再発防止策・解決策」を独立した列にする。欧米の Issue Log は「影響分析・推奨対応・決定・最終解決策」で、原因分析は RCA や 8D の別文書に回るのが普通（推測）。PRINCE2 は「決定」を列にする点が特徴。
- **分類列の意味。** 日本の「課題分類」は業務の領域を表す。PRINCE2 の種別は扱い方（変更要求か仕様外か）を表す。
- **優先度の段階。** どちらも3〜5段階。Linear は No priority / Low / Medium / High / Urgent に固定している: https://linear.app/docs/issue-property
- **状態の数。** 日本の例は3つ前後（未対応・対応中・継続対応・完了）。Linear は区分を固定し（Backlog > Todo > In Progress > Done > Canceled、加えて Triage と Duplicate）、区分の中でだけ名前を足せる: https://linear.app/docs/configuring-workflows 。Punch list は「検収」の段がある。

### 2. ツールの項目の足し引き

| ツール | 項目の足し方 | 上限（出典） |
|---|---|---|
| Jira | フィールドは全体で共有。画面とスキームで項目ごとに出し分ける（company-managed）。team-managed はプロジェクト単位で共有できない | 公式は 800 未満を目標、1,200 を目安とする。強制の上限はない |
| Asana | 組織のフィールドライブラリで再利用 | プロジェクトあたり 150。タスクの画面に出るのは60まで（フォーラム情報） |
| GitHub Projects | 単一選択・テキスト・数値・日付・イテレーション | 1プロジェクト 50（既定の項目も含む）。選択肢は 50 |
| Trello | チェック・日付・ドロップダウン・数値・テキスト | ボードあたり 50 |
| monday.com | 列の型が豊富 | 公式の開発者文書では 1,000列・10,000アイテム。コミュニティでは 300〜500 の報告もある |
| Smartsheet | 列の型 | 400列・20,000行・500,000セル（セル数の上限が先に効く） |
| Airtable | 型が豊富。リンクとロールアップ | テーブルあたり 500 フィールド（スタッフのフォーラム回答） |
| Notion | プロパティ | 500 プロパティ・250,000 行 |
| Linear | 固定プロパティとラベル。優先度は変えられない | 一般的なカスタムフィールドは無いという二次情報あり（未確認） |

出典:
- Jira: https://confluence.atlassian.com/enterprise/managing-number-of-custom-fields-in-jira-data-center-1488597907.html 、https://support.atlassian.com/jira-service-desk-cloud/docs/project-configuration-in-classic-and-next-gen-projects
- Asana: https://forum.asana.com/t/list-of-technical-and-data-limitations-in-asana/236641
- GitHub Projects: https://docs.github.com/zh/issues/tracking-your-work-with-issues/using-issues/managing-issue-fields-in-your-organization
- Trello: https://jira.atlassian.com/browse/TRELLO-151
- monday.com: https://developer.monday.com/apps/docs/limits
- Smartsheet: https://developers.smartsheet.com/api/smartsheet/guides/basics/limitations
- Airtable: https://community.airtable.com/base-design-9/maximum-fields-in-airtable-31111
- Notion: https://www.notion.com/en-gb/help/optimize-database-load-times-and-performance
- Linear: https://www.resumelens.org/blog/linear/linear-vs-jira

ビュー（表・ボード・タイムライン）は、ほとんどのツールが「同じデータを違う形で見せる」作りになっている（推測。各製品について一般に知られていることから）。

### 項目が増えすぎる問題（Jira の custom field bloat 等）と対策
- **Jira。** カスタムフィールドの多さは性能劣化の第一の原因で、表示・検索・作成・編集が遅くなる。再索引の時間も延びる。対策として公式が勧めているのは、追加要求の審査（作業に使うのか、報告にしか使わないのか）、四半期ごとの使用状況の棚卸し、既存の項目の再利用、コンテキストで適用範囲を絞ること、画面に載っていない項目の削除。出典: https://confluence.atlassian.com/enterprise/managing-custom-fields-in-jira-effectively-945523781.html
- 「8割以上のプロジェクトで使う項目だけを足す」という経験則もある: https://www.praecipio.com/resources/blog/three-things-no-one-tells-you-about-custom-fields-in-jira
- **Notion。** 表示するプロパティが多いほど読み込みが遅くなる。関連（リレーション）とロールアップが重い。古い行はアーカイブに分ける: https://www.notion.com/en-gb/help/optimize-database-load-times-and-performance 。「5,000行・30項目で重くなる」という数字は二次情報で、裏付けはない: https://developers.fabric.so/blog/why-notion-is-slow
- **対策の型。** 上限を固定する（GitHub と Trello の 50）、ライブラリやスキームで項目を共有して重複を防ぐ、テンプレートに既定値を持たせる。

### 3. Linear 路線と Airtable 路線の得失

| | オピニオン型（Linear） | 何でも足せる型（Airtable・Notion・Jira） |
|---|---|---|
| 速さ | 項目の形が全件同じなので速くしやすい（推測） | 項目数・関連・数式に比例して遅くなる（Jira・Notion が公式に認めている） |
| 学習と一貫性 | チームが増えても形が崩れない。「柔軟さは規模が大きくなると混乱を生む」という主張 | 部署ごとに独自の表ができ、統合と集計が難しくなる |
| 適合範囲 | 規制のある変更管理や PMO の統制、異なる業務の混在には窮屈 | 建設の手直しリスト、PRINCE2 の台帳、日本型の原因分析列まで表現できる |
| 足りないときの逃げ道 | ラベル、状態の追加 | 項目を足す（増えすぎる問題を招く） |

- Linear 側の主張と得失（二次情報）: https://research2.contrary.com/company/linear 、https://www.resumelens.org/blog/linear/linear-vs-jira 、https://aicoolies.com/reviews/linear-review
- 中間の解として実例があるもの（推測を含む）:
  - 項目の数に低い上限を固定する（GitHub と Trello の 50）
  - 状態は区分を固定し、名前だけを足せるようにする（Linear）
  - 項目は組織のライブラリで共有する（Asana、Jira のスキーム）

  RAID・Punch list・日本型を「既定の項目セット（テンプレート）を出し入れするもの」として用意すれば、自由に足す方式より速さと一貫性を保ちやすいと考える（推測）。

### 注意
PMBOK と PRINCE2 の列構成、Airtable・Asana・Linear のカスタムフィールドの扱いは、二次資料かフォーラムの情報で、本体や公式ページでは確認していない。各社の上限値は改定されやすい。

## 3. アジア（日本以外） — アジア（日本以外）の課題管理表：列構成・使われ方・承認文化

### 0. 調査の前提と限界
- 中国語と韓国語の検索では、英語の汎用テンプレート記事が上位を占めた。現地の Excel テンプレートで列名まで読めたものは少ない。
- 韓国の yesform は文字化け（EUC-KR）のため、列名を読めなかった。bizforms は説明文だけを読めた。
- CSDN（HTTP 521）と禅道の手册（リダイレクト先が無効）は取得できなかった。
- 台湾・東南アジア・インドに固有の様式は見つからなかった。ここは汎用様式と RAID log から推した部分が多い。
- 今回の調査では画像 URL を割り当てられていない。そのため画像は見ていない。

### 1. 地域別の典型的な列と使われ方

### 中国
- **Excel の「问题清单 / 问题跟踪表」**
  - 列の例1：序号・问题描述・责任人・解决状态・解决方案・截止日期（[百度文库](https://wenku.baidu.com/view/23fd3f75f9b069dc5022aaea998fcc22bdd14373.html)、検索要約経由）
  - 列の例2：序号・问题描述・责任人・提出日期・预计解决日期・实际解决日期・当前状态・备注（[CSDN](https://blog.csdn.net/joeyon1985/article/details/42678457)、検索要約経由。本体は取得失敗）
  - **予定日と実績日を並べる**点は日本の課題管理表とほぼ同じ形である。
- **問題日志の型分け**（[SegmentFault](https://segmentfault.com/a/1190000047051093)）
  - 基础问题日志：问题编号・描述・报告人・发现日期・优先级・状态。大規模なら「影响范围」「预计解决时间」を足す。
  - 深度分析型：根本原因・方案・实施计划・结果
  - 风险日志、跨团队型（涉及团队・分工・协作计划）、客户反馈型（严重程度・客户满意度）
  - 同じ記事は、**每日晨会または周会で状態を見直す**ことを勧めている。
- **飞书多维表格**
  - 列ごとに型（単一選択・複数選択・日付・人員・添付）を設定できる。ビューは表・看板・甘特・画册の4つ（[飞书](https://www.feishu.cn/content/article/7574713887522639055)）。
  - 需求収集テンプレートの列：描述・分类・提出人・状态・优先级・提出日期・预计/实际完成时间・负责人（[人人都是产品经理](https://www.woshipm.com/pd/5791246.html)）
- **TAPD・禅道（研発向け）**
  - 禅道の Bug は severity（1〜4）・pri・assignedTo・openedBy・resolvedBy を持つ。状態は active → resolved → closed で、操作は确认・解决・关闭・激活（[glama zentao-mcp](https://glama.ai/mcp/servers/@Tytt/zentao-mcp/blob/ce1d7d184d04801339f008f5fb4721df590d1a14/dist/types.d.ts)、[lobehub](https://lobehub.com/mcp/tytt-zentao-mcp)。どちらも第三者資料）。
  - TAPD のワークフローは、状態を遷移させるときに**追加の入力欄（Appendfield）と遷移の権限（AuthorizedUser）**を要求できる（[TAPD API](https://open.tapd.cn/document/api-doc/API文档/api_reference/workflow/get_workflow_all_transitions.html)）。
- **钉钉・企业微信**：課題表そのものより、審批の流れと待办を連携させて「闭环」にする使い方が目立つ（後述）。企业微信は今回裏付けが取れていない。

### 韓国
- **이슈관리대장（Excel・PMO 様式）**
  - 「ID・제목・등록일자・의사결정시기 등」で構成される（[bizforms](https://www.bizforms.co.kr/form/part/generalbusiness/10004771)）。
  - 別の様式は「일련번호・이슈명・등록일자・결정일자」（[yesform](https://excel.yesform.com/xls/100367285.html)、検索要約）。
  - **「意思決定の時期・決定日」が独立した列になっている**のが特徴である。課題を「誰かが決めるべき事項」として扱う色が濃い。
- 公共の情報化事業では、PMO 運営マニュアルの中に「위험/이슈관리」が手順として組み込まれている（[KISAC PMO マニュアル](http://kisac.co.kr/bbs/data/pds/126_1_.pdf)、検索要約）。
- **Notion** は GS建設・暁星・Kakao Style・당근・SOCAR などが全社で導入している（[ditoday](https://ditoday.com/?p=114025)）。
- **Flow・Jandi**：状態の体系（요청/진행/피드백/완료/보류）は検索で確認できなかった。
  - 推測：Flow の業務機能はこの5段の状態を持つと記憶しているが、未確認である。
  - Jira は IT 企業の研究開発で使われ、PMO や非 IT 部門では Excel 台帳と Notion が多いと見られる。これも推測。

### 台湾
- Microsoft の繁体字テンプレート「Project issue tracker」がある（[Microsoft Create zh-tw](https://create.microsoft.com/zh-tw/template/project-issue-tracker-e9a761b1-72f4-421d-950e-5419dbe2cca6)）。
- 汎用の列は11本：問題編號・回報日期・標題・說明・分類・優先順序・負責人・狀態・截止日・解決日期・備註（[spreadsheetdaddy](https://spreadsheetdaddy.com/?p=21689)、検索要約）。
- 推測：台湾に固有の様式は検索で浮かばなかった。製造業では「缺失追蹤表」や 8D 報告（[CSDN 8D](https://blog.csdn.net/qq_37117595/article/details/147167149)）の系統が強いと見られる。

### 東南アジア・インド
- **Zoho Projects**（インド発）の Issue の列：Severity（None/Show Stopper/Critical/Major/Minor）・Classification（Security, Crash/Hang…）・Reproducible・Flag（Internal/External）・Module・Affected/Release Milestone・Due Date（[Zoho Help](https://help.zoho.com/portal/kb/articles/customize-columns)、[Issue Details](https://help.zoho.com/portal/en/kb/projects/issue-tracker/issue-tracker-introduction/articles/issue-details)）
  - **Flag で「顧客に見せる／社内だけ」を分ける**のは、受託開発らしい特徴である。
- オフショア・受託の現場では、課題（Issue）を RAID log（Risk・Assumption・Issue・Dependency）の一部として持ち、週次の状況報告とエスカレーションに流す型が一般的とされる（[pmworld360](https://www.pmworld360.com/?p=38020)、[learnleansigma](https://www.learnleansigma.com/template/raid-log-template/)）。
  - 推測：インドの IT サービス企業は、顧客との統治の場（週次・月次の steering）でこの型を使うと見られる。インドに固有の資料は見つかっていない。

### 2. 承認・上長確認の文化の表れ方
- **中国（审批）**：課題表の中に承認欄を持たず、**別系統の审批を呼んで、結果を状態に書き戻す**形が多い。
  - 飞书：表単体での承認は弱い。そのため「多维表格＋飞书审批」を連携させ、部門責任者の审批結果を状態列へ自動で反映する（[飞书 多维表格连通审批与督办](https://www.feishu.cn/content/7272345292303974428)、[事例](https://www.feishu.cn/content/515976987951)、[woshipm 評測](https://www.woshipm.com/evaluating/6150755.html)）。
  - 钉钉：会签・或签・条件分岐・期限超過で上長へ自動移送（[woshipm](https://www.woshipm.com/pd/6318712.html)）。宜搭のフォームから待办を作れるが、状態の同期には独自の口が要る（[阿里云](https://developer.aliyun.com/ask/491837)）。
  - TAPD の「遷移に権限と必須欄」は、**承認を状態遷移の門として埋め込む**型である。
- **韓国（결재）**
  - 台帳の「意思決定の時期・決定日」列が、결재を待つことを表している（前掲）。
  - 週報の様式は「이슈 및 요청 사항」「향후 계획」を定番の欄に持ち、「上司には結論とリスクを中心に」書くよう勧められる（[wikidocs](https://wikidocs.net/271634)）。
  - 推測：課題台帳は週報や전자결재の添付として上へ上がる。押印欄に当たる결재란が様式の頭に付くことも多いと見られるが、今回の資料では確認できなかった。
- **インド・東南アジア**：承認の主体は社内の上長より**顧客**である。RAID で週次に見直し、塞がった項目はエスカレーションする（前掲）。

### 3. 日本・欧米との比較

| 観点 | 日本 | 中国 | 韓国 | 台湾 | 東南アジア・インド | 欧米（参考） |
|---|---|---|---|---|---|---|
| 主な道具 | Excel 課題管理表 | Excel・飞书/钉钉・TAPD/禅道 | Excel 台帳・Notion・Jira | Excel・汎用テンプレート | Excel RAID・Jira・Zoho | Jira・Excel issue log |
| 特徴的な列 | 起票日・対応期限・完了日・対応内容 | 预计/实际解决日期・解决方案・备注 | **의사결정시기・결정일자** | 回報日期・解決日期 | **Internal/External の Flag**・Severity | Severity/Priority・Reporter/Assignee |
| 状態 | 未着手/対応中/完了 | 待处理/处理中/已解决/已关闭 | 신규/진행/해결/종료（推測を含む） | Open/In progress/Resolved/Closed | 同左＋Show Stopper 等の Severity | 同左 |
| 承認の表れ | 確認者欄・押印（推測） | 别系统の审批 → 状態へ書き戻し | 결재と週報の「이슈 및 요청」 | 推測：日本と中国の中間 | 顧客承認・エスカレーション | 解決者と検証者を分ける程度 |
| 定例での使い方 | 定例会で読み合わせる（推測） | 晨会・周会で見直す | 週報で上へ上げる | 推測：周會 | 週次の状況報告・steering | weekly status・RAID review |

**共通点**
- どの地域も ID・内容・担当・期限・状態・解決日が核になっている。
- 予定日と実績日を並べ、遅れを強調表示する使い方も共通する（[thebricks](https://www.website.thebricks.com/resources/how-to-create-an-issue-tracker-in-excel)）。

**違い**
- 承認を入れる場所が地域で違う。
  - 中国：外部の审批フローに置く。
  - 韓国：台帳の「決定日」と週報に置く。
  - インド：顧客向けの Flag とエスカレーションに置く。
- 推測：設定で足し引きするなら、次の3つを任意の列や遷移規則として持てれば、これらの地域を広くカバーできる。
  1. 決定・承認の日付と承認者
  2. 公開範囲（Internal/External）
  3. 状態遷移ごとの必須欄と権限
