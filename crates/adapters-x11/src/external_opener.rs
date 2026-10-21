//! OS の既定のアプリでファイルを開く（FR-022）。`xdg-open` を起動する。プロセスの起動はアダプタだけ（ARC-007）。
//! 起動したプロセスの終わりは別のスレッドで待つ（`xdg-open` はすぐ戻る・ゾンビを残さない）。

use std::process::{Command, Stdio};
use std::thread;

use nene_docket_application::open_failure::OpenFailure;
use nene_docket_application::opener_port::OpenerPort;

/// 起動する命令の名前（既定は `xdg-open`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalOpener {
    command: String,
}

impl ExternalOpener {
    /// `xdg-open` で開く。
    #[must_use]
    pub fn xdg_open() -> Self {
        Self::with_command("xdg-open")
    }

    /// 別の命令で開く（テスト・環境の都合）。
    #[must_use]
    pub fn with_command(command: &str) -> Self {
        Self {
            command: command.to_owned(),
        }
    }
}

impl OpenerPort for ExternalOpener {
    fn open(&self, path: &str) -> Result<(), OpenFailure> {
        let mut child = Command::new(&self.command)
            .arg(path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_error| OpenFailure::CannotOpen)?;
        thread::spawn(move || {
            let _ignored = child.wait();
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_present_command_opens_and_a_missing_one_fails() {
        assert_eq!(ExternalOpener::with_command("true").open("/tmp/x"), Ok(()));
        assert_eq!(
            ExternalOpener::with_command("nene-docket-no-such-command").open("/tmp/x"),
            Err(OpenFailure::CannotOpen)
        );
        assert_eq!(ExternalOpener::xdg_open().command, "xdg-open");
    }
}
