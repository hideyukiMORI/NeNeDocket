//! 共有フォルダの速さを測る（Issue #75・`--probe-share`）。`stat`・末尾の読み・追記 ＋ 同期・一時名からの rename の 1 回あたりの時間。
//! 台帳の正本には触らず、`probe/` を作って消す。数字は gate-proofs に残し、読み直しの間隔を決める根拠にする（QLT-013）。

use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use crate::journal_file;
use crate::store_failure::StoreFailure;

/// 測る相手のフォルダ名。
const DIRECTORY: &str = "probe";
/// 計測用のファイルの行（1 行 200 バイト強を 300 行）。
const LINES: usize = 300;

/// 1 回あたりの時間（平均）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShareProbe {
    /// `metadata()`（読み直しの伸びの確認）。
    pub stat: Duration,
    /// 末尾 4 KiB の読み（新しい操作の読み）。
    pub read_tail: Duration,
    /// 1 行の追記 ＋ `sync_data`（操作の記録）。
    pub append_sync: Duration,
    /// 一時名に書いて rename（スナップショット・権限・添付）。
    pub rename: Duration,
}

impl ShareProbe {
    /// 1 行の報告（ms）。
    #[must_use]
    pub fn describe(&self) -> String {
        let millis = |duration: Duration| duration.as_secs_f64() * 1000.0_f64;
        format!(
            "stat {:.2} ms / read tail {:.2} ms / append+sync {:.2} ms / rename {:.2} ms",
            millis(self.stat),
            millis(self.read_tail),
            millis(self.append_sync),
            millis(self.rename)
        )
    }
}

/// `rounds` 回ずつ測る。
///
/// # Errors
///
/// フォルダを作れない・書けない・読めない。
pub fn measure(root: &Path, rounds: u32) -> Result<ShareProbe, StoreFailure> {
    let directory = root.join(DIRECTORY);
    fs::create_dir_all(&directory)?;
    let file = directory.join("probe.log");
    let line = "x".repeat(200);
    let mut text = String::new();
    for _ in 0..LINES {
        text.push_str(&line);
        text.push('\n');
    }
    fs::write(&file, &text)?;
    let rounds = rounds.max(1);
    let stat = timed(rounds, || {
        fs::metadata(&file)?;
        Ok(())
    })?;
    let read_tail = timed(rounds, || {
        journal_file::read_tail(&file, 4096).map(|_read| ())
    })?;
    let append_sync = timed(rounds, || {
        let mut handle = fs::OpenOptions::new().append(true).open(&file)?;
        handle.write_all(text.get(..201).unwrap_or("").as_bytes())?;
        handle.sync_data()?;
        Ok(())
    })?;
    let rename = timed(rounds, || {
        let temporary = directory.join("probe.tmp");
        fs::write(&temporary, b"probe\n")?;
        fs::rename(&temporary, directory.join("probe.txt"))?;
        Ok(())
    })?;
    fs::remove_dir_all(&directory)?;
    Ok(ShareProbe {
        stat,
        read_tail,
        append_sync,
        rename,
    })
}

/// `rounds` 回走らせた平均。
fn timed<F: FnMut() -> Result<(), StoreFailure>>(
    rounds: u32,
    mut step: F,
) -> Result<Duration, StoreFailure> {
    let start = Instant::now();
    for _ in 0..rounds {
        step()?;
    }
    Ok(start.elapsed() / rounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_four_operations_and_leaves_no_probe_folder() {
        let root = std::env::temp_dir().join(format!("nene-docket-probe-{}", std::process::id()));
        let _ignored = fs::remove_dir_all(&root);
        let probe = measure(&root, 3).unwrap();
        assert!(!root.join(DIRECTORY).exists());
        assert!(probe.stat < Duration::from_secs(5));
        assert!(probe.read_tail < Duration::from_secs(5));
        assert!(probe.append_sync < Duration::from_secs(5));
        assert!(probe.rename < Duration::from_secs(5));
        assert!(probe.describe().contains("append+sync"));
        let _cleaned = fs::remove_dir_all(&root);
    }
}
