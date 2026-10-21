//! コマンドラインの引数。`nene-docket [FOLDER]`。合成ルートに許された唯一の入力（ARC-006）。
//! フォルダが無ければ設定の最近の台帳（FR-012）、それも無ければ `./docket`。

use std::path::{Path, PathBuf};

use crate::launch_failure::LaunchFailure;

/// 台帳のフォルダが無いときの既定。
const DEFAULT_FOLDER: &str = "docket";
/// `docket.v1.txt` を新しく作るときの名前の既定（フォルダ名が取れないとき）。
const DEFAULT_NAME: &str = "Docket";

/// 台帳のフォルダ（引数に無ければ `None`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Arguments {
    folder: Option<PathBuf>,
}

impl Arguments {
    /// プログラム名を除いた引数から。1 つなら台帳のフォルダ。2 つ以上と `-` で始まる語は使い方の誤り。
    pub(crate) fn parse<I: Iterator<Item = String>>(
        mut arguments: I,
    ) -> Result<Self, LaunchFailure> {
        let folder = arguments.next();
        if arguments.next().is_some() || folder.as_deref().is_some_and(|text| text.starts_with('-'))
        {
            return Err(LaunchFailure::Usage);
        }
        Ok(Self {
            folder: folder.map(PathBuf::from),
        })
    }

    /// 台帳のフォルダ。引数、無ければ最近の台帳、それも無ければ `./docket`。
    pub(crate) fn folder_or(&self, latest: Option<&str>) -> PathBuf {
        self.folder
            .clone()
            .unwrap_or_else(|| PathBuf::from(latest.map_or(DEFAULT_FOLDER, |recent| recent)))
    }

    /// 台帳を新しく作るときの名前。フォルダ名。
    pub(crate) fn docket_name(folder: &Path) -> String {
        folder
            .file_name()
            .and_then(|name| name.to_str())
            .map_or_else(|| String::from(DEFAULT_NAME), str::to_owned)
    }
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
        let given = parse(&["/srv/team"]).unwrap();
        assert_eq!(given.folder_or(Some("/srv/recent")), Path::new("/srv/team"));
        assert_eq!(Arguments::docket_name(Path::new("/srv/team")), "team");
        assert_eq!(Arguments::docket_name(Path::new("/")), "Docket");
    }

    #[test]
    fn flags_and_extra_words_are_a_usage_error() {
        assert!(matches!(parse(&["--help"]), Err(LaunchFailure::Usage)));
        assert!(matches!(parse(&["a", "b"]), Err(LaunchFailure::Usage)));
    }
}
