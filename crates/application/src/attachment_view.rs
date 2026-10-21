//! 添付の見せ方（FR-022）。種類で決める: 文（読み取り専用の欄）・表（CSV / TSV）・それ以外はパスと「開く」。
//! 画像（PNG / JPEG）は画素の列（ホイールで拡大・ドラッグで移動）。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::attachment::Attachment;
use nene_docket_core::attachment_kind::AttachmentKind;
use nene_docket_core::csv;
use nene_docket_core::item_id::ItemId;
use nene_docket_core::shared_bitmap::SharedBitmap;
use nene_docket_core::text_edit::TextEdit;

use crate::journal_failure::JournalFailure;
use crate::journal_port::JournalPort;

/// 読んだ中身。閉じた集合（RS-002）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentView {
    /// 文。
    Text {
        /// 読み取り専用の欄に出す文。
        edit: TextEdit,
        /// 先頭だけか。
        truncated: bool,
    },
    /// 表。
    Table {
        /// 行と欄。最初の行が見出し。
        rows: Vec<Vec<String>>,
        /// 列ごとのいちばん長い欄の文字数。
        widths: Vec<usize>,
        /// 先頭だけか。
        truncated: bool,
    },
    /// 画像。UI が伸縮して描く。
    Image(SharedBitmap),
    /// OS の既定のアプリで開く。写しの場所。
    Path(String),
}

impl AttachmentView {
    /// 種類に応じて読む。文と表はポートから文を読み、それ以外は場所だけ。
    ///
    /// # Errors
    ///
    /// 写しを読めない。
    pub fn load(
        journal: &dyn JournalPort,
        item: ItemId,
        attachment: &Attachment,
    ) -> Result<Self, JournalFailure> {
        let kind = AttachmentKind::of(attachment.name());
        let separator = match kind {
            AttachmentKind::Csv => Some(','),
            AttachmentKind::Tsv => Some('\t'),
            AttachmentKind::Text => None,
            AttachmentKind::Image => {
                return journal
                    .read_attachment_image(item, attachment)
                    .map(Self::Image);
            }
            AttachmentKind::Other => {
                return Ok(Self::Path(journal.attachment_path(item, attachment)));
            }
        };
        let text = journal.read_attachment(item, attachment)?;
        let truncated = text.is_truncated();
        Ok(match separator {
            Some(separator) => {
                let rows = csv::parse(text.text(), separator);
                let widths = csv::widest(&rows);
                Self::Table {
                    rows,
                    widths,
                    truncated,
                }
            }
            None => Self::Text {
                edit: TextEdit::showing(text.text()),
                truncated,
            },
        })
    }

    /// 先頭だけか。
    #[must_use]
    pub const fn is_truncated(&self) -> bool {
        match self {
            Self::Text { truncated, .. } | Self::Table { truncated, .. } => *truncated,
            Self::Image(_) | Self::Path(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use nene_docket_core::account_id::AccountId;
    use nene_docket_core::admin_command::AdminCommand;
    use nene_docket_core::bitmap::Bitmap;
    use nene_docket_core::op::Op;
    use nene_docket_core::permissions::Permissions;

    use crate::admission_report::AdmissionReport;
    use crate::attachment_text::AttachmentText;
    use crate::ledger_snapshot::LedgerSnapshot;

    use super::*;

    /// 名前が `.csv` なら表の文、`.tsv` なら `\t` 区切り、他は 2 行の文。`big` で始まれば先頭だけ。
    struct TextJournal;

    impl JournalPort for TextJournal {
        fn append(&mut self, _op: &Op) -> Result<(), JournalFailure> {
            Ok(())
        }
        fn sync(&mut self) -> Result<(), JournalFailure> {
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
            Ok(Vec::new())
        }
        fn checkpoint(&mut self, _snapshot: &LedgerSnapshot) -> Result<(), JournalFailure> {
            Ok(())
        }
        fn rejected_lines(&self) -> usize {
            0
        }
        fn admission(&self) -> AdmissionReport {
            AdmissionReport::new(0, false, false)
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
        fn copy_attachment(
            &mut self,
            _item: ItemId,
            sequence: u32,
            source: &str,
        ) -> Result<Attachment, JournalFailure> {
            Ok(Attachment::new(
                sequence,
                String::from(source),
                0,
                [0_u8; 32],
            ))
        }
        fn attachment_is_intact(&self, _item: ItemId, _attachment: &Attachment) -> bool {
            true
        }
        fn attachment_path(&self, _item: ItemId, attachment: &Attachment) -> String {
            alloc::format!("/d/{}", attachment.file_name())
        }
        fn read_attachment_image(
            &self,
            _item: ItemId,
            attachment: &Attachment,
        ) -> Result<SharedBitmap, JournalFailure> {
            if attachment.name().starts_with("broken") {
                return Err(JournalFailure::AttachmentUndecodable);
            }
            let bitmap = Bitmap::new(2, 1, alloc::vec![1, 2, 3, 4, 5, 6, 7, 8])
                .ok_or(JournalFailure::AttachmentUndecodable)?;
            Ok(SharedBitmap::new(1, bitmap))
        }

        fn read_attachment(
            &self,
            _item: ItemId,
            attachment: &Attachment,
        ) -> Result<AttachmentText, JournalFailure> {
            let name = attachment.name();
            let text = match AttachmentKind::of(name) {
                AttachmentKind::Csv => "a,b\n1,2\n",
                AttachmentKind::Tsv => "a\tb\n",
                AttachmentKind::Text | AttachmentKind::Image | AttachmentKind::Other => "one\ntwo",
            };
            Ok(AttachmentText::new(
                String::from(text),
                name.starts_with("big"),
            ))
        }
        fn administer(&mut self, _commands: &[AdminCommand]) -> Result<(), JournalFailure> {
            Ok(())
        }
    }

    fn load(name: &str) -> AttachmentView {
        let attachment = Attachment::new(1, String::from(name), 0, [0_u8; 32]);
        AttachmentView::load(&TextJournal, ItemId::new(AccountId::new(1), 1), &attachment).unwrap()
    }

    #[test]
    fn text_kinds_become_a_read_only_edit() {
        let text = load("a.txt");
        assert_eq!(
            text,
            AttachmentView::Text {
                edit: TextEdit::showing("one\ntwo"),
                truncated: false
            }
        );
        assert!(!text.is_truncated());
        assert!(load("big.md").is_truncated());
    }

    #[test]
    fn table_kinds_become_rows_and_other_kinds_a_path() {
        let csv = load("a.csv");
        assert_eq!(
            csv,
            AttachmentView::Table {
                rows: alloc::vec![
                    alloc::vec![String::from("a"), String::from("b")],
                    alloc::vec![String::from("1"), String::from("2")]
                ],
                widths: alloc::vec![1, 1],
                truncated: false
            }
        );
        assert!(!csv.is_truncated());
        assert_eq!(
            load("a.tsv"),
            AttachmentView::Table {
                rows: alloc::vec![alloc::vec![String::from("a"), String::from("b")]],
                widths: alloc::vec![1, 1],
                truncated: false
            }
        );
        assert_eq!(
            load("a.xlsx"),
            AttachmentView::Path(String::from("/d/1-a.xlsx"))
        );
        let image = load("a.png");
        assert!(matches!(&image, AttachmentView::Image(bitmap) if bitmap.bitmap().width() == 2));
        assert!(!image.is_truncated());
        let broken = AttachmentView::load(
            &TextJournal,
            ItemId::new(AccountId::new(1), 1),
            &Attachment::new(1, String::from("broken.png"), 0, [0_u8; 32]),
        );
        assert_eq!(broken, Err(JournalFailure::AttachmentUndecodable));
    }
}
