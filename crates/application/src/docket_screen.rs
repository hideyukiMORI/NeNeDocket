//! 画面の状態の所有者（ARC-004）。検索欄・絞り込み・完了の表示・表の選択とスクロール・選んだ課題の編集・Undo / Redo。
//!
//! 即時モード（ADR 0006）。`step` が 1 フレームの入力を受け、鍵を前のフレームのフォーカスに配り、
//! 部品を問い合わせ、描画命令を返す。業務の判断（どの行を出すか・何が選ばれたか・どの操作を作るか）は
//! すべてここにあり、UI には無い（ARC-011）。課題の正本は [`Ledger`] が持つジャーナルの併合で、
//! 自分の操作はポート（[`Ports`]）へ追記して同期する（ADR 0004）。

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::button;
use nene_docket_core::checkbox;
use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::field_value::FieldValue;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::item::Item;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::item_state::ItemState;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::layout::Layout;
use nene_docket_core::op::Op;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::paint::Paint;
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
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;

use crate::account::Account;
use crate::docket_frame::DocketFrame;
use crate::environment::Environment;
use crate::frame_input::FrameInput;
use crate::journal_failure::JournalFailure;
use crate::ledger::Ledger;
use crate::notice::Notice;
use crate::ports::Ports;

/// 部品の ID。候補は `*_OPTION` に番号を足し、表の行は `ROW` に見えている順の番号を足す。
const SEARCH: WidgetId = WidgetId::new(1);
const FILTER: WidgetId = WidgetId::new(2);
const SHOW_DONE: WidgetId = WidgetId::new(3);
const ADD: WidgetId = WidgetId::new(4);
const SCROLL: WidgetId = WidgetId::new(5);
const TITLE: WidgetId = WidgetId::new(6);
const STATE: WidgetId = WidgetId::new(7);
const DELETE: WidgetId = WidgetId::new(8);
const FILTER_OPTION: WidgetId = WidgetId::new(100);
const STATE_OPTION: WidgetId = WidgetId::new(200);
const ROW: WidgetId = WidgetId::new(1_000);

/// 1x の寸法（SPECIFICATION 第 2 節・決め打ちしない）。
const MARGIN_PX: u16 = 8;
const ROW_PX: u16 = 28;
const FILTER_WIDTH_PX: u16 = 140;
const SHOW_DONE_WIDTH_PX: u16 = 120;
const BUTTON_WIDTH_PX: u16 = 80;
const SCROLL_WIDTH_PX: u16 = 12;
const NUMBER_WIDTH_PX: u16 = 72;
const STATE_WIDTH_PX: u16 = 110;
const ASSIGNEE_WIDTH_PX: u16 = 90;
/// ホイール 1 刻みで送る行数。
const WHEEL_ROWS: i32 = 3;
/// 番号の列に出す作成者の ID の桁数。
const CREATOR_DIGITS: usize = 4;

/// 絞り込みの候補。先頭は「すべて」、以降は [`ItemState::ALL`] の順。
const FILTER_OPTIONS: [&str; 4] = ["All", "Open", "In progress", "Done"];
/// 状態の候補。[`ItemState::ALL`] の順。
const STATE_OPTIONS: [&str; 3] = ["Open", "In progress", "Done"];

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
    state: SelectState,
    ledger: Ledger,
    ports: Ports,
    write_failure: Option<JournalFailure>,
    read_failure: Option<JournalFailure>,
}

impl DocketScreen {
    /// 環境・アカウント・ポートから始める。ジャーナルを最初に読み、台帳を併合する。
    #[must_use]
    pub fn open(environment: Environment, account: Account, mut ports: Ports) -> Self {
        let (ops, read_failure) = match ports.journal_mut().refresh() {
            Ok(ops) => (ops, None),
            Err(failure) => (Vec::new(), Some(failure)),
        };
        Self {
            scale: environment.scale(),
            interaction: Interaction::idle(),
            search: TextEdit::empty(),
            filter: SelectState::closed(FILTER_OPTIONS.len(), 0),
            show_done: true,
            scroll: ScrollState::top(0, 0),
            selected: None,
            title: TextEdit::empty(),
            state: SelectState::closed(STATE_OPTIONS.len(), 0),
            ledger: Ledger::open(account, ops),
            ports,
            write_failure: None,
            read_failure,
        }
    }

    /// 1 フレーム。読み直し → 鍵 → 部品の問い合わせ → 描画命令 → 題名の確定。
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
        let (filter_rect, filter_response) = self.probe_toolbar(&toolbar, &mut commands);
        let (state_rect, state_response) = self.probe_editor(&editor, &mut commands);
        self.probe_table(&table, &visible, &mut commands);
        self.probe_scroll(track, &mut commands);
        self.probe_filter_options(filter_rect);
        self.probe_state_options(state_rect);
        commands.extend(select_box::select_box(
            filter_rect,
            &FILTER_OPTIONS,
            self.filter,
            filter_response,
        ));
        commands.extend(select_box::select_box(
            state_rect,
            &STATE_OPTIONS,
            self.state,
            state_response,
        ));
        self.notice_commands(notice_rect, &mut commands);
        self.interaction.end();
        if self.interaction.focus() != Some(TITLE) {
            self.commit_title();
        }
        DocketFrame::new(self.scale, commands)
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

    /// 利用者に示す降格（FR-014）。無ければ空。
    #[must_use]
    pub fn notices(&self) -> Vec<Notice> {
        let mut notices = Vec::new();
        notices.extend(self.write_failure.map(Notice::Journal));
        notices.extend(self.read_failure.map(Notice::Journal));
        let orphans = self.ledger.docket().orphans();
        if orphans > 0 {
            notices.push(Notice::Orphans(orphans));
        }
        let rejected = self.ports.journal().rejected_lines();
        if rejected > 0 {
            notices.push(Notice::RejectedLines(rejected));
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
            .filter(|item| {
                self.search.text().is_empty() || item.title().contains(self.search.text())
            })
            .map(Item::id)
            .collect()
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

    fn toolbar_widths(&self) -> [ColumnWidth; 4] {
        [
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(FILTER_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(SHOW_DONE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
        ]
    }

    fn editor_widths(&self) -> [ColumnWidth; 3] {
        [
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(STATE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(BUTTON_WIDTH_PX)),
        ]
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
        let width = remaining.width().saturating_sub(self.px(SCROLL_WIDTH_PX));
        let bounds = PixelRect::new(remaining.x(), remaining.y(), width, remaining.height());
        let rows = u32::try_from(visible).unwrap_or(u32::MAX);
        TableLayout::new(bounds, self.px(ROW_PX), self.px(ROW_PX), rows).with_columns(alloc::vec![
            ColumnWidth::Fixed(self.px(NUMBER_WIDTH_PX)),
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(STATE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(ASSIGNEE_WIDTH_PX)),
        ])
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
        match self.interaction.focus() {
            Some(SEARCH) => self.search = self.search.with_preedit(input.preedit().cloned()),
            Some(TITLE) => self.title = self.title.with_preedit(input.preedit().cloned()),
            Some(_) | None => {}
        }
    }

    /// 鍵を、前のフレームのフォーカスの持ち主へ。Tab・Esc・Undo・Redo は持ち主に依らない。
    fn key(&mut self, key: KeyIntent) {
        match key {
            KeyIntent::Tab => self.interaction.focus_next(),
            KeyIntent::BackTab => self.interaction.focus_previous(),
            KeyIntent::Escape if !self.filter.is_open() && !self.state.is_open() => {
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
        match self.interaction.focus() {
            Some(SEARCH) => self.search = self.search.applied(key),
            Some(FILTER) => self.filter = self.filter.applied(key),
            Some(TITLE) if key == KeyIntent::Enter => self.commit_title(),
            Some(TITLE) => self.title = self.title.applied(key),
            Some(STATE) => {
                let before = self.state.chosen();
                self.state = self.state.applied(key);
                self.state_changed(before);
            }
            Some(SHOW_DONE) if matches!(key, KeyIntent::Enter | KeyIntent::Char(' ')) => {
                self.show_done = !self.show_done;
            }
            Some(ADD) if key == KeyIntent::Enter => self.add_item(),
            Some(DELETE) if key == KeyIntent::Enter => self.delete_item(),
            Some(_) | None => self.key_to_table(key),
        }
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

    fn undo(&mut self) {
        let now = self.ports.clock().now();
        if let Some(op) = self.ledger.compose_undo(now) {
            self.write(&op);
            self.reload_editor();
        }
    }

    fn redo(&mut self) {
        let now = self.ports.clock().now();
        if let Some(op) = self.ledger.compose_redo(now) {
            self.write(&op);
            self.reload_editor();
        }
    }

    /// 他人のジャーナルの伸びた分を取り込み、編集欄を合わせる（刻みの合図）。
    fn refresh(&mut self) {
        self.absorb_others();
        self.reload_editor();
    }

    /// 他人のジャーナルの伸びた分を取り込む。編集欄は触らない。
    fn absorb_others(&mut self) {
        match self.ports.journal_mut().refresh() {
            Ok(fresh) => {
                self.read_failure = None;
                self.ledger.absorb(fresh);
            }
            Err(failure) => self.read_failure = Some(failure),
        }
    }

    /// 課題を作る。検索欄に字があればそれを題名にし、題名の欄にフォーカスを置く。
    fn add_item(&mut self) {
        self.commit_title();
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
            self.reload_editor();
        }
    }

    /// 行を選ぶ。編集中の題名はその前に確定する。
    fn select(&mut self, id: ItemId) {
        self.commit_title();
        self.selected = Some(id);
        let item = self.selected_item();
        self.title = TextEdit::showing(item.as_ref().map_or("", Item::title));
        self.state = SelectState::closed(
            STATE_OPTIONS.len(),
            item.map_or(0, |found| Self::state_index(found.state())),
        );
    }

    /// 台帳が変わった後、編集欄をいまの値に合わせる。入力中の欄は触らない。
    fn reload_editor(&mut self) {
        let item = self.selected_item();
        if self.interaction.focus() != Some(TITLE) {
            self.title = TextEdit::showing(item.as_ref().map_or("", Item::title));
        }
        if !self.state.is_open() {
            self.state = SelectState::closed(
                STATE_OPTIONS.len(),
                item.map_or(0, |found| Self::state_index(found.state())),
            );
        }
    }

    /// 題名の欄が課題と違えば `Set` を作る。
    fn commit_title(&mut self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        if item.title() != self.title.text() {
            self.compose(OpKind::Set {
                item: item.id(),
                value: FieldValue::Title(self.title.text().to_owned()),
            });
        }
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

    /// 検索欄・絞り込み・完了の表示・追加。絞り込みの箱は候補が表の上に乗るので、最後に描く。
    fn probe_toolbar(
        &mut self,
        toolbar: &[PixelRect],
        commands: &mut Vec<DrawCommand>,
    ) -> (PixelRect, Response) {
        let [search_rect, filter_rect, done_rect, add_rect] = Self::four(toolbar);
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
            "Show done",
            self.show_done,
            done,
        ));
        let add = self.interaction.probe(ADD, add_rect, true);
        if add.clicked() {
            self.add_item();
        }
        commands.extend(button::button(add_rect, "Add", add));
        (filter_rect, filter)
    }

    /// 選んだ課題の題名・状態・削除。状態の箱は最後に描く。
    fn probe_editor(
        &mut self,
        editor: &[PixelRect],
        commands: &mut Vec<DrawCommand>,
    ) -> (PixelRect, Response) {
        let [title_rect, state_rect, delete_rect, _] = Self::four(editor);
        let title = self.interaction.probe(TITLE, title_rect, true);
        commands.extend(text_field::text_field(title_rect, &self.title, title));
        let state = self.interaction.probe(STATE, state_rect, true);
        if state.clicked() {
            self.state = self.state.toggled();
        }
        let delete = self.interaction.probe(DELETE, delete_rect, true);
        if delete.clicked() {
            self.delete_item();
        }
        commands.extend(button::button(delete_rect, "Delete", delete));
        (state_rect, state)
    }

    /// 4 列の段。足りなければ空の矩形で埋める（起こらないが型の上では `Vec`）。
    fn four(rects: &[PixelRect]) -> [PixelRect; 4] {
        let empty = PixelRect::new(0, 0, 0, 0);
        let at = |index: usize| rects.get(index).copied().unwrap_or(empty);
        [at(0), at(1), at(2), at(3)]
    }

    /// 開いた絞り込みの候補。表より後に問い合わせるので、重なった所では候補が勝つ。候補の列は `select_box` が描く。
    fn probe_filter_options(&mut self, filter_rect: PixelRect) {
        if !self.filter.is_open() {
            return;
        }
        let rects = select_box::option_rects(filter_rect, FILTER_OPTIONS.len());
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
        let rects = select_box::option_rects(state_rect, STATE_OPTIONS.len());
        for (index, rect) in rects.into_iter().enumerate() {
            let id = STATE_OPTION.indexed(u32::try_from(index).unwrap_or(u32::MAX));
            if self.interaction.probe(id, rect, false).clicked() {
                self.state = self.state.chosen_at(index);
                self.interaction.focus_on(STATE);
                self.state_changed(before);
            }
        }
    }

    /// 候補の列が開いているあいだ、表は押せない。
    fn any_options_open(&self) -> bool {
        self.filter.is_open() || self.state.is_open()
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
        for (cell, heading) in table
            .cells(header)
            .into_iter()
            .zip(["#", "Title", "State", "Assignee"])
        {
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
        let texts = [
            Self::number_text(item.id()),
            item.title().to_owned(),
            item.state().label().to_owned(),
            item.assignee().to_owned(),
        ];
        for (cell, text) in cells.into_iter().zip(texts) {
            commands.push(Self::cell_text(cell, &text, Paint::Ink, self.px(MARGIN_PX)));
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
        let words: Vec<String> = self.notices().into_iter().map(Notice::describe).collect();
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

    /// スクロールバー。掴んで引けば位置が追い、ホイールは 3 行ずつ。
    fn probe_scroll(&mut self, track: PixelRect, commands: &mut Vec<DrawCommand>) {
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
        let wheel = i32::from(pointer.wheel())
            .saturating_mul(WHEEL_ROWS)
            .saturating_mul(i32::from(self.px(ROW_PX)));
        self.scroll = self.scroll.scrolled_by(wheel);
        commands.extend(scroll_bar::scroll_bar(track, self.scroll, response));
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::rc::Rc;
    use core::cell::RefCell;

    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::button_phase::ButtonPhase;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::pixel_point::PixelPoint;
    use nene_docket_core::pointer_input::PointerInput;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;
    use crate::clock_port::ClockPort;
    use crate::journal_port::JournalPort;

    const BOUNDS: PixelRect = PixelRect::new(0, 0, 640, 400);
    const ME: AccountId = AccountId::new(1);
    const OTHER: AccountId = AccountId::new(2);
    /// 1 行目の中央。段は 8..36（道具）・44..72（編集）・80..108（ヘッダ）・108 から行。
    const FIRST_ROW_Y: u16 = 120;
    const SECOND_ROW_Y: u16 = 150;
    const TOOLBAR_Y: u16 = 20;
    const EDITOR_Y: u16 = 58;

    /// 書き込みを覚え、読み直しで渡す分を積める偽のジャーナル。
    #[derive(Debug, Default)]
    struct FakeJournal {
        written: Rc<RefCell<Vec<Op>>>,
        pending: Rc<RefCell<Vec<Op>>>,
        synced: Rc<RefCell<usize>>,
        fail_write: bool,
        fail_read: bool,
        rejected: usize,
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
    }

    #[derive(Debug)]
    struct FixedClock;

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
    }

    fn screen_with(journal: FakeJournal) -> (DocketScreen, Spy) {
        let spy = Spy {
            written: Rc::clone(&journal.written),
            pending: Rc::clone(&journal.pending),
            synced: Rc::clone(&journal.synced),
        };
        let ports = Ports::new(Box::new(journal), Box::new(FixedClock));
        let account = Account::new(ME, String::from("me"));
        (
            DocketScreen::open(Environment::new(None), account, ports),
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
        click(&mut screen, 400, TOOLBAR_Y);
        assert!(screen.filter().is_open());
        click(&mut screen, 400, 130);
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
        click(&mut screen, 500, TOOLBAR_Y);
        assert!(!screen.shows_done());
        assert_eq!(screen.visible_items().len(), 4);
        keys(&mut screen, &[KeyIntent::Char(' ')]);
        assert!(screen.shows_done());
    }

    #[test]
    fn add_creates_an_item_writes_it_and_focuses_the_title() {
        let (mut screen, spy) = screen(2);
        click(&mut screen, 600, TOOLBAR_Y);
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
        keys(
            &mut screen,
            &[
                KeyIntent::Tab,
                KeyIntent::Tab,
                KeyIntent::Tab,
                KeyIntent::Enter,
            ],
        );
        assert_eq!(title_of(&screen, ItemId::new(ME, 2)), "Issue");
    }

    #[test]
    fn editing_the_title_commits_when_focus_leaves() {
        let (mut screen, spy) = screen(3);
        click(&mut screen, 100, FIRST_ROW_Y);
        click(&mut screen, 100, EDITOR_Y);
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
        click(&mut screen, 500, EDITOR_Y);
        assert!(!screen.filter().is_open());
        click(&mut screen, 500, 142);
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
        click(&mut screen, 600, EDITOR_Y);
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
        keys(&mut screen, &[KeyIntent::Tab; 7]);
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(spy.written.borrow().len(), 0);
        click(&mut screen, 100, FIRST_ROW_Y);
        keys(&mut screen, &[KeyIntent::Tab; 7]);
        keys(&mut screen, &[KeyIntent::Enter]);
        assert_eq!(spy.written.borrow().len(), 1);
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
        click(&mut screen, 100, EDITOR_Y);
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
        click(&mut screen, 600, TOOLBAR_Y);
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

    #[test]
    fn dragging_the_scroll_bar_moves_the_offset() {
        let (mut screen, _) = screen(200);
        screen.step(&pointer(630, 200, ButtonPhase::Pressed));
        screen.step(&pointer(630, 300, ButtonPhase::Held));
        assert!(screen.scroll().offset() > 0);
        assert!(screen.scroll().offset() <= screen.scroll().max_offset());
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
        click(&mut screen, 100, EDITOR_Y);
        let in_title = screen.step(
            &FrameInput::new(BOUNDS, PointerInput::absent())
                .with_preedit(Some(nene_docket_core::preedit::Preedit::new("き", 1))),
        );
        assert!(texts(&in_title).iter().any(|text| text.contains('き')));
    }
}
