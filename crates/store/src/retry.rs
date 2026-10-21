//! Windows の一時的なロック（共有違反・ウイルス対策が作ったばかりのファイルを開く・Explorer のプレビュー）への再試行（Issue #79）。
//! 50 ms × 5 回。OS に依らず同じコードで、Linux では一時的な失敗が起きないので 1 回で決まる。

use std::io;
use std::thread;
use std::time::Duration;

/// 試す回数（最初の 1 回を含む）。
const ATTEMPTS: u32 = 5;
/// 失敗と次の試みの間。
const PAUSE: Duration = Duration::from_millis(50);

/// 一時的な失敗なら少し待って試し直し、それでも駄目なら最後の失敗を返す。
pub(crate) fn retried<T>(mut operation: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    let mut attempt = 1_u32;
    loop {
        match operation() {
            Err(error) if attempt < ATTEMPTS && is_transient(&error) => {
                attempt = attempt.saturating_add(1);
                thread::sleep(PAUSE);
            }
            outcome => return outcome,
        }
    }
}

/// 待てば通るかもしれない失敗か。`ERROR_ACCESS_DENIED`（Rust は `PermissionDenied`）と、Windows の
/// `ERROR_SHARING_VIOLATION`（32）/ `ERROR_LOCK_VIOLATION`（33）。Linux の `EACCES` も `PermissionDenied` だが、5 回で諦めるだけ。
fn is_transient(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::PermissionDenied
        || (cfg!(windows) && matches!(error.raw_os_error(), Some(32_i32 | 33_i32)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failing(times: u32, kind: io::ErrorKind) -> impl FnMut() -> io::Result<u32> {
        let mut left = times;
        move || {
            if left == 0 {
                Ok(7)
            } else {
                left -= 1;
                Err(io::Error::from(kind))
            }
        }
    }

    #[test]
    fn a_transient_failure_is_retried_until_it_passes() {
        assert_eq!(
            retried(failing(3, io::ErrorKind::PermissionDenied)).unwrap(),
            7
        );
        assert_eq!(
            retried(failing(0, io::ErrorKind::PermissionDenied)).unwrap(),
            7
        );
    }

    #[test]
    fn five_failures_in_a_row_give_up_with_the_last_failure() {
        let outcome = retried(failing(5, io::ErrorKind::PermissionDenied));
        assert_eq!(outcome.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn a_permanent_failure_is_not_retried() {
        let mut calls = 0_u32;
        let outcome: io::Result<()> = retried(|| {
            calls += 1;
            Err(io::Error::from(io::ErrorKind::NotFound))
        });
        assert_eq!(outcome.unwrap_err().kind(), io::ErrorKind::NotFound);
        assert_eq!(calls, 1);
        assert!(!is_transient(&io::Error::from(io::ErrorKind::BrokenPipe)));
    }
}
