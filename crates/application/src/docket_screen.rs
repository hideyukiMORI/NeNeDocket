//! 画面の状態の所有者（ARC-004）。検索欄・絞り込み・完了の表示・表の選択とスクロール・選んだ課題の編集（題名・担当・期限・ラベル・状態・削除）・履歴の段・Undo / Redo。
//!
//! 即時モード（ADR 0006）。`step` が 1 フレームの入力を受け、鍵を前のフレームのフォーカスに配り、
//! 部品を問い合わせ、描画命令を返す。業務の判断（どの行を出すか・何が選ばれたか・どの操作を作るか）は
//! すべてここにあり、UI には無い（ARC-011）。課題の正本は [`Ledger`] が持つジャーナルの併合で、
//! 自分の操作はポート（[`Ports`]）へ追記して同期する（ADR 0004）。

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::button;
use nene_docket_core::checkbox;
use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::field::Field;
use nene_docket_core::field_value::FieldValue;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::item::Item;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::item_state::ItemState;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::language::Language;
use nene_docket_core::language_choice::LanguageChoice;
use nene_docket_core::layout::Layout;
use nene_docket_core::op::Op;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::paint::Paint;
use nene_docket_core::pixel_point::PixelPoint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::response::Response;
use nene_docket_core::response_phase::ResponsePhase;
use nene_docket_core::scroll_bar;
use nene_docket_core::scroll_state::ScrollState;
use nene_docket_core::select_box;
use nene_docket_core::select_state::SelectState;
use nene_docket_core::table_layout::TableLayout;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::text_edit::TextEdit;
use nene_docket_core::text_field;
use nene_docket_core::theme::Theme;
use nene_docket_core::theme_choice::ThemeChoice;
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::account::Account;
use crate::admin_command::AdminCommand;
use crate::body_pane::BodyPane;
use crate::docket_frame::DocketFrame;
use crate::edit_slot::EditSlot;
use crate::environment::Environment;
use crate::frame_input::FrameInput;
use crate::history_change::HistoryChange;
use crate::history_line::HistoryLine;
use crate::history_pane::HistoryPane;
use crate::journal_failure::JournalFailure;
use crate::ledger::Ledger;
use crate::member::Member;
use crate::members_action::MembersAction;
use crate::members_pane::MembersPane;
use crate::notice::Notice;
use crate::ports::Ports;
use crate::settings::Settings;
use crate::settings_failure::SettingsFailure;
use crate::table_column::TableColumn;

/// 部品の ID。候補は `*_OPTION` に番号を足し、表の行は `ROW` に見えている順の番号を足す。
const SEARCH: WidgetId = WidgetId::new(1);
const FILTER: WidgetId = WidgetId::new(2);
const SHOW_DONE: WidgetId = WidgetId::new(3);
const ADD: WidgetId = WidgetId::new(4);
const SCROLL: WidgetId = WidgetId::new(5);
const TITLE: WidgetId = WidgetId::new(6);
const STATE: WidgetId = WidgetId::new(7);
const DELETE: WidgetId = WidgetId::new(8);
const ASSIGNEE: WidgetId = WidgetId::new(9);
const DUE: WidgetId = WidgetId::new(10);
/// 11 は履歴の段のスクロールバー（`history_pane`）。
const HISTORY_TOGGLE: WidgetId = WidgetId::new(12);
const LABELS: WidgetId = WidgetId::new(13);
const LANGUAGE: WidgetId = WidgetId::new(14);
const THEME: WidgetId = WidgetId::new(15);
/// 16 は本文の段の欄（`body_pane`）。
const BODY_TOGGLE: WidgetId = WidgetId::new(17);
const MEMBERS_TOGGLE: WidgetId = WidgetId::new(18);
/// 19 は参加者の段の「権限」のチェック、21 はそのスクロールバー（`members_pane`）。20 は本文のスクロールバー（`body_pane`）。
const FILTER_OPTION: WidgetId = WidgetId::new(100);
const STATE_OPTION: WidgetId = WidgetId::new(200);
const LANGUAGE_OPTION: WidgetId = WidgetId::new(300);
const ROW: WidgetId = WidgetId::new(1_000);
/// 表のヘッダの列の境（ドラッグで幅を変える・FR-012）。左からの番号を足す。
const COLUMN_GRIP: WidgetId = WidgetId::new(5_000);

/// 1x の寸法（SPECIFICATION 第 2 節・決め打ちしない）。
const MARGIN_PX: u16 = 8;
const ROW_PX: u16 = 28;
const FILTER_WIDTH_PX: u16 = 140;
const SHOW_DONE_WIDTH_PX: u16 = 120;
const HISTORY_WIDTH_PX: u16 = 100;
const BODY_WIDTH_PX: u16 = 80;
const MEMBERS_WIDTH_PX: u16 = 100;
/// 本文の段の 1 行。
const BODY_LINE_PX: u16 = 20;
const LANGUAGE_WIDTH_PX: u16 = 110;
const THEME_WIDTH_PX: u16 = 130;
const BUTTON_WIDTH_PX: u16 = 80;
const SCROLL_WIDTH_PX: u16 = 12;
/// 表のヘッダの列の境の、つまめる幅。
const GRIP_PX: u16 = 6;
/// 編集行の状態の箱。
const STATE_WIDTH_PX: u16 = 110;
/// 編集行の担当と期限の欄。
const EDIT_FIELD_WIDTH_PX: u16 = 100;
/// 編集行のラベルの欄。
const LABELS_FIELD_WIDTH_PX: u16 = 140;
/// ホイール 1 刻みで送る行数。
const WHEEL_ROWS: i32 = 3;
/// 番号の列に出す作成者の ID の桁数。
const CREATOR_DIGITS: usize = 4;
/// 過去の時点を再現している行の番号に付ける印。
const PREVIEW_MARK: &str = "←";

/// 絞り込みの候補の数。先頭は「すべて」、以降は [`ItemState::ALL`] の順。
const FILTER_COUNT: usize = ItemState::ALL.len() + 1;

/// 画面の状態。
#[derive(Debug)]
pub struct DocketScreen {
    scale: UiScale,
    interaction: Interaction,
    search: TextEdit,
    filter: SelectState,
    show_done: bool,
    scroll: ScrollState,
    selected: Option<ItemId>,
    title: TextEdit,
    assignee: TextEdit,
    due: TextEdit,
    labels: TextEdit,
    body: BodyPane,
    members: MembersPane,
    /// 編集欄を読み込んだ時点の課題。確定のとき、他人がその後に変えていれば上書きと分かる（FR-008）。
    base: Option<Item>,
    /// 履歴の行の上にポインタがあるとき、その時点の課題（表の行と本文の段に再現する・FR-006）。
    preview: Option<Item>,
    /// 列の境をドラッグ中か。離したときに設定を書く。
    resizing: Option<TableColumn>,
    bad_due: bool,
    /// 他人が変えた後に上書きした欄（FR-008）。別の行を選ぶまで示す。
    overwrote: Option<EditSlot>,
    state: SelectState,
    history: HistoryPane,
    ledger: Ledger,
    ports: Ports,
    /// 台帳の `keys/` にある名前（履歴の「誰が」）。読み直しのたびに取り直す。
    names: Vec<(AccountId, String)>,
    write_failure: Option<JournalFailure>,
    read_failure: Option<JournalFailure>,
    admin_failure: Option<JournalFailure>,
    environment: Environment,
    settings: Settings,
    language: Language,
    theme: Theme,
    language_select: SelectState,
    theme_edit: TextEdit,
    unknown_theme: bool,
    settings_failure: Option<SettingsFailure>,
}

impl DocketScreen {
    /// 環境・アカウント・ポート・設定から始める。ジャーナルを最初に読み、台帳を併合する。言語と配色は設定と環境から解決する。
    #[must_use]
    pub fn open(
        environment: Environment,
        account: Account,
        mut ports: Ports,
        settings: Settings,
    ) -> Self {
        let (ops, read_failure) = match ports.journal_mut().refresh() {
            Ok(ops) => (ops, None),
            Err(failure) => (Vec::new(), Some(failure)),
        };
        let language = settings.language().resolve(environment.language());
        let theme = settings.theme().resolve(environment.appearance());
        let names = ports.journal().names();
        Self {
            scale: environment.scale(),
            interaction: Interaction::idle(),
            search: TextEdit::empty(),
            filter: SelectState::closed(FILTER_COUNT, 0),
            show_done: true,
            scroll: ScrollState::top(0, 0),
            selected: None,
            title: TextEdit::empty(),
            assignee: TextEdit::empty(),
            due: TextEdit::empty(),
            labels: TextEdit::empty(),
            body: BodyPane::hidden(),
            members: MembersPane::hidden(environment.scale()),
            base: None,
            preview: None,
            resizing: None,
            bad_due: false,
            overwrote: None,
            state: SelectState::closed(ItemState::ALL.len(), 0),
            history: HistoryPane::hidden(environment.scale()),
            ledger: Ledger::open(account, ops),
            ports,
            names,
            write_failure: None,
            read_failure,
            admin_failure: None,
            environment,
            language_select: SelectState::closed(
                LanguageChoice::ALL.len(),
                settings.language().position(),
            ),
            theme_edit: TextEdit::showing(settings.theme().keyword()),
            settings,
            language,
            theme,
            unknown_theme: false,
            settings_failure: None,
        }
    }

    /// 1 フレーム。読み直し → 鍵 → 部品の問い合わせ → 描画命令 → フォーカスの無い欄の確定。
    pub fn step(&mut self, input: &FrameInput) -> DocketFrame {
        if input.refresh() {
            self.refresh();
        }
        for key in input.keys() {
            self.key(*key);
        }
        self.take_preedit(input);
        let bounds = input.bounds();
        let mut layout = Layout::inside(bounds.inset(self.px(MARGIN_PX)), self.px(MARGIN_PX));
        let toolbar = layout.row(self.px(ROW_PX), &self.toolbar_widths());
        let editor = layout.row(self.px(ROW_PX), &self.editor_widths());
        let (table_area, notice_rect) = self.split_notice(layout.remaining());
        let (table_area, [body_area, members_area, history_area]) = self.split_panes(table_area);
        self.preview = self.history_preview(history_area, input);
        let visible = self.visible_ids();
        let table = self.table_layout(table_area, visible.len());
        let track = Self::track_of(table_area, self.px(SCROLL_WIDTH_PX));
        self.scroll = self
            .scroll
            .resized(table.content_height(), u32::from(table.body().height()));

        self.interaction.begin(input.pointer());
        let mut commands = alloc::vec![DrawCommand::Fill {
            rect: bounds,
            paint: Paint::Background,
        }];
        let ((filter_rect, filter_response), (language_rect, language_response)) =
            self.probe_toolbar(&toolbar, &mut commands);
        let (state_rect, state_response) = self.probe_editor(&editor, &mut commands);
        self.probe_table(&table, &visible, &mut commands);
        self.probe_scroll(
            track,
            [body_area, members_area, history_area],
            &mut commands,
        );
        self.probe_body(body_area, &mut commands);
        self.probe_members(members_area, &mut commands);
        self.probe_history(history_area, &mut commands);
        self.probe_filter_options(filter_rect);
        self.probe_state_options(state_rect);
        self.probe_language_options(language_rect);
        self.box_commands(
            [
                (filter_rect, filter_response),
                (state_rect, state_response),
                (language_rect, language_response),
            ],
            &mut commands,
        );
        self.notice_commands(notice_rect, &mut commands);
        self.interaction.end();
        self.commit_unfocused_slots();
        DocketFrame::new((self.scale, self.language, self.theme), commands)
    }

    /// 3 つの箱（絞り込み・状態・言語）を最後に描く（開いた候補が他の部品の上に来る）。
    fn box_commands(&self, boxes: [(PixelRect, Response); 3], commands: &mut Vec<DrawCommand>) {
        let [
            (filter_rect, filter_response),
            (state_rect, state_response),
            (language_rect, language_response),
        ] = boxes;
        commands.extend(select_box::select_box(
            filter_rect,
            &self.filter_options(),
            self.filter,
            filter_response,
        ));
        commands.extend(select_box::select_box(
            state_rect,
            &self.state_options(),
            self.state,
            state_response,
        ));
        commands.extend(select_box::select_box(
            language_rect,
            &self.language_options(),
            self.language_select,
            language_response,
        ));
    }

    /// 表示言語の語。
    fn word(&self, word: Word) -> &'static str {
        wording::text(self.language, word)
    }

    /// 絞り込みの候補の文言。
    fn filter_options(&self) -> [&'static str; FILTER_COUNT] {
        let [open, in_progress, done] = self.state_options();
        [self.word(Word::All), open, in_progress, done]
    }

    /// 状態の候補の文言。
    fn state_options(&self) -> [&'static str; 3] {
        ItemState::ALL.map(|state| self.word(Word::StateName(state)))
    }

    /// 言語の候補の文言。先頭は「OS に従う」、以降は各言語の自分の名前。
    fn language_options(&self) -> [&'static str; 7] {
        LanguageChoice::ALL.map(|choice| match choice {
            LanguageChoice::System => self.word(Word::FollowSystem),
            LanguageChoice::Fixed(language) => language.native_name(),
        })
    }

    /// 表示言語。
    #[must_use]
    pub const fn language(&self) -> Language {
        self.language
    }

    /// 配色。
    #[must_use]
    pub const fn theme(&self) -> Theme {
        self.theme
    }

    /// 設定（画面で変えた言語と配色を含む）。
    #[must_use]
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    /// 検索欄の文字列。
    #[must_use]
    pub fn search_text(&self) -> &str {
        self.search.text()
    }

    /// 選んでいる課題。
    #[must_use]
    pub const fn selected(&self) -> Option<ItemId> {
        self.selected
    }

    /// 完了を表に出すか。
    #[must_use]
    pub const fn shows_done(&self) -> bool {
        self.show_done
    }

    /// 絞り込みの状態。
    #[must_use]
    pub const fn filter(&self) -> SelectState {
        self.filter
    }

    /// スクロールの位置。
    #[must_use]
    pub const fn scroll(&self) -> ScrollState {
        self.scroll
    }

    /// 台帳。
    #[must_use]
    pub const fn ledger(&self) -> &Ledger {
        &self.ledger
    }

    /// 履歴の段を出しているか。
    #[must_use]
    pub const fn shows_history(&self) -> bool {
        self.history.is_shown()
    }

    /// 本文の段を出しているか。
    #[must_use]
    pub const fn shows_body(&self) -> bool {
        self.body.is_shown()
    }

    /// 参加者の段を出しているか。
    #[must_use]
    pub const fn shows_members(&self) -> bool {
        self.members.is_shown()
    }

    /// 台帳に参加したアカウント（FR-009）。`keys/` の名前と権限から。
    #[must_use]
    pub fn members(&self) -> Vec<Member> {
        Member::list(&self.names, self.ports.journal().permissions().as_ref())
    }

    /// 本文の欄の文字列（編集中の値）。
    #[must_use]
    pub fn body_text(&self) -> &str {
        self.body.edit().text()
    }

    /// 選んだ課題の履歴。新しいものが先。選んでいなければ空。
    #[must_use]
    pub fn history_lines(&self) -> Vec<HistoryLine> {
        self.selected
            .map(|id| HistoryLine::lines(self.ledger.ops(), id, &self.names))
            .unwrap_or_default()
    }

    /// 利用者に示す降格（FR-014）。無ければ空。
    #[must_use]
    pub fn notices(&self) -> Vec<Notice> {
        let mut notices = Vec::new();
        notices.extend(self.write_failure.map(Notice::Journal));
        notices.extend(self.read_failure.map(Notice::Journal));
        notices.extend(self.admin_failure.map(Notice::Journal));
        let orphans = self.ledger.docket().orphans();
        if orphans > 0 {
            notices.push(Notice::Orphans(orphans));
        }
        let rejected = self.ports.journal().rejected_lines();
        if rejected > 0 {
            notices.push(Notice::RejectedLines(rejected));
        }
        if self.bad_due {
            notices.push(Notice::NotADate);
        }
        if self.overwrote.is_some() {
            notices.push(Notice::Overwrote);
        }
        notices.extend(self.settings_failure.map(Notice::Settings));
        if self.unknown_theme {
            notices.push(Notice::UnknownTheme);
        }
        let admission = self.ports.journal().admission();
        if admission.unverified_lines() > 0 {
            notices.push(Notice::UnverifiedLines(admission.unverified_lines()));
        }
        if admission.key_mismatch() {
            notices.push(Notice::KeyMismatch);
        }
        if !admission.may_edit() {
            notices.push(Notice::NotAnEditor);
        }
        notices
    }

    /// 絞り込んだ後の課題。表に出す順。
    #[must_use]
    pub fn visible_items(&self) -> Vec<&Item> {
        self.visible_ids()
            .into_iter()
            .filter_map(|id| self.ledger.docket().item(id))
            .collect()
    }

    /// 絞り込んだ後の課題の ID。表に出す順。1 フレームに 1 度だけ数える（FR-002）。
    fn visible_ids(&self) -> Vec<ItemId> {
        let wanted = self
            .filter
            .chosen()
            .checked_sub(1)
            .and_then(|index| ItemState::ALL.get(index).copied());
        self.ledger
            .docket()
            .live_items()
            .into_iter()
            .filter(|item| wanted.is_none_or(|state| item.state() == state))
            .filter(|item| self.show_done || item.state() != ItemState::Done)
            .filter(|item| self.matches_search(item))
            .map(Item::id)
            .collect()
    }

    /// 検索欄が空か、題名か本文かラベルに含まれる。
    fn matches_search(&self, item: &Item) -> bool {
        let wanted = self.search.text();
        wanted.is_empty()
            || item.title().contains(wanted)
            || item.body().contains(wanted)
            || item.labels().iter().any(|label| label.contains(wanted))
    }

    /// 選んでいる課題の、いまの値。
    fn selected_item(&self) -> Option<Item> {
        self.selected
            .and_then(|id| self.ledger.docket().item(id))
            .cloned()
    }

    /// 1x の寸法に倍率を掛ける。
    const fn px(&self, base: u16) -> u16 {
        self.scale.px(base)
    }

    fn toolbar_widths(&self) -> [ColumnWidth; 9] {
        [
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(FILTER_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(SHOW_DONE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(HISTORY_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(BODY_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(MEMBERS_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(LANGUAGE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(THEME_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
        ]
    }

    /// 表の下の段（上から本文・参加者・履歴）。本文と履歴は課題を選んでいるときだけ。
    /// 高さは 1 つなら 2/5、2 つなら 1/4 ずつ、3 つなら 1/5 ずつ（ヘッダ ＋ 3 行以上）。
    fn split_panes(&self, area: PixelRect) -> (PixelRect, [Option<PixelRect>; 3]) {
        let shown = [
            self.body.is_shown() && self.selected.is_some(),
            self.members.is_shown(),
            self.history.is_shown() && self.selected.is_some(),
        ];
        let count = shown.iter().filter(|flag| **flag).count();
        let share = match count {
            0 | 1 => self.history.height_in(area.height()),
            2 => area.height() / 4,
            _ => area.height() / 5,
        };
        let height = share.max(self.px(ROW_PX).saturating_mul(4));
        let mut bottom = area.bottom();
        let mut panes = [None, None, None];
        for (slot, flag) in panes.iter_mut().zip(shown).rev() {
            if !flag {
                continue;
            }
            let rect = PixelRect::new(
                area.x(),
                bottom.saturating_sub(height).saturating_add(1),
                area.width(),
                height,
            );
            bottom = rect
                .y()
                .saturating_sub(self.px(MARGIN_PX))
                .saturating_sub(1);
            *slot = Some(rect);
        }
        let table = PixelRect::new(
            area.x(),
            area.y(),
            area.width(),
            bottom.saturating_add(1).saturating_sub(area.y()),
        );
        (table, panes)
    }

    /// ポインタの下の履歴の行の、その時点の課題（FR-006）。段が無い・行の上に無ければ `None`。
    fn history_preview(&self, area: Option<PixelRect>, input: &FrameInput) -> Option<Item> {
        let area = area?;
        let point = input
            .pointer()
            .position()
            .filter(|point| area.contains(*point))?;
        let lines = self.history_lines();
        let index = self.history.line_at(area, lines.len(), point)?;
        lines.get(index).map(|line| line.snapshot().clone())
    }

    /// 参加者の段。持ち主なら行の押下と「権限」のチェックを受け、権限を書く。
    fn probe_members(&mut self, area: Option<PixelRect>, commands: &mut Vec<DrawCommand>) {
        let Some(area) = area else {
            return;
        };
        let members = self.members();
        let owner = self.ports.journal().owner() == Some(self.ledger.account().id());
        let enabled = self.ports.journal().permissions().is_some();
        let (drawn, action) = self.members.probe(
            &mut self.interaction,
            (area, &members, self.language),
            (owner, enabled),
        );
        commands.extend(drawn);
        if let Some(action) = action {
            self.administer(action, enabled);
        }
    }

    /// 持ち主の求めを権限の命令にして書く。書けなければ示す（FR-014）。
    fn administer(&mut self, action: MembersAction, enabled: bool) {
        let commands = match action {
            MembersAction::TogglePermissions => alloc::vec![AdminCommand::Permissions(!enabled)],
            MembersAction::ToggleEditor(id) => {
                let editor = self
                    .ports
                    .journal()
                    .permissions()
                    .is_some_and(|permissions| permissions.may_edit(id));
                alloc::vec![if editor {
                    AdminCommand::Disallow(id)
                } else {
                    AdminCommand::Allow(id)
                }]
            }
        };
        self.admin_failure = self.ports.journal_mut().administer(&commands).err();
        self.names = self.ports.journal().names();
    }

    /// 本文の段。複数行の欄 1 つ。
    fn probe_body(&mut self, area: Option<PixelRect>, commands: &mut Vec<DrawCommand>) {
        let Some(area) = area else {
            return;
        };
        let (line_px, scroll_width) = (self.px(BODY_LINE_PX), self.px(SCROLL_WIDTH_PX));
        let preview = self.preview.as_ref().map(Item::body);
        self.body.probe(
            &mut self.interaction,
            (area, line_px, scroll_width, preview),
            commands,
        );
    }

    fn probe_history(&mut self, area: Option<PixelRect>, commands: &mut Vec<DrawCommand>) {
        let Some(area) = area else {
            return;
        };
        let lines = self.history_lines();
        let clicked = self.history.probe(
            &mut self.interaction,
            (area, &lines, self.language),
            commands,
        );
        if let Some(line) = clicked.and_then(|index| lines.get(index)) {
            self.repick(line.change().clone());
        }
    }

    fn editor_widths(&self) -> [ColumnWidth; 6] {
        [
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(EDIT_FIELD_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(EDIT_FIELD_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(LABELS_FIELD_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(STATE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
        ]
    }

    /// 欄の部品の ID。
    const fn slot_id(slot: EditSlot) -> WidgetId {
        match slot {
            EditSlot::Title => TITLE,
            EditSlot::Assignee => ASSIGNEE,
            EditSlot::Due => DUE,
            EditSlot::Labels => LABELS,
            EditSlot::Body => BodyPane::id(),
        }
    }

    /// 鍵を持つ欄。
    fn focused_slot(&self) -> Option<EditSlot> {
        EditSlot::ALL
            .into_iter()
            .find(|&slot| self.interaction.focus() == Some(Self::slot_id(slot)))
    }

    const fn slot(&self, slot: EditSlot) -> &TextEdit {
        match slot {
            EditSlot::Title => &self.title,
            EditSlot::Assignee => &self.assignee,
            EditSlot::Due => &self.due,
            EditSlot::Labels => &self.labels,
            EditSlot::Body => self.body.edit(),
        }
    }

    const fn slot_mut(&mut self, slot: EditSlot) -> &mut TextEdit {
        match slot {
            EditSlot::Title => &mut self.title,
            EditSlot::Assignee => &mut self.assignee,
            EditSlot::Due => &mut self.due,
            EditSlot::Labels => &mut self.labels,
            EditSlot::Body => self.body.edit_mut(),
        }
    }

    /// 降格があれば、残りの下 1 段を文言に使う。
    fn split_notice(&self, remaining: PixelRect) -> (PixelRect, Option<PixelRect>) {
        if self.notices().is_empty() {
            return (remaining, None);
        }
        let line = self.px(ROW_PX);
        let table_height = remaining
            .height()
            .saturating_sub(line)
            .saturating_sub(self.px(MARGIN_PX));
        let table = PixelRect::new(
            remaining.x(),
            remaining.y(),
            remaining.width(),
            table_height,
        );
        let notice = PixelRect::new(
            remaining.x(),
            remaining.bottom().saturating_sub(line).saturating_add(1),
            remaining.width(),
            line,
        );
        (table, Some(notice))
    }

    fn table_layout(&self, remaining: PixelRect, visible: usize) -> TableLayout {
        let inner = remaining.width().saturating_sub(self.px(SCROLL_WIDTH_PX));
        let bounds = PixelRect::new(remaining.x(), remaining.y(), inner, remaining.height());
        let rows = u32::try_from(visible).unwrap_or(u32::MAX);
        let widths = self.settings.columns();
        let columns = TableColumn::ALL
            .into_iter()
            .map(|column| {
                widths.get(column).map_or(ColumnWidth::Fill, |width| {
                    ColumnWidth::Fixed(self.px(width))
                })
            })
            .collect();
        TableLayout::new(bounds, self.px(ROW_PX), self.px(ROW_PX), rows).with_columns(columns)
    }

    /// ヘッダの列の境をつまんで幅を変える（FR-012）。境の左の列、題名の右の境なら右の列。離したら設定に書く。
    fn probe_column_grips(&mut self, header: PixelRect, cells: &[PixelRect]) {
        let grip = self.px(GRIP_PX);
        let mut held = None;
        for (index, (left, right)) in cells.iter().zip(cells.iter().skip(1)).enumerate() {
            let rect = PixelRect::new(
                left.right().saturating_sub(grip / 2),
                header.y(),
                grip,
                header.height(),
            );
            let response = self.interaction.probe(
                COLUMN_GRIP.indexed(u32::try_from(index).unwrap_or(0)),
                rect,
                false,
            );
            if response.phase() != ResponsePhase::Held {
                continue;
            }
            let Some(x) = self.interaction.pointer().position().map(PixelPoint::x) else {
                continue;
            };
            // 境を動かさなければ幅はそのまま: 左の列は x - 左端 + 1、右の列は 右端 - x - 隙間。
            let gap = right.x().saturating_sub(left.right()).saturating_sub(1);
            let (column, physical) = match TableColumn::ALL.get(index).copied() {
                Some(column) if column.is_resizable() => {
                    (column, x.saturating_sub(left.x()).saturating_add(1))
                }
                _ => (
                    TableColumn::ALL
                        .get(index.saturating_add(1))
                        .copied()
                        .unwrap_or(TableColumn::Labels),
                    right.right().saturating_sub(x).saturating_sub(gap),
                ),
            };
            let logical = physical / self.scale.factor().max(1);
            let widths = self.settings.columns().with(column, logical);
            self.settings = self.settings.clone().with_columns(widths);
            held = Some(column);
        }
        if held.is_none() && self.resizing.is_some() {
            self.save_settings();
        }
        self.resizing = held;
    }

    const fn track_of(remaining: PixelRect, width: u16) -> PixelRect {
        PixelRect::new(
            remaining.right().saturating_sub(width).saturating_add(1),
            remaining.y(),
            width,
            remaining.height(),
        )
    }

    /// 確定前の文字列は、フォーカスのある入力欄へ。
    fn take_preedit(&mut self, input: &FrameInput) {
        if self.interaction.focus() == Some(SEARCH) {
            self.search = self.search.with_preedit(input.preedit().cloned());
        } else if self.interaction.focus() == Some(THEME) {
            self.theme_edit = self.theme_edit.with_preedit(input.preedit().cloned());
        } else if let Some(slot) = self.focused_slot() {
            let edit = self.slot_mut(slot);
            *edit = edit.with_preedit(input.preedit().cloned());
        }
    }

    /// 鍵を、前のフレームのフォーカスの持ち主へ。Tab・Esc・Undo・Redo は持ち主に依らない。
    fn key(&mut self, key: KeyIntent) {
        match key {
            KeyIntent::Tab => self.interaction.focus_next(),
            KeyIntent::BackTab => self.interaction.focus_previous(),
            KeyIntent::Escape if !self.any_options_open() => {
                self.interaction.blur();
            }
            KeyIntent::Undo => self.undo(),
            KeyIntent::Redo => self.redo(),
            KeyIntent::Char(_)
            | KeyIntent::Move { .. }
            | KeyIntent::Backspace
            | KeyIntent::Delete
            | KeyIntent::SelectAll
            | KeyIntent::Enter
            | KeyIntent::Escape
            | KeyIntent::Up
            | KeyIntent::Down => self.key_to_focus(key),
        }
    }

    fn key_to_focus(&mut self, key: KeyIntent) {
        if let Some(slot) = self.focused_slot() {
            self.key_to_slot(slot, key);
            return;
        }
        match self.interaction.focus() {
            Some(SEARCH) => self.search = self.search.applied(key),
            Some(FILTER) => self.filter = self.filter.applied(key),
            Some(LANGUAGE) => {
                let before = self.language_select.chosen();
                self.language_select = self.language_select.applied(key);
                self.language_changed(before);
            }
            Some(THEME) => self.theme_key(key),
            Some(STATE) => {
                let before = self.state.chosen();
                self.state = self.state.applied(key);
                self.state_changed(before);
            }
            Some(id) => {
                if !self.key_to_control(id, key) {
                    self.key_to_table(key);
                }
            }
            None => self.key_to_table(key),
        }
    }

    /// 文字の欄の鍵。本文は複数行（Enter は改行）、ほかは Enter で確定。
    fn key_to_slot(&mut self, slot: EditSlot, key: KeyIntent) {
        if slot == EditSlot::Body {
            let edit = self.body.edit_mut();
            *edit = edit.applied_in_area(key);
        } else if key == KeyIntent::Enter {
            self.commit_slot(slot);
        } else {
            let edit = self.slot_mut(slot);
            *edit = edit.applied(key);
        }
    }

    /// チェックボックスとボタンの鍵。Enter か空白で押す。扱ったら真。
    fn key_to_control(&mut self, id: WidgetId, key: KeyIntent) -> bool {
        let pressed = matches!(key, KeyIntent::Enter | KeyIntent::Char(' '));
        match id {
            SHOW_DONE if pressed => self.show_done = !self.show_done,
            HISTORY_TOGGLE if pressed => self.history.toggle(),
            BODY_TOGGLE if pressed => self.body.toggle(),
            MEMBERS_TOGGLE if pressed => self.members.toggle(),
            ADD if key == KeyIntent::Enter => self.add_item(),
            DELETE if key == KeyIntent::Enter => self.delete_item(),
            _ => return false,
        }
        true
    }

    /// フォーカスが無いときの上下はスクロール。
    fn key_to_table(&mut self, key: KeyIntent) {
        let rows = match key {
            KeyIntent::Up => -1_i32,
            KeyIntent::Down => 1_i32,
            KeyIntent::Char(_)
            | KeyIntent::Move { .. }
            | KeyIntent::Backspace
            | KeyIntent::Delete
            | KeyIntent::SelectAll
            | KeyIntent::Enter
            | KeyIntent::Escape
            | KeyIntent::Tab
            | KeyIntent::BackTab
            | KeyIntent::Undo
            | KeyIntent::Redo => 0_i32,
        };
        self.scroll = self
            .scroll
            .scrolled_by(rows.saturating_mul(i32::from(self.px(ROW_PX))));
    }

    /// 自分の操作を 1 つ作り、ジャーナルへ追記して同期する（FR-004）。
    /// その前に他人の伸びた分を取り込むので、自分の操作は見たすべての操作より後に並ぶ（ADR 0004 の 8）。
    fn compose(&mut self, kind: OpKind) {
        self.absorb_others();
        let now = self.ports.clock().now();
        let op = self.ledger.compose(now, kind);
        self.write(&op);
    }

    fn write(&mut self, op: &Op) {
        let journal = self.ports.journal_mut();
        self.write_failure = journal.append(op).and_then(|()| journal.sync()).err();
    }

    /// Undo は利用者が明示した戻しなので、入力中の欄も戻った値に合わせる（残すと、フォーカスが外れた瞬間に戻す前の値を追記してしまう）。
    fn undo(&mut self) {
        let now = self.ports.clock().now();
        if let Some(op) = self.ledger.compose_undo(now) {
            self.write(&op);
            self.reload_editor(true);
        }
    }

    fn redo(&mut self) {
        let now = self.ports.clock().now();
        if let Some(op) = self.ledger.compose_redo(now) {
            self.write(&op);
            self.reload_editor(true);
        }
    }

    /// 他人のジャーナルの伸びた分を取り込み、編集欄を合わせる（刻みの合図）。入力中の欄は触らない。
    fn refresh(&mut self) {
        self.absorb_others();
        self.reload_editor(false);
    }

    /// 他人のジャーナルの伸びた分を取り込む。編集欄は触らない。
    fn absorb_others(&mut self) {
        match self.ports.journal_mut().refresh() {
            Ok(fresh) => {
                self.read_failure = None;
                self.names = self.ports.journal().names();
                self.ledger.absorb(fresh);
            }
            Err(failure) => self.read_failure = Some(failure),
        }
    }

    /// 課題を作る。検索欄に字があればそれを題名にし、題名の欄にフォーカスを置く。
    fn add_item(&mut self) {
        self.commit_unfocused_slots();
        let id = self.ledger.fresh_item_id();
        self.compose(OpKind::Create(id));
        if !self.search.text().is_empty() {
            self.compose(OpKind::Set {
                item: id,
                value: FieldValue::Title(self.search.text().to_owned()),
            });
        }
        self.select(id);
        self.interaction.focus_on(TITLE);
    }

    /// 選んでいる課題を消す（ジャーナルには残る）。
    fn delete_item(&mut self) {
        if let Some(id) = self.selected {
            self.compose(OpKind::Delete(id));
            self.selected = None;
            self.reload_editor(true);
        }
    }

    /// 履歴の行の値を選び直す（FR-008）。`Set` の行で、いまの値と違えば新しい `Set` を追記する。
    fn repick(&mut self, change: HistoryChange) {
        let HistoryChange::Set { field, after, .. } = change else {
            return;
        };
        let Some(item) = self.selected_item() else {
            return;
        };
        if item.value_of(field) == after {
            return;
        }
        self.compose(OpKind::Set {
            item: item.id(),
            value: after,
        });
        self.reload_editor(true);
    }

    /// 行を選ぶ。編集中の欄はその前に確定する。
    fn select(&mut self, id: ItemId) {
        self.commit_unfocused_slots();
        self.selected = Some(id);
        self.bad_due = false;
        self.overwrote = None;
        let item = self.selected_item();
        self.base.clone_from(&item);
        for slot in EditSlot::ALL {
            *self.slot_mut(slot) = TextEdit::showing(
                &item
                    .as_ref()
                    .map_or_else(String::new, |found| slot.text_of(found)),
            );
        }
        self.state = SelectState::closed(
            ItemState::ALL.len(),
            item.map_or(0, |found| Self::state_index(found.state())),
        );
    }

    /// 台帳が変わった後、編集欄をいまの値に合わせる。`force` でなければ入力中の欄は触らない。
    fn reload_editor(&mut self, force: bool) {
        let item = self.selected_item();
        for slot in EditSlot::ALL {
            if force || self.interaction.focus() != Some(Self::slot_id(slot)) {
                *self.slot_mut(slot) = TextEdit::showing(
                    &item
                        .as_ref()
                        .map_or_else(String::new, |found| slot.text_of(found)),
                );
                self.rebase(slot, item.as_ref());
            }
        }
        if !self.state.is_open() {
            self.state = SelectState::closed(
                ItemState::ALL.len(),
                item.map_or(0, |found| Self::state_index(found.state())),
            );
        }
    }

    /// フォーカスの無い欄を確定する（フレームの終わり・行の選択・追加の前）。
    fn commit_unfocused_slots(&mut self) {
        for slot in EditSlot::ALL {
            if self.interaction.focus() != Some(Self::slot_id(slot)) {
                self.commit_slot(slot);
            }
        }
    }

    /// 欄が課題と違えば `Set` を作る。期限の形が違えば追記せず示す（FR-014）。
    fn commit_slot(&mut self, slot: EditSlot) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let text = self.slot(slot).text().to_owned();
        let Some(value) = slot.value_of(&text) else {
            self.bad_due = true;
            return;
        };
        if slot == EditSlot::Due {
            self.bad_due = false;
        }
        if item.value_of(slot.field()) != value {
            let changed_by_others = self
                .base
                .as_ref()
                .is_some_and(|base| base.value_of(slot.field()) != item.value_of(slot.field()));
            if changed_by_others {
                self.overwrote = Some(slot);
            }
            self.compose(OpKind::Set {
                item: item.id(),
                value,
            });
            // 欄を正規の形（`a, b`・`2026-12-31`）に揃え、読み込んだ時点の値も自分の値にする。
            if let Some(updated) = self.selected_item() {
                *self.slot_mut(slot) = TextEdit::showing(&slot.text_of(&updated));
                self.rebase(slot, Some(&updated));
            }
        }
    }

    /// 欄を読み込んだ時点の値を、いまの値に合わせる（その欄は読み込み直した）。
    fn rebase(&mut self, slot: EditSlot, item: Option<&Item>) {
        self.base = match (self.base.take(), item) {
            (Some(base), Some(current)) => Some(base.with(current.value_of(slot.field()))),
            (_, current) => current.cloned(),
        };
    }

    /// 状態の箱の選びが変わっていれば `Set` を作る。
    fn state_changed(&mut self, before: usize) {
        let chosen = self.state.chosen();
        let Some(state) = ItemState::ALL.get(chosen).copied() else {
            return;
        };
        if let Some(id) = self.selected.filter(|_| chosen != before) {
            self.compose(OpKind::Set {
                item: id,
                value: FieldValue::State(state),
            });
        }
    }

    fn state_index(state: ItemState) -> usize {
        ItemState::ALL
            .iter()
            .position(|&candidate| candidate == state)
            .unwrap_or(0)
    }

    /// 言語の箱の選びが変わっていれば、言語を変えて設定に残す。
    fn language_changed(&mut self, before: usize) {
        let chosen = self.language_select.chosen();
        if chosen == before {
            return;
        }
        let Some(choice) = LanguageChoice::ALL.get(chosen).copied() else {
            return;
        };
        self.settings = self.settings.clone().with_language(choice);
        self.language = choice.resolve(self.environment.language());
        self.save_settings();
    }

    /// 配色の欄の鍵。Enter で名前を適用（`system` も）、上下で表を巡る、ほかは字の編集。
    fn theme_key(&mut self, key: KeyIntent) {
        match key {
            KeyIntent::Enter => self.apply_theme_text(),
            KeyIntent::Up => self.set_theme(self.settings.theme().previous()),
            KeyIntent::Down => self.set_theme(self.settings.theme().next()),
            KeyIntent::Char(_)
            | KeyIntent::Move { .. }
            | KeyIntent::Backspace
            | KeyIntent::Delete
            | KeyIntent::SelectAll
            | KeyIntent::Escape
            | KeyIntent::Tab
            | KeyIntent::BackTab
            | KeyIntent::Undo
            | KeyIntent::Redo => self.theme_edit = self.theme_edit.applied(key),
        }
    }

    /// 欄の名前の配色にする。知らない名前なら変えずに示す（FR-014）。
    fn apply_theme_text(&mut self) {
        match ThemeChoice::from_keyword(self.theme_edit.text().trim()) {
            Some(choice) => self.set_theme(choice),
            None => self.unknown_theme = true,
        }
    }

    /// 配色を変えて設定に残す。欄は正規の名前に揃える。
    fn set_theme(&mut self, choice: ThemeChoice) {
        self.unknown_theme = false;
        self.settings = self.settings.clone().with_theme(choice);
        self.theme = choice.resolve(self.environment.appearance());
        self.theme_edit = TextEdit::showing(choice.keyword());
        self.save_settings();
    }

    /// 設定を書く。書けなければ示す。
    fn save_settings(&mut self) {
        self.settings_failure = self.ports.settings_mut().save(&self.settings).err();
    }

    /// 検索欄・絞り込み・完了の表示・履歴・言語・配色・追加。箱は候補が表の上に乗るので、最後に描く。
    fn probe_toolbar(
        &mut self,
        toolbar: &[PixelRect],
        commands: &mut Vec<DrawCommand>,
    ) -> ((PixelRect, Response), (PixelRect, Response)) {
        let [
            search_rect,
            filter_rect,
            done_rect,
            history_rect,
            body_rect,
            members_rect,
            language_rect,
            theme_rect,
            add_rect,
        ] = Self::rects(toolbar);
        let search = self.interaction.probe(SEARCH, search_rect, true);
        commands.extend(text_field::text_field(search_rect, &self.search, search));
        let filter = self.interaction.probe(FILTER, filter_rect, true);
        if filter.clicked() {
            self.filter = self.filter.toggled();
        }
        let done = self.interaction.probe(SHOW_DONE, done_rect, true);
        if done.clicked() {
            self.show_done = !self.show_done;
        }
        commands.extend(checkbox::checkbox(
            done_rect,
            self.word(Word::ShowDone),
            self.show_done,
            done,
        ));
        self.probe_pane_toggles([history_rect, body_rect, members_rect], commands);
        let language = self.interaction.probe(LANGUAGE, language_rect, true);
        if language.clicked() {
            self.language_select = self.language_select.toggled();
        }
        let theme = self.interaction.probe(THEME, theme_rect, true);
        commands.extend(text_field::text_field(theme_rect, &self.theme_edit, theme));
        let add = self.interaction.probe(ADD, add_rect, true);
        if add.clicked() {
            self.add_item();
        }
        commands.extend(button::button(add_rect, self.word(Word::Add), add));
        ((filter_rect, filter), (language_rect, language))
    }

    /// 表の下の段を出すチェック（履歴・本文・参加者）。
    fn probe_pane_toggles(&mut self, rects: [PixelRect; 3], commands: &mut Vec<DrawCommand>) {
        let [history_rect, body_rect, members_rect] = rects;
        let history = self.interaction.probe(HISTORY_TOGGLE, history_rect, true);
        if history.clicked() {
            self.history.toggle();
        }
        commands.extend(checkbox::checkbox(
            history_rect,
            self.word(Word::History),
            self.history.is_shown(),
            history,
        ));
        let body = self.interaction.probe(BODY_TOGGLE, body_rect, true);
        if body.clicked() {
            self.body.toggle();
        }
        commands.extend(checkbox::checkbox(
            body_rect,
            self.word(Word::FieldName(Field::Body)),
            self.body.is_shown(),
            body,
        ));
        let members = self.interaction.probe(MEMBERS_TOGGLE, members_rect, true);
        if members.clicked() {
            self.members.toggle();
        }
        commands.extend(checkbox::checkbox(
            members_rect,
            self.word(Word::Members),
            self.members.is_shown(),
            members,
        ));
    }

    /// 選んだ課題の題名・担当・期限・ラベル・状態・削除。状態の箱は最後に描く。
    fn probe_editor(
        &mut self,
        editor: &[PixelRect],
        commands: &mut Vec<DrawCommand>,
    ) -> (PixelRect, Response) {
        let [
            title_rect,
            assignee_rect,
            due_rect,
            labels_rect,
            state_rect,
            delete_rect,
        ] = Self::rects(editor);
        for (slot, rect) in
            EditSlot::ROW
                .into_iter()
                .zip([title_rect, assignee_rect, due_rect, labels_rect])
        {
            let response = self.interaction.probe(Self::slot_id(slot), rect, true);
            commands.extend(text_field::text_field(rect, self.slot(slot), response));
        }
        let state = self.interaction.probe(STATE, state_rect, true);
        if state.clicked() {
            self.state = self.state.toggled();
        }
        let delete = self.interaction.probe(DELETE, delete_rect, true);
        if delete.clicked() {
            self.delete_item();
        }
        commands.extend(button::button(delete_rect, self.word(Word::Delete), delete));
        (state_rect, state)
    }

    /// 段の列。足りなければ空の矩形で埋める（起こらないが型の上では `Vec`）。
    fn rects<const N: usize>(rects: &[PixelRect]) -> [PixelRect; N] {
        let empty = PixelRect::new(0, 0, 0, 0);
        core::array::from_fn(|index| rects.get(index).copied().unwrap_or(empty))
    }

    /// 開いた絞り込みの候補。表より後に問い合わせるので、重なった所では候補が勝つ。候補の列は `select_box` が描く。
    fn probe_filter_options(&mut self, filter_rect: PixelRect) {
        if !self.filter.is_open() {
            return;
        }
        let rects = select_box::option_rects(filter_rect, FILTER_COUNT);
        for (index, rect) in rects.into_iter().enumerate() {
            let id = FILTER_OPTION.indexed(u32::try_from(index).unwrap_or(u32::MAX));
            if self.interaction.probe(id, rect, false).clicked() {
                self.filter = self.filter.chosen_at(index);
                self.interaction.focus_on(FILTER);
            }
        }
    }

    /// 開いた状態の候補。選べば `Set` を作る。選んだ後も箱にフォーカスが残る（続けて上下で変えられる）。
    fn probe_state_options(&mut self, state_rect: PixelRect) {
        if !self.state.is_open() {
            return;
        }
        let before = self.state.chosen();
        let rects = select_box::option_rects(state_rect, ItemState::ALL.len());
        for (index, rect) in rects.into_iter().enumerate() {
            let id = STATE_OPTION.indexed(u32::try_from(index).unwrap_or(u32::MAX));
            if self.interaction.probe(id, rect, false).clicked() {
                self.state = self.state.chosen_at(index);
                self.interaction.focus_on(STATE);
                self.state_changed(before);
            }
        }
    }

    /// 開いた言語の候補。選べば言語を変えて設定に残す。
    fn probe_language_options(&mut self, language_rect: PixelRect) {
        if !self.language_select.is_open() {
            return;
        }
        let before = self.language_select.chosen();
        let rects = select_box::option_rects(language_rect, LanguageChoice::ALL.len());
        for (index, rect) in rects.into_iter().enumerate() {
            let id = LANGUAGE_OPTION.indexed(u32::try_from(index).unwrap_or(u32::MAX));
            if self.interaction.probe(id, rect, false).clicked() {
                self.language_select = self.language_select.chosen_at(index);
                self.interaction.focus_on(LANGUAGE);
                self.language_changed(before);
            }
        }
    }

    /// 候補の列が開いているあいだ、表は押せない。
    fn any_options_open(&self) -> bool {
        self.filter.is_open() || self.state.is_open() || self.language_select.is_open()
    }

    /// ヘッダと、見えている行だけ。行はクリックで選ぶ。
    fn probe_table(
        &mut self,
        table: &TableLayout,
        visible: &[ItemId],
        commands: &mut Vec<DrawCommand>,
    ) {
        let header = table.header();
        commands.push(DrawCommand::Fill {
            rect: header,
            paint: Paint::Surface,
        });
        let headings = [
            Word::Number,
            Word::FieldName(Field::Title),
            Word::FieldName(Field::State),
            Word::FieldName(Field::Assignee),
            Word::FieldName(Field::Due),
            Word::FieldName(Field::Labels),
        ]
        .map(|word| self.word(word));
        let cells = table.cells(header);
        self.probe_column_grips(header, &cells);
        for (cell, heading) in cells.into_iter().zip(headings) {
            commands.push(Self::cell_text(
                cell,
                heading,
                Paint::Muted,
                self.px(MARGIN_PX),
            ));
        }
        commands.push(DrawCommand::Clip(table.body()));
        let mut chosen = None;
        for (index, rect) in table.visible_rows(self.scroll.offset()) {
            let Some(item) = usize::try_from(index)
                .ok()
                .and_then(|i| visible.get(i))
                .and_then(|&id| self.ledger.docket().item(id))
                .cloned()
            else {
                continue;
            };
            let response = if self.any_options_open() {
                Response::new(ResponsePhase::Idle, false, false)
            } else {
                self.interaction.probe(ROW.indexed(index), rect, false)
            };
            if response.clicked() {
                chosen = Some(item.id());
            }
            self.row_commands((rect, table.cells(rect)), &item, commands);
        }
        commands.push(DrawCommand::Unclip);
        if let Some(id) = chosen {
            self.select(id);
        }
    }

    fn row_commands(
        &self,
        (rect, cells): (PixelRect, Vec<PixelRect>),
        item: &Item,
        commands: &mut Vec<DrawCommand>,
    ) {
        if self.selected == Some(item.id()) {
            commands.push(DrawCommand::Fill {
                rect,
                paint: Paint::Selection,
            });
        }
        // 履歴の行の上にポインタがあれば、選んだ行はその時点の値で（控えめな色・番号に印）。
        let past = self
            .preview
            .as_ref()
            .filter(|_| self.selected == Some(item.id()));
        let shown = past.unwrap_or(item);
        let paint = if past.is_some() {
            Paint::Muted
        } else {
            Paint::Ink
        };
        let number = if past.is_some() {
            format!("{PREVIEW_MARK} {}", Self::number_text(item.id()))
        } else {
            Self::number_text(item.id())
        };
        let texts = [
            number,
            shown.title().to_owned(),
            self.word(Word::StateName(shown.state())).to_owned(),
            shown.assignee().to_owned(),
            EditSlot::Due.text_of(shown),
            EditSlot::Labels.text_of(shown),
        ];
        for (cell, text) in cells.into_iter().zip(texts) {
            commands.push(Self::cell_text(cell, &text, paint, self.px(MARGIN_PX)));
        }
        commands.push(DrawCommand::Stroke {
            rect: PixelRect::new(rect.x(), rect.bottom(), rect.width(), 1),
            paint: Paint::Border,
        });
    }

    /// 番号の列。作成者の ID の先頭と、その作成者の中での連番。複数アカウントでも重ならない。
    fn number_text(id: ItemId) -> String {
        let creator: String = id.creator().hex().chars().take(CREATOR_DIGITS).collect();
        format!("{creator}-{}", id.sequence())
    }

    fn cell_text(cell: PixelRect, text: &str, paint: Paint, padding: u16) -> DrawCommand {
        DrawCommand::Text {
            rect: PixelRect::new(
                cell.x().saturating_add(padding),
                cell.y(),
                cell.width().saturating_sub(padding.saturating_mul(2)),
                cell.height(),
            ),
            text: text.to_owned(),
            align: TextAlign::Start,
            paint,
            caret: None,
        }
    }

    /// 降格の文言を 1 行に並べる（FR-014）。
    fn notice_commands(&self, rect: Option<PixelRect>, commands: &mut Vec<DrawCommand>) {
        let Some(rect) = rect else {
            return;
        };
        let words: Vec<String> = self
            .notices()
            .into_iter()
            .map(|notice| notice.describe(self.language))
            .collect();
        commands.push(DrawCommand::Fill {
            rect,
            paint: Paint::Surface,
        });
        commands.push(Self::cell_text(
            rect,
            &words.join(" / "),
            Paint::Ink,
            self.px(MARGIN_PX),
        ));
    }

    /// スクロールバー。掴んで引けば位置が追い、ホイールは 3 行ずつ。ポインタが履歴の段の上ならホイールはそちらへ。
    /// 表のスクロールバー。ホイールは、ポインタが下の段（本文・履歴）の上に無いときだけ表へ。
    fn probe_scroll(
        &mut self,
        track: PixelRect,
        panes: [Option<PixelRect>; 3],
        commands: &mut Vec<DrawCommand>,
    ) {
        let response = self.interaction.probe(SCROLL, track, false);
        let pointer = self.interaction.pointer();
        let dragged = pointer
            .position()
            .filter(|_| response.phase() == ResponsePhase::Held);
        if let Some(point) = dragged {
            let into = u64::from(point.y().saturating_sub(track.y()));
            let offset = into * u64::from(self.scroll.content()) / u64::from(track.height().max(1));
            self.scroll = self
                .scroll
                .scrolled_to(u32::try_from(offset).unwrap_or(u32::MAX));
        }
        let over_pane = pointer
            .position()
            .is_some_and(|point| panes.into_iter().flatten().any(|area| area.contains(point)));
        if !over_pane {
            let wheel = i32::from(pointer.wheel())
                .saturating_mul(WHEEL_ROWS)
                .saturating_mul(i32::from(self.px(ROW_PX)));
            self.scroll = self.scroll.scrolled_by(wheel);
        }
        commands.extend(scroll_bar::scroll_bar(track, self.scroll, response));
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::rc::Rc;
    use core::cell::RefCell;

    use nene_docket_core::appearance::Appearance;
    use nene_docket_core::button_phase::ButtonPhase;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::permissions::Permissions;
    use nene_docket_core::pixel_point::PixelPoint;
    use nene_docket_core::signing_key::SigningKey;

    use crate::admission_report::AdmissionReport;
    use crate::member_role::MemberRole;
    use nene_docket_core::pointer_input::PointerInput;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;
    use crate::clock_port::ClockPort;
    use crate::journal_port::JournalPort;
    use crate::settings_port::SettingsPort;

    const BOUNDS: PixelRect = PixelRect::new(0, 0, 1060, 400);
    const ME: AccountId = AccountId::new(1);
    const OTHER: AccountId = AccountId::new(2);
    /// 1 行目の中央。段は 8..36（道具）・44..72（編集）・80..108（ヘッダ）・108 から行。
    const FIRST_ROW_Y: u16 = 120;
    const SECOND_ROW_Y: u16 = 150;
    const TOOLBAR_Y: u16 = 20;
    const EDITOR_Y: u16 = 58;
    /// 道具の段（1060 幅）: 検索 8..128・絞り込み 136..276・完了 284..404・履歴 412..512・本文 520..600・参加者 608..708・言語 716..826・配色 834..964・追加 972..1052。
    const FILTER_X: u16 = 200;
    const SHOW_DONE_X: u16 = 340;
    const HISTORY_X: u16 = 460;
    const BODY_X: u16 = 560;
    const MEMBERS_X: u16 = 660;
    const LANGUAGE_X: u16 = 770;
    const THEME_X: u16 = 900;
    const ADD_X: u16 = 1010;
    /// 編集行: 題名 8..482・担当 490..590・期限 598..698・ラベル 706..846・状態 854..964・削除 972..1052。
    const TITLE_X: u16 = 100;
    const STATE_X: u16 = 910;
    const DELETE_X: u16 = 1010;
    const LABELS_X: u16 = 780;
    /// スクロールバーの溝（1041..1052）。
    const TRACK_X: u16 = 1046;
    const ASSIGNEE_X: u16 = 540;
    const DUE_X: u16 = 650;

    /// 書き込みを覚え、読み直しで渡す分を積める偽のジャーナル。
    #[derive(Debug)]
    struct FakeJournal {
        written: Rc<RefCell<Vec<Op>>>,
        pending: Rc<RefCell<Vec<Op>>>,
        synced: Rc<RefCell<usize>>,
        fail_write: bool,
        fail_read: bool,
        rejected: usize,
        admission: AdmissionReport,
        permissions: Rc<RefCell<Option<Permissions>>>,
    }

    impl Default for FakeJournal {
        fn default() -> Self {
            Self {
                written: Rc::default(),
                pending: Rc::default(),
                synced: Rc::default(),
                fail_write: false,
                fail_read: false,
                rejected: 0,
                admission: AdmissionReport::open(),
                permissions: Rc::default(),
            }
        }
    }

    impl JournalPort for FakeJournal {
        fn append(&mut self, op: &Op) -> Result<(), JournalFailure> {
            if self.fail_write {
                return Err(JournalFailure::Unwritable);
            }
            self.written.borrow_mut().push(op.clone());
            Ok(())
        }

        fn sync(&mut self) -> Result<(), JournalFailure> {
            *self.synced.borrow_mut() += 1;
            Ok(())
        }

        fn refresh(&mut self) -> Result<Vec<Op>, JournalFailure> {
            if self.fail_read {
                return Err(JournalFailure::Unreadable);
            }
            Ok(core::mem::take(&mut *self.pending.borrow_mut()))
        }

        fn rejected_lines(&self) -> usize {
            self.rejected
        }

        fn admission(&self) -> AdmissionReport {
            self.admission
        }

        fn names(&self) -> Vec<(AccountId, String)> {
            alloc::vec![(OTHER, String::from("other")), (ME, String::from("me"))]
        }

        fn owner(&self) -> Option<AccountId> {
            Some(ME)
        }

        fn permissions(&self) -> Option<Permissions> {
            self.permissions.borrow().clone()
        }

        fn administer(&mut self, commands: &[AdminCommand]) -> Result<(), JournalFailure> {
            if self.fail_write {
                return Err(JournalFailure::PermissionsUnwritable);
            }
            let mut current = self.permissions.borrow_mut();
            for command in commands {
                *current = match (command, current.take()) {
                    (AdminCommand::Permissions(true), _) => Some(Permissions::owner_only(ME)),
                    (AdminCommand::Permissions(false), _)
                    | (
                        AdminCommand::Allow(_)
                        | AdminCommand::Disallow(_)
                        | AdminCommand::Group(..)
                        | AdminCommand::Ungroup(..),
                        None,
                    ) => None,
                    (AdminCommand::Allow(id), Some(p)) => Some(p.allowing(*id)),
                    (AdminCommand::Disallow(id), Some(p)) => Some(p.disallowing(*id)),
                    (AdminCommand::Group(name, id), Some(p)) => Some(p.grouped(name, *id)),
                    (AdminCommand::Ungroup(name, id), Some(p)) => Some(p.ungrouped(name, *id)),
                };
            }
            Ok(())
        }
    }

    #[derive(Debug)]
    struct FixedClock;

    /// 書いた設定を覚える偽の設定の置き場。
    #[derive(Debug)]
    struct FakeSettings {
        saved: Rc<RefCell<Vec<Settings>>>,
        fail: bool,
    }

    impl SettingsPort for FakeSettings {
        fn save(&mut self, settings: &Settings) -> Result<(), SettingsFailure> {
            if self.fail {
                return Err(SettingsFailure::Unwritable);
            }
            self.saved.borrow_mut().push(settings.clone());
            Ok(())
        }
    }

    impl ClockPort for FixedClock {
        fn now(&self) -> WallClock {
            WallClock::new(1_700_000_000)
        }
    }

    /// 偽のジャーナルの共有部分。画面が持った後も中を覗ける。
    struct Spy {
        written: Rc<RefCell<Vec<Op>>>,
        pending: Rc<RefCell<Vec<Op>>>,
        synced: Rc<RefCell<usize>>,
        saved: Rc<RefCell<Vec<Settings>>>,
    }

    fn screen_with(journal: FakeJournal) -> (DocketScreen, Spy) {
        screen_in(journal, Environment::new(None), false)
    }

    fn screen_in(
        journal: FakeJournal,
        environment: Environment,
        fail_settings: bool,
    ) -> (DocketScreen, Spy) {
        let saved = Rc::new(RefCell::new(Vec::new()));
        let spy = Spy {
            written: Rc::clone(&journal.written),
            pending: Rc::clone(&journal.pending),
            synced: Rc::clone(&journal.synced),
            saved: Rc::clone(&saved),
        };
        let settings_port = FakeSettings {
            saved,
            fail: fail_settings,
        };
        let ports = Ports::new(
            Box::new(journal),
            Box::new(FixedClock),
            Box::new(settings_port),
        );
        let account = Account::new(ME, String::from("me"), SigningKey::from_bytes(&[1_u8; 32]));
        (
            DocketScreen::open(environment, account, ports, Settings::initial()),
            spy,
        )
    }

    /// 他人が `count` 件作った台帳から始める。題名は `Issue #n`、状態は巡る。
    fn screen(count: u32) -> (DocketScreen, Spy) {
        let journal = FakeJournal::default();
        journal.pending.borrow_mut().extend(others_items(count));
        screen_with(journal)
    }

    fn others_items(count: u32) -> Vec<Op> {
        let mut ops = Vec::new();
        for number in 1..=count {
            let item = ItemId::new(OTHER, number);
            let clock = u64::from(number).saturating_mul(3);
            let state = ItemState::ALL
                .get(usize::try_from(number).unwrap_or(0) % ItemState::ALL.len())
                .copied()
                .unwrap_or(ItemState::Open);
            ops.push(other_op(clock, OpKind::Create(item)));
            ops.push(other_op(
                clock.saturating_add(1),
                OpKind::Set {
                    item,
                    value: FieldValue::Title(format!("Issue #{number}")),
                },
            ));
            ops.push(other_op(
                clock.saturating_add(2),
                OpKind::Set {
                    item,
                    value: FieldValue::State(state),
                },
            ));
        }
        ops
    }

    fn other_op(clock: u64, kind: OpKind) -> Op {
        Op::new(
            OpId::new(LogicalClock::new(clock), OTHER),
            WallClock::new(0),
            kind,
        )
    }

    fn pointer(x: u16, y: u16, phase: ButtonPhase) -> FrameInput {
        FrameInput::new(
            BOUNDS,
            PointerInput::new(Some(PixelPoint::new(x, y)), phase, 0),
        )
    }

    fn click(screen: &mut DocketScreen, x: u16, y: u16) {
        screen.step(&pointer(x, y, ButtonPhase::Pressed));
        screen.step(&pointer(x, y, ButtonPhase::Released));
    }

    fn keys(screen: &mut DocketScreen, keys: &[KeyIntent]) -> DocketFrame {
        screen.step(&FrameInput::new(BOUNDS, PointerInput::absent()).with_keys(keys.to_vec()))
    }

    fn idle(screen: &mut DocketScreen) -> DocketFrame {
        screen.step(&FrameInput::new(BOUNDS, PointerInput::absent()))
    }

    fn chars(text: &str) -> Vec<KeyIntent> {
        text.chars().map(KeyIntent::Char).collect()
    }

    fn title_of(screen: &DocketScreen, id: ItemId) -> String {
        screen
            .ledger()
            .docket()
            .item(id)
            .map(|item| item.title().to_owned())
            .unwrap_or_default()
    }

    fn texts(frame: &DocketFrame) -> Vec<String> {
        frame
            .commands()
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.clone()),
                DrawCommand::Fill { .. }
                | DrawCommand::Stroke { .. }
                | DrawCommand::Clip(_)
                | DrawCommand::Unclip => None,
            })
            .collect()
    }

    #[test]
    fn a_frame_starts_with_the_background_and_lists_only_visible_rows() {
        let (mut screen, _) = screen(10_000);
        let frame = idle(&mut screen);
        assert_eq!(
            frame.commands().first(),
            Some(&DrawCommand::Fill {
                rect: BOUNDS,
                paint: Paint::Background
            })
        );
        let rows = frame
            .commands()
            .iter()
            .filter(
                |command| matches!(command, DrawCommand::Stroke { rect, .. } if rect.height() == 1),
            )
            .count();
        assert!(rows < 20, "{rows}");
        assert_eq!(screen.ledger().docket().len(), 10_000);
        assert_eq!(frame.font_px(), 13);
        assert!(screen.notices().is_empty());
    }

    #[test]
    fn clicking_a_row_selects_it_and_the_wheel_scrolls() {
        let (mut screen, _) = screen(50);
        click(&mut screen, 100, FIRST_ROW_Y);
        assert_eq!(screen.selected(), Some(ItemId::new(OTHER, 1)));
        click(&mut screen, 100, SECOND_ROW_Y);
        assert_eq!(screen.selected(), Some(ItemId::new(OTHER, 2)));
        screen.step(&FrameInput::new(
            BOUNDS,
            PointerInput::new(Some(PixelPoint::new(100, 200)), ButtonPhase::Idle, 1),
        ));
        assert_eq!(screen.scroll().offset(), 84);
        click(&mut screen, 100, FIRST_ROW_Y);
        assert_eq!(screen.selected(), Some(ItemId::new(OTHER, 4)));
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().any(|text| text == "0000-4"));
    }

    #[test]
    fn typing_goes_to_the_focused_search_field_and_filters_the_table() {
        let (mut screen, _) = screen(50);
        click(&mut screen, 100, TOOLBAR_Y);
        keys(&mut screen, &chars("#4"));
        assert_eq!(screen.search_text(), "#4");
        let shown = screen.visible_items();
        assert!(shown.iter().all(|item| item.title().contains("#4")));
        assert_eq!(shown.len(), 11);
        keys(&mut screen, &[KeyIntent::Escape]);
        keys(&mut screen, &[KeyIntent::Char('x')]);
        assert_eq!(screen.search_text(), "#4");
    }

    #[test]
    fn the_filter_box_opens_and_a_click_on_an_option_chooses_it() {
        let (mut screen, _) = screen(30);
        click(&mut screen, FILTER_X, TOOLBAR_Y);
        assert!(screen.filter().is_open());
        click(&mut screen, FILTER_X, 130);
        assert!(!screen.filter().is_open());
        assert_eq!(screen.filter().chosen(), 3);
        assert!(
            screen
                .visible_items()
                .iter()
                .all(|item| item.state() == ItemState::Done)
        );
        click(&mut screen, 100, FIRST_ROW_Y);
        assert_eq!(screen.selected(), Some(ItemId::new(OTHER, 2)));
    }

    #[test]
    fn show_done_toggles_by_click_and_by_key() {
        let (mut screen, _) = screen(6);
        click(&mut screen, SHOW_DONE_X, TOOLBAR_Y);
        assert!(!screen.shows_done());
        assert_eq!(screen.visible_items().len(), 4);
        keys(&mut screen, &[KeyIntent::Char(' ')]);
        assert!(screen.shows_done());
    }

    #[test]
    fn add_creates_an_item_writes_it_and_focuses_the_title() {
        let (mut screen, spy) = screen(2);
        click(&mut screen, ADD_X, TOOLBAR_Y);
        let mine = ItemId::new(ME, 1);
        assert_eq!(screen.selected(), Some(mine));
        assert_eq!(spy.written.borrow().len(), 1);
        assert_eq!(*spy.synced.borrow(), 1);
        keys(&mut screen, &chars("Fix"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(title_of(&screen, mine), "Fix");
        assert_eq!(spy.written.borrow().len(), 2);
        keys(&mut screen, &[KeyIntent::Tab, KeyIntent::Enter]);
        assert_eq!(screen.visible_items().len(), 3);
        // 検索欄に字があれば、それが題名になる。
        click(&mut screen, 100, TOOLBAR_Y);
        keys(&mut screen, &chars("Issue"));
        keys(&mut screen, &[KeyIntent::Tab; 8]);
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(title_of(&screen, ItemId::new(ME, 2)), "Issue");
    }

    #[test]
    fn editing_the_title_commits_when_focus_leaves() {
        let (mut screen, spy) = screen(3);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, TITLE_X, EDITOR_Y);
        keys(&mut screen, &[KeyIntent::SelectAll]);
        keys(&mut screen, &chars("Renamed"));
        assert_eq!(spy.written.borrow().len(), 0);
        click(&mut screen, 100, SECOND_ROW_Y);
        assert_eq!(title_of(&screen, ItemId::new(OTHER, 1)), "Renamed");
        assert_eq!(spy.written.borrow().len(), 1);
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().any(|text| text == "Issue #2"));
    }

    #[test]
    fn the_state_box_sets_the_state_by_option_click_and_by_key() {
        let (mut screen, spy) = screen(3);
        let first = ItemId::new(OTHER, 1);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, STATE_X, EDITOR_Y);
        assert!(!screen.filter().is_open());
        click(&mut screen, STATE_X, 142);
        assert_eq!(
            screen.ledger().docket().item(first).map(Item::state),
            Some(ItemState::Done)
        );
        keys(&mut screen, &[KeyIntent::Up]);
        assert_eq!(
            screen.ledger().docket().item(first).map(Item::state),
            Some(ItemState::InProgress)
        );
        assert_eq!(spy.written.borrow().len(), 2);
    }

    #[test]
    fn delete_removes_the_row_and_undo_redo_walk_back_and_forth() {
        let (mut screen, spy) = screen(3);
        let first = ItemId::new(OTHER, 1);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, DELETE_X, EDITOR_Y);
        assert_eq!(screen.selected(), None);
        assert_eq!(screen.visible_items().len(), 2);
        keys(&mut screen, &[KeyIntent::Undo]);
        assert_eq!(screen.visible_items().len(), 3);
        keys(&mut screen, &[KeyIntent::Redo]);
        assert_eq!(screen.visible_items().len(), 2);
        keys(&mut screen, &[KeyIntent::Redo]);
        assert_eq!(spy.written.borrow().len(), 3);
        assert!(
            spy.written
                .borrow()
                .iter()
                .skip(1)
                .all(|op| op.undoes().is_some())
        );
        assert!(
            screen
                .ledger()
                .docket()
                .item(first)
                .is_some_and(Item::is_deleted)
        );
    }

    #[test]
    fn delete_by_key_needs_the_focus_and_a_selection() {
        let (mut screen, spy) = screen(2);
        keys(&mut screen, &[KeyIntent::Tab; 15]);
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(spy.written.borrow().len(), 0);
        click(&mut screen, 100, FIRST_ROW_Y);
        keys(&mut screen, &[KeyIntent::Tab; 15]);
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(spy.written.borrow().len(), 1);
    }

    #[test]
    fn the_assignee_field_commits_on_enter_and_shows_in_the_table() {
        let (mut screen, spy) = screen(2);
        let first = ItemId::new(OTHER, 1);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, ASSIGNEE_X, EDITOR_Y);
        keys(&mut screen, &chars("rina"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(
            screen.ledger().docket().item(first).map(Item::assignee),
            Some("rina")
        );
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(spy.written.borrow().len(), 1);
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().filter(|text| *text == "rina").count() >= 2);
    }

    #[test]
    fn the_due_field_takes_a_date_refuses_the_rest_and_clears_with_empty() {
        let (mut screen, spy) = screen(2);
        let first = ItemId::new(OTHER, 1);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, DUE_X, EDITOR_Y);
        keys(&mut screen, &chars("soon"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(screen.notices(), alloc::vec![Notice::NotADate]);
        assert_eq!(spy.written.borrow().len(), 0);
        keys(&mut screen, &[KeyIntent::SelectAll]);
        keys(&mut screen, &chars("2026-12-31"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert!(screen.notices().is_empty());
        assert_eq!(
            screen.ledger().docket().item(first).and_then(Item::due),
            nene_docket_core::date::Date::new(2026, 12, 31)
        );
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().any(|text| text == "2026-12-31"));
        keys(&mut screen, &[KeyIntent::SelectAll, KeyIntent::Backspace]);
        click(&mut screen, 100, SECOND_ROW_Y);
        assert_eq!(
            screen.ledger().docket().item(first).and_then(Item::due),
            None
        );
        assert_eq!(spy.written.borrow().len(), 2);
    }

    #[test]
    fn labels_are_comma_separated_and_the_search_matches_them() {
        let (mut screen, spy) = screen(2);
        let first = ItemId::new(OTHER, 1);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, LABELS_X, EDITOR_Y);
        keys(&mut screen, &chars("bug, ui ,"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(
            screen.ledger().docket().item(first).map(Item::labels),
            Some(&[String::from("bug"), String::from("ui")][..])
        );
        assert_eq!(spy.written.borrow().len(), 1);
        let frame = idle(&mut screen);
        assert!(
            texts(&frame)
                .iter()
                .filter(|text| *text == "bug, ui")
                .count()
                >= 2
        );
        click(&mut screen, 100, TOOLBAR_Y);
        keys(&mut screen, &chars("ui"));
        assert_eq!(screen.visible_items().len(), 1);
        keys(&mut screen, &[KeyIntent::SelectAll, KeyIntent::Backspace]);
        assert_eq!(screen.visible_items().len(), 2);
    }

    #[test]
    fn selecting_another_row_reloads_every_field_and_drops_the_date_notice() {
        let (mut screen, _) = screen(2);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, DUE_X, EDITOR_Y);
        keys(&mut screen, &chars("x"));
        click(&mut screen, 100, SECOND_ROW_Y);
        assert!(screen.notices().is_empty());
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().all(|text| text != "x"));
    }

    #[test]
    fn a_refresh_absorbs_others_ops_and_reloads_the_editor() {
        let (mut screen, spy) = screen(1);
        let first = ItemId::new(OTHER, 1);
        click(&mut screen, 100, FIRST_ROW_Y);
        spy.pending.borrow_mut().push(other_op(
            100,
            OpKind::Set {
                item: first,
                value: FieldValue::Title(String::from("Theirs")),
            },
        ));
        idle(&mut screen);
        assert_eq!(title_of(&screen, first), "Issue #1");
        screen.step(&FrameInput::new(BOUNDS, PointerInput::absent()).with_refresh());
        assert_eq!(title_of(&screen, first), "Theirs");
        let frame = idle(&mut screen);
        assert!(
            texts(&frame)
                .iter()
                .filter(|text| *text == "Theirs")
                .count()
                >= 2
        );
    }

    #[test]
    fn an_own_op_reads_others_first_and_orders_after_them() {
        let (mut screen, spy) = screen(1);
        let first = ItemId::new(OTHER, 1);
        spy.pending.borrow_mut().push(other_op(
            50,
            OpKind::Set {
                item: first,
                value: FieldValue::Title(String::from("Theirs")),
            },
        ));
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, TITLE_X, EDITOR_Y);
        keys(&mut screen, &[KeyIntent::SelectAll]);
        keys(&mut screen, &chars("Mine"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(title_of(&screen, first), "Mine");
        let written = spy.written.borrow();
        assert_eq!(
            written.first().map(|op| op.id().clock()),
            Some(LogicalClock::new(51))
        );
        drop(written);
        idle(&mut screen);
        assert_eq!(title_of(&screen, first), "Mine");
        assert_eq!(spy.written.borrow().len(), 1);
    }

    #[test]
    fn failures_and_rejected_lines_show_as_notices() {
        let (mut screen, _) = screen_with(FakeJournal {
            fail_write: true,
            rejected: 2,
            ..FakeJournal::default()
        });
        click(&mut screen, ADD_X, TOOLBAR_Y);
        assert_eq!(
            screen.notices(),
            alloc::vec![
                Notice::Journal(JournalFailure::Unwritable),
                Notice::RejectedLines(2)
            ]
        );
        let frame = idle(&mut screen);
        assert!(
            texts(&frame)
                .iter()
                .any(|text| text.contains("cannot write"))
        );
    }

    #[test]
    fn an_unreadable_journal_is_shown_and_clears_when_it_reads_again() {
        let (mut screen, _) = screen_with(FakeJournal {
            fail_read: true,
            ..FakeJournal::default()
        });
        assert_eq!(
            screen.notices(),
            alloc::vec![Notice::Journal(JournalFailure::Unreadable)]
        );
        screen.step(&FrameInput::new(BOUNDS, PointerInput::absent()).with_refresh());
        assert_eq!(screen.notices().len(), 1);
    }

    #[test]
    fn orphan_ops_count_as_a_notice() {
        let journal = FakeJournal::default();
        journal.pending.borrow_mut().push(other_op(
            1,
            OpKind::Set {
                item: ItemId::new(OTHER, 9),
                value: FieldValue::Title(String::from("ghost")),
            },
        ));
        let (screen, _) = screen_with(journal);
        assert_eq!(screen.notices(), alloc::vec![Notice::Orphans(1)]);
    }

    #[test]
    fn tab_moves_focus_through_the_toolbar_and_the_editor() {
        let (mut screen, _) = screen(3);
        idle(&mut screen);
        keys(&mut screen, &[KeyIntent::Tab]);
        keys(&mut screen, &[KeyIntent::Char('a')]);
        assert_eq!(screen.search_text(), "a");
        keys(&mut screen, &[KeyIntent::Tab, KeyIntent::Down]);
        assert_eq!(screen.filter().chosen(), 1);
        keys(&mut screen, &[KeyIntent::BackTab, KeyIntent::Char('b')]);
        assert_eq!(screen.search_text(), "ab");
        keys(&mut screen, &[KeyIntent::Escape, KeyIntent::Down]);
        assert_eq!(screen.scroll().offset(), 0);
    }

    /// 履歴を出し、1 行目を選んで状態を Done にし、Undo する。
    fn screen_with_history() -> DocketScreen {
        let (mut screen, _) = screen(3);
        click(&mut screen, HISTORY_X, TOOLBAR_Y);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, STATE_X, EDITOR_Y);
        click(&mut screen, STATE_X, 142);
        keys(&mut screen, &[KeyIntent::Undo]);
        screen
    }

    #[test]
    fn undo_while_a_field_is_focused_reloads_it_so_nothing_is_re_appended() {
        let (mut screen, spy) = screen(2);
        let first = ItemId::new(OTHER, 1);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, TITLE_X, EDITOR_Y);
        keys(&mut screen, &[KeyIntent::SelectAll]);
        keys(&mut screen, &chars("Renamed"));
        keys(&mut screen, &[KeyIntent::Enter]);
        keys(&mut screen, &[KeyIntent::Undo]);
        assert_eq!(title_of(&screen, first), "Issue #1");
        click(&mut screen, HISTORY_X, TOOLBAR_Y);
        idle(&mut screen);
        assert_eq!(title_of(&screen, first), "Issue #1");
        assert_eq!(spy.written.borrow().len(), 2);
        keys(&mut screen, &[KeyIntent::Redo]);
        assert_eq!(title_of(&screen, first), "Renamed");
        let frame = idle(&mut screen);
        assert!(
            texts(&frame)
                .iter()
                .filter(|text| *text == "Renamed")
                .count()
                >= 2
        );
    }

    #[test]
    fn the_history_pane_is_empty_without_a_selection() {
        let (mut screen, _) = screen(3);
        click(&mut screen, HISTORY_X, TOOLBAR_Y);
        assert!(screen.shows_history());
        assert!(screen.history_lines().is_empty());
        let without_selection = idle(&mut screen);
        assert!(texts(&without_selection).iter().all(|text| text != "When"));
    }

    #[test]
    fn the_history_pane_lists_the_selected_items_ops_newest_first() {
        let mut screen = screen_with_history();
        let lines: Vec<[String; 3]> = screen
            .history_lines()
            .iter()
            .map(|line| line.columns(Language::English))
            .collect();
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0][2], "undo: State : Done -> In progress");
        assert_eq!(lines[0][1], "me");
        assert_eq!(lines[1][2], "State : In progress -> Done");
        assert_eq!(lines[4][2], "created");
        let shown = texts(&idle(&mut screen));
        assert!(shown.iter().any(|text| text == "When"));
        assert!(shown.iter().any(|text| text == "created"));
    }

    /// 履歴の段（1060×400・残り 80..391 の 2/5 = 124）: 段は 268..391、ヘッダ 268..292、行は 292 から 24 ずつ。
    const FIRST_HISTORY_Y: u16 = 304;
    const SECOND_HISTORY_Y: u16 = 328;

    #[test]
    fn clicking_a_history_line_re_picks_its_value_as_a_new_op() {
        let mut screen = screen_with_history();
        let item = ItemId::new(OTHER, 1);
        // 履歴（新しい順）: undo の行（Done -> In progress）・In progress -> Done・…
        let before = screen.ledger().ops().len();
        click(&mut screen, 100, SECOND_HISTORY_Y);
        assert_eq!(screen.ledger().ops().len(), before + 1);
        assert_eq!(
            screen.ledger().docket().item(item).map(Item::state),
            Some(ItemState::Done)
        );
        // いまと同じ値の行を押しても何も追記しない。
        click(&mut screen, 100, FIRST_HISTORY_Y);
        assert_eq!(screen.ledger().ops().len(), before + 1);
        // 「作成」の行は何もしない（一番下。スクロールして押す代わりに直接）。
        let created = screen.history_lines().last().cloned().unwrap();
        screen.repick(created.change().clone());
        assert_eq!(screen.ledger().ops().len(), before + 1);
    }

    #[test]
    fn overwriting_a_change_that_arrived_while_editing_is_shown() {
        let (mut screen, spy) = screen(2);
        let item = ItemId::new(OTHER, 1);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, TITLE_X, EDITOR_Y);
        keys(&mut screen, &[KeyIntent::SelectAll]);
        keys(&mut screen, &chars("mine"));
        // 他人が同じ題名を変えた分が届く。
        spy.pending.borrow_mut().push(other_op(
            50,
            OpKind::Set {
                item,
                value: FieldValue::Title(String::from("theirs")),
            },
        ));
        screen.step(&FrameInput::new(BOUNDS, PointerInput::absent()).with_refresh());
        assert!(screen.notices().is_empty());
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(title_of(&screen, item), "mine");
        assert_eq!(screen.notices(), alloc::vec![Notice::Overwrote]);
        // 別の行を選ぶと消える。自分だけが変えた欄では出ない。
        click(&mut screen, 100, SECOND_ROW_Y);
        assert!(screen.notices().is_empty());
        click(&mut screen, TITLE_X, EDITOR_Y);
        keys(&mut screen, &chars("!"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert!(screen.notices().is_empty());
    }

    #[test]
    fn hovering_a_history_line_shows_the_item_as_it_was_then() {
        let mut screen = screen_with_history();
        // いまは In progress（Done を Undo した）。2 行目は「In progress -> Done」で、その直後は Done。
        let now = texts(&idle(&mut screen));
        assert!(now.iter().any(|text| text == "In progress"));
        let hover = screen.step(&FrameInput::new(
            BOUNDS,
            PointerInput::new(
                Some(PixelPoint::new(100, SECOND_HISTORY_Y)),
                ButtonPhase::Idle,
                0,
            ),
        ));
        let shown = texts(&hover);
        assert!(shown.iter().any(|text| text == "Done"));
        assert!(shown.iter().any(|text| text.starts_with("← ")));
        assert!(screen.preview.is_some());
        // 離せば戻る。ジャーナルには何も足さない。
        let back = texts(&idle(&mut screen));
        assert!(back.iter().any(|text| text == "In progress"));
        assert!(!back.iter().any(|text| text.starts_with("← ")));
        assert!(screen.preview.is_none());
    }

    #[test]
    fn the_wheel_over_the_history_pane_leaves_the_table_and_the_key_hides_it() {
        let mut screen = screen_with_history();
        screen.step(&FrameInput::new(
            BOUNDS,
            PointerInput::new(Some(PixelPoint::new(100, 380)), ButtonPhase::Idle, 1),
        ));
        assert_eq!(screen.scroll().offset(), 0);
        keys(&mut screen, &[KeyIntent::Escape]);
        keys(&mut screen, &[KeyIntent::Tab; 4]);
        keys(&mut screen, &[KeyIntent::Char(' ')]);
        assert!(!screen.shows_history());
    }

    #[test]
    fn the_language_box_changes_every_word_and_saves_the_choice() {
        let (mut screen, spy) = screen(2);
        assert_eq!(screen.language(), Language::English);
        click(&mut screen, LANGUAGE_X, TOOLBAR_Y);
        click(&mut screen, LANGUAGE_X, 64);
        assert_eq!(screen.language(), Language::Japanese);
        assert_eq!(
            spy.saved.borrow().last().map(Settings::language),
            Some(LanguageChoice::Fixed(Language::Japanese))
        );
        let frame = idle(&mut screen);
        let shown = texts(&frame);
        assert!(shown.iter().any(|text| text == "追加"));
        assert!(shown.iter().any(|text| text == "題名"));
        assert!(shown.iter().any(|text| text == "着手中"));
        keys(&mut screen, &[KeyIntent::Down, KeyIntent::Down]);
        assert_eq!(screen.language(), Language::SimplifiedChinese);
        assert_eq!(spy.saved.borrow().len(), 3);
    }

    #[test]
    fn system_language_follows_the_environment() {
        let journal = FakeJournal::default();
        let environment = Environment::new(None).with_language(Some(Language::Korean));
        let (mut screen, _) = screen_in(journal, environment, false);
        assert_eq!(screen.language(), Language::Korean);
        assert_eq!(screen.notices(), Vec::new());
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().any(|text| text == "추가"));
    }

    #[test]
    fn the_theme_field_takes_a_name_cycles_with_arrows_and_refuses_unknown_names() {
        let (mut screen, spy) = screen(1);
        assert_eq!(screen.theme().name(), "neutral-light");
        click(&mut screen, THEME_X, TOOLBAR_Y);
        keys(&mut screen, &[KeyIntent::SelectAll]);
        keys(&mut screen, &chars("dracula"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(screen.theme().name(), "dracula");
        assert_eq!(
            spy.saved.borrow().last().map(|s| s.theme().keyword()),
            Some("dracula")
        );
        keys(&mut screen, &[KeyIntent::Down]);
        assert_eq!(screen.theme().name(), "one-dark");
        keys(&mut screen, &[KeyIntent::Up, KeyIntent::Up]);
        assert_eq!(screen.theme().name(), "monokai");
    }

    #[test]
    fn an_unknown_theme_name_is_shown_and_system_goes_back_to_the_default() {
        let (mut screen, _) = screen(1);
        click(&mut screen, THEME_X, TOOLBAR_Y);
        keys(&mut screen, &[KeyIntent::SelectAll]);
        keys(&mut screen, &chars("no-such"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(screen.theme().name(), "neutral-light");
        assert_eq!(screen.notices(), alloc::vec![Notice::UnknownTheme]);
        keys(&mut screen, &[KeyIntent::SelectAll]);
        keys(&mut screen, &chars("system"));
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(screen.theme().name(), "neutral-light");
        assert!(screen.notices().is_empty());
        let frame = idle(&mut screen);
        assert_eq!(frame.theme().name(), "neutral-light");
        assert_eq!(frame.language(), Language::English);
    }

    #[test]
    fn a_dark_system_follows_the_environment_and_an_unwritable_settings_file_is_shown() {
        let journal = FakeJournal::default();
        let environment = Environment::new(None).with_appearance(Some(Appearance::Dark));
        let (mut screen, _) = screen_in(journal, environment, true);
        assert_eq!(screen.theme().name(), "ubuntu-aubergine");
        click(&mut screen, THEME_X, TOOLBAR_Y);
        keys(&mut screen, &[KeyIntent::Down]);
        assert_eq!(
            screen.notices(),
            alloc::vec![Notice::Settings(SettingsFailure::Unwritable)]
        );
    }

    #[test]
    fn dragging_the_scroll_bar_moves_the_offset() {
        let (mut screen, _) = screen(200);
        screen.step(&pointer(TRACK_X, 200, ButtonPhase::Pressed));
        screen.step(&pointer(TRACK_X, 300, ButtonPhase::Held));
        assert!(screen.scroll().offset() > 0);
        assert!(screen.scroll().offset() <= screen.scroll().max_offset());
    }

    /// 本文の段を出し、1 行目を選んで段を押す。
    fn screen_with_body() -> (DocketScreen, Spy) {
        let (mut screen, spy) = screen(3);
        click(&mut screen, BODY_X, TOOLBAR_Y);
        assert!(screen.shows_body());
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, 100, 380);
        (screen, spy)
    }

    #[test]
    fn the_body_pane_takes_lines_and_commits_when_focus_leaves() {
        let (mut screen, spy) = screen_with_body();
        keys(&mut screen, &chars("first"));
        keys(&mut screen, &[KeyIntent::Enter]);
        keys(&mut screen, &chars("second"));
        assert_eq!(screen.body_text(), "first\nsecond");
        assert_eq!(spy.written.borrow().len(), 0);
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().any(|text| text == "second"));
        keys(&mut screen, &[KeyIntent::Escape]);
        let written = spy.written.borrow();
        assert_eq!(written.len(), 1);
        assert_eq!(
            written[0].kind(),
            &OpKind::Set {
                item: ItemId::new(OTHER, 1),
                value: FieldValue::Body(String::from("first\nsecond")),
            }
        );
        drop(written);
        let item = screen
            .ledger()
            .docket()
            .item(ItemId::new(OTHER, 1))
            .unwrap();
        assert_eq!(item.body(), "first\nsecond");
        // 検索は本文にも効く。
        click(&mut screen, 50, TOOLBAR_Y);
        keys(&mut screen, &chars("second"));
        assert_eq!(screen.visible_items().len(), 1);
        // 履歴は改行を印にして 1 行に。
        click(&mut screen, HISTORY_X, TOOLBAR_Y);
        let lines = screen.history_lines();
        assert_eq!(
            lines
                .first()
                .map(|line| line.columns(Language::English)[2].clone()),
            Some(String::from("Body :  -> first↵second"))
        );
    }

    #[test]
    fn the_body_pane_follows_the_selection_and_hides_by_key() {
        let (mut screen, _) = screen_with_body();
        keys(&mut screen, &chars("note"));
        click(&mut screen, 100, SECOND_ROW_Y);
        assert_eq!(screen.body_text(), "");
        assert_eq!(
            screen
                .ledger()
                .docket()
                .item(ItemId::new(OTHER, 1))
                .map(Item::body),
            Some("note")
        );
        // ホイールは段の上では表を動かさない。
        screen.step(&FrameInput::new(
            BOUNDS,
            PointerInput::new(Some(PixelPoint::new(100, 380)), ButtonPhase::Idle, 1),
        ));
        assert_eq!(screen.scroll().offset(), 0);
        keys(&mut screen, &[KeyIntent::Escape]);
        keys(&mut screen, &[KeyIntent::Tab; 5]);
        keys(&mut screen, &[KeyIntent::Char(' ')]);
        assert!(!screen.shows_body());
        // 本文と履歴を両方出しても表は残る。
        click(&mut screen, BODY_X, TOOLBAR_Y);
        click(&mut screen, HISTORY_X, TOOLBAR_Y);
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().any(|text| text == "When"));
        assert!(texts(&frame).iter().any(|text| text == "Issue #1"));
    }

    /// 参加者の段（1060×400・残り 80..391 の 2/5 = 124）: 段は 268..391、ヘッダ 268..292、行は 292 から 24 ずつ。
    const MEMBERS_HEADER_Y: u16 = 280;
    const FIRST_MEMBER_Y: u16 = 304;
    const SECOND_MEMBER_Y: u16 = 328;

    #[test]
    fn the_members_pane_lists_accounts_and_the_owner_changes_permissions() {
        let (mut screen, _) = screen(2);
        click(&mut screen, MEMBERS_X, TOOLBAR_Y);
        assert!(screen.shows_members());
        let members = screen.members();
        assert_eq!(members.len(), 2);
        assert!(members.iter().all(|m| m.role() == MemberRole::Anyone));
        let frame = idle(&mut screen);
        assert!(texts(&frame).iter().any(|text| text == "Anyone may edit"));
        assert!(texts(&frame).iter().any(|text| text == "Permissions"));
        keys(&mut screen, &[KeyIntent::Escape]);
        keys(&mut screen, &[KeyIntent::Tab; 6]);
        keys(&mut screen, &[KeyIntent::Char(' ')]);
        assert!(!screen.shows_members());
    }

    #[test]
    fn the_owner_turns_permissions_on_and_toggles_editors_from_the_pane() {
        let (mut screen, _) = screen(2);
        click(&mut screen, MEMBERS_X, TOOLBAR_Y);
        // 「権限」のチェック（段のヘッダの右端）を押す。
        click(&mut screen, 1000, MEMBERS_HEADER_Y);
        let roles: Vec<MemberRole> = screen.members().iter().map(Member::role).collect();
        assert_eq!(roles, [MemberRole::Owner, MemberRole::Viewer]);
        // 他人の行を押す → 編集者。もう一度 → 閲覧のみ。
        click(&mut screen, 100, SECOND_MEMBER_Y);
        assert_eq!(screen.members()[1].role(), MemberRole::Editor);
        click(&mut screen, 100, SECOND_MEMBER_Y);
        assert_eq!(screen.members()[1].role(), MemberRole::Viewer);
        click(&mut screen, 100, FIRST_MEMBER_Y);
        assert_eq!(screen.members()[0].role(), MemberRole::Owner);
        assert!(screen.notices().is_empty());
        click(&mut screen, 1000, MEMBERS_HEADER_Y);
        assert!(
            screen
                .members()
                .iter()
                .all(|m| m.role() == MemberRole::Anyone)
        );
    }

    #[test]
    fn an_unwritable_permissions_file_is_shown() {
        let (mut screen, _) = screen_with(FakeJournal {
            fail_write: true,
            ..FakeJournal::default()
        });
        click(&mut screen, MEMBERS_X, TOOLBAR_Y);
        click(&mut screen, 1000, MEMBERS_HEADER_Y);
        assert_eq!(
            screen.notices(),
            alloc::vec![Notice::Journal(JournalFailure::PermissionsUnwritable)]
        );
    }

    #[test]
    fn dragging_a_header_grip_resizes_the_column_and_saves_the_settings() {
        let (mut screen, spy) = screen(3);
        // 番号の列は 8..79 で、境は x=79 の辺り（ヘッダ y = 80..108）。
        let header_y = 94;
        screen.step(&pointer(79, header_y, ButtonPhase::Pressed));
        screen.step(&pointer(120, header_y, ButtonPhase::Held));
        assert_eq!(
            screen.settings().columns().get(TableColumn::Number),
            Some(113)
        );
        assert!(spy.saved.borrow().is_empty());
        screen.step(&pointer(120, header_y, ButtonPhase::Released));
        idle(&mut screen);
        assert_eq!(spy.saved.borrow().len(), 1);
        assert_eq!(
            spy.saved.borrow()[0].columns().get(TableColumn::Number),
            Some(113)
        );
        // 題名の右の境は右の列（状態）の幅を変える。列の間に隙間は無く、右の固定の列は 1039 から 120+90+90+110 なので題名の右端は 629。
        screen.step(&pointer(629, header_y, ButtonPhase::Pressed));
        assert_eq!(
            screen.settings().columns().get(TableColumn::State),
            Some(110)
        );
        screen.step(&pointer(599, header_y, ButtonPhase::Held));
        assert_eq!(
            screen.settings().columns().get(TableColumn::State),
            Some(140)
        );
        // 幅は 40 未満にならない。
        screen.step(&pointer(1040, header_y, ButtonPhase::Held));
        assert_eq!(
            screen.settings().columns().get(TableColumn::State),
            Some(40)
        );
        screen.step(&pointer(1040, header_y, ButtonPhase::Released));
        idle(&mut screen);
        assert_eq!(spy.saved.borrow().len(), 2);
    }

    #[test]
    fn preedit_shows_in_the_focused_field() {
        let (mut screen, _) = screen(3);
        click(&mut screen, 100, TOOLBAR_Y);
        let frame = screen.step(
            &FrameInput::new(BOUNDS, PointerInput::absent())
                .with_preedit(Some(nene_docket_core::preedit::Preedit::new("か", 1))),
        );
        assert!(frame.commands().iter().any(
            |command| matches!(command, DrawCommand::Text { text, caret: Some(_), .. } if text == "か")
        ));
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, TITLE_X, EDITOR_Y);
        let in_title = screen.step(
            &FrameInput::new(BOUNDS, PointerInput::absent())
                .with_preedit(Some(nene_docket_core::preedit::Preedit::new("き", 1))),
        );
        assert!(texts(&in_title).iter().any(|text| text.contains('き')));
    }
}
