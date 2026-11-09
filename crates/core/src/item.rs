//! 併合した課題 1 件（FR-001）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::attachment::Attachment;
use crate::date::Date;
use crate::field_value::FieldValue;
use crate::item_id::ItemId;
use crate::item_state::ItemState;
use crate::op::Op;
use crate::op_kind::OpKind;
use crate::wall_clock::WallClock;

/// 題名・本文・状態・担当・期限・ラベル・並び順と、消されているか、添付（生きているもの・連番の順）と、
/// 添付に使った最大の連番（外しても戻らない）と、自動の項目（起票日・更新日・完了日・Issue #117）。
/// すべて併合の派生値で、正本はジャーナル。起票者は作ったアカウント（[`ItemId::creator`]）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    id: ItemId,
    title: String,
    body: String,
    state: ItemState,
    assignee: String,
    due: Option<Date>,
    labels: Vec<String>,
    rank: i64,
    deleted: bool,
    attachments: Vec<Attachment>,
    last_attachment: u32,
    /// 起票日: 作る操作の壁時計。
    opened: WallClock,
    /// 更新日: 全順序で最後にこの課題に当たった操作の壁時計。
    updated: WallClock,
    /// 完了日: 全順序で最後に「完了」へ移した操作の壁時計。いま完了でなければ `None`。
    closed: Option<WallClock>,
}

impl Item {
    /// `Create` の直後。項目は空、状態は `Open`。
    #[must_use]
    pub const fn created(id: ItemId) -> Self {
        Self {
            id,
            title: String::new(),
            body: String::new(),
            state: ItemState::Open,
            assignee: String::new(),
            due: None,
            labels: Vec::new(),
            rank: 0,
            deleted: false,
            attachments: Vec::new(),
            last_attachment: 0,
            opened: WallClock::new(0),
            updated: WallClock::new(0),
            closed: None,
        }
    }

    /// 自動の項目を置いた状態（スナップショットから戻すとき・ADR 0023）。
    #[must_use]
    pub fn with_stamps(
        self,
        opened: WallClock,
        updated: WallClock,
        closed: Option<WallClock>,
    ) -> Self {
        Self {
            opened,
            updated,
            closed,
            ..self
        }
    }

    /// 起票日（作る操作の壁時計）。
    #[must_use]
    pub const fn opened(&self) -> WallClock {
        self.opened
    }

    /// 更新日（全順序で最後に当たった操作の壁時計）。
    #[must_use]
    pub const fn updated(&self) -> WallClock {
        self.updated
    }

    /// 完了日（全順序で最後に完了へ移した操作の壁時計）。いま完了でなければ `None`。
    #[must_use]
    pub const fn closed(&self) -> Option<WallClock> {
        self.closed
    }

    /// 添付を足した状態（同じ連番があれば置き換える）。連番の順に並ぶ。
    #[must_use]
    pub fn attached(mut self, attachment: Attachment) -> Self {
        self.last_attachment = self.last_attachment.max(attachment.sequence());
        self.attachments
            .retain(|known| known.sequence() != attachment.sequence());
        self.attachments.push(attachment);
        self.attachments.sort_by_key(Attachment::sequence);
        self
    }

    /// 添付を外した状態。無ければそのまま。
    #[must_use]
    pub fn detached(mut self, sequence: u32) -> Self {
        self.attachments
            .retain(|known| known.sequence() != sequence);
        self
    }

    /// 添付（生きているもの・連番の順）。
    #[must_use]
    pub fn attachments(&self) -> &[Attachment] {
        &self.attachments
    }

    /// 連番から添付。
    #[must_use]
    pub fn attachment(&self, sequence: u32) -> Option<&Attachment> {
        self.attachments
            .iter()
            .find(|known| known.sequence() == sequence)
    }

    /// 次に添付するときの連番（使った最大 ＋ 1・外しても戻らない）。
    #[must_use]
    pub const fn next_attachment(&self) -> u32 {
        self.last_attachment.saturating_add(1)
    }

    /// 添付の列と使った最大の連番から（スナップショットから戻す）。
    #[must_use]
    pub fn with_attachments(mut self, attachments: Vec<Attachment>, last_attachment: u32) -> Self {
        self.attachments = attachments;
        self.attachments.sort_by_key(Attachment::sequence);
        self.last_attachment = last_attachment.max(
            self.attachments
                .iter()
                .map(Attachment::sequence)
                .max()
                .unwrap_or(0),
        );
        self
    }

    /// 添付に使った最大の連番。
    #[must_use]
    pub const fn last_attachment(&self) -> u32 {
        self.last_attachment
    }

    /// 項目を置いた状態。
    #[must_use]
    pub fn with(mut self, value: FieldValue) -> Self {
        match value {
            FieldValue::Title(title) => self.title = title,
            FieldValue::Body(body) => self.body = body,
            FieldValue::State(state) => self.state = state,
            FieldValue::Assignee(assignee) => self.assignee = assignee,
            FieldValue::Due(due) => self.due = due,
            FieldValue::Labels(labels) => self.labels = labels,
            FieldValue::Rank(rank) => self.rank = rank,
        }
        self
    }

    /// 操作を当てた課題（`Docket::applied` の 1 件版）。全順序に並べた後で呼ぶ。権限の記録は何もしない。
    /// 自動の項目もここで刻む: 作る操作は起票日、課題に当たる操作は更新日、完了でない状態から完了へ移す操作は完了日
    /// （完了から外せば消える）。壁時計は機械ごとにずれるので、値は表示にだけ使う（ADR 0004 / 0023）。
    #[must_use]
    pub fn applied(self, op: &Op) -> Self {
        let wall = op.wall();
        let was_done = self.state == ItemState::Done;
        let changed = match op.kind() {
            OpKind::Admin(_) => return self,
            OpKind::Create(_) => Self {
                opened: wall,
                ..self
            },
            OpKind::Set { value, .. } => self.with(value.clone()),
            OpKind::Delete(_) => self.with_deleted(true),
            OpKind::Restore(_) => self.with_deleted(false),
            OpKind::Attach { attachment, .. } => self.attached(attachment.clone()),
            OpKind::Detach { sequence, .. } => self.detached(*sequence),
        };
        let closed = match (was_done, changed.state == ItemState::Done) {
            (false, true) => Some(wall),
            (true, true) => changed.closed,
            (_, false) => None,
        };
        Self {
            updated: wall,
            closed,
            ..changed
        }
    }

    /// 消した・戻した状態。
    #[must_use]
    pub fn with_deleted(self, deleted: bool) -> Self {
        Self { deleted, ..self }
    }

    /// いまの値を、項目の名前から取り出す。逆の操作（Undo）を作るのに使う。
    #[must_use]
    pub fn value_of(&self, field: crate::field::Field) -> FieldValue {
        match field {
            crate::field::Field::Title => FieldValue::Title(self.title.clone()),
            crate::field::Field::Body => FieldValue::Body(self.body.clone()),
            crate::field::Field::State => FieldValue::State(self.state),
            crate::field::Field::Assignee => FieldValue::Assignee(self.assignee.clone()),
            crate::field::Field::Due => FieldValue::Due(self.due),
            crate::field::Field::Labels => FieldValue::Labels(self.labels.clone()),
            crate::field::Field::Rank => FieldValue::Rank(self.rank),
        }
    }

    /// ID。
    #[must_use]
    pub const fn id(&self) -> ItemId {
        self.id
    }

    /// 題名。
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 本文。
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// 状態。
    #[must_use]
    pub const fn state(&self) -> ItemState {
        self.state
    }

    /// 担当。
    #[must_use]
    pub fn assignee(&self) -> &str {
        &self.assignee
    }

    /// 期限。
    #[must_use]
    pub const fn due(&self) -> Option<Date> {
        self.due
    }

    /// ラベル。
    #[must_use]
    pub fn labels(&self) -> &[String] {
        &self.labels
    }

    /// 並び順。
    #[must_use]
    pub const fn rank(&self) -> i64 {
        self.rank
    }

    /// 消されているか。
    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account_id::AccountId;
    use crate::field::Field;

    fn filled() -> Item {
        Item::created(ItemId::new(AccountId::new(1), 1))
            .with(FieldValue::Title(String::from("題")))
            .with(FieldValue::Body(String::from("本文")))
            .with(FieldValue::State(ItemState::Done))
            .with(FieldValue::Assignee(String::from("mori")))
            .with(FieldValue::Due(Date::new(2026, 12, 31)))
            .with(FieldValue::Labels(alloc::vec![String::from("core")]))
            .with(FieldValue::Rank(-5))
            .with_deleted(true)
    }

    #[test]
    fn values_set_and_read_back() {
        let item = filled();
        assert_eq!(item.title(), "題");
        assert_eq!(item.body(), "本文");
        assert_eq!(item.state(), ItemState::Done);
        assert_eq!(item.assignee(), "mori");
        assert_eq!(item.due(), Date::new(2026, 12, 31));
        assert_eq!(item.labels(), [String::from("core")]);
        assert_eq!(item.rank(), -5);
        assert!(item.is_deleted());
        assert_eq!(item.id(), ItemId::new(AccountId::new(1), 1));
    }

    fn stamped(clock: u64, wall: i64, kind: OpKind) -> Op {
        use crate::logical_clock::LogicalClock;
        use crate::op_id::OpId;
        Op::new(
            OpId::new(LogicalClock::new(clock), AccountId::new(1)),
            WallClock::new(wall),
            kind,
        )
    }

    fn set_state(value: ItemState) -> OpKind {
        OpKind::Set {
            item: ItemId::new(AccountId::new(1), 1),
            value: FieldValue::State(value),
        }
    }

    #[test]
    fn create_stamps_the_opened_date_and_every_op_the_updated_date() {
        let id = ItemId::new(AccountId::new(1), 1);
        let opened = Item::created(id).applied(&stamped(1, 100, OpKind::Create(id)));
        assert_eq!(opened.opened(), WallClock::new(100));
        assert_eq!(opened.updated(), WallClock::new(100));
        assert_eq!(opened.closed(), None);
        let titled = opened.applied(&stamped(
            2,
            150,
            OpKind::Set {
                item: id,
                value: FieldValue::Title(String::from("t")),
            },
        ));
        assert_eq!(titled.updated(), WallClock::new(150));
        assert_eq!(titled.opened(), WallClock::new(100));
        let admin = titled.clone().applied(&stamped(
            3,
            600,
            OpKind::Admin(crate::admin_command::AdminCommand::Permissions(true)),
        ));
        assert_eq!(admin, titled);
    }

    #[test]
    fn the_closed_date_is_the_last_move_into_done_and_leaving_done_clears_it() {
        let id = ItemId::new(AccountId::new(1), 1);
        let done = Item::created(id)
            .applied(&stamped(1, 100, OpKind::Create(id)))
            .applied(&stamped(2, 200, set_state(ItemState::Done)));
        assert_eq!(done.closed(), Some(WallClock::new(200)));
        // 完了のまま完了を置いても、完了日は最初に移したときのまま。更新日は進む。
        let again = done.applied(&stamped(3, 300, set_state(ItemState::Done)));
        assert_eq!(again.closed(), Some(WallClock::new(200)));
        assert_eq!(again.updated(), WallClock::new(300));
        let reopened = again.applied(&stamped(4, 400, set_state(ItemState::Open)));
        assert_eq!(reopened.closed(), None);
        let redone = reopened.applied(&stamped(5, 500, set_state(ItemState::Done)));
        assert_eq!(redone.closed(), Some(WallClock::new(500)));
    }

    #[test]
    fn value_of_reads_every_field_and_a_new_item_is_open() {
        let item = filled();
        for field in Field::ALL {
            assert_eq!(item.value_of(field).field(), field);
        }
        assert_eq!(
            item.value_of(Field::Title),
            FieldValue::Title(String::from("題"))
        );
        assert_eq!(
            Item::created(ItemId::new(AccountId::new(1), 1)).state(),
            ItemState::Open
        );
    }
}
