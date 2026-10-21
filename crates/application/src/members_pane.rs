//! 参加者の段（FR-009）。表の下に、台帳に参加したアカウントの名前・ID・役（グループがあれば `役 (qa, dev)`）。
//! 持ち主は「権限」で有効・無効を切り替え、行を押して選ぶと段の上に編集行（「編集者」のチェックとグループの欄・`,` 区切り・
//! 「引き継ぎ先」の箱・ADR 0013）が出る（FR-010）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::checkbox;
use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::key_intent::KeyIntent;
use nene_docket_core::language::Language;
use nene_docket_core::layout::Layout;
use nene_docket_core::paint::Paint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::preedit::Preedit;
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
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::member::Member;
use crate::member_role::MemberRole;
use crate::members_action::MembersAction;

/// 「権限」のチェックの ID。画面の他の部品（1〜18・20・100〜・200〜・300〜・1000〜・4000〜）と重ならない。
const PERMISSIONS_TOGGLE: WidgetId = WidgetId::new(19);
/// 参加者のスクロールバーの ID。
const MEMBERS_SCROLL: WidgetId = WidgetId::new(21);
/// 編集行の「編集者」のチェックの ID。
const MEMBER_EDITOR: WidgetId = WidgetId::new(22);
/// 編集行のグループの欄の ID。
const MEMBER_GROUPS: WidgetId = WidgetId::new(23);
/// 編集行の「引き継ぎ先」の箱の ID。
const MEMBER_SUCCESSOR: WidgetId = WidgetId::new(24);
/// 「引き継ぎ先」の候補の ID。番号を足す。
const MEMBER_SUCCESSOR_OPTION: WidgetId = WidgetId::new(7_000);
/// 参加者の行の ID。見えている順の番号を足す。
const MEMBER_ROW: WidgetId = WidgetId::new(3_000);

/// 1x の寸法。
const ROW_PX: u16 = 24;
const ID_WIDTH_PX: u16 = 56;
const ROLE_WIDTH_PX: u16 = 150;
const PERMISSIONS_WIDTH_PX: u16 = 120;
const PADDING_PX: u16 = 8;
const SCROLL_WIDTH_PX: u16 = 12;
const EDITOR_WIDTH_PX: u16 = 120;
const SUCCESSOR_WIDTH_PX: u16 = 200;
const GAP_PX: u16 = 4;
/// グループの区切り。名前に `:` は使えない（`permissions.v1.txt` の形）。
const GROUP_SEPARATOR: char = ',';
const FORBIDDEN_IN_GROUP: char = ':';
/// ホイール 1 刻みで送る行数。
const WHEEL_ROWS: i32 = 3;

/// 出すかどうかとスクロールと倍率と、選んだ行とそのグループの欄。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembersPane {
    shown: bool,
    scroll: ScrollState,
    scale: UiScale,
    selected: Option<AccountId>,
    groups: TextEdit,
    /// グループの欄を読み込んだ時点の文字列。違えば利用者が変えた。
    groups_base: String,
    bad_groups: bool,
    /// 「引き継ぎ先」の箱。0 は引き継がない、1 からは `candidates` の順。
    successor: SelectState,
    /// 引き継ぎ先の候補（選んだ人の他の参加者）と、その表示。
    candidates: Vec<(AccountId, String)>,
}

impl MembersPane {
    /// 隠れている。
    #[must_use]
    pub fn hidden(scale: UiScale) -> Self {
        Self {
            shown: false,
            scroll: ScrollState::top(0, 0),
            scale,
            selected: None,
            groups: TextEdit::empty(),
            groups_base: String::new(),
            bad_groups: false,
            successor: SelectState::closed(1, 0),
            candidates: Vec::new(),
        }
    }

    /// スクロールの位置。
    #[must_use]
    pub const fn scroll(&self) -> ScrollState {
        self.scroll
    }

    /// 出す・隠すを切り替える。
    pub const fn toggle(&mut self) {
        self.shown = !self.shown;
    }

    /// 出しているか。
    #[must_use]
    pub const fn is_shown(&self) -> bool {
        self.shown
    }

    /// 選んだ行のアカウント。
    #[must_use]
    pub const fn selected(&self) -> Option<AccountId> {
        self.selected
    }

    /// グループの欄に `:` を含む名前がある（追記しない・FR-014 で示す）。
    #[must_use]
    pub const fn has_bad_groups(&self) -> bool {
        self.bad_groups
    }

    /// 「権限」のチェックの ID（鍵で押すとき）。
    #[must_use]
    pub const fn permissions_toggle() -> WidgetId {
        PERMISSIONS_TOGGLE
    }

    /// 編集行の「編集者」のチェックの ID。
    #[must_use]
    pub const fn editor_id() -> WidgetId {
        MEMBER_EDITOR
    }

    /// 編集行のグループの欄の ID。
    #[must_use]
    pub const fn groups_id() -> WidgetId {
        MEMBER_GROUPS
    }

    /// 編集行の「引き継ぎ先」の箱の ID。
    #[must_use]
    pub const fn successor_id() -> WidgetId {
        MEMBER_SUCCESSOR
    }

    /// 「引き継ぎ先」の候補の列が開いているか（開いているあいだ、表と行は押せない）。
    #[must_use]
    pub const fn options_open(&self) -> bool {
        self.successor.is_open()
    }

    /// 「引き継ぎ先」の箱の鍵。閉じた箱の上下では選ばない（引き継ぎは戻せないので、開いて Enter で決める）。
    pub fn successor_key(&mut self, key: KeyIntent) -> Option<MembersAction> {
        if !self.successor.is_open() && key != KeyIntent::Enter {
            return None;
        }
        self.successor = self.successor.applied(key);
        self.chosen_successor()
    }

    /// 箱が選んでいる候補への引き継ぎ。引き継がない（0）なら `None`。
    fn chosen_successor(&self) -> Option<MembersAction> {
        let old = self.selected?;
        let chosen = self.successor.chosen().checked_sub(1)?;
        let (new, _) = self.candidates.get(chosen)?;
        Some(MembersAction::Succeed(*new, old))
    }

    /// グループの欄の鍵。Enter で確定（選んだ行のグループをその集合にする求め）。
    pub fn groups_key(&mut self, key: KeyIntent) -> Option<MembersAction> {
        if key == KeyIntent::Enter {
            let action = self.groups_action();
            self.groups_base = String::from(self.groups.text());
            return action;
        }
        self.groups = self.groups.applied(key);
        None
    }

    /// 編集行の「編集者」を鍵で押した。
    #[must_use]
    pub fn editor_key(&self) -> Option<MembersAction> {
        self.selected.map(MembersAction::ToggleEditor)
    }

    /// 確定前の文字列をグループの欄へ。
    pub fn set_preedit(&mut self, preedit: Option<Preedit>) {
        self.groups = self.groups.with_preedit(preedit);
    }

    /// 欄の文字列を集合にした求め。`:` を含む名前があれば示して `None`。
    fn groups_action(&mut self) -> Option<MembersAction> {
        let selected = self.selected?;
        let names: Vec<String> = self
            .groups
            .text()
            .split(GROUP_SEPARATOR)
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(String::from)
            .collect();
        self.bad_groups = names.iter().any(|name| name.contains(FORBIDDEN_IN_GROUP));
        if self.bad_groups {
            return None;
        }
        Some(MembersAction::SetGroups(selected, names))
    }

    /// 段を問い合わせ、描画命令と求めを返す。`owner` なら行を押して選べ、ヘッダの右に「権限」のチェック（`enabled`）が出る。
    /// 持ち主でない人を選んでいれば、段の上に編集行（「編集者」のチェックとグループの欄）が出る。
    #[must_use]
    pub fn probe(
        &mut self,
        interaction: &mut Interaction,
        (area, members, language): (PixelRect, &[Member], Language),
        (owner, enabled): (bool, bool),
    ) -> (Vec<DrawCommand>, Vec<MembersAction>) {
        let mut commands = Vec::new();
        let mut actions = Vec::new();
        let editing = self.editing(members, owner && enabled);
        let mut layout = Layout::inside(area, self.px(GAP_PX));
        let mut successor_box = None;
        if let Some(member) = editing {
            let row = layout.row(
                self.px(ROW_PX),
                &[
                    ColumnWidth::Fixed(self.px(EDITOR_WIDTH_PX)),
                    ColumnWidth::Fill,
                    ColumnWidth::Fixed(self.px(SUCCESSOR_WIDTH_PX)),
                ],
            );
            self.load_candidates(members, &member);
            let (acted, response) =
                self.edit_row(interaction, (&row, &member, language), &mut commands);
            actions.extend(acted);
            successor_box = row.get(2).copied().zip(response);
        }
        let table_area = layout.remaining();
        let track = PixelRect::new(
            table_area
                .right()
                .saturating_sub(self.px(SCROLL_WIDTH_PX))
                .saturating_add(1),
            table_area.y(),
            self.px(SCROLL_WIDTH_PX),
            table_area.height(),
        );
        let table = self.table(table_area, members.len());
        self.scroll = self
            .scroll
            .resized(table.content_height(), u32::from(table.body().height()));
        actions.extend(self.header(
            interaction,
            (&table, language, (owner, enabled)),
            &mut commands,
        ));
        actions.extend(self.rows(
            interaction,
            (&table, members, language, owner && enabled),
            &mut commands,
        ));
        self.scroll_bar(interaction, (track, table_area), &mut commands);
        if let Some((rect, response)) = successor_box {
            // 候補は行より後に問い合わせ（重なった所では候補が勝つ）、箱は最後に描く（候補が行の上に来る）。
            actions.extend(self.successor_options(interaction, rect, language));
            let labels = self.successor_labels(language);
            let options: Vec<&str> = labels.iter().map(String::as_str).collect();
            commands.extend(select_box::select_box(
                rect,
                &options,
                self.successor,
                response,
            ));
        }
        (commands, actions)
    }

    /// 選んだ人の他の参加者を候補にする。数が変われば箱を閉じて「引き継がない」に戻す。
    fn load_candidates(&mut self, members: &[Member], selected: &Member) {
        self.candidates = members
            .iter()
            .filter(|member| member.id() != selected.id())
            .map(|member| {
                (
                    member.id(),
                    alloc::format!("{} ({})", member.name(), member.short_id()),
                )
            })
            .collect();
        let count = self.candidates.len().saturating_add(1);
        if self.successor.count() != count {
            self.successor = SelectState::closed(count, 0);
        }
    }

    /// 箱の候補の文言。先頭は「引き継ぎ先なし」。
    fn successor_labels(&self, language: Language) -> Vec<String> {
        let mut labels = alloc::vec![String::from(wording::text(language, Word::NoSuccessor))];
        labels.extend(self.candidates.iter().map(|(_, label)| label.clone()));
        labels
    }

    /// 開いた「引き継ぎ先」の候補。押せば選んで閉じ、引き継ぐ。
    fn successor_options(
        &mut self,
        interaction: &mut Interaction,
        rect: PixelRect,
        language: Language,
    ) -> Option<MembersAction> {
        if !self.successor.is_open() {
            return None;
        }
        let count = self.successor_labels(language).len();
        let mut action = None;
        for (index, option) in select_box::option_rects(rect, count)
            .into_iter()
            .enumerate()
        {
            let id = MEMBER_SUCCESSOR_OPTION.indexed(u32::try_from(index).unwrap_or(u32::MAX));
            if interaction.probe(id, option, false).clicked() {
                self.successor = self.successor.chosen_at(index);
                interaction.focus_on(MEMBER_SUCCESSOR);
                action = self.chosen_successor();
            }
        }
        action
    }

    /// 見えている行。`administering` なら押して選べる。選んだ行は強調。
    fn rows(
        &mut self,
        interaction: &mut Interaction,
        (table, members, language, administering): (&TableLayout, &[Member], Language, bool),
        commands: &mut Vec<DrawCommand>,
    ) -> Vec<MembersAction> {
        let mut actions = Vec::new();
        commands.push(DrawCommand::Clip(table.body()));
        for (index, rect) in table.visible_rows(self.scroll.offset()) {
            let Some(member) = usize::try_from(index).ok().and_then(|i| members.get(i)) else {
                continue;
            };
            let response = if self.successor.is_open() {
                Response::new(ResponsePhase::Idle, false, false)
            } else {
                interaction.probe(MEMBER_ROW.indexed(index), rect, false)
            };
            if administering && response.clicked() {
                actions.extend(self.select(member));
            }
            if self.selected == Some(member.id()) {
                commands.push(DrawCommand::Fill {
                    rect,
                    paint: Paint::Selection,
                });
            } else if administering && response.phase() != ResponsePhase::Idle {
                commands.push(DrawCommand::Fill {
                    rect,
                    paint: Paint::Surface,
                });
            }
            let texts = [
                String::from(member.name()),
                member.short_id(),
                role_text(language, member),
            ];
            for (cell, text) in table.cells(rect).into_iter().zip(texts) {
                commands.push(Self::cell_text(
                    cell,
                    &text,
                    Paint::Ink,
                    self.px(PADDING_PX),
                ));
            }
        }
        commands.push(DrawCommand::Unclip);
        actions
    }

    /// 編集行を出す相手。持ち主が権限を有効にしていて、持ち主でない人を選んでいるとき。権限が無効か選んだ人が居なくなれば選びを外す。
    fn editing(&mut self, members: &[Member], administering: bool) -> Option<Member> {
        let found = self
            .selected
            .filter(|_| administering)
            .and_then(|id| members.iter().find(|member| member.id() == id))
            .cloned();
        if found.is_none() {
            self.selected = None;
        }
        found.filter(|member| member.role() != MemberRole::Owner)
    }

    /// 行を押した。グループの欄に書きかけがあれば先に確定し、その行を選んで欄を読み込む。
    fn select(&mut self, member: &Member) -> Vec<MembersAction> {
        let mut actions = Vec::new();
        if self.groups.text() != self.groups_base {
            actions.extend(self.groups_action());
        }
        self.selected = Some(member.id());
        self.load_groups(member);
        self.successor = SelectState::closed(1, 0);
        actions
    }

    fn load_groups(&mut self, member: &Member) {
        let text = member.groups().join(", ");
        self.groups = TextEdit::showing(&text);
        self.groups_base = text;
        self.bad_groups = false;
    }

    /// 編集行: 「編集者」のチェックとグループの欄と「引き継ぎ先」の箱（描くのは最後なので、問い合わせの結果を返す）。
    /// 欄はフォーカスを失ったときに確定し、違わなければいまの値に合わせる。
    fn edit_row(
        &mut self,
        interaction: &mut Interaction,
        (row, member, language): (&[PixelRect], &Member, Language),
        commands: &mut Vec<DrawCommand>,
    ) -> (Vec<MembersAction>, Option<Response>) {
        let mut actions = Vec::new();
        let (Some(editor_rect), Some(groups_rect), Some(successor_rect)) =
            (row.first(), row.get(1), row.get(2))
        else {
            return (actions, None);
        };
        let editor = interaction.probe(MEMBER_EDITOR, *editor_rect, true);
        commands.extend(checkbox::checkbox(
            *editor_rect,
            wording::text(language, Word::Editor),
            member.is_listed_editor(),
            editor,
        ));
        if editor.clicked() {
            actions.push(MembersAction::ToggleEditor(member.id()));
        }
        let groups = interaction.probe(MEMBER_GROUPS, *groups_rect, true);
        if !groups.focused() {
            if self.groups.text() == self.groups_base {
                self.load_groups(member);
            } else {
                actions.extend(self.groups_action());
                self.load_groups(member);
            }
        }
        commands.extend(text_field::text_field(*groups_rect, &self.groups, groups));
        let successor = interaction.probe(MEMBER_SUCCESSOR, *successor_rect, true);
        if successor.clicked() {
            self.successor = self.successor.toggled();
        }
        (actions, Some(successor))
    }

    /// つまみのドラッグとホイール（ポインタが段の上にあるときだけ）。
    fn scroll_bar(
        &mut self,
        interaction: &mut Interaction,
        (track, area): (PixelRect, PixelRect),
        commands: &mut Vec<DrawCommand>,
    ) {
        let response = interaction.probe(MEMBERS_SCROLL, track, false);
        let pointer = interaction.pointer();
        if let Some(point) = pointer
            .position()
            .filter(|_| response.phase() == ResponsePhase::Held)
        {
            let into = u64::from(point.y().saturating_sub(track.y()));
            let offset = into * u64::from(self.scroll.content()) / u64::from(track.height().max(1));
            self.scroll = self
                .scroll
                .scrolled_to(u32::try_from(offset).unwrap_or(u32::MAX));
        }
        if pointer.position().is_some_and(|point| area.contains(point)) {
            let wheel = i32::from(pointer.wheel())
                .saturating_mul(WHEEL_ROWS)
                .saturating_mul(i32::from(self.px(ROW_PX)));
            self.scroll = self.scroll.scrolled_by(wheel);
        }
        commands.extend(scroll_bar::scroll_bar(track, self.scroll, response));
    }

    const fn px(&self, base: u16) -> u16 {
        self.scale.px(base)
    }

    fn table(&self, area: PixelRect, count: usize) -> TableLayout {
        let bounds = PixelRect::new(
            area.x(),
            area.y(),
            area.width().saturating_sub(self.px(SCROLL_WIDTH_PX)),
            area.height(),
        );
        let rows = u32::try_from(count).unwrap_or(u32::MAX);
        TableLayout::new(bounds, self.px(ROW_PX), self.px(ROW_PX), rows).with_columns(alloc::vec![
            ColumnWidth::Fill,
            ColumnWidth::Fixed(self.px(ID_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(ROLE_WIDTH_PX)),
            ColumnWidth::Fixed(self.px(PERMISSIONS_WIDTH_PX)),
        ])
    }

    /// ヘッダ: 名前・ID・役と、持ち主には 4 列目に「権限」のチェック。
    fn header(
        &self,
        interaction: &mut Interaction,
        (table, language, owner_access): (&TableLayout, Language, (bool, bool)),
        commands: &mut Vec<DrawCommand>,
    ) -> Option<MembersAction> {
        let (owner, enabled) = owner_access;
        let header = table.header();
        commands.push(DrawCommand::Stroke {
            rect: PixelRect::new(header.x(), header.y(), header.width(), 1),
            paint: Paint::Border,
        });
        commands.push(DrawCommand::Fill {
            rect: header,
            paint: Paint::Surface,
        });
        let headings =
            [Word::Name, Word::Number, Word::Role].map(|word| wording::text(language, word));
        for (cell, heading) in table.cells(header).into_iter().zip(headings) {
            commands.push(Self::cell_text(
                cell,
                heading,
                Paint::Muted,
                self.px(PADDING_PX),
            ));
        }
        if !owner {
            return None;
        }
        let toggle = table.cells(header).get(3).copied()?;
        let response = interaction.probe(PERMISSIONS_TOGGLE, toggle, true);
        commands.extend(checkbox::checkbox(
            toggle,
            wording::text(language, Word::Permissions),
            enabled,
            response,
        ));
        response
            .clicked()
            .then_some(MembersAction::TogglePermissions)
    }

    fn cell_text(cell: PixelRect, text: &str, paint: Paint, padding: u16) -> DrawCommand {
        DrawCommand::Text {
            rect: PixelRect::new(
                cell.x().saturating_add(padding),
                cell.y(),
                cell.width().saturating_sub(padding.saturating_mul(2)),
                cell.height(),
            ),
            text: String::from(text),
            align: TextAlign::Start,
            paint,
            caret: None,
        }
    }
}

/// 役の文字列。グループに入っていれば `編集者 (qa, dev)`。
fn role_text(language: Language, member: &Member) -> String {
    let role = wording::text(language, member.role().word());
    if member.groups().is_empty() {
        String::from(role)
    } else {
        alloc::format!("{role} ({})", member.groups().join(", "))
    }
}
