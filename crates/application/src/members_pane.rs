//! 参加者の段（FR-009）。表の下に、台帳に参加したアカウントの名前・ID・役。持ち主は行を押して編集者を切り替え、「権限」で有効・無効を切り替える（FR-010）。

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
use nene_docket_core::table_layout::TableLayout;
use nene_docket_core::text_align::TextAlign;
use nene_docket_core::ui_scale::UiScale;
use nene_docket_core::widget_id::WidgetId;
use nene_docket_core::word::Word;
use nene_docket_core::wording;

use crate::member::Member;
use crate::members_action::MembersAction;

/// 「権限」のチェックの ID。画面の他の部品（1〜18・100〜・200〜・300〜・1000〜）と重ならない。
const PERMISSIONS_TOGGLE: WidgetId = WidgetId::new(19);
/// 参加者の行の ID。見えている順の番号を足す。
const MEMBER_ROW: WidgetId = WidgetId::new(3_000);

/// 1x の寸法。
const ROW_PX: u16 = 24;
const ID_WIDTH_PX: u16 = 56;
const ROLE_WIDTH_PX: u16 = 150;
const PERMISSIONS_WIDTH_PX: u16 = 120;
const PADDING_PX: u16 = 8;

/// 出すかどうかと倍率。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MembersPane {
    shown: bool,
    scale: UiScale,
}

impl MembersPane {
    /// 隠れている。
    #[must_use]
    pub const fn hidden(scale: UiScale) -> Self {
        Self {
            shown: false,
            scale,
        }
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
        self,
        interaction: &mut Interaction,
        (area, members, language): (PixelRect, &[Member], Language),
        (owner, enabled): (bool, bool),
    ) -> (Vec<DrawCommand>, Option<MembersAction>) {
        let table = self.table(area, members.len());
        let mut commands = Vec::new();
        let mut action = self.header(
            interaction,
            (&table, language, (owner, enabled)),
            &mut commands,
        );
        commands.push(DrawCommand::Clip(table.body()));
        for (index, rect) in table.visible_rows(0) {
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
                String::from(wording::text(language, member.role().word())),
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
        (commands, action)
    }

    const fn px(self, base: u16) -> u16 {
        self.scale.px(base)
    }

    fn table(self, area: PixelRect, count: usize) -> TableLayout {
        let rows = u32::try_from(count).unwrap_or(u32::MAX);
        TableLayout::new(area, self.px(ROW_PX), self.px(ROW_PX), rows).with_columns(alloc::vec![
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
