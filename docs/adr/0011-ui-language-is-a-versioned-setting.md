# ADR 0011 — 表示言語は 6 言語の閉じた集合で、文言は core の 1 表から出し、希望は版付きの設定に残して OS の観測値で解決する

- 状態: 受理
- 日付: 2026-10-21
- Issue: #27
- 影響する規則: ARC-001 / ARC-004 / ARC-007 / ARC-009 / ARC-011 / RS-002 / CNF-012 / QLT-013。FR-019 を足す

## 文脈

施主の依頼（2026-10-21）: 「日米中など多言語化を 6 ヶ国語くらい対応したい」。画面の文言は Issue #11〜#20 で application の中に英語の文字列として散っていた（`"Show done"`・`"Add"`・列の見出し・状態の名前・通知）。

nene-nib の ADR 0123（表示言語は版付き設定）と ADR 0154（文言の表は core に 1 つ）の形を継ぐ。違いは、Nib が 2 言語（日本語・English）で OS の観測を Windows だけに閉じたのに対し、Docket は X11 と Windows の両方で観測する点。

制約: core / application は既定ロケール・環境変数を読まない（ARC-007）。文字列は UI に置かない（ARC-011）。閉じた `enum` の `match` に `_` を書かない（RS-002）。

## 決定

**表示言語は `Language`（日本語・English・简体中文・繁體中文・한국어・Español）の閉じた集合。画面の語は `Word`（閉じた集合）で名指しし、`wording::text(language, word)` の 1 表だけが文字列を持つ。希望は `LanguageChoice`（`system` か決まった言語）として設定の版 2 に残し、表示する言語は希望と起動時に観測した OS の言語から純関数で解決する。**

1. **言語の集合**（決め打ちしない）: `ja` `en` `zh-hans` `zh-hant` `ko` `es`。足すときは `Language` の変種と `wording` の関数を 1 つ足す（網羅性でコンパイルが落ちる）。選択肢には各言語の自分の名前（`native_name`）を出し、どの言語で見ていても読める
2. **文言の表は core の `wording`**。言語ごとに 1 関数（`Word` の `match`）。application は `Word` で語を名指しし、UI は文字列を書かない。通知の数（宛先の無い操作・読めなかった行）は `<語>: <数>` の形で言語に依らない
3. **希望と表示の分離**: 設定に残すのは `LanguageChoice`（`language = system|<tag>`）。`system` は OS の言語（分からなければ English）。起動時に `Environment::with_language` で観測値を渡し、`LanguageChoice::resolve` が決める。起動後の OS の変更には追従しない（決め打ちしない）
4. **観測はアダプタの 1 か所**: X11 は `LC_ALL` → `LC_MESSAGES` → `LANG`、Windows は `GetUserDefaultLocaleName`。どちらも core の `Language::from_locale`（`ja_JP.UTF-8`・`zh-Hant-TW`・`zh_CN`・`C` …）で言語にする。`zh` は `hant` か台湾・香港・マカオなら繁体字、ほかは簡体字
5. **切り替え**: 道具の段のセレクトボックス（「OS に従う」＋ 6 言語）。選べばその場で全部の語が変わり、設定に書く（`SettingsPort`）。書けなければ通知（FR-014）
6. **字形**: X11 は言語の確かめる字（`Language::probe`: 日・简・繁・한・A・ñ）を描ける家族を言語ごとの候補から選び直す（ADR 0007 の拡張。言語が変わったら選び直す）。Windows は DirectWrite の地域名（`ja-jp` `zh-cn` `zh-tw` `ko-kr` `en-us` `es-es`）を渡して fallback の字形を選ばせる
7. **設定の版 2**: `settings.v2.txt` に `language` と `theme`（ADR 0012）を足す。版 1 しか無ければ読んで `system` で補い、書くのは版 2 だけ。版 1 は消さない（ARC-009）

## 強制

- active: `Word` と `Language` は閉じた `enum`。表に抜けがあればコンパイルが落ちる。全言語 × 全語が空でないテスト（`wording`）
- active: core は環境変数を読めない（`#![no_std]`・CNF-012）。観測は adapters の `locale` だけ
- active: 設定の版 2 の往復と、版 1 の読み込み・版 2 への書き出しのテスト（application・adapters）
- planned: 画面の文字列を字句で拒む検査（CNF）。いまは `"` の文字列が application の描画に残っていないことをレビューで見る

## 結果

- 得たもの: 6 言語で同じ画面。文言は 1 か所。Xvfb で日本語・简体中文・한국어・Español の表と履歴が描けた（gate-proofs 第 21 節）。コンテナの文泉驛正黑が中韓の字形を持っていた
- 失うもの: 翻訳は施主の目を通していない（機械翻訳でなく自分で書いたが、母語話者の確認はまだ）。日付は `YYYY-MM-DD`・時刻は UTC のまま（言語に依らない）
- 残る穴: アカウントの名前・ラベル・題名は利用者の入力なので言語に依らない。設定の `theme` の名前も英語の固有名。Windows の `WM_CHAR` は確定した字だけで、確定前の文字列は出ない（ADR 0009 のまま）

## 却下した選択肢

| 選択肢 | 却下の理由 |
| --- | --- |
| gettext / fluent などの外部の仕組み | 語は 31 個で、表 1 つで足りる。外部クレートを増やさない（ARC-013） |
| 言語ごとのファイルを起動時に読む | 読めないファイルの扱いが増え、FR-014 の経路が増える。言語の集合は閉じているので表で持つ |
| OS の言語を描画のたびに読む | 純粋性が壊れ（ARC-007）、切り替えの時機が曖昧になる。起動時に 1 度観測し、設定で上書きする |
| `system` の既定を日本語にする | 施主は日本語だが、`system` は観測値に従うのが名前の通り。分からなければ English |
