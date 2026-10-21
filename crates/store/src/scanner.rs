//! 他人のジャーナルの読み直しを別のスレッドで走らせる（Issue #75・ARC-007: スレッドを持てるのは store とアダプタ）。
//! 画面の読み直しの合図は「頼む」と「届いていれば受け取る」に分かれ、SMB の遅い `stat` や読みでフレームが止まらない。
//! 頼みは 1 つしか溜めない（遅い共有では合図が重なっても 1 回の読みにまとまる）。

use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread;
use std::time::Duration;

use nene_docket_application::key_record::KeyRecord;
use nene_docket_core::account_id::AccountId;

use crate::journal_folder::JournalFolder;
use crate::other_journals::OtherJournals;
use crate::scan::Scan;
use crate::verifier::{self, Verifier};

/// テストや終了時に 1 回の読みを待つ上限。
const WAIT: Duration = Duration::from_secs(30);
/// 刻みごとに頼んだ後、結果を待つ短い上限。ローカルの台帳ならこの間に届く（同じ刻みで画面に出る）。遅い共有では次の刻みに回る。
const BRIEF_WAIT: Duration = Duration::from_millis(50);

/// 走っているスレッドへの頼みの口と、結果の受け口。落とすと（`drop`）頼みの口が閉じ、スレッドは次の頼みで終わる。
#[derive(Debug)]
pub(crate) struct Scanner {
    requests: SyncSender<()>,
    results: Receiver<Scan>,
}

impl Scanner {
    /// `FileJournal` の今の状態の写しから始める（権限が有効なら持ち主を検証の相手に）。
    pub(crate) fn start_for(
        folder: &JournalFolder,
        others: &OtherJournals,
        (keys, verifier): (&[KeyRecord], Option<&Verifier>),
    ) -> Self {
        Self::start(
            folder.clone(),
            others.clone(),
            (keys.to_vec(), verifier.cloned()),
            folder.docket().owner().filter(|_| verifier.is_some()),
        )
    }

    /// フォルダ・読み位置・鍵と検証器（権限が有効なら持ち主）を渡してスレッドを始める。
    pub(crate) fn start(
        folder: JournalFolder,
        others: OtherJournals,
        (keys, verifier): (Vec<KeyRecord>, Option<Verifier>),
        verified_owner: Option<AccountId>,
    ) -> Self {
        let (requests, incoming) = mpsc::sync_channel::<()>(1);
        let (outgoing, results) = mpsc::channel::<Scan>();
        let _handle = thread::spawn(move || {
            serve(
                &folder,
                (incoming, &outgoing),
                (others, keys, verifier),
                verified_owner,
            );
        });
        Self { requests, results }
    }

    /// 1 回の読みを頼む。既に頼みが溜まっていれば（遅い共有）まとめる。スレッドが終わっていれば何もしない。
    pub(crate) fn request(&self) {
        match self.requests.try_send(()) {
            Ok(()) | Err(TrySendError::Full(()) | TrySendError::Disconnected(())) => {}
        }
    }

    /// 届いている結果を全部受け取る（待たない）。
    pub(crate) fn take(&self) -> Vec<Scan> {
        self.results.try_iter().collect()
    }

    /// 次の結果を待って受け取る（テストと、確実に最新を見たいとき）。上限を過ぎれば `None`。
    pub(crate) fn wait(&self) -> Option<Scan> {
        self.results.recv_timeout(WAIT).ok()
    }

    /// 次の結果を 50 ms だけ待つ。届かなければ `None`（次の刻みで受け取る）。
    pub(crate) fn wait_briefly(&self) -> Option<Scan> {
        self.results.recv_timeout(BRIEF_WAIT).ok()
    }
}

/// 頼みが来るたびに 1 回読み、結果を送る。頼みの口が閉じるか、受け口が無くなれば終わる。
fn serve(
    folder: &JournalFolder,
    (incoming, outgoing): (Receiver<()>, &mpsc::Sender<Scan>),
    mut state: (OtherJournals, Vec<KeyRecord>, Option<Verifier>),
    verified_owner: Option<AccountId>,
) {
    for () in incoming {
        let scan = scan_once(folder, &mut state, verified_owner);
        if outgoing.send(scan).is_err() {
            return;
        }
    }
}

/// 鍵と検証器を読み直してから、他人の伸びた分を読む。失敗は結果に添える（スレッドは続ける）。
fn scan_once(
    folder: &JournalFolder,
    (others, keys, verifier): &mut (OtherJournals, Vec<KeyRecord>, Option<Verifier>),
    verified_owner: Option<AccountId>,
) -> Scan {
    let mut failure = None;
    match verifier::load_if_enabled(folder.root(), verified_owner) {
        Ok((fresh_keys, fresh_verifier)) => {
            *keys = fresh_keys;
            *verifier = fresh_verifier;
        }
        Err(error) => failure = Some(error),
    }
    let fresh = match others.refresh_located(folder, verifier.as_ref()) {
        Ok(fresh) => fresh,
        Err(error) => {
            failure = Some(error);
            Vec::new()
        }
    };
    Scan::new(
        (fresh, failure),
        (others.clone(), keys.clone(), verifier.clone()),
    )
}
