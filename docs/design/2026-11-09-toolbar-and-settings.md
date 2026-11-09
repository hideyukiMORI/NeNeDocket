# 道具の行・設定・追加・0 件・F1（運用の筋書きの B の後半・2026-11-09）

> 状態: 採用（施主 2026-11-09「1，よい。2，了解。」）。画は /design のキャンバス https://claude.ai/artifact/7xwidbC6Yqgbxg4spi6mF8（施主だけが開ける・5 枚: 道具の行とペイン・設定のペイン・課題を足している途中・0 件の案内 3 種・F1 の案内）。
> 施主 2026-11-09「後デザインが絡むところは /design とデザイン系の専門家ペルソナ複数に相談して。web 検索もして資料も使って」。4 人のペルソナ（日本の業務アプリの UX・課題管理ツールのプロダクトデザイナー・Windows / Fluent のインタラクション・アクセシビリティと情報設計）が web で資料を集めて案を出し、リナが議長としてまとめた。
> 前提: [運用の筋書き](2026-11-09-operating-scenario.md)・[採用した見た目](2026-10-22-look.md)（ADR 0022）・[カタログ先行](2026-10-22-issue-list-catalog-first.md)（ADR 0021）。B の前半（詳細のペイン）は Issue #122。

## 1. 道具の行

```text
[⌕ 題名・本文で検索  Ctrl+F] 絞り込み (✓未完了 37)(期限超過 5)(未割り当て 3)(確認待ち 2)(すべて 124)  ……  37 件を表示（全 124 件） [? ヘルプ] [⚙ 設定] [＋ 課題を追加 Ctrl+N]
```

- **絞り込みは札 1 列・1 つだけ選ぶ**（ラジオの組）。既定は「未完了」。状態の選択箱と「完了も表示」のチェックは札に吸収する（同じことを 3 通りでできる状態をなくす・ARC-001 の精神）。「すべて」が閉じた課題（完了・中止）も出す
  - 未完了: 閉じていない（完了・中止でない）
  - 期限超過: 閉じていなくて期限が今日より前
  - 未割り当て: 閉じていなくて担当が空
  - 確認待ち: 状態が確認中
- **件数は札ごとに、検索だけを当てた数**（札どうしは影響しない）。0 件でも出す（「期限超過 0」は安心材料）
- **選んだ札は色だけに頼らない**: ✓・太字・太い縁
- 検索の欄の前に ⌕、案内の字「題名・本文で検索」と `Ctrl+F`。案内の字だけでは欄の名にしない（⌕ と置き場で示す）
- 「37 件を表示（全 124 件）」を右に。絞り込み中であることを隠さない
- 言語・配色・参加者は道具の行から外す（設定へ）
- 状態ごとの細かい絞り込みは段階 2 の見出しの絞り込み。「担当 ▾」は置かず、C の「自分の担当」タブへ（施主 2026-11-09）

## 2. 課題を足す

- 「＋課題を追加」（Ctrl+N）は右端に固定。押すと右のペインが「新しい課題」の下書きになり、題名の欄にフォーカス
- **Enter で初めてジャーナルに追記する**（作る ＋ 題名）。題名が空なら何もしない。続けて次の題名を打てる。Esc で下書きを捨てる（何も残らない・ADR 0004 の追記専用で空の課題を作らない）
- **検索の欄の字は黙って使わない**。検索で 0 件のときだけ「『◯◯』を題名にして追加」を出す
- 足した課題がいまの絞り込みに合わなくても、別の行を選ぶか札を替えるまで表に残し、「森-13 を追加しました。今の絞り込みでは表示されません［絞り込みを外す］」を出す

## 3. 設定

- 「⚙ 設定」で右のペインが設定になる（窓やダイアログは出さない・表を見ながら選べる）。Esc か「閉じる」で戻る。C でタイトルバーに歯車が来ても開く先は同じ
- **自分だけ（この PC）**: 表示言語（各言語の自分の名・「適用」で切り替える＝WCAG 3.2.2）・配色（選ぶとすぐ効く・見比べる）・表の列の出し入れ（No と題名は常に）
- **この台帳（全員に効く）**: 参加者と権限（持ち主だけが変えられる）。一段深く置いて誤操作を防ぐ

## 4. 0 件のとき

| 場合 | 文言 | 次の一手 |
| --- | --- | --- |
| 台帳が空 | まだ課題がありません。気づいたことを、題名だけで足せます | ＋課題を追加（取り込みは E が入るまで出さない） |
| 絞り込みで 0 件 | 期限超過の課題はありません（全 124 件） | 絞り込みを外す |
| 検索で 0 件 | 「◯◯」を含む課題はありません | 検索を消す・「◯◯」を題名にして追加 |

## 5. F1

運用の筋書きの 6 段の表を重ねて出し、各段から当たる札やボタンへ飛べる。下に鍵（Ctrl+N・Ctrl+F・Ctrl+Z / Y・Tab・← →・Esc）。Esc で閉じ、フォーカスは開く前の場所へ。

## 6. 詳細のペインの手直し（B の前半への指摘）

- 「削除」は右上の「…」の中へ（普段は「中止」で閉じる）
- 題名に項目名。番号の横に状態、下に「起票 10-01 森 · 更新 11-09 田中」
- 状態の札は 1 段目に 未着手→対応中→確認中→完了、2 段目に保留・中止（控えめ）。選んだ札に ✓
- 本文・添付・履歴は状態の札と形を変えて**下線のタブ**に、件数を添える（添付 2・履歴 14）
- 担当は参加者から選ぶ（自由入力で「alice」と「alicemori」が混ざった）
- 期限の欄に書き方の案内（例 11/30・+3）と曜日
- 境のつまみとスクロールバーを軽く（1px の線と 3×32 のつまみ）

## 7. コントラスト（アクセシビリティの実測）

| 箇所 | 実測 | 基準 | 対応 |
| --- | --- | --- | --- |
| 欄・札・チェックの縁 | 約 1.35:1 | 1.4.11 の 3:1 | 縁の色を 3:1 以上に（配色の札を足す・ADR） |
| 選んでいる完了の行（灰色の字 × 選びの面） | 4.46:1 | 1.4.3 の 4.5:1 | `muted` を濃く、または `selection` を明るく |
| 選んでいる行の面 | 地に対して 1.42:1 | 1.4.1 色だけに頼らない | 左に 3px の強調の帯 |
| フォーカスと選びの印 | 同じ 1px の橙 | 2.4.7 / 2.4.13 | フォーカスは外側 2px の輪で別の形に |

## 8. 4 人が割れた点と議長の裁定

| 争点 | 裁定 | 理由 |
| --- | --- | --- |
| 札は複数選べる（AND）か 1 つだけか | 1 つだけ | Excel から来た人に AND / OR を考えさせない。保存した見方は C のタブ |
| 「担当 ▾」を置くか | 置かない（施主了解） | C の「自分の担当」タブと重なる。道具の行を増やさない |
| 設定はすぐ効くか「適用」か | 配色はすぐ・言語は「適用」 | 配色は見比べたい。言語は全部の字が変わる（3.2.2） |
| 下書きは表の中かペインか | ペインの題名の欄 | 表の中の仮の行は並び・絞り込み・スクロールと絡んで壊れやすい |
| 主ボタンは左か右か | 右端に固定 | 採用案（ADR 0022）と Fluent のコマンドバー。C で ⚙ が移っても動かない |
| 「完了・中止も表示」をチェックで残すか | 札「すべて」に吸収 | 1 つのことは 1 つの方法で |

## 9. 実装の分け方

B2-1 道具の行（札・件数・追加の下書き・Ctrl+N / Ctrl+F）→ B2-2 設定のペインと F1 → B2-3 0 件の案内 → B2-4 ペインの手直し → B2-5 コントラスト（配色の札・ADR）。

## 10. 根拠（ペルソナが集めた資料）

- Linear: 課題の作成 https://linear.app/docs/creating-issues ・フィルタ https://linear.app/docs/filters ・表示オプション https://linear.app/docs/display-options
- GitHub Projects の絞り込み https://docs.github.com/en/issues/planning-and-tracking-with-projects/customizing-views-in-your-project/filtering-projects
- Jira のクイックフィルタは AND（コミュニティ） https://community.atlassian.com/t5/Jira-questions/Can-boards-be-set-to-treat-multiple-selected-quick-filters-as/qaq-p/2834973
- Backlog の検索（「完了以外」が既定） https://support-ja.backlog.com/hc/ja/articles/360034889913
- Excel の完了行の塗り分け https://canon.jp/biz/solution/smb/tips/excel/table/01
- Microsoft コマンドバー https://learn.microsoft.com/en-us/windows/apps/design/controls/command-bar ・アプリの設定 https://learn.microsoft.com/en-us/windows/apps/design/app-settings/guidelines-for-app-settings ・キーボード https://learn.microsoft.com/en-us/windows/apps/develop/input/keyboard-interactions ・間隔 https://learn.microsoft.com/en-us/windows/apps/design/basics/content-basics
- Fluent 2 Toolbar https://fluent2.microsoft.design/components/web/react/toolbar/usage
- NN/g 空の状態 https://www.nngroup.com/videos/empty-states-in-application-design-guidelines/ ・SAP Fiori 空の状態 https://www.sap.com/design-system/fiori-design-web/v1-139/foundations/best-practices/global-patterns/designing-for-empty-states
- WCAG 2.2: 1.4.1 https://www.w3.org/WAI/WCAG22/Understanding/use-of-color ・1.4.3 https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum ・1.4.11 https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast ・2.4.7 https://www.w3.org/WAI/WCAG22/Understanding/focus-visible ・3.2.2 https://www.w3.org/WAI/WCAG22/Understanding/on-input ・3.3.2 https://www.w3.org/WAI/WCAG22/Understanding/labels-or-instructions
- デジタル庁デザインシステム: インプットテキスト https://design.digital.go.jp/dads/components/input-text/usage/ ・チップタグ https://design.digital.go.jp/dads/components/chip-tag/

確かめきれていないこと: Backlog のヘルプ本文は 403 で読めず検索結果の要約での確認。「kintone は一覧の左上に＋」はペルソナの記憶で裏を取れていない。
