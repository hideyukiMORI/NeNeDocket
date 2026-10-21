//! コマンドラインの引数。合成ルートに許された唯一の入力（ARC-006）。
//!
//! `nene-docket [FOLDER]` で台帳を開く。フォルダが無ければ設定の最近の台帳（FR-012）、それも無ければ `./docket`。
//! 持ち主は `--permissions on|off` / `--allow <account>` / `--disallow <account>` / `--group <name> <account>` /
//! `--ungroup <name> <account>` で権限を変える（FR-010・ADR 0010）。これらがあれば画面は開かず、書いて終わる。

use std::path::{Path, PathBuf};

use nene_docket_core::account_id::AccountId;
use nene_docket_store::admin_command::AdminCommand;

use crate::launch_failure::LaunchFailure;

/// 台帳のフォルダが無いときの既定。
const DEFAULT_FOLDER: &str = "docket";
/// `docket.v1.txt` を新しく作るときの名前の既定（フォルダ名が取れないとき）。
const DEFAULT_NAME: &str = "Docket";

/// 台帳のフォルダ（引数に無ければ `None`）と、権限の命令（無ければ空）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Arguments {
    folder: Option<PathBuf>,
    admin: Vec<AdminCommand>,
}

impl Arguments {
    /// プログラム名を除いた引数から。`-` で始まらない語は 1 つまでで台帳のフォルダ。知らない旗・値の無い旗は使い方の誤り。
    pub(crate) fn parse<I: Iterator<Item = String>>(
        mut arguments: I,
    ) -> Result<Self, LaunchFailure> {
        let mut parsed = Self {
            folder: None,
            admin: Vec::new(),
        };
        while let Some(word) = arguments.next() {
            if !word.starts_with('-') {
                parsed.take_folder(word)?;
                continue;
            }
            let command = match word.as_str() {
                "--permissions" => AdminCommand::Permissions(switch(&mut arguments)?),
                "--allow" => AdminCommand::Allow(account(&mut arguments)?),
                "--disallow" => AdminCommand::Disallow(account(&mut arguments)?),
                "--group" => AdminCommand::Group(name(&mut arguments)?, account(&mut arguments)?),
                "--ungroup" => {
                    AdminCommand::Ungroup(name(&mut arguments)?, account(&mut arguments)?)
                }
                _ => return Err(LaunchFailure::Usage),
            };
            parsed.admin.push(command);
        }
        Ok(parsed)
    }

    /// 旗でない語は台帳のフォルダ。2 つ目は使い方の誤り。
    fn take_folder(&mut self, word: String) -> Result<(), LaunchFailure> {
        if self.folder.replace(PathBuf::from(word)).is_some() {
            return Err(LaunchFailure::Usage);
        }
        Ok(())
    }

    /// 台帳のフォルダ。引数、無ければ最近の台帳、それも無ければ `./docket`。
    pub(crate) fn folder_or(&self, latest: Option<&str>) -> PathBuf {
        self.folder
            .clone()
            .unwrap_or_else(|| PathBuf::from(latest.map_or(DEFAULT_FOLDER, |recent| recent)))
    }

    /// 権限の命令。空なら画面を開く。
    pub(crate) fn admin(&self) -> &[AdminCommand] {
        &self.admin
    }

    /// 台帳を新しく作るときの名前。フォルダ名。
    pub(crate) fn docket_name(folder: &Path) -> String {
        folder
            .file_name()
            .and_then(|name| name.to_str())
            .map_or_else(|| String::from(DEFAULT_NAME), str::to_owned)
    }
}

fn switch<I: Iterator<Item = String>>(arguments: &mut I) -> Result<bool, LaunchFailure> {
    match arguments.next().as_deref() {
        Some("on") => Ok(true),
        Some("off") => Ok(false),
        _ => Err(LaunchFailure::Usage),
    }
}

fn account<I: Iterator<Item = String>>(arguments: &mut I) -> Result<AccountId, LaunchFailure> {
    arguments
        .next()
        .and_then(|text| AccountId::from_hex(&text))
        .ok_or(LaunchFailure::Usage)
}

fn name<I: Iterator<Item = String>>(arguments: &mut I) -> Result<String, LaunchFailure> {
    arguments
        .next()
        .filter(|text| !text.is_empty() && !text.starts_with('-') && !text.contains(':'))
        .ok_or(LaunchFailure::Usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(words: &[&str]) -> Result<Arguments, LaunchFailure> {
        Arguments::parse(words.iter().map(|word| String::from(*word)))
    }

    #[test]
    fn the_argument_wins_then_the_latest_then_the_default() {
        let none = parse(&[]).unwrap();
        assert_eq!(none.folder_or(None), Path::new("docket"));
        assert_eq!(
            none.folder_or(Some("/srv/recent")),
            Path::new("/srv/recent")
        );
        assert!(none.admin().is_empty());
        let given = parse(&["/srv/team"]).unwrap();
        assert_eq!(given.folder_or(Some("/srv/recent")), Path::new("/srv/team"));
        assert_eq!(Arguments::docket_name(Path::new("/srv/team")), "team");
        assert_eq!(Arguments::docket_name(Path::new("/")), "Docket");
    }

    #[test]
    fn admin_flags_are_read_in_order_with_the_folder_anywhere() {
        let id = AccountId::new(7);
        let hex = id.hex();
        let parsed = parse(&[
            "--permissions",
            "on",
            "/srv/team",
            "--allow",
            &hex,
            "--group",
            "qa",
            &hex,
            "--ungroup",
            "qa",
            &hex,
            "--disallow",
            &hex,
        ])
        .unwrap();
        assert_eq!(parsed.folder_or(None), Path::new("/srv/team"));
        assert_eq!(
            parsed.admin(),
            [
                AdminCommand::Permissions(true),
                AdminCommand::Allow(id),
                AdminCommand::Group(String::from("qa"), id),
                AdminCommand::Ungroup(String::from("qa"), id),
                AdminCommand::Disallow(id),
            ]
        );
        assert_eq!(
            parse(&["--permissions", "off"]).unwrap().admin(),
            [AdminCommand::Permissions(false)]
        );
    }

    #[test]
    fn unknown_flags_missing_values_and_extra_words_are_a_usage_error() {
        let hex = AccountId::new(7).hex();
        for words in [
            vec!["--help"],
            vec!["a", "b"],
            vec!["--permissions"],
            vec!["--permissions", "maybe"],
            vec!["--allow"],
            vec!["--allow", "zz"],
            vec!["--group", "qa"],
            vec!["--group", "", &hex],
            vec!["--group", "a:b", &hex],
            vec!["--ungroup", "--allow", &hex],
        ] {
            assert!(
                matches!(parse(&words), Err(LaunchFailure::Usage)),
                "{words:?}"
            );
        }
    }
}
