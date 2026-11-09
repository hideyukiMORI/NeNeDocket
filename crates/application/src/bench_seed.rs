//! 計測用の台帳を作る（`--bench`・Issue #111）。自分のアカウントで課題を作り、1 件に 5 操作を追記し、最後に 1 度だけ同期する。
//! 書く経路は画面と同じ（`Ledger::compose` → `JournalPort::append`）。計測用の第 2 の書き方は作らない（ARC-001）。

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::field_value::FieldValue;
use nene_docket_core::item_state::ItemState;
use nene_docket_core::op_kind::OpKind;
use nene_docket_core::wall_clock::WallClock;

use crate::journal_failure::JournalFailure;
use crate::journal_port::JournalPort;
use crate::ledger::Ledger;

/// 1 件あたりの操作の数（作る・題名・担当・ラベル・状態）。
pub const OPS_PER_ITEM: u32 = 5;

/// 担当の名前の巡回。
const ASSIGNEES: [&str; 4] = ["alice", "bob", "carol", "dave"];
/// 状態の巡回。
const STATES: [ItemState; 3] = [ItemState::Open, ItemState::InProgress, ItemState::Done];

/// `count` 件を追記して同期する。壁時計は表示にだけ使うので、全部の操作に同じ `now` を添える。
///
/// # Errors
///
/// ジャーナルに書けない・同期できない。
pub fn seed(
    ledger: &mut Ledger,
    journal: &mut dyn JournalPort,
    now: WallClock,
    count: u32,
) -> Result<(), JournalFailure> {
    for number in 0..count {
        let item = ledger.fresh_item_id();
        let lap = usize::try_from(number).unwrap_or(0);
        let assignee = ASSIGNEES
            .get(lap % ASSIGNEES.len())
            .copied()
            .unwrap_or("alice");
        let state = STATES
            .get(lap % STATES.len())
            .copied()
            .unwrap_or(ItemState::Open);
        let kinds = [
            OpKind::Create(item),
            OpKind::Set {
                item,
                value: FieldValue::Title(format!("計測の課題 {number}")),
            },
            OpKind::Set {
                item,
                value: FieldValue::Assignee(String::from(assignee)),
            },
            OpKind::Set {
                item,
                value: FieldValue::Labels(Vec::from([format!("label-{}", number % 7)])),
            },
            OpKind::Set {
                item,
                value: FieldValue::State(state),
            },
        ];
        for kind in kinds {
            let op = ledger.compose(now, kind);
            journal.append(&op)?;
        }
    }
    journal.sync()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Account;
    use crate::admission_report::AdmissionReport;
    use crate::attachment_text::AttachmentText;
    use crate::ledger_snapshot::LedgerSnapshot;
    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::admin_command::AdminCommand;
    use nene_docket_core::attachment::Attachment;
    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::op::Op;
    use nene_docket_core::permissions::Permissions;
    use nene_docket_core::shared_bitmap::SharedBitmap;
    use nene_docket_core::signing_key::SigningKey;

    /// 追記を数えるだけのジャーナル。
    struct Counting {
        ops: Vec<Op>,
        syncs: usize,
    }

    impl JournalPort for Counting {
        fn append(&mut self, op: &Op) -> Result<(), JournalFailure> {
            self.ops.push(op.clone());
            Ok(())
        }

        fn sync(&mut self) -> Result<(), JournalFailure> {
            self.syncs += 1;
            Ok(())
        }

        fn refresh(&mut self) -> Result<Vec<Op>, JournalFailure> {
            Ok(Vec::new())
        }

        fn snapshot(&mut self) -> Option<LedgerSnapshot> {
            None
        }

        fn ops_of(&self, _item: ItemId) -> Result<Vec<Op>, JournalFailure> {
            Ok(Vec::new())
        }

        fn recent_ops(&self, _limit: usize) -> Result<Vec<Op>, JournalFailure> {
            Ok(Vec::new())
        }

        fn all_ops(&self) -> Result<Vec<Op>, JournalFailure> {
            Ok(self.ops.clone())
        }

        fn checkpoint(&mut self, _snapshot: &LedgerSnapshot) -> Result<(), JournalFailure> {
            Ok(())
        }

        fn rejected_lines(&self) -> usize {
            0
        }

        fn admission(&self) -> AdmissionReport {
            AdmissionReport::open()
        }

        fn names(&self) -> Vec<(AccountId, String)> {
            Vec::new()
        }

        fn owner(&self) -> Option<AccountId> {
            None
        }

        fn permissions(&self) -> Option<Permissions> {
            None
        }

        fn administer(&mut self, _commands: &[AdminCommand]) -> Result<(), JournalFailure> {
            Ok(())
        }

        fn copy_attachment(
            &mut self,
            _item: ItemId,
            _sequence: u32,
            _source: &str,
        ) -> Result<Attachment, JournalFailure> {
            Err(JournalFailure::Unwritable)
        }

        fn attachment_is_intact(&self, _item: ItemId, _attachment: &Attachment) -> bool {
            true
        }

        fn attachment_path(&self, _item: ItemId, attachment: &Attachment) -> String {
            attachment.file_name()
        }

        fn read_attachment(
            &self,
            _item: ItemId,
            _attachment: &Attachment,
        ) -> Result<AttachmentText, JournalFailure> {
            Ok(AttachmentText::new(String::new(), false))
        }

        fn read_attachment_image(
            &self,
            _item: ItemId,
            _attachment: &Attachment,
        ) -> Result<SharedBitmap, JournalFailure> {
            Err(JournalFailure::AttachmentUndecodable)
        }
    }

    #[test]
    fn seeding_appends_five_ops_per_item_and_syncs_once() {
        let account = Account::new(
            AccountId::new(1),
            String::from("me"),
            SigningKey::from_bytes(&[1_u8; 32]),
        );
        let mut ledger = Ledger::open(account, &[]);
        let mut journal = Counting {
            ops: Vec::new(),
            syncs: 0,
        };
        seed(&mut ledger, &mut journal, WallClock::new(0), 7).unwrap();
        assert_eq!(journal.ops.len(), 35);
        assert_eq!(journal.syncs, 1);
        let docket = ledger.docket();
        let first = docket.item(ItemId::new(AccountId::new(1), 1)).unwrap();
        assert_eq!(first.title(), "計測の課題 0");
        assert_eq!(first.assignee(), "alice");
        let third = docket.item(ItemId::new(AccountId::new(1), 3)).unwrap();
        assert_eq!(third.state(), ItemState::Done);
    }
}
