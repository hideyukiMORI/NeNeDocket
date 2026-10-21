//! 参加者の段（FR-009）。表の下に、台帳に参加したアカウントの名前・ID・役（グループがあれば `役 (qa, dev)`）。持ち主は行を押して編集者を切り替え、「権限」で有効・無効を切り替える（FR-010）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::checkbox;
use nene_docket_core::column_width::ColumnWidth;
use nene_docket_core::draw_command::DrawCommand;
use nene_docket_core::interaction::Interaction;
use nene_docket_core::language::Language;
use nene_docket_core::paint::Paint;
use nene_docket_core::pixel_rect::PixelRect;
use nene_docket_core::response_phase::ResponsePhase;
use nene_docket_core::scroll_bar;
use nene_docket_core::scroll_state::ScrollState;
use nene_docket_core::table_layout::TableLayout;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::member::Member;
use crate::members_action::MembersAction;

/// 「権限」のチェックの ID。画面の他の部品（1〜18・20・100〜・200〜・300〜・1000〜・4000〜）と重ならない。
const PERMISSIONS_TOGGLE: WidgetId = WidgetId::new(19);
/// 参加者のスクロールバーの ID。
const MEMBERS_SCROLL: WidgetId = WidgetId::new(21);
/// 参加者の行の ID。見えている順の番号を足す。
const MEMBER_ROW: WidgetId = WidgetId::new(3_000);

/// 1x の寸法。
const ROW_PX: u16 = 24;
const ID_WIDTH_PX: u16 = 56;
const ROLE_WIDTH_PX: u16 = 150;
const PERMISSIONS_WIDTH_PX: u16 = 120;
const PADDING_PX: u16 = 8;
const SCROLL_WIDTH_PX: u16 = 12;
/// ホイール 1 刻みで送る行数。
const WHEEL_ROWS: i32 = 3;

/// 出すかどうかとスクロールと倍率。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MembersPane {
    shown: bool,
    scroll: ScrollState,
    scale: UiScale,
}

impl MembersPane {
    /// 隠れている。
    #[must_use]
    pub const fn hidden(scale: UiScale) -> Self {
        Self {
            shown: false,
            scroll: ScrollState::top(0, 0),
            scale,
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

    /// 段を問い合わせ、描画命令と求めを返す。`owner` なら行を押せ、ヘッダの右に「権限」のチェック（`enabled`）が出る。
    #[must_use]
    pub fn probe(
        &mut self,
        interaction: &mut Interaction,
        (area, members, language): (PixelRect, &[Member], Language),
        (owner, enabled): (bool, bool),
    ) -> (Vec<DrawCommand>, Option<MembersAction>) {
        let track = PixelRect::new(
            area.right()
                .saturating_sub(self.px(SCROLL_WIDTH_PX))
                .saturating_add(1),
            area.y(),
            self.px(SCROLL_WIDTH_PX),
            area.height(),
        );
        let table = self.table(area, members.len());
        self.scroll = self
            .scroll
            .resized(table.content_height(), u32::from(table.body().height()));
        let mut commands = Vec::new();
        let mut action = self.header(
            interaction,
            (&table, language, (owner, enabled)),
            &mut commands,
        );
        commands.push(DrawCommand::Clip(table.body()));
        for (index, rect) in table.visible_rows(self.scroll.offset()) {
            let Some(member) = usize::try_from(index).ok().and_then(|i| members.get(i)) else {
                continue;
            };
            let response = interaction.probe(MEMBER_ROW.indexed(index), rect, false);
            if owner && enabled && response.clicked() {
                action = Some(MembersAction::ToggleEditor(member.id()));
            }
            if owner && enabled && response.phase() != ResponsePhase::Idle {
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
        self.scroll_bar(interaction, (track, area), &mut commands);
        (commands, action)
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

    const fn px(self, base: u16) -> u16 {
        self.scale.px(base)
    }

    fn table(self, area: PixelRect, count: usize) -> TableLayout {
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
        self,
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
    /// 「権限」のチェックの ID（鍵で押すとき）。
    #[must_use]
    pub const fn permissions_toggle() -> WidgetId {
        PERMISSIONS_TOGGLE
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
