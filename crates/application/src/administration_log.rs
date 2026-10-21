//! 権限の変更を操作としてジャーナルに追記する（監査・ADR 0014）。画面も合成ルートも、権限を書いた後にここを通る。

use nene_docket_core::admin_command::AdminCommand;
use nene_docket_core::op_kind::OpKind;

use crate::journal_failure::JournalFailure;
use crate::ledger::Ledger;
use crate::ports::Ports;

/// 命令を 1 つずつ `OpKind::Admin` の操作にして追記し、同期する。台帳の状態は変わらない。
///
/// # Errors
///
/// ジャーナルに書けない・同期できない。
pub fn record(
    ledger: &mut Ledger,
    ports: &mut Ports,
    commands: &[AdminCommand],
) -> Result<(), JournalFailure> {
    for command in commands {
        let now = ports.clock().now();
        let op = ledger.compose(now, OpKind::Admin(command.clone()));
        ports.journal_mut().append(&op)?;
    }
    ports.journal_mut().sync()
}
