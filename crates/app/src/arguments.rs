//! コマンドラインの引数。`nene-docket [FOLDER]`。合成ルートに許された唯一の入力（ARC-006）。

use std::path::{Path, PathBuf};

use crate::launch_failure::LaunchFailure;

/// 台帳のフォルダが無いときの既定。
const DEFAULT_FOLDER: &str = "docket";
/// `docket.v1.txt` を新しく作るときの名前の既定（フォルダ名が取れないとき）。
const DEFAULT_NAME: &str = "Docket";

/// 台帳のフォルダ。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Arguments {
    folder: PathBuf,
}

impl Arguments {
    /// プログラム名を除いた引数から。1 つなら台帳のフォルダ、無ければ `./docket`。それ以外は使い方の誤り。
    pub(crate) fn parse<I: Iterator<Item = String>>(
        mut arguments: I,
    ) -> Result<Self, LaunchFailure> {
        let folder = arguments.next();
        if arguments.next().is_some() || folder.as_deref().is_some_and(|text| text.starts_with('-'))
        {
            return Err(LaunchFailure::Usage);
        }
        Ok(Self {
            folder: PathBuf::from(folder.unwrap_or_else(|| String::from(DEFAULT_FOLDER))),
        })
    }

    /// 台帳のフォルダ。
    pub(crate) fn folder(&self) -> &Path {
        &self.folder
    }

    /// 台帳を新しく作るときの名前。フォルダ名。
    pub(crate) fn docket_name(&self) -> String {
        self.folder
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
    fn one_folder_or_the_default() {
        assert_eq!(parse(&[]).unwrap().folder(), Path::new("docket"));
        let given = parse(&["/srv/team"]).unwrap();
        assert_eq!(given.folder(), Path::new("/srv/team"));
        assert_eq!(given.docket_name(), "team");
        assert_eq!(parse(&["/"]).unwrap().docket_name(), "Docket");
    }

    #[test]
    fn flags_and_extra_words_are_a_usage_error() {
        assert!(matches!(parse(&["--help"]), Err(LaunchFailure::Usage)));
        assert!(matches!(parse(&["a", "b"]), Err(LaunchFailure::Usage)));
    }
}
