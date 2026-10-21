//! 台帳全体の時系列をテキストにする（監査・ADR 0014）。`nene-docket <フォルダ> --audit` が標準出力に出す。
//! 1 行 1 操作: `<いつ UTC>\t<誰が>\t<何を>`（課題の操作は番号が頭に付く）。古いものが先。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::language::Language;
use nene_docket_core::op::Op;

use crate::history_line::HistoryLine;

/// 欄の区切り。
const SEPARATOR: char = '\t';

/// 全部の操作を古い順に。空なら空文字列。
#[must_use]
pub fn text<'a, I>(ops: I, names: &[(AccountId, String)], language: Language) -> String
where
    I: IntoIterator<Item = &'a Op>,
{
    let mut lines = HistoryLine::all(ops, names);
    lines.reverse();
    let rows: Vec<String> = lines
        .iter()
        .map(|line| {
            let [when, who, change] = line.columns(language);
            alloc::format!("{when}{SEPARATOR}{who}{SEPARATOR}{change}\n")
        })
        .collect();
    rows.concat()
}

#[cfg(test)]
mod tests {
    use nene_docket_core::admin_command::AdminCommand;
    use nene_docket_core::item_id::ItemId;
    use nene_docket_core::logical_clock::LogicalClock;
    use nene_docket_core::op_id::OpId;
    use nene_docket_core::op_kind::OpKind;
    use nene_docket_core::wall_clock::WallClock;

    use super::*;

    const A: AccountId = AccountId::new(0xAB00);

    #[test]
    fn the_report_is_one_line_per_op_oldest_first_with_the_item_number() {
        let item = ItemId::new(A, 1);
        let ops = [
            Op::new(
                OpId::new(LogicalClock::new(2), A),
                WallClock::new(120),
                OpKind::Admin(AdminCommand::Permissions(true)),
            ),
            Op::new(
                OpId::new(LogicalClock::new(1), A),
                WallClock::new(60),
                OpKind::Create(item),
            ),
        ];
        let names = [(A, String::from("alice"))];
        assert_eq!(
            text(&ops, &names, Language::English),
            "1970-01-01 00:01\talice\t0000-1: created\n1970-01-01 00:02\talice\tturned permissions on\n"
        );
        assert_eq!(text(&[], &names, Language::English), "");
    }
}
